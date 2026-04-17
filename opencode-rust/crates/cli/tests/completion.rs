mod common;

use common::TestHarness;

#[test]
fn test_bash_completion_generates_valid_script() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["completion", "bash"]);

    assert!(output.status.success(), "completion bash should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("# opencode-rs bash completion"),
        "Should contain bash completion header"
    );
    assert!(
        stdout.contains("_opencode_rs()"),
        "Should contain completion function"
    );
    assert!(
        stdout.contains("complete -F _opencode_rs opencode-rs"),
        "Should register completion for opencode-rs"
    );
}

#[test]
fn test_bash_completion_includes_all_cli_commands() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["completion", "bash"]);

    assert!(output.status.success(), "completion bash should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let expected_commands = [
        "run", "serve", "desktop", "account", "config", "agent", "bash",
        "models", "providers", "mcp", "session", "list", "stats", "terminal",
        "db", "github", "gitlab", "pr", "export", "import", "generate",
        "web", "thread", "attach", "uninstall", "upgrade", "debug", "acp",
        "workspace-serve", "palette", "shortcuts", "workspace", "ui",
        "project", "files", "prompt", "quick", "tui", "completion",
    ];

    for cmd in expected_commands {
        assert!(
            stdout.contains(cmd),
            "Bash completion should contain command: {}",
            cmd
        );
    }
}

#[test]
fn test_zsh_completion_generates_valid_script() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["completion", "zsh"]);

    assert!(output.status.success(), "completion zsh should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("#compdef opencode-rs"),
        "Should contain zsh compdef header"
    );
    assert!(
        stdout.contains("_opencode_rs()"),
        "Should contain completion function"
    );
}

#[test]
fn test_fish_completion_generates_valid_script() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["completion", "fish"]);

    assert!(output.status.success(), "completion fish should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("# opencode-rs fish completion"),
        "Should contain fish completion header"
    );
    assert!(
        stdout.contains("complete -c opencode-rs"),
        "Should register fish completion"
    );
}

#[test]
fn test_powershell_completion_generates_valid_script() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["completion", "powershell"]);

    assert!(output.status.success(), "completion powershell should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("# opencode-rs PowerShell completion"),
        "Should contain PowerShell completion header"
    );
    assert!(
        stdout.contains("$script:OpenCodeCommands"),
        "Should contain PowerShell variable"
    );
    assert!(
        stdout.contains("Register-ArgumentCompleter"),
        "Should register argument completer"
    );
}

#[test]
fn test_completion_bash_is_default() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["completion"]);

    assert!(output.status.success(), "completion without shell arg should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("# opencode-rs bash completion"),
        "Default shell should be bash"
    );
}