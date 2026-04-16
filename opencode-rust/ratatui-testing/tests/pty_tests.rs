use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui_testing::PtySimulator;
use std::time::Duration;

#[test]
fn test_pty_simulator_new_without_arguments() {
    let result = PtySimulator::new();
    assert!(
        result.is_ok(),
        "PtySimulator::new() without arguments should create a valid PTY"
    );

    let pty = result.unwrap();
    assert!(pty.master.is_some(), "Master PTY should be Some");
    assert!(pty.child.borrow().is_some(), "Child process should be Some");
    assert!(pty.writer.is_some(), "Writer should be Some");
    assert!(pty.reader.is_some(), "Reader should be Some");
}

#[test]
fn test_pty_simulator_new_creates_valid_pty_pair() {
    let result = PtySimulator::new_with_command(&["echo", "hello"]);
    assert!(
        result.is_ok(),
        "PtySimulator::new should create a valid PTY pair"
    );

    let pty = result.unwrap();
    assert!(pty.master.is_some(), "Master PTY should be Some");
    assert!(pty.child.borrow().is_some(), "Child process should be Some");
    assert!(pty.writer.is_some(), "Writer should be Some");
    assert!(pty.reader.is_some(), "Reader should be Some");
}

#[test]
fn test_pty_simulator_child_process_is_spawned_and_running() {
    let result = PtySimulator::new_with_command(&["sleep", "10"]);
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
    let result = PtySimulator::new_with_command(&["cat"]);
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

    drop(pty);
}

#[test]
fn test_pty_simulator_error_when_command_is_invalid() {
    let result = PtySimulator::new_with_command(&["nonexistent_command_12345"]);
    assert!(
        result.is_err(),
        "PtySimulator::new should fail with invalid command"
    );
}

#[test]
fn test_key_event_injection_works() {
    let result = PtySimulator::new_with_command(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
    let inject_result = pty.inject_key_event(key_event);
    assert!(inject_result.is_ok(), "KeyEvent injection should succeed");

    drop(pty);
}

#[test]
fn test_key_event_injection_with_modifiers() {
    let result = PtySimulator::new_with_command(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let inject_result = pty.inject_key_event(ctrl_c);
    assert!(
        inject_result.is_ok(),
        "Ctrl+C KeyEvent injection should succeed"
    );

    drop(pty);
}

#[test]
fn test_mouse_event_injection_works() {
    let result = PtySimulator::new_with_command(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    let inject_result = pty.inject_mouse_event(mouse_event);
    assert!(inject_result.is_ok(), "MouseEvent injection should succeed");

    drop(pty);
}

#[test]
fn test_mouse_event_injection_scroll() {
    let result = PtySimulator::new_with_command(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let scroll_event = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 15,
        row: 20,
        modifiers: KeyModifiers::NONE,
    };
    let inject_result = pty.inject_mouse_event(scroll_event);
    assert!(
        inject_result.is_ok(),
        "Scroll mouse event injection should succeed"
    );

    drop(pty);
}

#[test]
fn test_key_event_injection_reaches_terminal() {
    let result = PtySimulator::new_with_command(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    let inject_result = pty.inject_key_event(key_event);
    assert!(inject_result.is_ok(), "KeyEvent injection should succeed");

    std::thread::sleep(Duration::from_millis(100));

    let output = pty.read_output(Duration::from_millis(200));
    assert!(output.is_ok(), "Reading output should succeed");
    let output_str = output.unwrap();
    assert!(
        !output_str.is_empty(),
        "Injected event should reach the terminal, got empty output"
    );
    assert!(
        output_str.contains("120;0u") || output_str.contains("120"),
        "Injected event should contain CSI-u encoded key code (120 for 'x'), got: {:?}",
        output_str
    );

    drop(pty);
}

#[test]
fn test_multiple_key_events_injection() {
    let result = PtySimulator::new_with_command(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    for ch in ['a', 'b', 'c'] {
        let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        let inject_result = pty.inject_key_event(key_event);
        assert!(
            inject_result.is_ok(),
            "KeyEvent injection for '{}' should succeed",
            ch
        );
    }

    drop(pty);
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_new_returns_error() {
    let result = PtySimulator::new();
    assert!(
        result.is_err(),
        "PtySimulator::new() should return error on Windows"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("PTY not supported on Windows"),
        "Error message should contain 'PTY not supported on Windows', got: {}",
        err
    );
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_new_with_command_returns_error() {
    let result = PtySimulator::new_with_command(&["echo", "hello"]);
    assert!(
        result.is_err(),
        "PtySimulator::new_with_command() should return error on Windows"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("PTY not supported on Windows"),
        "Error message should contain 'PTY not supported on Windows', got: {}",
        err
    );
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_write_input_returns_error() {
    let mut pty = PtySimulator::default();
    let result = pty.write_input("test");
    assert!(
        result.is_err(),
        "write_input() should return error on Windows"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("PTY not supported on Windows"),
        "Error message should contain 'PTY not supported on Windows', got: {}",
        err
    );
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_read_output_returns_error() {
    let mut pty = PtySimulator::default();
    let result = pty.read_output(Duration::from_secs(1));
    assert!(
        result.is_err(),
        "read_output() should return error on Windows"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("PTY not supported on Windows"),
        "Error message should contain 'PTY not supported on Windows', got: {}",
        err
    );
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_resize_returns_error() {
    let mut pty = PtySimulator::default();
    let result = pty.resize(80, 24);
    assert!(result.is_err(), "resize() should return error on Windows");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("PTY not supported on Windows"),
        "Error message should contain 'PTY not supported on Windows', got: {}",
        err
    );
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_inject_key_event_returns_error() {
    let mut pty = PtySimulator::default();
    let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    let result = pty.inject_key_event(key_event);
    assert!(
        result.is_err(),
        "inject_key_event() should return error on Windows"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("PTY not supported on Windows"),
        "Error message should contain 'PTY not supported on Windows', got: {}",
        err
    );
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_inject_mouse_event_returns_error() {
    let mut pty = PtySimulator::default();
    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    let result = pty.inject_mouse_event(mouse_event);
    assert!(
        result.is_err(),
        "inject_mouse_event() should return error on Windows"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("PTY not supported on Windows"),
        "Error message should contain 'PTY not supported on Windows', got: {}",
        err
    );
}

#[cfg(windows)]
#[test]
fn test_pty_simulator_windows_is_child_running_returns_false() {
    let pty = PtySimulator::default();
    assert!(
        !pty.is_child_running(),
        "is_child_running() should return false on Windows"
    );
}

#[cfg(unix)]
#[test]
fn test_pty_simulator_unix_new_works_normally() {
    let result = PtySimulator::new();
    assert!(
        result.is_ok(),
        "PtySimulator::new() should work on Unix platforms"
    );
}

#[cfg(unix)]
#[test]
fn test_pty_simulator_unix_new_with_command_works_normally() {
    let result = PtySimulator::new_with_command(&["echo", "test"]);
    assert!(
        result.is_ok(),
        "PtySimulator::new_with_command() should work on Unix platforms"
    );
}

#[cfg(unix)]
#[test]
fn test_pty_simulator_unix_is_child_running_returns_correct_value() {
    let pty = PtySimulator::new_with_command(&["sleep", "10"]).unwrap();
    assert!(
        pty.is_child_running(),
        "is_child_running() should return true for running child on Unix"
    );
    drop(pty);
}
