//! Repository maintenance commands.

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const SOURCE_COMMIT: &str = "902886c2149261ccb53a9c982068b7ccd0990237";
const EXPECTED_JAVA_FILES: usize = 895;
const EXPECTED_PACKAGE_INFO_FILES: usize = 14;
const EXPECTED_MIGRATION_FILES: usize = 881;
const MAP_PATH: &str = "docs/migration/file-map.csv";

type TaskResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn main() -> TaskResult {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("migration-generate") => {
            let java_root = args
                .next()
                .map_or_else(|| PathBuf::from("../Sa-Token"), PathBuf::from);
            generate_migration_map(&java_root)
        }
        Some("migration-audit") => audit_migration_map(parse_audit_mode(args)?),
        Some("migration-audit-strict") => audit_migration_map(true),
        _ => {
            eprintln!(
                "usage: cargo xtask <migration-generate [java-root]|migration-audit [--strict]|migration-audit-strict>"
            );
            std::process::exit(2);
        }
    }
}

fn parse_audit_mode(mut args: impl Iterator<Item = String>) -> TaskResult<bool> {
    match (args.next().as_deref(), args.next()) {
        (None, None) => Ok(false),
        (Some("--strict"), None) => Ok(true),
        _ => Err("usage: cargo xtask migration-audit [--strict]".into()),
    }
}

fn generate_migration_map(java_root: &Path) -> TaskResult {
    let mut java_files = Vec::new();
    collect_files(java_root, OsStr::new("java"), &mut java_files)?;
    java_files.retain(|path| path.components().any(|part| part.as_os_str() == "main"));
    java_files.retain(|path| path.to_string_lossy().contains("/src/main/java/"));
    java_files.sort();

    if java_files.len() != EXPECTED_JAVA_FILES {
        return Err(format!(
            "expected {EXPECTED_JAVA_FILES} Java production files, found {}",
            java_files.len()
        )
        .into());
    }

    let output = Path::new(MAP_PATH);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = BufWriter::new(File::create(output)?);
    writeln!(
        writer,
        "java_file,rust_file,target_crate,rust_type,capability,status,test_evidence,source_commit"
    )?;

    for java_file in java_files {
        let relative = java_file.strip_prefix(java_root)?;
        let java_path = slash(relative);
        if java_file.file_name() == Some(OsStr::new("package-info.java")) {
            write_csv_row(
                &mut writer,
                &[
                    &java_path,
                    "",
                    "",
                    "",
                    "package_metadata",
                    "excluded",
                    "",
                    SOURCE_COMMIT,
                ],
            )?;
            continue;
        }

        let target = target_for_java(relative)?;
        let rust_type = java_file
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or("Java filename is not valid UTF-8")?;
        let target_crate = crate_for_java(relative)?;
        let capability = capability_for_java(relative);
        let status = if Path::new(&target).is_file() {
            "in_progress"
        } else {
            "planned"
        };
        write_csv_row(
            &mut writer,
            &[
                &java_path,
                &target,
                &target_crate,
                rust_type,
                capability,
                status,
                "",
                SOURCE_COMMIT,
            ],
        )?;
    }
    writer.flush()?;
    println!("generated {MAP_PATH} from {}", java_root.display());
    Ok(())
}

fn audit_migration_map(strict: bool) -> TaskResult {
    let file = BufReader::new(File::open(MAP_PATH)?);
    let mut lines = file.lines();
    let header = lines.next().ok_or("migration map is empty")??;
    if header
        != "java_file,rust_file,target_crate,rust_type,capability,status,test_evidence,source_commit"
    {
        return Err("unexpected migration map header".into());
    }

    let mut source_paths = HashSet::new();
    let mut target_paths = HashSet::new();
    let mut total = 0;
    let mut excluded = 0;
    let mut mapped = 0;
    let mut completed = 0;
    let mut errors = Vec::new();

    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let fields = parse_csv_line(&line?)?;
        if fields.len() != 8 {
            errors.push(format!("line {line_number}: expected 8 fields"));
            continue;
        }
        total += 1;
        if !source_paths.insert(fields[0].clone()) {
            errors.push(format!(
                "line {line_number}: duplicate Java path {}",
                fields[0]
            ));
        }
        if fields[7] != SOURCE_COMMIT {
            errors.push(format!("line {line_number}: unexpected source commit"));
        }
        if fields[5] == "excluded" {
            excluded += 1;
            if !fields[0].ends_with("/package-info.java") {
                errors.push(format!(
                    "line {line_number}: only package-info.java may be excluded"
                ));
            }
            continue;
        }

        mapped += 1;
        if fields[1].is_empty() || !target_paths.insert(fields[1].clone()) {
            errors.push(format!(
                "line {line_number}: missing or duplicate Rust path"
            ));
        }
        if !is_snake_rust_path(Path::new(&fields[1])) {
            errors.push(format!(
                "line {line_number}: non-snake Rust path {}",
                fields[1]
            ));
        }
        if fields[5] == "complete" {
            completed += 1;
            let target = Path::new(&fields[1]);
            if !target.is_file() || fs::metadata(target)?.len() == 0 {
                errors.push(format!(
                    "line {line_number}: completed target is missing or empty"
                ));
            }
            if fields[6].is_empty() {
                errors.push(format!(
                    "line {line_number}: completed target has no test evidence"
                ));
            }
        }
    }

    audit_actual_rust_layout(Path::new("crates"), &mut errors)?;
    if total != EXPECTED_JAVA_FILES {
        errors.push(format!(
            "expected {EXPECTED_JAVA_FILES} rows, found {total}"
        ));
    }
    if excluded != EXPECTED_PACKAGE_INFO_FILES {
        errors.push(format!(
            "expected {EXPECTED_PACKAGE_INFO_FILES} exclusions, found {excluded}"
        ));
    }
    if mapped != EXPECTED_MIGRATION_FILES {
        errors.push(format!(
            "expected {EXPECTED_MIGRATION_FILES} mappings, found {mapped}"
        ));
    }
    if strict && completed != EXPECTED_MIGRATION_FILES {
        errors.push(format!(
            "strict audit requires {EXPECTED_MIGRATION_FILES} completed mappings, found {completed}"
        ));
    }

    if errors.is_empty() {
        println!(
            "migration audit passed: total={total}, mapped={mapped}, excluded={excluded}, complete={completed}"
        );
        Ok(())
    } else {
        for error in &errors {
            eprintln!("error: {error}");
        }
        Err(format!("migration audit failed with {} error(s)", errors.len()).into())
    }
}

