use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub json: Value,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiffEntry {
    pub path: String,
    pub diff_type: DiffType,
    pub expected: Option<Value>,
    pub actual: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
}

impl fmt::Display for DiffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffType::Added => write!(f, "added"),
            DiffType::Removed => write!(f, "removed"),
            DiffType::Modified => write!(f, "modified"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub differences: Vec<StateDiffEntry>,
    pub total_diffs: usize,
}

impl fmt::Display for StateDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.differences.is_empty() {
            return write!(f, "State is identical");
        }

        writeln!(f, "StateDiff: {} difference(s) found", self.total_diffs)?;
        writeln!(f, "{}", "=".repeat(60))?;

        for diff in &self.differences {
            writeln!(f, "Path: {}", diff.path)?;
            writeln!(f, "  Type: {}", diff.diff_type)?;
            if let Some(expected) = &diff.expected {
                writeln!(f, "  Expected: {}", expected)?;
            }
            if let Some(actual) = &diff.actual {
                writeln!(f, "  Actual: {}", actual)?;
            }
            writeln!(f, "{}", "-".repeat(40))?;
        }

        Ok(())
    }
}

pub struct StateTester {
    snapshots: HashMap<String, StateSnapshot>,
    default_path: String,
}

impl Default for StateTester {
    fn default() -> Self {
        Self::new()
    }
}

