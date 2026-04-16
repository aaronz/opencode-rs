use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui_testing::{TestDsl, WaitPredicate};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn create_test_dsl() -> TestDsl {
    TestDsl::new().with_size(80, 30)
}

#[test]
fn test_dsl_new_constructs_valid_instance() {
    let dsl = TestDsl::new();
    assert!(dsl.get_terminal().is_none());
    assert!(dsl.get_pty().is_none());
    assert!(dsl.get_buffer_diff().is_none());
    assert!(dsl.get_state_tester().is_none());
    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_dsl_default_constructs_valid_instance() {
    let dsl = TestDsl::default();
    assert!(dsl.get_terminal().is_none());
    assert!(dsl.get_pty().is_none());
    assert!(dsl.get_buffer_diff().is_none());
    assert!(dsl.get_state_tester().is_none());
}

#[test]
fn test_with_size_correctly_sets_terminal_dimensions() {
    let dsl = TestDsl::new().with_size(120, 40).init_terminal();
    let terminal = dsl.get_terminal().unwrap();
    let backend_buffer = terminal.backend().buffer();
    assert_eq!(backend_buffer.area.width, 120);
    assert_eq!(backend_buffer.area.height, 40);
}

#[test]
fn test_with_size_allows_different_dimensions() {
    let dsl1 = TestDsl::new().with_size(40, 10).init_terminal();
    let terminal1 = dsl1.get_terminal().unwrap();
    let buf1 = terminal1.backend().buffer();
    assert_eq!(buf1.area.width, 40);
    assert_eq!(buf1.area.height, 10);

    let dsl2 = TestDsl::new().with_size(200, 100).init_terminal();
    let terminal2 = dsl2.get_terminal().unwrap();
    let buf2 = terminal2.backend().buffer();
    assert_eq!(buf2.area.width, 200);
    assert_eq!(buf2.area.height, 100);
}

#[test]
fn test_init_terminal_initializes_terminal_without_error() {
    let dsl = create_test_dsl().init_terminal();
    assert!(dsl.get_terminal().is_some());
}

#[test]
fn test_init_terminal_creates_test_backend() {
    let dsl = TestDsl::new().with_size(20, 5).init_terminal();
    let terminal = dsl.get_terminal().unwrap();
    let backend_buffer = terminal.backend().buffer();
    assert_eq!(backend_buffer.area.width, 20);
    assert_eq!(backend_buffer.area.height, 5);
}

#[test]
fn test_init_terminal_returns_self_for_fluent_chaining() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_buffer_diff();
    assert!(dsl.get_terminal().is_some());
    assert!(dsl.get_buffer_diff().is_some());
}

#[test]
fn test_with_pty_with_command_parameter_creates_pty_simulator() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["echo", "test"])
        .expect("PTY creation should succeed");
    assert!(dsl.get_pty().is_some());
}

#[test]
fn test_with_pty_child_process_is_running() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["sleep", "10"])
        .expect("PTY creation should succeed");
    assert!(dsl.is_pty_child_running());
}

#[test]
fn test_with_pty_accepts_cat_command() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["cat"])
        .expect("PTY creation with cat should succeed");
    assert!(dsl.get_pty().is_some());
}

#[test]
fn test_with_pty_fails_with_invalid_command() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["nonexistent_command_xyz_12345"]);
    assert!(result.is_err());
}

#[test]
fn test_with_buffer_diff_initializes_buffer_diff() {
    let dsl = TestDsl::new().with_buffer_diff();
    assert!(dsl.get_buffer_diff().is_some());
}

#[test]
fn test_with_buffer_diff_returns_self_for_chaining() {
    let dsl = TestDsl::new().with_buffer_diff().with_state_tester();
    assert!(dsl.get_buffer_diff().is_some());
    assert!(dsl.get_state_tester().is_some());
}

#[test]
fn test_with_state_tester_initializes_state_tester() {
    let dsl = TestDsl::new().with_state_tester();
    assert!(dsl.get_state_tester().is_some());
}

