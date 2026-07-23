#!/usr/bin/env python3
"""Normalize Rust source paths and module declarations.

The rename is deliberately two-step so it also works on case-insensitive file
systems. Run without ``--apply`` to preview the operation.
"""

from __future__ import annotations

import argparse
import os
import re
import uuid
from pathlib import Path


ACRONYMS = {
    "OAuth": "Oauth",
    "BCrypt": "Bcrypt",
    "URL": "Url",
    "URI": "Uri",
    "HTTP": "Http",
    "JSON": "Json",
    "JWT": "Jwt",
    "TOTP": "Totp",
    "SSO": "Sso",
    "API": "Api",
    "EL": "El",
    "JDK": "Jdk",
    "ISO": "Iso",
}

COMPOUND_MODULES = {
    "granttype": "grant_type",
    "httpauth": "http_auth",
    "timedcache": "timed_cache",
    "wrapperinfo": "wrapper_info",
}


def snake_case(name: str) -> str:
    if name in COMPOUND_MODULES:
        return COMPOUND_MODULES[name]
    for source, replacement in ACRONYMS.items():
        name = name.replace(source, replacement)
    name = re.sub(r"[- .]+", "_", name)
    name = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name)
    name = re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", "_", name)
    return re.sub(r"_+", "_", name).strip("_").lower()


def case_safe_rename(source: Path, target: Path) -> None:
    if source == target:
        return
    if target.exists() and not os.path.samefile(source, target):
        raise RuntimeError(f"rename collision: {source} -> {target}")
    temporary = source.with_name(f".__sa_token_tmp_{uuid.uuid4().hex}")
    source.rename(temporary)
    temporary.rename(target)


def collect_renames(root: Path) -> tuple[list[tuple[Path, Path]], dict[str, str]]:
    renames: list[tuple[Path, Path]] = []
    identifiers: dict[str, str] = {}

    for rust_file in root.glob("**/src/**/*.rs"):
        source = rust_file.read_text(encoding="utf-8")
        for module_name in re.findall(r"\b(?:pub\s+)?mod\s+([A-Z][A-Za-z0-9_]*)\b", source):
            identifiers[module_name] = snake_case(module_name)

    directories = sorted(
        {path.parent for path in root.glob("**/src/**/*.rs")},
        key=lambda path: len(path.parts),
        reverse=True,
    )
    for directory in directories:
        if directory.name == "src":
            continue
        normalized = snake_case(directory.name)
        if normalized != directory.name:
            identifiers[directory.name] = normalized
            renames.append((directory, directory.with_name(normalized)))

    for source, target in renames:
        case_safe_rename(source, target)

    file_renames: list[tuple[Path, Path]] = []
    for source in root.glob("**/src/**/*.rs"):
        if source.name in {"lib.rs", "main.rs", "mod.rs"}:
            continue
        normalized = snake_case(source.stem)
        if normalized != source.stem:
            identifiers[source.stem] = normalized
            file_renames.append((source, source.with_name(f"{normalized}.rs")))
    for source, target in file_renames:
        case_safe_rename(source, target)
    renames.extend(file_renames)

    module_renames: list[tuple[Path, Path]] = []
    mod_files = sorted(root.glob("**/src/**/mod.rs"), key=lambda path: len(path.parts), reverse=True)
    for source in mod_files:
        module_name = snake_case(source.parent.name)
        target = source.parent.parent / f"{module_name}.rs"
        if target.exists():
            raise RuntimeError(f"modern module collision: {source} -> {target}")
        module_renames.append((source, target))
        case_safe_rename(source, target)
    renames.extend(module_renames)
    return renames, identifiers


