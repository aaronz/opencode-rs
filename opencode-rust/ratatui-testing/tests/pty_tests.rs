use ratatui_testing::PtySimulator;
use std::time::Duration;

#[test]
fn test_pty_simulator_new_creates_valid_pty_pair() {
    let result = PtySimulator::new(&["echo", "hello"]);
    assert!(
        result.is_ok(),
        "PtySimulator::new should create a valid PTY pair"
    );

    let pty = result.unwrap();
    assert!(pty.master.is_some(), "Master PTY should be Some");
    assert!(pty.child.is_some(), "Child process should be Some");
    assert!(pty.writer.is_some(), "Writer should be Some");
    assert!(pty.reader.is_some(), "Reader should be Some");
}

#[test]
fn test_pty_simulator_child_process_is_spawned_and_running() {
    let result = PtySimulator::new(&["sleep", "10"]);
    assert!(
        result.is_ok(),
        "PtySimulator::new should succeed with sleep command"
    );

    let pty = result.unwrap();
    assert!(pty.is_child_running(), "Child process should be running");

    drop(pty);
}

#[test]
fn test_pty_simulator_writer_and_reader_properly_initialized() {
    let result = PtySimulator::new(&["cat"]);
    assert!(
        result.is_ok(),
        "PtySimulator::new should succeed with cat command"
    );

    let mut pty = result.unwrap();
    assert!(
        pty.writer.is_some(),
        "Writer should be properly initialized"
    );
    assert!(
        pty.reader.is_some(),
        "Reader should be properly initialized"
    );

    let write_result = pty.write_input("test input\n");
    assert!(write_result.is_ok(), "Writing to PTY should succeed");
}

#[test]
fn test_pty_simulator_error_when_command_is_invalid() {
    let result = PtySimulator::new(&["nonexistent_command_12345"]);
    assert!(
        result.is_err(),
        "PtySimulator::new should fail with invalid command"
    );
}