fn target_for_java(relative: &Path) -> TaskResult<String> {
    let source = slash(relative);
    let (prefix, crate_name) = if source.starts_with("sa-token-core/") {
        (
            "crates/sa-token-core/src".to_owned(),
            "sa-token-core".to_owned(),
        )
    } else if let Some(module) = module_after(&source, "sa-token-plugin/") {
        (
            format!("crates/sa-token-plugin/{module}/src"),
            module.to_owned(),
        )
    } else if let Some(module) = module_after(&source, "sa-token-starter/") {
        (
            format!("crates/sa-token-web/sa-token-web-core/src/source/{module}"),
            "sa-token-web-core".to_owned(),
        )
    } else if let Some(demo_path) = source
        .strip_prefix("sa-token-demo/")
        .and_then(|tail| tail.split_once("/src/main/java/").map(|(path, _)| path))
    {
        (
            format!("crates/sa-token-demo/sa-token-demo-suite/src/source/{demo_path}"),
            "sa-token-demo-suite".to_owned(),
        )
    } else {
        return Err(format!("unsupported Java module: {source}").into());
    };
    let _ = crate_name;

    let marker = "/src/main/java/";
    let package_path = source
        .split_once(marker)
        .map(|(_, path)| path)
        .ok_or_else(|| format!("missing {marker} in {source}"))?;
    let package_path = strip_java_namespace(package_path);
    let mut target = PathBuf::from(prefix);
    let package = Path::new(package_path);
    if let Some(parent) = package.parent() {
        for component in parent.components() {
            target.push(to_snake_case(&component.as_os_str().to_string_lossy()));
        }
    }
    let stem = package
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or("Java filename is not valid UTF-8")?;
    target.push(format!("{}.rs", to_snake_case(stem)));
    Ok(slash(&target))
}

fn crate_for_java(relative: &Path) -> TaskResult<String> {
    let source = slash(relative);
    if source.starts_with("sa-token-core/") {
        Ok("sa-token-core".to_owned())
    } else if let Some(module) = module_after(&source, "sa-token-plugin/") {
        Ok(module.to_owned())
    } else if source.starts_with("sa-token-starter/") {
        Ok("sa-token-web-core".to_owned())
    } else if source.starts_with("sa-token-demo/") {
        Ok("sa-token-demo-suite".to_owned())
    } else {
        Err(format!("unsupported Java module: {source}").into())
    }
}

fn capability_for_java(relative: &Path) -> &'static str {
    let source = slash(relative);
    if source.starts_with("sa-token-core/") {
        "core"
    } else if source.starts_with("sa-token-plugin/") {
        "plugin"
    } else if source.starts_with("sa-token-starter/") {
        "web_integration"
    } else {
        "demo"
    }
}

fn module_after<'a>(source: &'a str, prefix: &str) -> Option<&'a str> {
    source
        .strip_prefix(prefix)
        .and_then(|tail| tail.split('/').next())
}

fn strip_java_namespace(path: &str) -> &str {
    for prefix in ["cn/dev33/satoken/", "com/pj/satoken/"] {
        if let Some(stripped) = path.strip_prefix(prefix) {
            return stripped;
        }
    }
    path
}

fn collect_files(root: &Path, extension: &OsStr, output: &mut Vec<PathBuf>) -> TaskResult {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, extension, output)?;
        } else if path.extension() == Some(extension) {
            output.push(path);
        }
    }
    Ok(())
}

