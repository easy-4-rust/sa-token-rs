//! Repository maintenance commands.

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE_COMMIT: &str = "902886c2149261ccb53a9c982068b7ccd0990237";
const EXPECTED_JAVA_FILES: usize = 895;
const EXPECTED_PACKAGE_INFO_FILES: usize = 14;
const EXPECTED_MIGRATION_FILES: usize = 881;
const MAP_PATH: &str = "docs/migration/file-map.csv";
const EXPORTER_DIR: &str = "scripts/java-golden-export";
const DEFAULT_GOLDEN_OUTPUT: &str =
    "crates/sa-token-test/sa-token-easy-test/tests/golden/core.json";

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
        Some("golden-refresh") => golden_refresh(&GoldenRefreshOptions::parse(args)?),
        Some("golden-split") => golden_split(&GoldenSplitOptions::parse(args)?),
        Some("--help") | Some("-h") | Some("help") => {
            print_help();
            Ok(())
        }
        _ => {
            eprintln!(
                "usage: cargo xtask <migration-generate [java-root]|migration-audit [--strict]|migration-audit-strict|golden-refresh [...]|golden-split [...]|help>"
            );
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!(
        "Repository maintenance tasks:\n\n\
         cargo xtask migration-generate [JAVA_ROOT]\n    \
         Build docs/migration/file-map.csv by scanning Java sources.\n\n\
         cargo xtask migration-audit [--strict]\n    \
         Verify docs/migration/file-map.csv against Rust crates.\n\n\
         cargo xtask golden-refresh\n    \
         Refresh the Java baseline used by java_golden_test.\n\n\
         Options for golden-refresh:\n  \
           --java-root <PATH>   Path to the Sa-Token (Java) checkout\n  \
                                 (default: $SA_TOKEN_JAVA_ROOT, then ../Sa-Token, then ../../Sa-Token)\n  \
           --ref <REF>          Check out <REF> inside --java-root before exporting\n  \
                                 (e.g. commit SHA, tag, or branch); updates fixture only\n  \
                                 when ref resolves to a 40-char SHA.\n  \
           --output <FILE>      Destination for the generated JSON\n  \
                                 (default: {DEFAULT_GOLDEN_OUTPUT})\n  \
           --skip-build         Skip `mvn package`; reuse exporter jar from target/.\n  \
           --clean              After success, delete exporter target/ directory.\n  \
           --mvn <PATH>         Use <PATH> as the `mvn` executable (default: `mvn`).\n\n\
         cargo xtask golden-split\n    \
         Slice the master core.json into per-domain fixtures for java_golden_test_<domain>.\n\n\
         Options for golden-split:\n  \
           --source <FILE>      Master JSON input (default: {DEFAULT_GOLDEN_OUTPUT})\n  \
           --out-dir <DIR>      Directory for per-domain fixtures\n  \
                                 (default: <source parent directory>)\n"
    );
}

fn parse_audit_mode(mut args: impl Iterator<Item = String>) -> TaskResult<bool> {
    match (args.next().as_deref(), args.next()) {
        (None, None) => Ok(false),
        (Some("--strict"), None) => Ok(true),
        _ => Err("usage: cargo xtask migration-audit [--strict]".into()),
    }
}

#[derive(Debug)]
struct GoldenRefreshOptions {
    java_root: Option<PathBuf>,
    git_ref: Option<String>,
    output: PathBuf,
    skip_build: bool,
    clean: bool,
    mvn: String,
}

impl GoldenRefreshOptions {
    fn parse(mut args: impl Iterator<Item = String>) -> TaskResult<Self> {
        let mut java_root = None;
        let mut git_ref = None;
        let mut output = PathBuf::from(DEFAULT_GOLDEN_OUTPUT);
        let mut skip_build = false;
        let mut clean = false;
        let mut mvn = env::var("MVN").unwrap_or_else(|_| "mvn".to_owned());

        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--java-root" => {
                    java_root = Some(PathBuf::from(args.next().ok_or(
                        "--java-root requires a path argument (usage: cargo xtask help)",
                    )?));
                }
                "--ref" => {
                    git_ref = Some(args.next().ok_or("--ref requires a ref argument")?);
                }
                "--output" => {
                    output = PathBuf::from(args.next().ok_or("--output requires a path argument")?);
                }
                "--skip-build" => skip_build = true,
                "--clean" => clean = true,
                "--mvn" => {
                    mvn = args.next().ok_or("--mvn requires a path argument")?;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => {
                    return Err(format!("unknown golden-refresh argument: {other}").into());
                }
            }
        }

        Ok(Self {
            java_root,
            git_ref,
            output,
            skip_build,
            clean,
            mvn,
        })
    }
}

