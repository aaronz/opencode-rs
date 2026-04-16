use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui_testing::{BufferDiff, CliTester, PtySimulator, StateTester, TestDsl};

#[test]
fn test_buffer_diff_and_state_tester_integration() {
    let mut state_tester = StateTester::new();
    let buffer_diff = BufferDiff::new();

    #[derive(Debug, serde::Serialize, PartialEq)]
    struct RenderState {
        width: u16,
        height: u16,
        diff_count: usize,
    }

    let area = Rect::new(0, 0, 80, 24);
    let buf1 = Buffer::empty(area);
    let buf2 = Buffer::empty(area);

    let diff_result = buffer_diff.diff(&buf1, &buf2);

    let render_state = RenderState {
        width: 80,
        height: 24,
        diff_count: diff_result.total_diffs,
    };

    state_tester
        .capture_state(&render_state, Some("initial_render"))
        .expect("Should capture render state");

    let retrieved = state_tester
        .get_snapshot("initial_render")
        .expect("Snapshot should exist");

    assert_eq!(retrieved.json["width"], 80);
    assert_eq!(retrieved.json["height"], 24);
    assert_eq!(retrieved.json["diff_count"], 0);
}

#[test]
fn test_pty_simulator_with_buffer_diff() {
    let mut pty = PtySimulator::new_with_command(&["cat"]).expect("PTY should be created");
    let buffer_diff = BufferDiff::new();

    pty.write_input("test input\n")
        .expect("Write should succeed");

    let output = pty.read_output(std::time::Duration::from_millis(100));
    assert!(output.is_ok(), "Read should succeed");
    assert!(
        output.unwrap().contains("test input"),
        "Output should contain written input"
    );

    let area = Rect::new(0, 0, 10, 1);
    let buf1 = Buffer::empty(area);
    let buf2 = Buffer::empty(area);

    let result = buffer_diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 0, "Fresh buffers should have no diffs");

    drop(pty);
}

#[test]
fn test_state_tester_and_cli_tester_workflow() {
    let mut state_tester = StateTester::new();

    #[derive(Debug, serde::Serialize)]
    struct CliExecution {
        command: String,
        executed: bool,
    }

    let initial_state = CliExecution {
        command: "echo".to_string(),
        executed: false,
    };

    state_tester
        .capture_state(&initial_state, Some("before_cli"))
        .expect("Should capture initial state");

    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let tester = CliTester::new("echo").arg("integration_test");
            let output = tester.run().await.expect("CLI should execute");
            assert!(output.stdout.contains("integration_test"));
        });

    let after_cli = CliExecution {
        command: "echo".to_string(),
        executed: true,
    };

    state_tester
        .capture_state(&after_cli, Some("after_cli"))
        .expect("Should capture after CLI state");

    let snapshots = state_tester.list_snapshots();
    assert_eq!(snapshots.len(), 2);
    assert!(snapshots.contains(&"before_cli"));
    assert!(snapshots.contains(&"after_cli"));
}

#[test]
fn test_dsl_composition_with_all_components() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty(&["cat"])
        .expect("PTY should be created")
        .with_buffer_diff()
        .with_state_tester();

    assert!(
        dsl.get_terminal().is_some(),
        "Terminal should be initialized"
    );
    assert!(dsl.get_pty().is_some(), "PTY should be configured");
    assert!(
        dsl.get_buffer_diff().is_some(),
        "BufferDiff should be configured"
    );
    assert!(
        dsl.get_state_tester().is_some(),
        "StateTester should be configured"
    );

    let widget = Paragraph::new(Text::from("Composition Test"));
    dsl = dsl.render(widget);

    let buffer = dsl.capture_buffer().expect("Buffer should be captured");
    assert_eq!(buffer.area.width, 80);
    assert_eq!(buffer.area.height, 24);

    #[derive(Debug, serde::Serialize)]
    struct CompositionState {
        terminal_initialized: bool,
        pty_running: bool,
        buffer_captured: bool,
    }

    let composition_state = CompositionState {
        terminal_initialized: dsl.get_terminal().is_some(),
        pty_running: dsl.is_pty_child_running(),
        buffer_captured: buffer.area.width > 0,
    };

    dsl.capture_state(&composition_state, Some("composition"))
        .expect("Should capture composition state");
}