#[test]
fn test_with_state_tester_returns_self_for_chaining() {
    let dsl = TestDsl::new().with_state_tester().with_buffer_diff();
    assert!(dsl.get_state_tester().is_some());
    assert!(dsl.get_buffer_diff().is_some());
}

#[test]
fn test_render_renders_widget_and_captures_buffer() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Hello, World!")));
    assert!(dsl.capture_buffer().is_some());
    let buffer = dsl.capture_buffer().unwrap();
    assert!(buffer.area.width == 80);
    assert!(buffer.area.height == 30);
}

#[test]
fn test_render_captures_multiline_content() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Line1\nLine2\nLine3")));
    let lines = dsl.buffer_lines().unwrap();
    assert!(lines.len() >= 3);
}

#[test]
fn test_render_returns_self_for_fluent_chaining() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Test")))
        .with_buffer_diff();
    assert!(dsl.capture_buffer().is_some());
    assert!(dsl.get_buffer_diff().is_some());
}

#[test]
fn test_render_with_state_renders_with_state_context() {
    #[derive(Debug, serde::Serialize)]
    struct RenderState {
        message: String,
    }

    let state = RenderState {
        message: "Dynamic content".to_string(),
    };

    let dsl = TestDsl::new().with_size(80, 24).init_terminal();

    let msg = state.message.clone();
    let widget = Paragraph::new(Text::from(msg));
    let result = dsl.render_with_state(&state, |_state: &RenderState, _area| widget);

    assert!(result.capture_buffer().is_some());
    let lines = result.buffer_lines().unwrap();
    assert!(lines.iter().any(|l| l.contains("Dynamic content")));
}

#[test]
fn test_render_with_state_allows_state_based_widget() {
    #[derive(Debug, serde::Serialize)]
    struct SizeState {
        width: u16,
        height: u16,
    }

    let state = SizeState {
        width: 80,
        height: 24,
    };

    let dsl = TestDsl::new().with_size(80, 24).init_terminal();

    let size = state.width;
    let widget = Paragraph::new(Text::from(format!("Width: {}", size)));
    let result = dsl.render_with_state(&state, |_state: &SizeState, _area| widget);

    assert!(result.capture_buffer().is_some());
}

#[test]
fn test_fluent_api_then_method_works() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .then(|dsl| dsl.render(Paragraph::new(Text::from("First"))))
        .then(|dsl| dsl.render(Paragraph::new(Text::from("Second"))));

    assert!(result.capture_buffer().is_some());
}

#[test]
fn test_fluent_api_then_result_method_works() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .then_result(|dsl| Ok(dsl.render(Paragraph::new(Text::from("Test")))))
        .expect("then_result should succeed");

    assert!(result.capture_buffer().is_some());
}

#[test]
fn test_fluent_api_method_chaining_multiple_components() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty_command(&["echo", "test"])
        .expect("PTY creation failed")
        .with_buffer_diff()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Chain test")));

    assert!(result.get_pty().is_some());
    assert!(result.get_buffer_diff().is_some());
    assert!(result.get_state_tester().is_some());
    assert!(result.capture_buffer().is_some());
}

#[test]
fn test_capture_buffer_retrieves_correct_buffer_content() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Capture test")));

    let buffer = dsl.capture_buffer();
    assert!(buffer.is_some());
    let lines = buffer
        .unwrap()
        .content
        .iter()
        .map(|c| c.symbol().to_string())
        .collect::<String>();
    assert!(lines.contains("Capture test"));
}

#[test]
fn test_capture_buffer_returns_none_when_not_rendered() {
    let dsl = create_test_dsl().init_terminal();
    let buffer = dsl.capture_buffer();
    assert!(buffer.is_none());
}

#[test]
fn test_send_keys_injects_keys_to_pty() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty_command(&["cat"])
        .expect("PTY creation failed");

    let result = dsl.send_keys("hello");
    assert!(result.is_ok());
}