fn resolve_java_root(explicit: Option<PathBuf>) -> TaskResult<PathBuf> {
    if let Some(path) = explicit {
        if !path.is_dir() {
            return Err(format!("--java-root does not exist: {}", path.display()).into());
        }
        return Ok(path);
    }

    let candidates: [PathBuf; 4] = [
        PathBuf::from("../Sa-Token"),
        PathBuf::from("../../Sa-Token"),
        PathBuf::from("../../../Sa-Token"),
        PathBuf::from("./Sa-Token"),
    ];
    if let Ok(env_root) = env::var("SA_TOKEN_JAVA_ROOT") {
        let candidate = PathBuf::from(env_root);
        if candidate.is_dir() {
            return Ok(candidate);
        }
        eprintln!(
            "warning: SA_TOKEN_JAVA_ROOT is set to {} but the directory is missing",
            candidate.display()
        );
    }
    for candidate in candidates {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Err(
        "could not locate Sa-Token Java checkout: pass --java-root or set $SA_TOKEN_JAVA_ROOT"
            .into(),
    )
}

fn current_or_ref_head(java_root: &Path, git_ref: Option<&str>) -> TaskResult<String> {
    let rev_parse = |spec: &str| -> TaskResult<Option<String>> {
        let output = Command::new("git")
            .args(["-C", &java_root.to_string_lossy(), "rev-parse", spec])
            .output()?;
        if !output.status.success() {
            return Ok(None);
        }
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if sha.is_empty() {
            Ok(None)
        } else {
            Ok(Some(sha))
        }
    };

    if let Some(spec) = git_ref {
        let sha = rev_parse(spec)?.ok_or_else(|| {
            format!(
                "could not resolve --ref {spec} inside {}; pass a commit SHA, tag, or branch",
                java_root.display()
            )
        })?;
        if sha.len() != 40 {
            return Err(format!(
                "git rev-parse {spec} returned '{sha}', not a full 40-char SHA; \
                 align HEAD to a real commit first inside {}",
                java_root.display()
            )
            .into());
        }
        return Ok(sha);
    }

    rev_parse("HEAD")?.ok_or_else(|| {
        format!(
            "{} is not a git working tree; pass --ref <commit> or initialize it first",
            java_root.display()
        )
        .into()
    })
}

fn run_command(program: &str, args: &[&str], cwd: &Path) -> TaskResult {
    let status = Command::new(program).args(args).current_dir(cwd).status()?;
    if !status.success() {
        return Err(format!(
            "{program} {} failed (status {status:?}) inside {}",
            args.join(" "),
            cwd.display()
        )
        .into());
    }
    Ok(())
}

fn golden_refresh(options: &GoldenRefreshOptions) -> TaskResult {
    let workspace_root = env::current_dir()?;

    let java_root = resolve_java_root(options.java_root.clone())?;
    if !java_root.join("pom.xml").is_file() {
        return Err(format!(
            "{} is not a Maven project (no pom.xml); pass --java-root that points at the Sa-Token repo root",
            java_root.display()
        )
        .into());
    }

    let exporter_dir = workspace_root.join(EXPORTER_DIR);
    let exporter_pom = exporter_dir.join("pom.xml");
    if !exporter_pom.is_file() {
        return Err(format!(
            "exporter module missing: {} (expected a standalone Maven module mirroring scripts/java-golden-export). \
             The exporter reads sa-token-core/apikey/jwt/sign/sso/oauth2 at its declared <sa-token.version>; \
             it does NOT need to be checked into the Java repo at this point.",
            exporter_pom.display()
        )
        .into());
    }

    let sha = current_or_ref_head(&java_root, options.git_ref.as_deref())?;
    println!("java baseline: {sha}");
    println!("  java checkout: {}", java_root.display());
    println!("  exporter module: {}", exporter_dir.display());

    if !options.skip_build {
        run_command(
            &options.mvn,
            &["-q", "-DskipTests", "-f", "pom.xml", "package"],
            &exporter_dir,
        )?;
    } else {
        println!("  --skip-build set: reusing whatever already lives under {EXPORTER_DIR}/target");
    }

    let exporter_output = exporter_dir.join("target/sa-token-java-golden-export-output.json");
    println!(
        "  invoking cn.dev33.satoken.golden.CoreGoldenExporter -> {}",
        exporter_output.display()
    );
    let exec_args = format!("{} {sha}", exporter_output.to_string_lossy());
    run_command(
        &options.mvn,
        &[
            "-q",
            "-f",
            "pom.xml",
            "exec:java",
            "-Dexec.mainClass=cn.dev33.satoken.golden.CoreGoldenExporter",
            &format!("-Dexec.args={exec_args}"),
        ],
        &exporter_dir,
    )?;

    if !exporter_output.is_file() {
        return Err(format!(
            "exporter did not produce {}; check Maven output above",
            exporter_output.display()
        )
        .into());
    }

    if let Some(parent) = options.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::copy(&exporter_output, &options.output)?;
    println!(
        "wrote {} ({} bytes)",
        options.output.display(),
        fs::metadata(&options.output)?.len()
    );

    println!("\nfollow-up actions:");
    println!(
        "  - if the SHA changed, update JAVA_BASELINE in crates/sa-token-test/sa-token-easy-test/tests/java_golden_test.rs"
    );
    println!("  - and SOURCE_COMMIT in xtask/src/main.rs");
    println!("  - rerun `cargo test -p sa-token-easy-test --test java_golden_test` to confirm the new fixture matches");

    if options.clean {
        let target = exporter_dir.join("target");
        if target.is_dir() {
            fs::remove_dir_all(&target)?;
            println!("  --clean: removed {}", target.display());
        }
    }
    Ok(())
}

const DOMAIN_CORE_SA_TOKEN: &str = "core_sa_token";
const DOMAIN_SERIALIZER: &str = "serializer";
const DOMAIN_JWT: &str = "jwt";
const DOMAIN_SIGN: &str = "sign";
const DOMAIN_SSO: &str = "sso";
const DOMAIN_OAUTH2: &str = "oauth2";
const DOMAIN_APIKEY: &str = "apikey";

/// Per-domain key map. Source-of-truth for both `golden_split` (which files to
/// emit) and the `java_golden_test_<domain>.rs` files (which keys each test
/// compares). Keep this in sync with ROADMAP M0.2 and CoreGoldenExporter.java.
fn domain_keys(domain: &str) -> &'static [&'static str] {
    match domain {
        DOMAIN_CORE_SA_TOKEN => &[
            "token_name",
            "timeout",
            "active_timeout",
            "is_concurrent",
            "max_login_count",
            "same_token_timeout",
            "token_session_check_login",
            "auto_renew",
            "token_key",
            "session_key",
            "token_session_key",
            "last_active_key",
            "switch_key",
            "disable_key",
            "disable_service_key",
            "safe_key",
            "safe_service_key",
        ],
        DOMAIN_SERIALIZER => &[
            "serializer_base64",
            "serializer_hex",
            "serializer_iso_8859_1",
            "serializer_emoji",
            "serializer_periodic_table",
            "serializer_special_symbols",
            "serializer_tian_gan",
        ],
        DOMAIN_JWT => &["jwt_hs256_token"],
        DOMAIN_SIGN => &[
            "sign_default_timestamp_disparity",
            "sign_default_digest",
            "sign_md5",
        ],
        DOMAIN_SSO => &[
            "sso_client_auth_url",
            "sso_client_is_http",
            "sso_client_is_slo",
            "sso_server_ticket_timeout",
            "sso_server_max_reg_client",
            "sso_api_check_ticket",
            "sso_param_secretkey",
            "sso_client_wildcard",
            "sso_last_error_code",
            "sso_server_auth_url",
            "sso_ticket_key",
            "sso_ticket_index_key",
            "sso_encoded_back_url",
        ],
        DOMAIN_OAUTH2 => &[
            "oauth2_oidc_id_token_timeout",
            "oauth2_grant_authorization_code",
            "oauth2_authorize_api",
            "oauth2_finally_work_scope",
            "oauth2_last_error_code",
            "oauth2_code_key",
            "oauth2_code_index_key",
            "oauth2_access_token_key",
            "oauth2_access_token_rsd",
            "oauth2_refresh_token_key",
            "oauth2_client_token_key",
            "oauth2_grant_scope_key",
            "oauth2_state_key",
            "oauth2_nonce_key",
            "oauth2_openid",
            "oauth2_unionid",
        ],
        DOMAIN_APIKEY => &[
            "api_key_prefix",
            "api_key_timeout",
            "api_key_record_index",
            "api_key_save_key",
            "api_key_invalid_code",
            "api_key_scope_code",
        ],
        other => panic!("unknown domain: {other}"),
    }
}

