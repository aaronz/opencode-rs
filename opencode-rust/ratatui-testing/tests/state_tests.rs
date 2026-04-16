use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui_testing::{StateDiff, StateTester, TerminalState};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct SimpleState {
    name: String,
    count: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct NestedState {
    inner: SimpleState,
    enabled: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct ListState {
    items: Vec<String>,
}

fn create_simple_state() -> SimpleState {
    SimpleState {
        name: "test".to_string(),
        count: 42,
    }
}

fn create_nested_state() -> NestedState {
    NestedState {
        inner: SimpleState {
            name: "nested".to_string(),
            count: 100,
        },
        enabled: true,
    }
}

fn create_list_state() -> ListState {
    ListState {
        items: vec!["item1".to_string(), "item2".to_string()],
    }
}

#[test]
fn test_state_tester_new() {
    let tester = StateTester::new();
    assert!(tester.list_snapshots().is_empty());
}

#[test]
fn test_state_tester_with_default_path() {
    let tester = StateTester::new().with_default_path("custom_path");
    assert!(tester.get_snapshot("custom_path").is_none());
}

#[test]
fn test_capture_state_simple() {
    let mut tester = StateTester::new();
    let state = create_simple_state();

    let snapshot = tester.capture_state(&state, None).unwrap();

    assert_eq!(snapshot.json["name"], "test");
    assert_eq!(snapshot.json["count"], 42);
    assert_eq!(snapshot.path, vec!["default"]);
}

#[test]
fn test_capture_state_named() {
    let mut tester = StateTester::new();
    let state = create_simple_state();

    let snapshot = tester.capture_state(&state, Some("my_snapshot")).unwrap();

    assert_eq!(snapshot.path, vec!["my_snapshot"]);
    assert!(tester.get_snapshot("my_snapshot").is_some());
}

#[test]
fn test_get_snapshot_existing() {
    let mut tester = StateTester::new();
    let state = create_simple_state();
    tester.capture_state(&state, Some("snap")).unwrap();

    let retrieved = tester.get_snapshot("snap");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().json["name"], "test");
}

#[test]
fn test_get_snapshot_nonexistent() {
    let tester = StateTester::new();
    assert!(tester.get_snapshot("nonexistent").is_none());
}

#[test]
fn test_list_snapshots() {
    let mut tester = StateTester::new();
    tester
        .capture_state(&create_simple_state(), Some("snap1"))
        .unwrap();
    tester
        .capture_state(&create_nested_state(), Some("snap2"))
        .unwrap();

    let snapshots = tester.list_snapshots();
    assert_eq!(snapshots.len(), 2);
    assert!(snapshots.contains(&"snap1"));
    assert!(snapshots.contains(&"snap2"));
}

#[test]
fn test_remove_snapshot() {
    let mut tester = StateTester::new();
    tester
        .capture_state(&create_simple_state(), Some("snap"))
        .unwrap();
    assert!(tester.get_snapshot("snap").is_some());

    let removed = tester.remove_snapshot("snap").unwrap();
    assert!(tester.get_snapshot("snap").is_none());
    assert_eq!(removed.json["name"], "test");
}

#[test]
fn test_remove_nonexistent_snapshot() {
    let mut tester = StateTester::new();
    assert!(tester.remove_snapshot("nonexistent").is_none());
}

#[test]
fn test_clear_snapshots() {
    let mut tester = StateTester::new();
    tester
        .capture_state(&create_simple_state(), Some("snap1"))
        .unwrap();
    tester
        .capture_state(&create_simple_state(), Some("snap2"))
        .unwrap();
    assert_eq!(tester.list_snapshots().len(), 2);

    tester.clear_snapshots();
    assert!(tester.list_snapshots().is_empty());
}

#[test]
fn test_compare_identical_states() {
    let mut tester = StateTester::new();
    let state = create_simple_state();
    tester.capture_state(&state, Some("snap")).unwrap();

    let current = serde_json::to_value(&state).unwrap();
    let snapshot = tester.get_snapshot("snap").unwrap();
    let diff = tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 0);
    assert!(diff.differences.is_empty());
}