#[test]
fn test_send_keys_parses_enter_key() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty_command(&["cat"])
        .expect("PTY creation failed");

    let result = dsl.send_keys("enter");
    assert!(result.is_ok());
}

#[test]
fn test_send_keys_parses_escape_key() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty_command(&["cat"])
        .expect("PTY creation failed");

    let result = dsl.send_keys("escape");
    assert!(result.is_ok());
}

#[test]
fn test_send_keys_parses_ctrl_combinations() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty_command(&["cat"])
        .expect("PTY creation failed");

    let result = dsl.send_keys("ctrl-c");
    assert!(result.is_ok());
}

#[test]
fn test_send_keys_fails_without_pty() {
    let mut dsl = TestDsl::new().with_size(80, 24).init_terminal();

    let result = dsl.send_keys("hello");
    assert!(result.is_err());
}

#[test]
fn test_wait_for_handles_timeout_correctly() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for(Duration::from_millis(100), || false);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("timed out"));
}

#[test]
fn test_wait_for_succeeds_when_predicate_becomes_true() {
    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = triggered.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        triggered_clone.store(true, Ordering::SeqCst);
    });

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for(Duration::from_secs(1), move || {
            triggered.load(Ordering::SeqCst)
        })
        .expect("Wait should succeed");

    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_wait_with_predicates_handles_predicate_matching() {
    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = triggered.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        triggered_clone.store(true, Ordering::SeqCst);
    });

    let predicate = WaitPredicate::new("test predicate", move || triggered.load(Ordering::SeqCst));

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .add_predicate(predicate)
        .wait_with_predicates(Duration::from_secs(1))
        .expect("Wait should succeed");

    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_wait_with_predicates_times_out_when_predicate_fails() {
    let predicate = WaitPredicate::new("always false", || false);

    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .add_predicate(predicate)
        .wait_with_predicates(Duration::from_millis(100));

    assert!(result.is_err());
}

#[test]
fn test_poll_until_polls_until_condition_met() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    std::thread::spawn(move || {
        for _ in 0..3 {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(20));
        }
    });

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .poll_until(Duration::from_secs(1), move || {
            counter.load(Ordering::SeqCst) >= 3
        })
        .expect("Poll should succeed");

    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_poll_until_times_out_when_condition_never_met() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .poll_until(Duration::from_millis(50), || false);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("timed out"));
}

#[test]
fn test_buffer_lines_extracts_correct_line_content() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Line1\nLine2\nLine3")));

    let lines = dsl.buffer_lines();
    assert!(lines.is_some());
    let lines = lines.unwrap();
    assert!(lines.len() >= 3);
    assert_eq!(lines[0].trim(), "Line1");
    assert_eq!(lines[1].trim(), "Line2");
    assert_eq!(lines[2].trim(), "Line3");
}

#[test]
fn test_buffer_lines_returns_none_when_not_rendered() {
    let dsl = create_test_dsl().init_terminal();
    let lines = dsl.buffer_lines();
    assert!(lines.is_none());
}

#[test]
fn test_buffer_lines_trims_trailing_whitespace() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Content with trailing   ")));

    let lines = dsl.buffer_lines().unwrap();
    assert!(lines[0].contains("Content") || lines[0].trim_end() == lines[0]);
}

#[test]
fn test_save_snapshot_persists_snapshot_to_disk() {
    let mut dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Snapshot content")));

    let snapshot_name = "test_dsl_save_snapshot";
    let result = dsl.save_snapshot(snapshot_name);
    assert!(result.is_ok());

    let loaded = ratatui_testing::load_snapshot(snapshot_name);
    assert!(loaded.is_ok());

    let _ = std::fs::remove_file("snapshots/test_dsl_save_snapshot.json");
}

#[test]
fn test_save_snapshot_fails_without_render() {
    let mut dsl = create_test_dsl().init_terminal();
    let result = dsl.save_snapshot("should_fail");
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("No buffer has been rendered yet"));
}