fn audit_actual_rust_layout(root: &Path, errors: &mut Vec<String>) -> TaskResult {
    let mut rust_files = Vec::new();
    collect_files(root, OsStr::new("rs"), &mut rust_files)?;
    for file in rust_files {
        if file.file_name() == Some(OsStr::new("mod.rs")) {
            errors.push(format!("legacy module entry remains: {}", file.display()));
        }
        if !is_snake_name(file.file_stem().and_then(OsStr::to_str).unwrap_or_default()) {
            errors.push(format!("non-snake Rust filename: {}", file.display()));
        }
        let mut under_src = false;
        for component in file.components() {
            let name = component.as_os_str().to_string_lossy();
            if name == "src" {
                under_src = true;
            } else if under_src && name.ends_with(".rs") {
                break;
            } else if under_src && !is_snake_name(&name) {
                errors.push(format!(
                    "non-snake Rust module directory: {}",
                    file.display()
                ));
                break;
            }
        }
    }
    Ok(())
}

fn is_snake_rust_path(path: &Path) -> bool {
    path.extension() == Some(OsStr::new("rs"))
        && path
            .file_stem()
            .and_then(OsStr::to_str)
            .is_some_and(is_snake_name)
}

fn is_snake_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && !name.starts_with('_')
        && !name.ends_with('_')
        && !name.contains("__")
}

fn to_snake_case(input: &str) -> String {
    match input.to_ascii_lowercase().as_str() {
        "granttype" => return "grant_type".to_owned(),
        "httpauth" => return "http_auth".to_owned(),
        "timedcache" => return "timed_cache".to_owned(),
        "wrapperinfo" => return "wrapper_info".to_owned(),
        _ => {}
    }
    let normalized = input
        .replace("OAuth", "Oauth")
        .replace("BCrypt", "Bcrypt")
        .replace("URL", "Url")
        .replace("URI", "Uri")
        .replace("HTTP", "Http")
        .replace("JSON", "Json")
        .replace("JWT", "Jwt")
        .replace("TOTP", "Totp")
        .replace("SSO", "Sso")
        .replace("API", "Api");
    let chars: Vec<char> = normalized.chars().collect();
    let mut output = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        if ch == '-' || ch == ' ' || ch == '.' {
            if !output.ends_with('_') {
                output.push('_');
            }
            continue;
        }
        if ch == '_' {
            if !output.ends_with('_') {
                output.push('_');
            }
            continue;
        }
        let previous = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(index + 1).copied();
        let boundary = ch.is_ascii_uppercase()
            && !output.is_empty()
            && (previous.is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
                || (previous.is_some_and(|value| value.is_ascii_uppercase())
                    && next.is_some_and(|value| value.is_ascii_lowercase())));
        if boundary && !output.ends_with('_') {
            output.push('_');
        }
        output.push(ch.to_ascii_lowercase());
    }
    output.trim_matches('_').to_owned()
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_csv_row(writer: &mut impl Write, fields: &[&str]) -> TaskResult {
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            writer.write_all(b",")?;
        }
        write_csv_field(writer, field)?;
    }
    writer.write_all(b"\n")?;
    Ok(())
}

fn write_csv_field(writer: &mut impl Write, field: &str) -> TaskResult {
    if field.contains([',', '"', '\n']) {
        writer.write_all(b"\"")?;
        writer.write_all(field.replace('"', "\"\"").as_bytes())?;
        writer.write_all(b"\"")?;
    } else {
        writer.write_all(field.as_bytes())?;
    }
    Ok(())
}

fn parse_csv_line(line: &str) -> TaskResult<Vec<String>> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.chars().peekable();
    let mut quoted = false;
    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
            _ => field.push(ch),
        }
    }
    if quoted {
        return Err("unterminated quoted CSV field".into());
    }
    fields.push(field);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::{is_snake_name, parse_audit_mode, parse_csv_line, to_snake_case};

    #[test]
    fn converts_java_names_to_rust_names() {
        assert_eq!(to_snake_case("StpLogic"), "stp_logic");
        assert_eq!(
            to_snake_case("SaOAuth2ServerConfig"),
            "sa_oauth2_server_config"
        );
        assert_eq!(to_snake_case("BCrypt"), "bcrypt");
        assert_eq!(to_snake_case("SaHTTPBasicUtil"), "sa_http_basic_util");
        assert_eq!(to_snake_case("timedcache"), "timed_cache");
    }

    #[test]
    fn validates_snake_names() {
        assert!(is_snake_name("sa_token_dao"));
        assert!(!is_snake_name("SaTokenDao"));
        assert!(!is_snake_name("sa__token"));
    }

    #[test]
    fn parses_quoted_csv() {
        assert_eq!(
            parse_csv_line("a,\"b,c\",\"d\"\"e\"").expect("CSV should parse"),
            ["a", "b,c", "d\"e"]
        );
    }

    #[test]
    fn parses_strict_audit_flag_without_ignoring_trailing_arguments() {
        assert!(!parse_audit_mode(Vec::<String>::new().into_iter()).expect("plain audit"));
        assert!(parse_audit_mode(vec!["--strict".to_owned()].into_iter()).expect("strict audit"));
        assert!(parse_audit_mode(vec!["--unknown".to_owned()].into_iter()).is_err());
        assert!(
            parse_audit_mode(vec!["--strict".to_owned(), "extra".to_owned()].into_iter()).is_err()
        );
    }
}