#[test]
fn test_compare_modified_state() {
    let mut tester = StateTester::new();
    let state1 = create_simple_state();
    tester.capture_state(&state1, Some("snap")).unwrap();

    let state2 = SimpleState {
        name: "changed".to_string(),
        count: 42,
    };
    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = tester.get_snapshot("snap").unwrap();
    let diff = tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 1);
    assert_eq!(
        diff.differences[0].diff_type,
        ratatui_testing::DiffType::Modified
    );
}

#[test]
fn test_compare_by_name() {
    let mut tester = StateTester::new();
    let state1 = create_simple_state();
    tester.capture_state(&state1, Some("named_snap")).unwrap();

    let state2 = SimpleState {
        name: "modified".to_string(),
        count: 99,
    };
    let current = serde_json::to_value(&state2).unwrap();

    let diff = tester.compare_by_name(&current, "named_snap").unwrap();
    assert_eq!(diff.total_diffs, 2);
}

#[test]
fn test_compare_nested_state() {
    let mut tester = StateTester::new();
    let state1 = create_nested_state();
    tester.capture_state(&state1, Some("nested_snap")).unwrap();

    let state2 = NestedState {
        inner: SimpleState {
            name: "nested".to_string(),
            count: 200,
        },
        enabled: true,
    };
    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = tester.get_snapshot("nested_snap").unwrap();
    let diff = tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 1);
    assert!(diff.differences[0].path.contains("count"));
}

#[test]
fn test_compare_array_states() {
    let mut tester = StateTester::new();
    let state1 = create_list_state();
    tester.capture_state(&state1, Some("list_snap")).unwrap();

    let state2 = ListState {
        items: vec!["item1".to_string(), "modified".to_string()],
    };
    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = tester.get_snapshot("list_snap").unwrap();
    let diff = tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 1);
    assert!(diff.differences[0].path.contains("[1]"));
}

#[test]
fn test_compare_added_array_element() {
    let mut tester = StateTester::new();
    let state1 = ListState {
        items: vec!["a".to_string()],
    };
    tester.capture_state(&state1, Some("snap")).unwrap();

    let state2 = ListState {
        items: vec!["a".to_string(), "b".to_string()],
    };
    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = tester.get_snapshot("snap").unwrap();
    let diff = tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 1);
    assert_eq!(
        diff.differences[0].diff_type,
        ratatui_testing::DiffType::Added
    );
}

#[test]
fn test_compare_removed_array_element() {
    let mut tester = StateTester::new();
    let state1 = ListState {
        items: vec!["a".to_string(), "b".to_string()],
    };
    tester.capture_state(&state1, Some("snap")).unwrap();

    let state2 = ListState {
        items: vec!["a".to_string()],
    };
    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = tester.get_snapshot("snap").unwrap();
    let diff = tester.compare(&current, snapshot).unwrap();

    assert_eq!(diff.total_diffs, 1);
    assert_eq!(
        diff.differences[0].diff_type,
        ratatui_testing::DiffType::Removed
    );
}

#[test]
fn test_assert_state_success() {
    let mut tester = StateTester::new();
    let state = create_simple_state();
    tester.capture_state(&state, Some("default")).unwrap();

    let result = tester.assert_state(&state);
    assert!(result.is_ok());
}

#[test]
fn test_assert_state_failure() {
    let mut tester = StateTester::new();
    let state1 = create_simple_state();
    tester.capture_state(&state1, Some("default")).unwrap();

    let state2 = SimpleState {
        name: "different".to_string(),
        count: 999,
    };

    let result = tester.assert_state(&state2);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("State mismatch"));
}

#[test]
fn test_assert_state_named_success() {
    let mut tester = StateTester::new();
    let state = create_simple_state();
    tester.capture_state(&state, Some("named")).unwrap();

    let result = tester.assert_state_named(&state, "named");
    assert!(result.is_ok());
}