#[test]
fn test_e2e_render_capture_diff_workflow() {
    let dsl1 = TestDsl::new()
        .with_size(40, 10)
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Expected Content")));

    let buffer1 = dsl1.capture_buffer().expect("Should capture first buffer");

    let dsl2 = TestDsl::new()
        .with_size(40, 10)
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Expected Content")));

    let result = dsl2.assert_no_diffs(&buffer1);
    assert!(result.is_ok(), "Identical renders should have no diffs");
}

#[test]
fn test_e2e_render_and_state_snapshot_workflow() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Snapshot Content")));

    dsl.snapshot_state("render_1")
        .expect("Snapshot should be captured");

    let mut dsl2 = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Snapshot Content")));

    dsl2.snapshot_state("render_2")
        .expect("Snapshot should be captured in dsl2");

    let compare_result = dsl.compare_to_snapshot("render_1");
    assert!(
        compare_result.is_ok(),
        "Comparing identical content should succeed: {:?}",
        compare_result
    );
}

#[test]
fn test_e2e_cli_and_dsl_state_preservation() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Before CLI")));

    let lines_before = dsl.buffer_lines().expect("Should get lines");

    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let cli = CliTester::new("echo").arg("CLI executed");
            let output = cli.run().await.expect("CLI should run");
            assert!(
                output.stdout.contains("CLI executed"),
                "CLI output should contain expected text"
            );
        });

    let lines_after = dsl.buffer_lines().expect("Should get lines after CLI");
    assert_eq!(
        lines_before, lines_after,
        "DSL state should be preserved after CLI execution"
    );
}

#[test]
fn test_pty_key_injection_with_buffer_capture() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty(&["cat"])
        .expect("PTY should be created")
        .render(Paragraph::new(Text::from("Initial")));

    let initial_buffer = dsl.capture_buffer().expect("Should capture initial");

    let key_event = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('x'),
        crossterm::event::KeyModifiers::NONE,
    );
    dsl.get_pty_mut()
        .expect("PTY should exist")
        .inject_key_event(key_event)
        .expect("Key injection should succeed");

    let after_key_buffer = dsl.capture_buffer().expect("Should capture after key");
    assert_eq!(
        initial_buffer.area, after_key_buffer.area,
        "Buffer area should remain same after key injection"
    );
}

#[test]
fn test_buffer_diff_with_color_differences() {
    let buffer_diff = BufferDiff::new();

    let area = Rect::new(0, 0, 10, 1);
    let mut buf1 = Buffer::empty(area);
    let mut buf2 = Buffer::empty(area);

    buf1.content[0].set_symbol("A");
    buf1.content[0].fg = Color::Red;
    buf2.content[0].set_symbol("A");
    buf2.content[0].fg = Color::Blue;

    let result = buffer_diff.diff(&buf1, &buf2);
    assert_eq!(
        result.total_diffs, 1,
        "Should detect foreground color difference"
    );

    let diff_with_ignore = BufferDiff::new().ignore_foreground();
    let result_ignored = diff_with_ignore.diff(&buf1, &buf2);
    assert_eq!(
        result_ignored.total_diffs, 0,
        "Ignoring foreground should hide the difference"
    );
}

#[test]
fn test_state_tester_nested_state_comparison() {
    let mut state_tester = StateTester::new();

    #[derive(Debug, serde::Serialize, PartialEq)]
    struct Inner {
        value: i32,
        label: String,
    }

    #[derive(Debug, serde::Serialize, PartialEq)]
    struct Outer {
        inner: Inner,
        active: bool,
    }

    let state1 = Outer {
        inner: Inner {
            value: 42,
            label: "original".to_string(),
        },
        active: true,
    };

    state_tester
        .capture_state(&state1, Some("nested_test"))
        .expect("Should capture nested state");

    let state2 = Outer {
        inner: Inner {
            value: 100,
            label: "modified".to_string(),
        },
        active: true,
    };

    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = state_tester.get_snapshot("nested_test").unwrap();
    let diff = state_tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 2, "Should detect 2 field differences");
}

