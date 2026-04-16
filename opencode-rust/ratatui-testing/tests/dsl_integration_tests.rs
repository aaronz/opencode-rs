use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui_testing::{BufferDiff, CliTester, PtySimulator, StateTester, TestDsl, WaitPredicate};

#[test]
fn test_dsl_complete_scenario_with_pty_and_terminal() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty(&["cat"])
        .expect("PTY creation should succeed")
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

    let widget = Paragraph::new(Text::from("Complete DSL Test"));
    dsl = dsl.render(widget);

    assert!(
        dsl.capture_buffer().is_some(),
        "Buffer should be captured after render"
    );

    let lines = dsl.buffer_lines().expect("Should get buffer lines");
    assert!(
        lines.iter().any(|l| l.contains("Complete DSL Test")),
        "Rendered content should contain expected text"
    );

    dsl.write_to_pty("test input\n")
        .expect("Write to PTY should succeed");
    assert!(
        dsl.is_pty_child_running(),
        "PTY child should still be running"
    );
}

#[test]
fn test_dsl_fluent_api_complex_chain() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_buffer_diff()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Initial")))
        .then(|dsl| {
            let widget = Paragraph::new(Text::from("After first then"));
            dsl.render(widget)
        })
        .then(|dsl| {
            let widget = Paragraph::new(Text::from("After second then"));
            dsl.render(widget)
        })
        .then_result(|dsl| {
            let widget = Paragraph::new(Text::from("Final state"));
            Ok(dsl.render(widget))
        })
        .expect("then_result should succeed");

    assert!(
        result.capture_buffer().is_some(),
        "Final render should have buffer"
    );
}

#[test]
fn test_dsl_state_capture_and_assertion_flow() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester();

    #[derive(Debug, serde::Serialize, PartialEq)]
    struct TestState {
        items: Vec<String>,
        count: usize,
    }

    let initial_state = TestState {
        items: vec!["item1".to_string(), "item2".to_string()],
        count: 2,
    };

    dsl.capture_state(&initial_state, Some("initial"))
        .expect("Should capture initial state");

    let captured = dsl
        .get_state_tester()
        .expect("StateTester should exist")
        .get_snapshot("initial")
        .expect("Snapshot should exist");

    assert_eq!(
        captured.json["count"], 2,
        "Captured state should have count 2"
    );

    dsl.capture_state(&initial_state, Some("verify"))
        .expect("Should capture verification state");

    let same_state = TestState {
        items: vec!["item1".to_string(), "item2".to_string()],
        count: 2,
    };
    dsl.get_state_tester()
        .expect("StateTester should exist")
        .assert_state_named(&same_state, "initial")
        .expect("Same state should pass assertion");
}

#[test]
fn test_dsl_buffer_diff_workflow() {
    let dsl1 = TestDsl::new()
        .with_size(40, 10)
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Expected content")));

    let buffer1 = dsl1.capture_buffer().expect("Should capture first buffer");

    let dsl2 = TestDsl::new()
        .with_size(40, 10)
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Expected content")));

    let result = dsl2.assert_no_diffs(&buffer1);
    assert!(result.is_ok(), "Identical buffers should have no diffs");
}

#[test]
fn test_dsl_with_predicates_wait() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = triggered.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        triggered_clone.store(true, Ordering::SeqCst);
    });

    let predicate = WaitPredicate::new("triggered becomes true", move || {
        triggered.load(Ordering::SeqCst)
    });

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .add_predicate(predicate)
        .wait_with_predicates(Duration::from_secs(1))
        .expect("Wait with predicates should succeed");

    assert!(
        dsl.get_terminal().is_some(),
        "Terminal should still be accessible after wait"
    );
}

