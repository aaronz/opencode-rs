mod common;

use common::TestHarness;

#[test]
fn test_run_prompt_mode_returns_structured_output() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["run", "--prompt", "hello parity", "--model", "gpt-4o"]);

    assert!(output.status.success(), "run should succeed in prompt mode");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Prompt:"),
        "stdout should include prompt echo"
    );
    assert!(
        stdout.contains("Model: gpt-4o"),
        "stdout should include selected model"
    );
    assert!(
        stdout.contains("Mode: non-interactive"),
        "stdout should include execution mode"
    );
}