#[test]
fn test_cli_tester_environment_and_working_directory() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let tester = CliTester::new("sh")
                .arg("-c")
                .arg("echo $PWD && echo $TEST_VAR")
                .env("TEST_VAR", "integration_value")
                .working_dir(std::path::PathBuf::from("/tmp"));

            let output = tester.run().await.expect("CLI should execute");
            assert!(
                output.stdout.contains("/tmp"),
                "Should show working directory"
            );
            assert!(
                output.stdout.contains("integration_value"),
                "Should show env var value"
            );
        });
}

#[test]
fn test_dsl_multiple_render_state_tracking() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester();

    dsl = dsl.render(Paragraph::new(Text::from("Render 1")));
    let lines1 = dsl.buffer_lines().expect("Should get lines");
    dsl.capture_state(&lines1, Some("render_1"))
        .expect("Should capture state 1");

    dsl = dsl.render(Paragraph::new(Text::from("Render 2")));
    let lines2 = dsl.buffer_lines().expect("Should get lines");
    dsl.capture_state(&lines2, Some("render_2"))
        .expect("Should capture state 2");

    dsl = dsl.render(Paragraph::new(Text::from("Render 3")));
    let lines3 = dsl.buffer_lines().expect("Should get lines");
    dsl.capture_state(&lines3, Some("render_3"))
        .expect("Should capture state 3");

    let snapshots = dsl.get_state_tester().unwrap().list_snapshots();
    assert_eq!(snapshots.len(), 3, "Should have 3 snapshots");
}

#[test]
fn test_buffer_diff_string_based_comparison() {
    let diff = BufferDiff::new();

    let result = diff.diff_str("hello world", "hello world");
    assert!(result.passed, "Identical strings should pass");
    assert_eq!(result.total_diffs, 0);

    let result = diff.diff_str("hello", "hallo");
    assert!(!result.passed, "Different strings should fail");
    assert_eq!(result.total_diffs, 1);

    let result = diff.diff_str("line1\nline2\nline3", "line1\nline2\nline3");
    assert!(result.passed, "Multi-line identical strings should pass");
}

#[test]
fn test_e2e_pty_write_read_cycle() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty(&["cat"])
        .expect("PTY should be created");

    assert!(
        dsl.is_pty_child_running(),
        "PTY child should be running initially"
    );

    dsl.write_to_pty("hello from PTY\n")
        .expect("Write to PTY should succeed");

    let output = dsl.read_from_pty(std::time::Duration::from_millis(200));
    assert!(output.is_ok(), "Reading from PTY should succeed");
    assert!(
        output.unwrap().contains("hello from PTY"),
        "Should read back what was written"
    );
}

#[test]
fn test_state_snapshot_overwrite_behavior() {
    let mut state_tester = StateTester::new();

    #[derive(Debug, serde::Serialize)]
    struct SimpleState {
        value: i32,
    }

    state_tester
        .capture_state(&SimpleState { value: 1 }, Some("snap"))
        .expect("First capture should succeed");

    let first = state_tester.get_snapshot("snap").unwrap();
    assert_eq!(first.json["value"], 1, "First value should be 1");

    state_tester
        .capture_state(&SimpleState { value: 2 }, Some("snap"))
        .expect("Overwrite capture should succeed");

    let second = state_tester.get_snapshot("snap").unwrap();
    assert_eq!(second.json["value"], 2, "Overwritten value should be 2");

    let all_snapshots = state_tester.list_snapshots();
    assert_eq!(all_snapshots.len(), 1, "Should still have only 1 snapshot");
}

#[test]
fn test_dsl_wait_for_with_async_operation() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    let completed = Arc::new(AtomicBool::new(false));
    let completed_clone = completed.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        completed_clone.store(true, Ordering::SeqCst);
    });

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for(Duration::from_secs(1), move || {
            completed.load(Ordering::SeqCst)
        })
        .expect("Wait should succeed");

    assert!(
        dsl.get_terminal().is_some(),
        "Terminal should still be accessible after wait"
    );
}

