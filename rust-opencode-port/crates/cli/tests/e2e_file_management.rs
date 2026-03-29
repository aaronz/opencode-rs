use crate::common::{TestHarness, EMPTY_VEC};
use std::fs;

mod common;

#[test]
fn test_file_tree_navigation() {
    let harness = TestHarness::setup();

    harness.setup_file("src/main.rs", "fn main() {}");
    harness.setup_file("src/lib.rs", "pub fn lib() {}");
    harness.setup_file("Cargo.toml", "[package]");

    let output = harness.run_cli(&["files", "list", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let files = json.as_array().expect("Expected array of files");
    assert!(files
        .iter()
        .any(|f| f["path"].as_str().unwrap_or("").contains("main.rs")));
    assert!(files
        .iter()
        .any(|f| f["path"].as_str().unwrap_or("").contains("lib.rs")));
}

#[test]
fn test_file_read_content() {
    let harness = TestHarness::setup();

    harness.setup_file("test.txt", "Hello, World!");

    let output = harness.run_cli(&["files", "read", "test.txt"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"));
}

#[test]
fn test_file_search() {
    let harness = TestHarness::setup();

    harness.setup_file("file1.rs", "fn foo() {}");
    harness.setup_file("file2.rs", "fn bar() {}");
    harness.setup_file("readme.md", "# README");

    let output = harness.run_cli(&["files", "search", "--pattern", "fn ", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let results = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(results.len() >= 2);
}

#[test]
fn test_file_filter_by_extension() {
    let harness = TestHarness::setup();

    harness.setup_file("test.rs", "fn main() {}");
    harness.setup_file("test.md", "# Test");
    harness.setup_file("test.txt", "Hello");

    let output = harness.run_cli(&["files", "list", "--ext", "rs", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let files = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(files
        .iter()
        .all(|f| f["path"].as_str().unwrap_or("").ends_with(".rs")));
}
