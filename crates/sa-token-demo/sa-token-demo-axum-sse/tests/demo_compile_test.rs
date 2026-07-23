//! Compile contract for `sa-token-demo-sse` scenario demos (Wave 7 ledger evidence).

#[test]
fn demo_axum_crate_has_main_entry() {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    assert!(manifest.contains("name = \"sa-token-demo-axum-sse\""));
    assert!(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs")).exists());
}

#[test]
fn dual_track_actix_crate_exists() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("workspace root");
    let actix = workspace.join("crates/sa-token-demo/sa-token-demo-actix-sse/src/main.rs");
    assert!(actix.exists(), "missing actix dual-track demo");
}