/// All known domains in canonical order. Used by `golden_split` to drive the
/// per-file emission, and by the test catalog when introspecting coverage.
const ALL_DOMAINS: &[&str] = &[
    DOMAIN_CORE_SA_TOKEN,
    DOMAIN_SERIALIZER,
    DOMAIN_JWT,
    DOMAIN_SIGN,
    DOMAIN_SSO,
    DOMAIN_OAUTH2,
    DOMAIN_APIKEY,
];

#[derive(Debug)]
struct GoldenSplitOptions {
    source: PathBuf,
    out_dir: PathBuf,
}

impl GoldenSplitOptions {
    fn parse(mut args: impl Iterator<Item = String>) -> TaskResult<Self> {
        let mut source = PathBuf::from(DEFAULT_GOLDEN_OUTPUT);
        let mut out_dir: Option<PathBuf> = None;

        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--source" => {
                    source = PathBuf::from(args.next().ok_or("--source requires a path argument")?);
                }
                "--out-dir" => {
                    out_dir = Some(PathBuf::from(
                        args.next().ok_or("--out-dir requires a path argument")?,
                    ));
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => {
                    return Err(format!("unknown golden-split argument: {other}").into());
                }
            }
        }

        let out_dir = out_dir.unwrap_or_else(|| {
            source
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        });

        Ok(Self { source, out_dir })
    }
}

