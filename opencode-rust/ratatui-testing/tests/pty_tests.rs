use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui_testing::PtySimulator;

#[test]
fn test_pty_simulator_new_creates_valid_pty_pair() {
    let result = PtySimulator::new(&["echo", "hello"]);
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

#[test]
fn test_key_event_injection_works() {
    let result = PtySimulator::new(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
    let inject_result = pty.inject_key_event(key_event);
    assert!(inject_result.is_ok(), "KeyEvent injection should succeed");

    drop(pty);
}

#[test]
fn test_key_event_injection_with_modifiers() {
    let result = PtySimulator::new(&["cat"]);
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
    let result = PtySimulator::new(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let mouse_event = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Left),
        10,
        5,
        KeyModifiers::NONE,
    );
    let inject_result = pty.inject_mouse_event(mouse_event);
    assert!(inject_result.is_ok(), "MouseEvent injection should succeed");

    drop(pty);
}

#[test]
fn test_mouse_event_injection_scroll() {
    let result = PtySimulator::new(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let scroll_event = MouseEvent::new(MouseEventKind::ScrollDown, 15, 20, KeyModifiers::NONE);
    let inject_result = pty.inject_mouse_event(scroll_event);
    assert!(
        inject_result.is_ok(),
        "Scroll mouse event injection should succeed"
    );

    drop(pty);
}

#[test]
fn test_key_event_injection_reaches_terminal() {
    let result = PtySimulator::new(&["cat"]);
    assert!(result.is_ok(), "PtySimulator::new should succeed");

    let mut pty = result.unwrap();

    let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    let inject_result = pty.inject_key_event(key_event);
    assert!(inject_result.is_ok(), "KeyEvent injection should succeed");

    std::thread::sleep(std::time::Duration::from_millis(50));

    let output = pty.read_output(std::time::Duration::from_millis(100));
    assert!(output.is_ok(), "Reading output should succeed");
    let output_str = output.unwrap();
    assert!(
        output_str.contains('x'),
        "Injected character 'x' should reach the terminal, got: {:?}",
        output_str
    );

    drop(pty);
}

#[test]
fn test_multiple_key_events_injection() {
    let result = PtySimulator::new(&["cat"]);
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
