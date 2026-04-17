use std::fs;
use std::path::Path;

#[test]
fn test_sdk_docsrs_metadata_exists() {
    let sdk_cargo_toml = Path::new("crates/sdk/Cargo.toml");
    let content = fs::read_to_string(sdk_cargo_toml).expect("Failed to read SDK Cargo.toml");

    assert!(
        content.contains("documentation = "),
        "SDK Cargo.toml should contain 'documentation' field for docs.rs publishing"
    );

    assert!(
        content.contains("https://docs.rs/opencode-sdk"),
        "SDK Cargo.toml should point to docs.rs URL"
    );
}

#[test]
fn test_sdk_has_description() {
    let sdk_cargo_toml = Path::new("crates/sdk/Cargo.toml");
    let content = fs::read_to_string(sdk_cargo_toml).expect("Failed to read SDK Cargo.toml");

    assert!(
        content.contains("description = "),
        "SDK Cargo.toml should contain 'description' field"
    );
}

#[test]
fn test_sdk_has_repository() {
    let sdk_cargo_toml = Path::new("crates/sdk/Cargo.toml");
    let content = fs::read_to_string(sdk_cargo_toml).expect("Failed to read SDK Cargo.toml");

    assert!(
        content.contains("repository = "),
        "SDK Cargo.toml should contain 'repository' field"
    );
}

#[test]
fn test_sdk_doc_builds_successfully() {
    let output = std::process::Command::new("cargo")
        .args(["doc", "--no-deps", "--all-features", "-p", "opencode-sdk"])
        .current_dir(".")
        .output();

    assert!(output.is_ok(), "cargo doc should execute without error");

    let output = output.unwrap();
    assert!(
        output.status.success(),
        "cargo doc should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_sdk_has_lib_with_public_api() {
    let sdk_lib_rs = Path::new("crates/sdk/src/lib.rs");
    let content = fs::read_to_string(sdk_lib_rs).expect("Failed to read SDK lib.rs");

    assert!(
        content.contains("pub use"),
        "SDK lib.rs should export public API items"
    );

    assert!(
        content.contains("//! # OpenCode SDK"),
        "SDK lib.rs should have crate-level doc comment"
    );
}