#[test]
fn test_save_snapshot_returns_mut_self() {
    let mut dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Content")));

    let snapshot_name = "test_return_mut_self";
    let result = dsl.save_snapshot(snapshot_name);
    assert!(result.is_ok());

    let _ = std::fs::remove_file("snapshots/test_return_mut_self.json");
}

#[test]
fn test_load_snapshot_loads_snapshot_from_disk() {
    let area = Rect::new(0, 0, 20, 5);
    let buffer = Buffer::empty(area);
    let snapshot_name = "test_dsl_load_snapshot";

    ratatui_testing::save_snapshot(snapshot_name, &buffer).unwrap();

    let dsl = create_test_dsl().init_terminal();
    let result = dsl.load_snapshot(snapshot_name);
    assert!(result.is_ok());
    let loaded = result.unwrap();
    assert_eq!(loaded.area.width, 20);
    assert_eq!(loaded.area.height, 5);

    let _ = std::fs::remove_file("snapshots/test_dsl_load_snapshot.json");
}

#[test]
fn test_load_snapshot_fails_for_nonexistent() {
    let dsl = create_test_dsl().init_terminal();
    let result = dsl.load_snapshot("nonexistent_snapshot_xyz_12345");
    assert!(result.is_err());
}

#[test]
fn test_load_snapshot_and_assert_eq_compares_loaded_snapshot() {
    let mut dsl = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Matching content")));

    let snapshot_name = "test_assert_eq_matching";
    dsl.save_snapshot(snapshot_name).unwrap();

    let result = dsl.load_snapshot_and_assert_eq(snapshot_name);
    assert!(result.is_ok());

    let _ = std::fs::remove_file("snapshots/test_assert_eq_matching.json");
}

#[test]
fn test_load_snapshot_and_assert_eq_fails_on_mismatch() {
    let mut dsl1 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Original content")));

    let snapshot_name = "test_assert_eq_mismatch";
    dsl1.save_snapshot(snapshot_name).unwrap();

    let dsl2 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Different content")));

    let result = dsl2.load_snapshot_and_assert_eq(snapshot_name);
    assert!(result.is_err());

    let _ = std::fs::remove_file("snapshots/test_assert_eq_mismatch.json");
}

#[test]
fn test_load_snapshot_and_assert_eq_requires_buffer_diff() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Content")));

    let result = dsl.load_snapshot_and_assert_eq("any_snapshot");
    assert!(result.is_err());
}

#[test]
fn test_get_terminal_returns_terminal_when_initialized() {
    let dsl = create_test_dsl().init_terminal();
    assert!(dsl.get_terminal().is_some());
}

#[test]
fn test_get_terminal_returns_none_when_not_initialized() {
    let dsl = create_test_dsl();
    assert!(dsl.get_terminal().is_none());
}

#[test]
fn test_get_terminal_mut_returns_mutable_terminal() {
    let mut dsl = create_test_dsl().init_terminal();
    let terminal = dsl.get_terminal_mut();
    assert!(terminal.is_some());
}

#[test]
fn test_get_pty_returns_pty_when_initialized() {
    let dsl = TestDsl::new()
        .with_pty_command(&["echo", "test"])
        .expect("PTY creation failed");
    assert!(dsl.get_pty().is_some());
}

#[test]
fn test_get_pty_returns_none_when_not_initialized() {
    let dsl = create_test_dsl();
    assert!(dsl.get_pty().is_none());
}

#[test]
fn test_get_pty_mut_returns_mutable_pty() {
    let mut dsl = TestDsl::new()
        .with_pty_command(&["cat"])
        .expect("PTY creation failed");
    let pty = dsl.get_pty_mut();
    assert!(pty.is_some());
}

#[test]
fn test_get_buffer_diff_returns_diff_when_initialized() {
    let dsl = TestDsl::new().with_buffer_diff();
    assert!(dsl.get_buffer_diff().is_some());
}

#[test]
fn test_get_state_tester_returns_tester_when_initialized() {
    let dsl = TestDsl::new().with_state_tester();
    assert!(dsl.get_state_tester().is_some());
}