impl StateTester {
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
            default_path: "default".to_string(),
        }
    }

    pub fn with_default_path(mut self, path: impl Into<String>) -> Self {
        self.default_path = path.into();
        self
    }

    pub fn capture_state<S>(&mut self, state: &S, name: Option<&str>) -> Result<StateSnapshot>
    where
        S: serde::Serialize,
    {
        let name = name.unwrap_or(&self.default_path);
        let json = serde_json::to_value(state).context("Failed to serialize state to JSON")?;

        let snapshot = StateSnapshot {
            json: json.clone(),
            path: vec![name.to_string()],
        };

        self.snapshots.insert(name.to_string(), snapshot.clone());
        Ok(snapshot)
    }

    pub fn get_snapshot(&self, name: &str) -> Option<&StateSnapshot> {
        self.snapshots.get(name)
    }

    pub fn list_snapshots(&self) -> Vec<&str> {
        self.snapshots.keys().map(|s| s.as_str()).collect()
    }

    pub fn compare(&self, current: &Value, snapshot: &StateSnapshot) -> Result<StateDiff> {
        let differences = self.diff_values(&current, &snapshot.json, "$");
        let total_diffs = differences.len();
        Ok(StateDiff {
            differences,
            total_diffs,
        })
    }

    pub fn compare_by_name(&self, current: &Value, name: &str) -> Result<StateDiff> {
        let snapshot = self
            .snapshots
            .get(name)
            .with_context(|| format!("Snapshot '{}' not found", name))?;
        self.compare(current, snapshot)
    }

    fn diff_values(&self, current: &Value, snapshot: &Value, path: &str) -> Vec<StateDiffEntry> {
        let mut differences = Vec::new();

        match (current, snapshot) {
            (Value::Object(cur_map), Value::Object(snap_map)) => {
                for (key, cur_val) in cur_map {
                    let key_path = format!("{}.{}", path, key);
                    if let Some(snap_val) = snap_map.get(key) {
                        differences.extend(self.diff_values(cur_val, snap_val, &key_path));
                    } else {
                        differences.push(StateDiffEntry {
                            path: key_path,
                            diff_type: DiffType::Added,
                            expected: None,
                            actual: Some(cur_val.clone()),
                        });
                    }
                }

                for (key, snap_val) in snap_map {
                    let key_path = format!("{}.{}", path, key);
                    if !cur_map.contains_key(key) {
                        differences.push(StateDiffEntry {
                            path: key_path,
                            diff_type: DiffType::Removed,
                            expected: Some(snap_val.clone()),
                            actual: None,
                        });
                    }
                }
            }

            (Value::Array(cur_arr), Value::Array(snap_arr)) => {
                let max_len = cur_arr.len().max(snap_arr.len());
                for i in 0..max_len {
                    let idx_path = format!("{}[{}]", path, i);
                    match (cur_arr.get(i), snap_arr.get(i)) {
                        (Some(cur_val), Some(snap_val)) => {
                            differences.extend(self.diff_values(cur_val, snap_val, &idx_path));
                        }
                        (Some(cur_val), None) => {
                            differences.push(StateDiffEntry {
                                path: idx_path,
                                diff_type: DiffType::Added,
                                expected: None,
                                actual: Some(cur_val.clone()),
                            });
                        }
                        (None, Some(snap_val)) => {
                            differences.push(StateDiffEntry {
                                path: idx_path,
                                diff_type: DiffType::Removed,
                                expected: Some(snap_val.clone()),
                                actual: None,
                            });
                        }
                        (None, None) => {}
                    }
                }
            }

            _ => {
                if current != snapshot {
                    differences.push(StateDiffEntry {
                        path: path.to_string(),
                        diff_type: DiffType::Modified,
                        expected: Some(snapshot.clone()),
                        actual: Some(current.clone()),
                    });
                }
            }
        }

        differences
    }

    pub fn assert_state<S>(&self, state: &S) -> Result<()>
    where
        S: serde::Serialize,
    {
        self.assert_state_named(state, &self.default_path)
    }

    pub fn assert_state_named<S>(&self, state: &S, name: &str) -> Result<()>
    where
        S: serde::Serialize,
    {
        let current = serde_json::to_value(state).context("Failed to serialize state to JSON")?;

        let snapshot = self.snapshots.get(name).with_context(|| {
            format!("Snapshot '{}' not found. Use capture_state() first.", name)
        })?;

        let diff = self.compare(&current, snapshot)?;

        if diff.total_diffs > 0 {
            anyhow::bail!("State mismatch:\n{}", diff);
        }

        Ok(())
    }

    pub fn assert_state_matches(&self, expected: &Value, actual: &Value) -> Result<()> {
        let diff = self.diff_values(expected, actual, "$");

        if !diff.is_empty() {
            let state_diff = StateDiff {
                total_diffs: diff.len(),
                differences: diff,
            };
            anyhow::bail!("State mismatch:\n{}", state_diff);
        }

        Ok(())
    }

    pub fn diff_to_string(&self, current: &Value, snapshot: &StateSnapshot) -> Result<String> {
        let diff = self.compare(current, snapshot)?;
        Ok(diff.to_string())
    }

    pub fn remove_snapshot(&mut self, name: &str) -> Option<StateSnapshot> {
        self.snapshots.remove(name)
    }

    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SimpleState {
        name: String,
        count: u32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct NestedState {
        inner: SimpleState,
        enabled: bool,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ListState {
        items: Vec<String>,
    }

    #[test]
    fn test_capture_simple_state() {
        let mut tester = StateTester::new();
        let state = SimpleState {
            name: "test".to_string(),
            count: 42,
        };

        let snapshot = tester.capture_state(&state, None).unwrap();

        assert!(snapshot.json.get("name").is_some());
        assert_eq!(snapshot.json["name"], "test");
        assert_eq!(snapshot.json["count"], 42);
    }

    #[test]
    fn test_capture_named_snapshot() {
        let mut tester = StateTester::new();
        let state1 = SimpleState {
            name: "first".to_string(),
            count: 1,
        };
        let state2 = SimpleState {
            name: "second".to_string(),
            count: 2,
        };

        tester.capture_state(&state1, Some("first")).unwrap();
        tester.capture_state(&state2, Some("second")).unwrap();

        let snapshots = tester.list_snapshots();
        assert!(snapshots.contains(&"first"));
        assert!(snapshots.contains(&"second"));
    }

    #[test]
    fn test_identical_states_match() {
        let mut tester = StateTester::new();
        let state1 = SimpleState {
            name: "test".to_string(),
            count: 42,
        };
        let state2 = SimpleState {
            name: "test".to_string(),
            count: 42,
        };

        tester.capture_state(&state1, Some("snap")).unwrap();

        let result = tester.assert_state_named(&state2, "snap");
        assert!(result.is_ok());
    }

    #[test]
    fn test_different_states_fail() {
        let mut tester = StateTester::new();
        let state1 = SimpleState {
            name: "test".to_string(),
            count: 42,
        };
        let state2 = SimpleState {
            name: "changed".to_string(),
            count: 99,
        };

        tester.capture_state(&state1, Some("snap")).unwrap();

        let result = tester.assert_state_named(&state2, "snap");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("State mismatch"));
    }

    #[test]
    fn test_nested_state_diff() {
        let mut tester = StateTester::new();
        let state1 = NestedState {
            inner: SimpleState {
                name: "test".to_string(),
                count: 42,
            },
            enabled: true,
        };
        let state2 = NestedState {
            inner: SimpleState {
                name: "test".to_string(),
                count: 99,
            },
            enabled: true,
        };

        tester.capture_state(&state1, Some("snap")).unwrap();

        let expected = serde_json::to_value(&state2).unwrap();
        let snapshot = tester.get_snapshot("snap").unwrap();
        let diff = tester.compare(&expected, snapshot).unwrap();

        assert_eq!(diff.total_diffs, 1);
        assert!(diff.differences[0].path.contains("count"));
        assert_eq!(diff.differences[0].diff_type, DiffType::Modified);
    }

    #[test]
    fn test_array_diff() {
        let mut tester = StateTester::new();
        let state1 = ListState {
            items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let state2 = ListState {
            items: vec!["a".to_string(), "x".to_string(), "c".to_string()],
        };

        tester.capture_state(&state1, Some("snap")).unwrap();

        let expected = serde_json::to_value(&state2).unwrap();
        let snapshot = tester.get_snapshot("snap").unwrap();
        let diff = tester.compare(&expected, snapshot).unwrap();

        assert_eq!(diff.total_diffs, 1);
        assert!(diff.differences[0].path.contains("[1]"));
        assert_eq!(diff.differences[0].diff_type, DiffType::Modified);
    }

    #[test]
    fn test_added_element() {
        let mut tester = StateTester::new();
        let state1 = ListState {
            items: vec!["a".to_string(), "b".to_string()],
        };
        let state2 = ListState {
            items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };

        tester.capture_state(&state1, Some("snap")).unwrap();

        let expected = serde_json::to_value(&state2).unwrap();
        let snapshot = tester.get_snapshot("snap").unwrap();
        let diff = tester.compare(&expected, snapshot).unwrap();

        assert_eq!(diff.total_diffs, 1);
        assert!(diff.differences[0].path.contains("[2]"));
        assert_eq!(diff.differences[0].diff_type, DiffType::Added);
    }

    #[test]
    fn test_removed_element() {
        let mut tester = StateTester::new();
        let state1 = ListState {
            items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let state2 = ListState {
            items: vec!["a".to_string(), "b".to_string()],
        };

        tester.capture_state(&state1, Some("snap")).unwrap();

        let expected = serde_json::to_value(&state2).unwrap();
        let snapshot = tester.get_snapshot("snap").unwrap();
        let diff = tester.compare(&expected, snapshot).unwrap();

        assert_eq!(diff.total_diffs, 1);
        assert!(diff.differences[0].path.contains("[2]"));
        assert_eq!(diff.differences[0].diff_type, DiffType::Removed);
    }

    #[test]
    fn test_diff_display() {
        let mut tester = StateTester::new();
        let state1 = SimpleState {
            name: "test".to_string(),
            count: 42,
        };
        let state2 = SimpleState {
            name: "changed".to_string(),
            count: 42,
        };

        tester.capture_state(&state1, Some("snap")).unwrap();

        let expected = serde_json::to_value(&state2).unwrap();
        let snapshot = tester.get_snapshot("snap").unwrap();
        let diff = tester.compare(&expected, snapshot).unwrap();

        let display = diff.to_string();
        assert!(display.contains("StateDiff"));
        assert!(display.contains("name"));
        assert!(display.contains("modified"));
    }

    #[test]
    fn test_identical_states_diff_is_empty() {
        let tester = StateTester::new();
        let json1 = serde_json::json!({"a": 1, "b": 2});
        let json2 = serde_json::json!({"a": 1, "b": 2});

        let diff = tester.diff_values(&json1, &json2, "$");
        assert!(diff.is_empty());
    }

    #[test]
    fn test_remove_snapshot() {
        let mut tester = StateTester::new();
        let state = SimpleState {
            name: "test".to_string(),
            count: 42,
        };

        tester.capture_state(&state, Some("snap")).unwrap();
        assert!(tester.get_snapshot("snap").is_some());

        let removed = tester.remove_snapshot("snap");
        assert!(removed.is_some());
        assert!(tester.get_snapshot("snap").is_none());
    }

    #[test]
    fn test_clear_snapshots() {
        let mut tester = StateTester::new();
        let state = SimpleState {
            name: "test".to_string(),
            count: 42,
        };

        tester.capture_state(&state, Some("snap1")).unwrap();
        tester.capture_state(&state, Some("snap2")).unwrap();
        assert_eq!(tester.list_snapshots().len(), 2);

        tester.clear_snapshots();
        assert!(tester.list_snapshots().is_empty());
    }

    #[test]
    fn test_snapshot_not_found_error() {
        let tester = StateTester::new();
        let state = SimpleState {
            name: "test".to_string(),
            count: 42,
        };

        let result = tester.assert_state_named(&state, "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_diff_to_string() {
        let mut tester = StateTester::new();
        let state1 = SimpleState {
            name: "test".to_string(),
            count: 42,
        };
        let state2 = SimpleState {
            name: "changed".to_string(),
            count: 42,
        };

        let snapshot = tester.capture_state(&state1, Some("snap")).unwrap();
        let expected = serde_json::to_value(&state2).unwrap();

        let output = tester.diff_to_string(&expected, &snapshot).unwrap();
        assert!(output.contains("StateDiff"));
        assert!(output.contains("name"));
    }

    #[test]
    fn test_compare_by_name() {
        let mut tester = StateTester::new();
        let state1 = SimpleState {
            name: "test".to_string(),
            count: 42,
        };
        let state2 = SimpleState {
            name: "changed".to_string(),
            count: 42,
        };

        tester.capture_state(&state1, Some("my_snap")).unwrap();
        let current = serde_json::to_value(&state2).unwrap();

        let diff = tester.compare_by_name(&current, "my_snap").unwrap();
        assert_eq!(diff.total_diffs, 1);
    }

    #[test]
    fn test_with_default_path() {
        let tester = StateTester::new().with_default_path("custom");
        assert!(tester.get_snapshot("custom").is_none());
    }

    #[test]
    fn test_assert_state_matches_success() {
        let tester = StateTester::new();
        let expected = serde_json::json!({"key": "value"});
        let actual = serde_json::json!({"key": "value"});

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
}