#[test]
fn test_dsl_poll_until_condition() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    std::thread::spawn(move || {
        for _ in 0..5 {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(20));
        }
    });

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .poll_until(Duration::from_secs(2), move || {
            counter.load(Ordering::SeqCst) >= 5
        })
        .expect("Poll until should succeed");

    assert!(
        dsl.get_terminal().is_some(),
        "Terminal should still be accessible after poll"
    );
}

#[test]
fn test_dsl_snapshot_and_compare_workflow() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Snapshot content")));

    dsl.snapshot_state("test_snapshot")
        .expect("Snapshot should be captured");

    let result = dsl.compare_to_snapshot("test_snapshot");
    assert!(
        result.is_ok(),
        "Comparing to identical snapshot should succeed: {:?}",
        result
    );
}

#[test]
fn test_dsl_buffer_content_access() {
    let dsl = TestDsl::new()
        .with_size(20, 5)
        .init_terminal()
        .render(Paragraph::new(Text::from("Line1\nLine2\nLine3")));

    assert_eq!(
        dsl.buffer_line_at(0).map(|l| l.trim().to_string()),
        Some("Line1".to_string()),
        "First line should be 'Line1'"
    );

    assert_eq!(
        dsl.buffer_line_at(1).map(|l| l.trim().to_string()),
        Some("Line2".to_string()),
        "Second line should be 'Line2'"
    );

    assert_eq!(
        dsl.buffer_line_at(2).map(|l| l.trim().to_string()),
        Some("Line3".to_string()),
        "Third line should be 'Line3'"
    );

    assert!(
        dsl.buffer_content_at(0, 0).is_some(),
        "Should be able to access content at (0,0)"
    );
}

#[test]
fn test_dsl_pty_operations_complete_flow() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty(&["cat"])
        .expect("PTY creation should succeed");

    assert!(
        dsl.is_pty_child_running(),
        "PTY child should be running initially"
    );

    dsl.write_to_pty("hello world\n")
        .expect("Write to PTY should succeed");

    let output = dsl.read_from_pty(std::time::Duration::from_millis(100));
    assert!(output.is_ok(), "Reading from PTY should succeed");

    dsl.resize_pty(120, 40).expect("PTY resize should succeed");

    dsl.write_to_pty("test after resize\n")
        .expect("Write after resize should succeed");
}

#[test]
fn test_dsl_pty_key_injection() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty(&["cat"])
        .expect("PTY creation should succeed");

    let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    let result = dsl
        .get_pty_mut()
        .expect("PTY should exist")
        .inject_key_event(key_event);
    assert!(result.is_ok(), "Key event injection should succeed");
}

#[test]
fn test_dsl_compose_all_components() {
    let _dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty(&["echo", "composed"])
        .expect("PTY should be created")
        .with_buffer_diff()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Composed test")))
        .then(|dsl| {
            let widget = Paragraph::new(Text::from("After composition"));
            dsl.render(widget)
        });
}

#[tokio::test]
async fn test_cli_tester_integration() {
    let tester = CliTester::new("echo")
        .arg("DSL")
        .arg("Integration")
        .arg("Test");

    let output = tester.run().await.expect("CLI should execute successfully");

    output.assert_success().expect("Echo should succeed");
    output
        .assert_stdout_contains("DSL")
        .expect("Should contain 'DSL'");
    output
        .assert_stdout_contains("Integration")
        .expect("Should contain 'Integration'");
    output
        .assert_stdout_contains("Test")
        .expect("Should contain 'Test'");
}

#[tokio::test]
async fn test_dsl_and_cli_combined_workflow() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Before CLI")));

    let cli = CliTester::new("echo").arg("CLI executed");
    let cli_output = cli.run().await.expect("CLI should run successfully");

    assert!(
        cli_output.stdout.contains("CLI executed"),
        "CLI output should contain expected text"
    );

    let lines = dsl.buffer_lines().expect("Should get buffer lines");
    assert!(
        lines.iter().any(|l| l.contains("Before CLI")),
        "DSL state should be preserved after CLI"
    );
}