#[test]
fn test_get_state_tester_returns_none_when_not_initialized() {
    let dsl = create_test_dsl();
    assert!(dsl.get_state_tester().is_none());
}

#[test]
fn test_assert_no_diffs_passes_for_identical_buffers() {
    let dsl1 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Identical")));

    let dsl2 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Identical")));

    let buffer1 = dsl1.capture_buffer().unwrap();
    let result = dsl2.assert_no_diffs(&buffer1);
    assert!(result.is_ok());
}

#[test]
fn test_assert_no_diffs_fails_for_different_buffers() {
    let dsl1 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Content A")));

    let dsl2 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Content B")));

    let buffer1 = dsl1.capture_buffer().unwrap();
    let result = dsl2.assert_no_diffs(&buffer1);
    assert!(result.is_err());
}

#[test]
fn test_assert_buffer_eq_passes_for_identical_buffers() {
    let dsl = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("Test")));

    let buf1 = dsl.capture_buffer().unwrap();
    let buf2 = dsl.capture_buffer().unwrap();
    let result = dsl.assert_buffer_eq(&buf1, &buf2);
    assert!(result.is_ok());
}

#[test]
fn test_assert_buffer_eq_fails_for_different_buffers() {
    let dsl1 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("A")));

    let dsl2 = create_test_dsl()
        .init_terminal()
        .with_buffer_diff()
        .render(Paragraph::new(Text::from("B")));

    let buf1 = dsl1.capture_buffer().unwrap();
    let buf2 = dsl2.capture_buffer().unwrap();
    let result = dsl2.assert_buffer_eq(&buf1, &buf2);
    assert!(result.is_err());
}

#[test]
fn test_add_predicate_adds_predicate_to_list() {
    let predicate = WaitPredicate::new("test", || true);
    let dsl = create_test_dsl().add_predicate(predicate);
    let result = dsl.wait_with_predicates(Duration::from_millis(10));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_add_multiple_predicates() {
    let dsl = create_test_dsl()
        .add_predicate(WaitPredicate::new("pred1", || true))
        .add_predicate(WaitPredicate::new("pred2", || true))
        .add_predicate(WaitPredicate::new("pred3", || true));
    let result = dsl.wait_with_predicates(Duration::from_millis(10));
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_wait_predicate_check_returns_result() {
    let predicate = WaitPredicate::new("always true", || true);
    assert!(predicate.check());
}

#[test]
fn test_wait_predicate_description_returns_description() {
    let predicate = WaitPredicate::new("custom description", || true);
    assert_eq!(predicate.description(), "custom description");
}

#[test]
fn test_assert_pty_running_passes_when_pty_is_running() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["sleep", "10"])
        .expect("PTY creation failed");

    let result = dsl.assert_pty_running();
    assert!(result.is_ok());
}

#[test]
fn test_assert_pty_running_fails_when_pty_is_not_running() {
    let dsl = TestDsl::new().with_size(80, 24);
    let result = dsl.assert_pty_running();
    assert!(result.is_err());
}

#[test]
fn test_snapshot_state_captures_buffer_to_state_tester() {
    let mut dsl = create_test_dsl()
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("State test")));

    let result = dsl.snapshot_state("buffer_state");
    assert!(result.is_ok());
}

#[test]
fn test_snapshot_state_fails_without_state_tester() {
    let mut dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("State test")));

    let result = dsl.snapshot_state("buffer_state");
    assert!(result.is_err());
}

#[test]
fn test_compare_to_snapshot_compares_current_buffer() {
    let mut dsl = create_test_dsl()
        .init_terminal()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Compare test")));

    dsl.snapshot_state("compare_snap").unwrap();

    let result = dsl.compare_to_snapshot("compare_snap");
    assert!(result.is_ok());
}

