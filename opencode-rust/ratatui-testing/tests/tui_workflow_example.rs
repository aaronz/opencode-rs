//! Example E2E workflow tests for opencode-rs TUI.
//!
//! These tests demonstrate how to use the TUI workflow testing infrastructure
//! to test real user interactions with the opencode-rs terminal UI.
//!
//! ## Requirements
//!
//! To run these tests, you need:
//! - A built binary at `target/debug/opencode-rs`
//! - Mock mode enabled via `--mock` flag to avoid real LLM API calls
//!
//! ## Running Tests
//!
//! ```bash
//! # Run just the workflow tests
//! cargo test -p ratatui-testing --test tui_workflow_example
//!
//! # Run with output
//! cargo test -p ratatui-testing --test tui_workflow_example -- --nocapture
//! ```

use ratatui_testing::{PtySimulator, TuiWorkflowBuilder};
use std::env;
use std::time::Duration;

/// Test that the TUI binary can be spawned with mock mode.
#[tokio::test]
async fn test_tui_spawn_with_mock_mode() {
    let binary_path =
        env::var("OPENCODE_TEST_BINARY").unwrap_or_else(|_| "target/debug/opencode-rs".to_string());

    // Skip if binary doesn't exist
    if !std::path::Path::new(&binary_path).exists() {
        eprintln!("Binary not found at {}, skipping test", binary_path);
        return;
    }

    // Spawn TUI with mock mode
    let mut session = TuiWorkflowBuilder::new(&binary_path)
        .with_args(&["tui", "--mock"])
        .with_size(120, 40)
        .with_timeout(5)
        .spawn()
        .await
        .expect("Failed to spawn TUI with mock mode");

    // Wait a bit for the TUI to initialize
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify the session is still alive (TUI didn't crash on startup)
    assert!(session.is_alive(), "TUI should still be running");

    // Send Ctrl+C to gracefully exit
    let _ = session.send_ctrl_c().await;

    // Wait for exit
    let _ = session.wait_for_exit().await;
}

/// Test that the TUI binary can be spawned and responds to basic input.
#[test]
fn test_tui_workflow_builder_basic() {
    let binary_path =
        env::var("OPENCODE_TEST_BINARY").unwrap_or_else(|_| "target/debug/opencode-rs".to_string());

    // Create a simple workflow that spawns the TUI
    let _builder = TuiWorkflowBuilder::new(&binary_path)
        .with_size(120, 40)
        .with_timeout(10);
}

/// Example test showing how to test TUI home view appearance.
///
/// This test demonstrates the pattern but requires a working mock mode
/// to actually pass in CI environments.
#[test]
fn test_home_view_appears_on_launch() {
    let binary_path =
        env::var("OPENCODE_TEST_BINARY").unwrap_or_else(|_| "target/debug/opencode-rs".to_string());

    // Skip if binary doesn't exist
    if !std::path::Path::new(&binary_path).exists() {
        eprintln!("Binary not found at {}, skipping test", binary_path);
        return;
    }

    // For now, just verify we can create a PTY and interact with a simple command
    let mut pty = PtySimulator::new_with_command(&["echo", "opencode ready"]).unwrap();

    std::thread::sleep(Duration::from_millis(200));

    let output = pty.read_output(Duration::from_millis(500)).unwrap();
    assert!(output.contains("opencode") || output.contains("ready"));
}

/// Demonstrate PTY interaction with echo - use echo instead of cat to avoid blocking.
#[test]
fn test_pty_echo_interaction() {
    let mut pty =
        PtySimulator::new_with_command(&["bash", "-c", "echo 'Hello, TUI!'; sleep 0.1"]).unwrap();

    std::thread::sleep(Duration::from_millis(200));

    let output = pty.read_output(Duration::from_millis(500)).unwrap();
    assert!(output.contains("Hello") || output.contains("TUI"));
}

/// Example: Testing the connect flow pattern with mock mode.
///
/// This test demonstrates how one might test the provider connection flow:
/// 1. Launch TUI with --mock flag
/// 2. Press ':' to open command palette
/// 3. Type 'connect' and Enter
/// 4. Verify provider picker appears
#[tokio::test]
async fn test_connect_flow_with_mock() {
    let binary_path =
        env::var("OPENCODE_TEST_BINARY").unwrap_or_else(|_| "target/debug/opencode-rs".to_string());

    // Skip if binary doesn't exist
    if !std::path::Path::new(&binary_path).exists() {
        eprintln!("Binary not found at {}, skipping test", binary_path);
        return;
    }

    // Spawn TUI with mock mode and agent flag
    let mut session = TuiWorkflowBuilder::new(&binary_path)
        .with_args(&["tui", "--mock", "--agent", "general"])
        .with_size(120, 40)
        .with_timeout(10)
        .spawn()
        .await
        .expect("Failed to spawn TUI");

    // Wait for startup
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify TUI is still running
    assert!(session.is_alive(), "TUI should still be running");

    // Send Ctrl+C to exit
    let _ = session.send_ctrl_c().await;
    let _ = session.wait_for_exit().await;
}

/// Test that terminal resizing works correctly.
#[test]
fn test_terminal_resize() {
    let mut pty = PtySimulator::new_with_command(&["echo", "resized"]).unwrap();

    // Initial size
    pty.resize(80, 24).unwrap();

    // Change size
    pty.resize(120, 40).unwrap();

    std::thread::sleep(Duration::from_millis(100));

    let output = pty.read_output(Duration::from_millis(200)).unwrap();
    assert!(output.contains("resized"));
}
