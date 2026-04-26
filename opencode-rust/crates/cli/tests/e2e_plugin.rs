mod common;

use common::TestHarness;
use std::fs;

#[test]
fn test_plugin_install_persists_to_config() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, "{}").unwrap();

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["plugin", "install", "test-plugin"]);
    assert!(
        output.status.success(),
        "plugin install should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config_content = fs::read_to_string(&config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    let plugins = config.get("plugin").and_then(|p| p.as_array());
    assert!(
        plugins
            .map(|p| p.contains(&serde_json::json!("test-plugin")))
            .unwrap_or(false),
        "plugin list should contain 'test-plugin' after install, got: {:?}",
        config
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}

#[test]
fn test_plugin_list_shows_installed_plugins() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    let config_content = serde_json::json!({
        "plugin": ["my-plugin", "other-plugin"]
    });
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&config_content).unwrap(),
    )
    .unwrap();

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["plugin", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my-plugin"),
        "plugin list should show 'my-plugin', got: {}",
        stdout
    );
    assert!(
        stdout.contains("other-plugin"),
        "plugin list should show 'other-plugin', got: {}",
        stdout
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}

#[test]
fn test_plugin_remove_removes_from_config() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    let initial_config = serde_json::json!({
        "plugin": ["plugin-to-remove", "keep-me"]
    });
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&initial_config).unwrap(),
    )
    .unwrap();

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["plugin", "remove", "plugin-to-remove"]);
    assert!(
        output.status.success(),
        "plugin remove should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config_content = fs::read_to_string(&config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    let plugins = config.get("plugin").and_then(|p| p.as_array()).unwrap();
    assert!(
        !plugins.contains(&serde_json::json!("plugin-to-remove")),
        "plugin list should not contain 'plugin-to-remove' after removal, got: {:?}",
        plugins
    );
    assert!(
        plugins.contains(&serde_json::json!("keep-me")),
        "plugin list should still contain 'keep-me', got: {:?}",
        plugins
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}

#[test]
fn test_plugin_search_filters_by_query() {
    let harness = TestHarness::setup();
    let plugins_dir = harness.temp_dir.path().join(".opencode-rs/plugins");
    fs::create_dir_all(&plugins_dir).unwrap();

    let plugin1_dir = plugins_dir.join("searchable-plugin");
    fs::create_dir_all(&plugin1_dir).unwrap();
    let plugin1_meta = serde_json::json!({
        "name": "searchable-plugin",
        "version": "1.0.0",
        "description": "A plugin that is searchable"
    });
    fs::write(
        plugin1_dir.join("plugin.json"),
        serde_json::to_string_pretty(&plugin1_meta).unwrap(),
    )
    .unwrap();

    let plugin2_dir = plugins_dir.join("other-plugin");
    fs::create_dir_all(&plugin2_dir).unwrap();
    let plugin2_meta = serde_json::json!({
        "name": "other-plugin",
        "version": "2.0.0",
        "description": "Another plugin"
    });
    fs::write(
        plugin2_dir.join("plugin.json"),
        serde_json::to_string_pretty(&plugin2_meta).unwrap(),
    )
    .unwrap();

    let output = harness.run_cli(&["plugin", "search", "searchable"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("searchable-plugin"),
        "search should find 'searchable-plugin', got: {}",
        stdout
    );
    assert!(
        !stdout.contains("other-plugin"),
        "search for 'searchable' should not include 'other-plugin', got: {}",
        stdout
    );
}

#[test]
fn test_plugin_search_with_no_query_returns_all_discovered() {
    let harness = TestHarness::setup();
    let plugins_dir = harness.temp_dir.path().join(".opencode-rs/plugins");
    fs::create_dir_all(&plugins_dir).unwrap();

    let plugin1_dir = plugins_dir.join("plugin-one");
    fs::create_dir_all(&plugin1_dir).unwrap();
    let plugin1_meta = serde_json::json!({
        "name": "plugin-one",
        "version": "1.0.0",
        "description": "First plugin"
    });
    fs::write(
        plugin1_dir.join("plugin.json"),
        serde_json::to_string_pretty(&plugin1_meta).unwrap(),
    )
    .unwrap();

    let plugin2_dir = plugins_dir.join("plugin-two");
    fs::create_dir_all(&plugin2_dir).unwrap();
    let plugin2_meta = serde_json::json!({
        "name": "plugin-two",
        "version": "2.0.0",
        "description": "Second plugin"
    });
    fs::write(
        plugin2_dir.join("plugin.json"),
        serde_json::to_string_pretty(&plugin2_meta).unwrap(),
    )
    .unwrap();

    let output = harness.run_cli(&["plugin", "search"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("plugin-one"),
        "search should find 'plugin-one', got: {}",
        stdout
    );
    assert!(
        stdout.contains("plugin-two"),
        "search should find 'plugin-two', got: {}",
        stdout
    );
}

#[test]
fn test_plugin_install_already_installed_fails() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    let initial_config = serde_json::json!({
        "plugin": ["existing-plugin"]
    });
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&initial_config).unwrap(),
    )
    .unwrap();

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["plugin", "install", "existing-plugin"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success() || stdout.contains("already installed"),
        "installing already installed plugin should fail or show message. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}

#[test]
fn test_plugin_remove_nonexistent_fails() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, "{}").unwrap();

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["plugin", "remove", "nonexistent-plugin"]);
    assert!(
        !output.status.success(),
        "removing nonexistent plugin should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed"),
        "error message should mention 'not installed', got: {}",
        stderr
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}