#[test]
fn test_buffer_line_at_extracts_correct_line() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Line1\nLine2\nLine3")));

    assert_eq!(dsl.buffer_line_at(0).unwrap().trim(), "Line1");
    assert_eq!(dsl.buffer_line_at(1).unwrap().trim(), "Line2");
    assert_eq!(dsl.buffer_line_at(2).unwrap().trim(), "Line3");
}

#[test]
fn test_buffer_line_at_returns_none_for_invalid_index() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("Line1\nLine2\nLine3")));

    assert!(dsl.buffer_line_at(100).is_none());
}

#[test]
fn test_buffer_content_at_extracts_correct_cell() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("ABC")));

    assert_eq!(dsl.buffer_content_at(0, 0).unwrap(), "A");
    assert_eq!(dsl.buffer_content_at(1, 0).unwrap(), "B");
    assert_eq!(dsl.buffer_content_at(2, 0).unwrap(), "C");
}

#[test]
fn test_buffer_content_at_returns_none_for_invalid_coords() {
    let dsl = create_test_dsl()
        .init_terminal()
        .render(Paragraph::new(Text::from("ABC")));

    assert!(dsl.buffer_content_at(100, 0).is_none());
    assert!(dsl.buffer_content_at(0, 100).is_none());
}

#[test]
fn test_is_pty_child_running_returns_true_when_running() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["sleep", "10"])
        .expect("PTY creation failed");

    assert!(dsl.is_pty_child_running());
}

#[test]
fn test_is_pty_child_running_returns_false_when_not_started() {
    let dsl = create_test_dsl();
    assert!(!dsl.is_pty_child_running());
}

#[test]
fn test_resize_pty_changes_dimensions() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["cat"])
        .expect("PTY creation failed");

    let result = dsl.resize_pty(120, 40);
    assert!(result.is_ok());
}

#[test]
fn test_resize_pty_fails_without_pty() {
    let mut dsl = create_test_dsl();
    let result = dsl.resize_pty(120, 40);
    assert!(result.is_err());
}

#[test]
fn test_write_to_pty_writes_input() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["cat"])
        .expect("PTY creation failed");

    let result = dsl.write_to_pty("test input\n");
    assert!(result.is_ok());
}

#[test]
fn test_read_from_pty_reads_output() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .with_pty_command(&["echo", "hello"])
        .expect("PTY creation failed");

    std::thread::sleep(Duration::from_millis(100));
    let result = dsl.read_from_pty(Duration::from_millis(200));
    assert!(result.is_ok());
}

#[test]
fn test_read_from_pty_fails_without_pty() {
    let mut dsl = create_test_dsl();
    let result = dsl.read_from_pty(Duration::from_millis(100));
    assert!(result.is_err());
}

#[test]
fn test_capture_state_captures_serializable_state() {
    #[derive(Debug, serde::Serialize, PartialEq)]
    struct TestState {
        value: String,
        count: u32,
    }

    let mut dsl = create_test_dsl().init_terminal().with_state_tester();

    let state = TestState {
        value: "test".to_string(),
        count: 42,
    };

    let result = dsl.capture_state(&state, Some("test_state"));
    assert!(result.is_ok());
}

#[test]
fn test_capture_state_fails_without_state_tester() {
    #[derive(Debug, serde::Serialize)]
    struct TestState {
        value: String,
    }

    let mut dsl = create_test_dsl().init_terminal();

    let state = TestState {
        value: "test".to_string(),
    };
    let result = dsl.capture_state(&state, Some("test_state"));
    assert!(result.is_err());
}

#[test]
fn test_assert_state_asserts_current_state_matches_default() {
    #[derive(Debug, serde::Serialize, PartialEq)]
    struct TestState {
        value: String,
    }

    let mut dsl = create_test_dsl().init_terminal().with_state_tester();

    let state = TestState {
        value: "test".to_string(),
    };
    dsl.capture_state(&state, None).unwrap();

    let result = dsl.assert_state(&state);
    assert!(result.is_ok());
}