#[test]
fn test_assert_state_named_not_found() {
    let tester = StateTester::new();
    let state = create_simple_state();

    let result = tester.assert_state_named(&state, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_assert_state_matches_success() {
    let tester = StateTester::new();
    let expected = serde_json::json!({"key": "value", "num": 42});
    let actual = serde_json::json!({"key": "value", "num": 42});

    let result = tester.assert_state_matches(&expected, &actual);
    assert!(result.is_ok());
}

#[test]
fn test_assert_state_matches_failure() {
    let tester = StateTester::new();
    let expected = serde_json::json!({"key": "value1"});
    let actual = serde_json::json!({"key": "value2"});

    let result = tester.assert_state_matches(&expected, &actual);
    assert!(result.is_err());
}

#[test]
fn test_diff_to_string() {
    let mut tester = StateTester::new();
    let state1 = create_simple_state();
    tester.capture_state(&state1, Some("snap")).unwrap();

    let state2 = SimpleState {
        name: "changed".to_string(),
        count: 42,
    };
    let current = serde_json::to_value(&state2).unwrap();
    let snapshot = tester.get_snapshot("snap").unwrap();

    let output = tester.diff_to_string(&current, snapshot).unwrap();
    assert!(output.contains("StateDiff"));
    assert!(output.contains("name"));
    assert!(output.contains("modified"));
}

#[test]
fn test_state_diff_display_empty() {
    let diff = StateDiff {
        differences: vec![],
        total_diffs: 0,
    };
    assert_eq!(diff.to_string(), "State is identical");
}

#[test]
fn test_state_diff_display_with_differences() {
    use ratatui_testing::{DiffType, StateDiffEntry};
    let diff = StateDiff {
        differences: vec![StateDiffEntry {
            path: "$.name".to_string(),
            diff_type: DiffType::Modified,
            expected: Some(serde_json::json!("old")),
            actual: Some(serde_json::json!("new")),
        }],
        total_diffs: 1,
    };
    let display = diff.to_string();
    assert!(display.contains("StateDiff"));
    assert!(display.contains("$.name"));
    assert!(display.contains("modified"));
}

#[test]
fn test_terminal_state_from_buffer() {
    let area = Rect::new(0, 0, 10, 2);
    let mut buffer = Buffer::empty(area);

    for (y, line) in ["Hello", "World"].iter().enumerate() {
        for (x, c) in line.char_indices() {
            let idx = y * 10 + x;
            buffer.content[idx].set_symbol(c.to_string().as_str());
        }
    }

    let state = TerminalState::from_buffer(&buffer, Some(5), Some(1));

    assert_eq!(state.width, 10);
    assert_eq!(state.height, 2);
    assert_eq!(state.cursor_x, Some(5));
    assert_eq!(state.cursor_y, Some(1));
}

#[test]
fn test_terminal_state_content_as_lines() {
    let area = Rect::new(0, 0, 5, 2);
    let mut buffer = Buffer::empty(area);

    buffer.content[0].set_symbol("H");
    buffer.content[1].set_symbol("e");
    buffer.content[2].set_symbol("l");
    buffer.content[3].set_symbol("l");
    buffer.content[4].set_symbol("o");
    buffer.content[5].set_symbol("W");
    buffer.content[6].set_symbol("o");
    buffer.content[7].set_symbol("r");
    buffer.content[8].set_symbol("l");
    buffer.content[9].set_symbol("d");

    let state = TerminalState::from_buffer(&buffer, None, None);
    let lines = state.content_as_lines();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "Hello");
    assert_eq!(lines[1], "World");
}

#[test]
fn test_terminal_state_cursor_position() {
    let buffer = Buffer::empty(Rect::new(0, 0, 80, 24));

    let with_cursor = TerminalState::from_buffer(&buffer, Some(10), Some(5));
    assert_eq!(with_cursor.cursor_position(), Some((10, 5)));

    let without_cursor = TerminalState::from_buffer(&buffer, None, None);
    assert_eq!(without_cursor.cursor_position(), None);
}

#[test]
fn test_terminal_state_serialization_roundtrip() {
    let area = Rect::new(0, 0, 20, 3);
    let buffer = Buffer::empty(area);

    let state = TerminalState::from_buffer(&buffer, Some(7), Some(2));

    let serialized = serde_json::to_string(&state).unwrap();
    let deserialized: TerminalState = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.width, 20);
    assert_eq!(deserialized.height, 3);
    assert_eq!(deserialized.cursor_x, Some(7));
    assert_eq!(deserialized.cursor_y, Some(2));
}