fn golden_split(options: &GoldenSplitOptions) -> TaskResult {
    if !options.source.is_file() {
        return Err(format!(
            "master golden file not found: {} — run `cargo xtask golden-refresh` first, or pass --source <path>",
            options.source.display()
        )
        .into());
    }

    let raw = fs::read_to_string(&options.source)?;
    let master: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| format!("master golden {} is not valid JSON: {err}", options.source.display()))?;
    let source_commit = master
        .get("source_commit")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            format!(
                "master golden {} is missing a string `source_commit` field",
                options.source.display()
            )
        })?
        .to_owned();

    let mut missing: Vec<String> = Vec::new();
    for domain in ALL_DOMAINS {
        for key in domain_keys(domain) {
            if !master.as_object().map_or(true, |obj| obj.contains_key(*key)) {
                missing.push(format!("{domain}:{key}"));
            }
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "master golden {} is missing {} keys: {}",
            options.source.display(),
            missing.len(),
            missing.join(", ")
        )
        .into());
    }

    fs::create_dir_all(&options.out_dir)?;

    let master_object = master
        .as_object()
        .expect("master golden is an object (parsed above)");

    let mut emitted: Vec<(String, usize)> = Vec::new();
    for domain in ALL_DOMAINS {
        let mut per_domain = serde_json::Map::with_capacity(domain_keys(domain).len() + 1);
        per_domain.insert(
            "source_commit".to_owned(),
            serde_json::Value::String(source_commit.clone()),
        );
        for key in domain_keys(domain) {
            let value = master_object.get(*key).cloned().unwrap_or(serde_json::Value::Null);
            per_domain.insert((*key).to_owned(), value);
        }
        let body = serde_json::Value::Object(per_domain);
        let path = options.out_dir.join(format!("{domain}.json"));
        let serialized = serde_json::to_string_pretty(&body)?;
        fs::write(&path, format!("{serialized}\n"))?;
        emitted.push((path.display().to_string(), serialized.len()));
    }

    println!("split {} into {} domain files:", options.source.display(), emitted.len());
    for (path, len) in &emitted {
        println!("  {path} ({len} bytes)");
    }
    Ok(())
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