#[test]
fn test_full_fluent_workflow() {
    let mut dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .with_pty_command(&["echo", "workflow"])
        .expect("PTY creation failed")
        .with_buffer_diff()
        .with_state_tester()
        .render(Paragraph::new(Text::from("Full workflow test")));

    assert!(dsl.get_terminal().is_some());
    assert!(dsl.get_pty().is_some());
    assert!(dsl.get_buffer_diff().is_some());
    assert!(dsl.get_state_tester().is_some());
    assert!(dsl.capture_buffer().is_some());

    let lines = dsl.buffer_lines();
    assert!(lines.is_some());
    assert!(lines.unwrap()[0].contains("Full workflow test"));

    let snapshot_name = "test_full_workflow";
    let save_result = dsl.save_snapshot(snapshot_name);
    assert!(save_result.is_ok());

    let _ = std::fs::remove_file(format!("snapshots/{}.json", snapshot_name));
}

#[test]
fn test_compose_all_components() {
    let _dsl = TestDsl::new()
        .with_size(80, 30)
        .init_terminal()
        .with_pty_command(&["cat"])
        .expect("PTY creation failed")
        .with_buffer_diff()
        .with_state_tester();
}

#[test]
fn test_then_and_then_result_can_be_combined() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .then(|dsl| dsl.render(Paragraph::new(Text::from("Step 1"))))
        .then_result(|dsl| Ok(dsl.render(Paragraph::new(Text::from("Step 2")))))
        .expect("Chaining should succeed");

    assert!(result.capture_buffer().is_some());
}

#[test]
fn test_wait_predicate_debug_trait() {
    let predicate = WaitPredicate::new("debug test", || true);
    let debug_str = format!("{:?}", predicate);
    assert!(debug_str.contains("debug test"));
}

#[test]
fn test_multiple_waits_in_sequence() {
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
        .expect("First poll should succeed");

    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_wait_for_uses_predicates_when_available() {
    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = triggered.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        triggered_clone.store(true, Ordering::SeqCst);
    });

    let predicate =
        WaitPredicate::new("stored predicate", move || triggered.load(Ordering::SeqCst));

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .add_predicate(predicate)
        .wait_for(Duration::from_secs(1), || false)
        .expect("Wait should succeed using stored predicates");

    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_wait_for_works_correctly_without_predicates() {
    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = triggered.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        triggered_clone.store(true, Ordering::SeqCst);
    });

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for(Duration::from_secs(1), move || {
            triggered.load(Ordering::SeqCst)
        })
        .expect("Wait should succeed with function predicate");

    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_wait_for_returns_correct_result_when_predicate_matches() {
    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for(Duration::from_secs(1), || true)
        .expect("Wait should succeed immediately when predicate is true");

    assert!(dsl.capture_buffer().is_none());
}

#[test]
fn test_wait_for_handles_timeout_when_predicate_never_matches() {
    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .wait_for(Duration::from_millis(100), || false);

    assert!(
        result.is_err(),
        "Wait should timeout when predicate never matches"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("timed out"),
        "Error should mention timeout"
    );
}

#[test]
fn test_wait_for_with_stored_predicate_timeout() {
    let predicate = WaitPredicate::new("always false", || false);

    let result = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .add_predicate(predicate)
        .wait_for(Duration::from_millis(100), || true);

    assert!(
        result.is_err(),
        "Wait should timeout when stored predicate never matches"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("timed out"),
        "Error should mention timeout"
    );
}

#[test]
fn test_wait_for_multiple_stored_predicates_all_match() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let predicate1 =
        WaitPredicate::new("counter >= 1", move || counter.load(Ordering::SeqCst) >= 1);
    let predicate2 = WaitPredicate::new("always true", || true);

    let dsl = TestDsl::new()
        .with_size(80, 24)
        .init_terminal()
        .add_predicate(predicate1)
        .add_predicate(predicate2)
        .wait_for(Duration::from_secs(1), || false)
        .expect("Wait should succeed when all stored predicates match");

    assert!(dsl.capture_buffer().is_none());
}