#[test]
fn test_capture_terminal_state() {
    let mut tester = StateTester::new();
    let area = Rect::new(0, 0, 80, 24);
    let buffer = Buffer::empty(area);

    let snapshot = tester
        .capture_terminal_state(&buffer, Some(0), Some(0), Some("term_snap"))
        .unwrap();

    assert_eq!(snapshot.json["width"], 80);
    assert_eq!(snapshot.json["height"], 24);
    assert_eq!(snapshot.json["cursor_x"], 0);
    assert_eq!(snapshot.json["cursor_y"], 0);
}

#[test]
fn test_multiple_snapshots_same_name_overwrites() {
    let mut tester = StateTester::new();
    let state1 = create_simple_state();
    let state2 = SimpleState {
        name: "overwritten".to_string(),
        count: 100,
    };

    tester.capture_state(&state1, Some("snap")).unwrap();
    tester.capture_state(&state2, Some("snap")).unwrap();

    let snapshot = tester.get_snapshot("snap").unwrap();
    assert_eq!(snapshot.json["name"], "overwritten");
    assert_eq!(snapshot.json["count"], 100);
}

#[test]
fn test_default_path_behavior() {
    let mut tester = StateTester::new();
    let state = create_simple_state();

    tester.capture_state(&state, None).unwrap();

    let snapshot = tester.get_snapshot("default").unwrap();
    assert_eq!(snapshot.json["name"], "test");
}

#[test]
fn test_state_snapshot_path() {
    let mut tester = StateTester::new();
    let state = create_simple_state();

    let snapshot = tester.capture_state(&state, Some("custom_path")).unwrap();
    assert_eq!(snapshot.path, vec!["custom_path"]);
}

#[test]
fn test_empty_terminal_state() {
    let area = Rect::new(0, 0, 80, 24);
    let buffer = Buffer::empty(area);

    let state = TerminalState::from_buffer(&buffer, None, None);

    assert_eq!(state.width, 80);
    assert_eq!(state.height, 24);
    assert!(state
        .content
        .iter()
        .all(|line| line.chars().all(|c| c == ' ')));
}

#[test]
fn test_snapshot_capture_and_retrieval_cycle() {
    let mut tester = StateTester::new();

    let state = create_nested_state();
    tester.capture_state(&state, Some("cycle_test")).unwrap();

    let retrieved = tester.get_snapshot("cycle_test").unwrap();
    let deserialized: NestedState = serde_json::from_value(retrieved.json.clone()).unwrap();

    assert_eq!(deserialized.inner.name, "nested");
    assert_eq!(deserialized.inner.count, 100);
    assert!(deserialized.enabled);
}

#[test]
fn test_compare_state_accepts_serializable_type() {
    let mut tester = StateTester::new();
    let state = create_simple_state();
    tester.capture_state(&state, Some("default")).unwrap();

    let result = tester.compare_state(&state);
    assert!(result.is_ok());
}

#[test]
fn test_compare_state_returns_state_diff() {
    let mut tester = StateTester::new();
    let state = create_simple_state();
    tester.capture_state(&state, Some("default")).unwrap();

    let result = tester.compare_state(&state).unwrap();
    assert!(result.differences.is_empty());
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_compare_state_matching_state_returns_empty_diff() {
    let mut tester = StateTester::new();
    let state = create_simple_state();
    tester.capture_state(&state, Some("default")).unwrap();

    let diff = tester.compare_state(&state).unwrap();
    assert_eq!(diff.total_diffs, 0);
    assert!(diff.differences.is_empty());
}

#[test]
fn test_compare_state_different_state_returns_diff() {
    let mut tester = StateTester::new();
    let state1 = create_simple_state();
    tester.capture_state(&state1, Some("default")).unwrap();

    let state2 = SimpleState {
        name: "changed".to_string(),
        count: 99,
    };

    let diff = tester.compare_state(&state2).unwrap();
    assert!(diff.total_diffs > 0);
    assert!(!diff.differences.is_empty());
}