def rewrite_modules(root: Path, identifiers: dict[str, str]) -> int:
    declared_types: dict[str, str] = {}
    for path in root.glob("**/src/**/*.rs"):
        source = path.read_text(encoding="utf-8")
        for type_name in re.findall(
            r"\b(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait|type)\s+([A-Z][A-Za-z0-9_]*)\b",
            source,
        ):
            declared_types[snake_case(type_name)] = type_name
        for alias_name in re.findall(
            r"\bpub\s+use\s+[A-Za-z][A-Za-z0-9_]*\s+as\s+([A-Z][A-Za-z0-9_]*)\b",
            source,
        ):
            declared_types[snake_case(alias_name)] = alias_name

    changed = 0
    names = sorted(identifiers, key=len, reverse=True)
    for path in root.glob("**/src/**/*.rs"):
        source = path.read_text(encoding="utf-8")
        target = source
        for old in names:
            new = identifiers[old]
            target = re.sub(rf"\b(pub\s+)?mod\s+{re.escape(old)}\b", lambda match: match.group(0).replace(old, new), target)
            target = re.sub(rf"(?<=::){re.escape(old)}(?=::|\b)", new, target)
            target = re.sub(rf"\b((?:pub\s+)?use\s+){re.escape(old)}(?=::)", rf"\1{new}", target)
            target = re.sub(rf"\b{re.escape(old)}::{re.escape(old)}\b", f"{new}::{old}", target)

        for snake_name, type_name in sorted(declared_types.items(), key=lambda item: -len(item[0])):
            target = re.sub(
                rf"::{re.escape(snake_name)}(?!::|!)(?=\b)",
                f"::{type_name}",
                target,
            )

        module_names = {type_name: snake_name for snake_name, type_name in declared_types.items()}
        module_names.update(
            {
                "Auto": "auto",
                "Enums": "enums",
                "GrantType": "grant_type",
                "Hooks": "hooks",
                "Model": "model",
                "Mock": "mock",
                "Parameter": "parameter",
                "Raw": "raw",
                "SaSerializerImpl": "sa_serializer_impl",
                "Strategy": "strategy",
                "TimedCache": "timed_cache",
                "Totp": "totp",
            }
        )

        def normalize_use(match: re.Match[str]) -> str:
            statement = match.group(0)
            for old, new in sorted(module_names.items(), key=lambda item: -len(item[0])):
                statement = re.sub(rf"\b{re.escape(old)}(?=::)", new, statement)
            return statement

        target = re.sub(r"(?ms)^\s*(?:pub\s+)?use\s+.*?;", normalize_use, target)
        for old, new in {
            "Model": "model",
            "Mock": "mock",
            "Parameter": "parameter",
            "Enums": "enums",
            "Hooks": "hooks",
            "Strategy": "strategy",
            "Auto": "auto",
            "TimedCache": "timed_cache",
            "Raw": "raw",
            "Totp": "totp",
            "SaSerializerImpl": "sa_serializer_impl",
            "GrantType": "grant_type",
        }.items():
            target = re.sub(rf"(?<=::){old}(?=::)", new, target)
        target = target.replace("tracing::Error!", "tracing::error!")
        if target != source:
            path.write_text(target, encoding="utf-8")
            changed += 1
    return changed


def planned_operations(root: Path) -> list[str]:
    operations: list[str] = []
    for path in sorted(root.glob("**/src/**/*.rs")):
        if path.name == "mod.rs":
            operations.append(f"MOVE {path} -> {path.parent.parent / (snake_case(path.parent.name) + '.rs')}")
        elif path.name not in {"lib.rs", "main.rs"} and snake_case(path.stem) != path.stem:
            operations.append(f"MOVE {path} -> {path.with_name(snake_case(path.stem) + '.rs')}")
    for directory in sorted({path.parent for path in root.glob("**/src/**/*.rs")}):
        if directory.name != "src" and snake_case(directory.name) != directory.name:
            operations.append(f"MOVE {directory} -> {directory.with_name(snake_case(directory.name))}")
    return operations


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--root", type=Path, default=Path("crates"))
    args = parser.parse_args()

    operations = planned_operations(args.root)
    if not args.apply:
        print("\n".join(operations))
        print(f"planned operations: {len(operations)}")
        return

    renames, identifiers = collect_renames(args.root)
    rewritten = rewrite_modules(args.root, identifiers)
    print(f"renamed paths: {len(renames)}")
    print(f"rewritten Rust files: {rewritten}")


if __name__ == "__main__":
    main()