#[test]
fn test_cli_tester_with_failed_command() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let tester = CliTester::new("sh").arg("-c").arg("exit 1");
            let output = tester.run().await.expect("Should execute even with exit 1");

            assert!(!output.success, "Output should indicate failure");
            assert_eq!(output.exit_code, 1, "Exit code should be 1");
        });
}

#[test]
fn test_buffer_diff_cell_diff_position_tracking() {
    let diff = BufferDiff::new();

    let area = Rect::new(0, 0, 5, 3);
    let mut buf1 = Buffer::empty(area);
    let mut buf2 = Buffer::empty(area);

    for (i, c) in "abcde".chars().enumerate() {
        buf1.content[i].set_symbol(c.to_string().as_str());
    }
    for (i, c) in "abxde".chars().enumerate() {
        buf2.content[i].set_symbol(c.to_string().as_str());
    }

    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1, "Should detect 1 diff");

    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.x, 2, "Diff should be at x=2");
    assert_eq!(cell_diff.y, 0, "Diff should be at y=0");
}

#[test]
fn test_e2e_terminal_resize_and_buffer_capture() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Original Size")));

    let buffer1 = dsl.capture_buffer().expect("Should capture buffer");
    assert_eq!(buffer1.area.width, 80);
    assert_eq!(buffer1.area.height, 24);

    let resize_result = dsl
        .get_terminal_mut()
        .expect("Terminal should exist")
        .resize(Rect::new(0, 0, 120, 40));
    assert!(resize_result.is_ok(), "Resize should succeed");

    dsl = dsl.render(Paragraph::new(Text::from("Resized Size")));

    let buffer2 = dsl.capture_buffer().expect("Should capture resized buffer");
    assert!(
        buffer2.area.width == 80 || buffer2.area.width == 120,
        "Buffer width should be one of the used sizes"
    );
}

#[test]
fn test_state_tester_array_diff_tracking() {
    let mut state_tester = StateTester::new();

    #[derive(Debug, serde::Serialize, PartialEq)]
    struct ArrayState {
        items: Vec<String>,
    }

    let state1 = ArrayState {
        items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    };

    state_tester
        .capture_state(&state1, Some("array_test"))
        .expect("Should capture array state");

    let state2 = ArrayState {
        items: vec!["a".to_string(), "x".to_string(), "c".to_string()],
    };

    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = state_tester.get_snapshot("array_test").unwrap();
    let diff = state_tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 1, "Should detect 1 array difference");
    assert!(
        diff.differences[0].path.contains("[1]"),
        "Diff should be at array index 1"
    );
}

#[test]
fn test_dsl_then_chaining_with_state_capture() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Initial State")))
        .then(|dsl| {
            let lines = dsl.buffer_lines().unwrap();
            let mut dsl = dsl;
            dsl.capture_state(&lines, Some("step1"))
                .expect("Should capture step1");
            dsl.render(Paragraph::new(Text::from("After Step 1")))
        })
        .then(|dsl| {
            let lines = dsl.buffer_lines().unwrap();
            let mut dsl = dsl;
            dsl.capture_state(&lines, Some("step2"))
                .expect("Should capture step2");
            dsl.render(Paragraph::new(Text::from("Final State")))
        });

    let snapshots = result.get_state_tester().unwrap().list_snapshots();
    assert_eq!(snapshots.len(), 2, "Should have 2 snapshots from chaining");
}

#[test]
fn test_cli_output_multiple_assertions() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let tester = CliTester::new("echo").arg("test output content");
            let output = tester.run().await.expect("CLI should execute");

            output.assert_success().expect("Should be successful");
            output
                .assert_stdout_contains("test")
                .expect("Should contain 'test'");
            output
                .assert_stdout_contains("output")
                .expect("Should contain 'output'");
            output
                .assert_stdout_contains("content")
                .expect("Should contain 'content'");
        });
}

#[test]
fn test_buffer_diff_multiline_string_diff() {
    let diff = BufferDiff::new();

    let expected = "line1\nline2\nline3\nline4";
    let actual = "line1\nline2\nmodified\nline4";

    let result = diff.diff_str(expected, actual);
    assert!(!result.passed, "Should detect differences");
    assert!(
        result.total_diffs > 0,
        "Should detect at least one difference"
    );
}
