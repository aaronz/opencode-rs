use std::process::Command;

fn workspace_root() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    std::path::PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .to_path_buf()
}

const PUBLIC_CRATES: &[&str] = &[
    "opencode-agent",
    "opencode-auth",
    "opencode-cli",
    "opencode-config",
    "opencode-control-plane",
    "opencode-core",
    "opencode-git",
    "opencode-llm",
    "opencode-lsp",
    "opencode-mcp",
    "opencode-permission",
    "opencode-plugin",
    "opencode-sdk",
    "opencode-server",
    "opencode-storage",
    "opencode-tools",
    "opencode-tui",
];

#[test]
fn test_all_public_crates_doc_build_successfully() {
    let root = workspace_root();
    for crate_name in PUBLIC_CRATES {
        let output = Command::new("cargo")
            .args(["doc", "--no-deps", "--all-features", "-p", crate_name])
            .current_dir(&root)
            .output();

        assert!(
            output.is_ok(),
            "cargo doc should execute without error for {}",
            crate_name
        );

        let output = output.unwrap();
        assert!(
            output.status.success(),
            "cargo doc should succeed for {}. stderr: {}",
            crate_name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_ci_doc_command_builds_successfully() {
    let root = workspace_root();
    let output = Command::new("cargo")
        .args(["doc", "--all-features", "--no-deps"])
        .current_dir(&root)
        .output();

    assert!(output.is_ok(), "cargo doc should execute without error");

    let output = output.unwrap();
    assert!(
        output.status.success(),
        "cargo doc should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