#[test]
fn test_state_tester_direct_usage() {
    let mut state_tester = StateTester::new();

    #[derive(Debug, serde::Serialize, PartialEq)]
    struct AppState {
        title: String,
        items: Vec<u32>,
    }

    let initial = AppState {
        title: "Test App".to_string(),
        items: vec![1, 2, 3],
    };

    state_tester
        .capture_state(&initial, Some("app_state"))
        .expect("Should capture state");

    let same_state = AppState {
        title: "Test App".to_string(),
        items: vec![1, 2, 3],
    };

    let result = state_tester.assert_state_named(&same_state, "app_state");
    assert!(result.is_ok(), "Same state should pass: {:?}", result);

    let modified_state = AppState {
        title: "Modified App".to_string(),
        items: vec![1, 2, 3],
    };

    let result = state_tester.assert_state_named(&modified_state, "app_state");
    assert!(result.is_err(), "Modified state should fail");
}

#[test]
fn test_buffer_diff_direct_comparison() {
    use ratatui::layout::Rect;

    fn create_buffer(width: u16, height: u16, content: &[&str]) -> ratatui::buffer::Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buffer = ratatui::buffer::Buffer::empty(area);

        for (y, line) in content.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let idx = y * width as usize + x;
                if idx < buffer.content.len() {
                    buffer.content[idx].set_symbol(c.to_string().as_str());
                }
            }
        }
        buffer
    }

    let diff = BufferDiff::new();

    let buf1 = create_buffer(5, 2, &["hello", "world"]);
    let buf2 = create_buffer(5, 2, &["hello", "world"]);

    let result = diff.diff(&buf1, &buf2).expect("Diff should succeed");
    assert_eq!(
        result.total_diffs, 0,
        "Identical buffers should have no diffs"
    );

    let buf3 = create_buffer(5, 2, &["hello", "w0rld"]);
    let result = diff.diff(&buf1, &buf3).expect("Diff should succeed");
    assert_eq!(result.total_diffs, 1, "Should detect one diff");

    let diff_with_ignore = BufferDiff::new().ignore_foreground().ignore_background();
    let result = diff_with_ignore
        .diff(&buf1, &buf3)
        .expect("Diff should succeed");
    assert_eq!(
        result.total_diffs, 1,
        "Ignore options should still detect symbol diffs"
    );
}

#[test]
fn test_dsl_wait_for_immediate_success() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for(std::time::Duration::from_secs(1), || true)
        .expect("Immediate success wait should succeed");

    assert!(
        dsl.get_terminal().is_some(),
        "Terminal should be accessible"
    );
}

#[test]
fn test_dsl_wait_for_async_example() {
    use std::time::Duration;

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for_async(Duration::from_secs(1), || async { true })
        .expect("Async wait should succeed");

    assert!(
        dsl.get_terminal().is_some(),
        "Terminal should be accessible after async wait"
    );
}

#[test]
fn test_pty_simulator_standalone() {
    let mut pty = PtySimulator::new(&["cat"]).expect("PTY should be created");

    assert!(pty.is_child_running(), "Child should be running");

    pty.write_input("test input\n")
        .expect("Write should succeed");

    let output = pty.read_output(std::time::Duration::from_millis(100));
    assert!(output.is_ok(), "Read should succeed");
    assert!(
        output.unwrap().contains("test input"),
        "Output should contain written input"
    );

    let resize_result = pty.resize(100, 30);
    assert!(resize_result.is_ok(), "Resize should succeed");
}

#[test]
fn test_dsl_buffer_line_trailing_whitespace_handling() {
    let dsl = TestDsl::new()
        .with_size(20, 3)
        .init_terminal()
        .render(Paragraph::new(Text::from("Hello      ")));

    let lines = dsl.buffer_lines().expect("Should get lines");
    assert!(
        lines[0].starts_with("Hello"),
        "Line should start with 'Hello'"
    );
}
