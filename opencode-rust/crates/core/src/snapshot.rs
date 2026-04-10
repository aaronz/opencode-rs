use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub files: HashMap<String, String>,
}

pub struct SnapshotManager {
    snapshots: Vec<Snapshot>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn create(&mut self, description: String, files: HashMap<String, String>) -> Snapshot {
        let snapshot = Snapshot {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            description,
            files,
        };
        self.snapshots.push(snapshot.clone());
        snapshot
    }

    pub fn get(&self, id: &str) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    pub fn list(&self) -> &[Snapshot] {
        &self.snapshots
    }

    pub fn revert(&self, id: &str) -> Option<&HashMap<String, String>> {
        self.get(id).map(|s| &s.files)
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_snapshot_manager_new() {
        let sm = SnapshotManager::new();
        assert!(sm.list().is_empty());
    }

    #[test]
    fn test_snapshot_manager_create() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), "content".to_string());

        let snapshot = sm.create("Test snapshot".to_string(), files);

        assert!(!snapshot.id.is_empty());
        assert_eq!(snapshot.description, "Test snapshot");
    }

    #[test]
    fn test_snapshot_manager_get() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), "content".to_string());

        let snapshot = sm.create("Test".to_string(), files);
        let id = snapshot.id.clone();

        assert!(sm.get(&id).is_some());
    }

    #[test]
    fn test_snapshot_manager_list() {
        let mut sm = SnapshotManager::new();
        let files: HashMap<String, String> = HashMap::new();
        sm.create("snapshot1".to_string(), files.clone());
        sm.create("snapshot2".to_string(), files);

        assert_eq!(sm.list().len(), 2);
    }

    #[test]
    fn test_snapshot_manager_revert() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert("file.txt".to_string(), "content".to_string());

        let snapshot = sm.create("Test".to_string(), files);

        let reverted = sm.revert(&snapshot.id);
        assert!(reverted.is_some());
    }

    // =========================================================================
    // Edge Case Tests for SnapshotManager
    // =========================================================================

    #[test]
    fn test_snapshot_with_empty_files_map() {
        let mut sm = SnapshotManager::new();
        let snapshot = sm.create("Empty snapshot".to_string(), HashMap::new());

        assert!(snapshot.files.is_empty());
        let reverted = sm.revert(&snapshot.id);
        assert!(reverted.is_some());
        assert!(reverted.unwrap().is_empty());
    }

    #[test]
    fn test_snapshot_with_unicode_content() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert(
            "unicode_file.txt".to_string(),
            "Hello 世界 🌍 مرحبا".to_string(),
        );
        files.insert(
            "path/with/unicode_日本語.rs".to_string(),
            "fn 日 本 語() {}".to_string(),
        );

        let snapshot = sm.create("Unicode content".to_string(), files);

        let reverted = sm.revert(&snapshot.id).unwrap();
        assert_eq!(
            reverted.get("unicode_file.txt").unwrap(),
            "Hello 世界 🌍 مرحبا"
        );
        assert_eq!(
            reverted.get("path/with/unicode_日本語.rs").unwrap(),
            "fn 日 本 語() {}"
        );
    }

    #[test]
    fn test_snapshot_with_special_characters_in_path() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        // File path with spaces, dots, dashes, underscores
        files.insert(
            "path with spaces/file-name_2024.test.txt".to_string(),
            "content".to_string(),
        );
        // Path with multiple consecutive slashes
        files.insert(
            "path///multiple////slashes".to_string(),
            "content".to_string(),
        );
        // Path ending with slash (technically invalid but should be stored as-is)
        files.insert("trailing_slash_/".to_string(), "content".to_string());

        let snapshot = sm.create("Special paths".to_string(), files);
        let reverted = sm.revert(&snapshot.id).unwrap();

        assert_eq!(
            reverted
                .get("path with spaces/file-name_2024.test.txt")
                .unwrap(),
            "content"
        );
        assert_eq!(
            reverted.get("path///multiple////slashes").unwrap(),
            "content"
        );
        assert_eq!(reverted.get("trailing_slash_/").unwrap(), "content");
    }

    #[test]
    fn test_snapshot_with_many_files() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        for i in 0..100 {
            files.insert(format!("file_{}.txt", i), format!("content_{}", i));
        }

        let snapshot = sm.create("Many files".to_string(), files);
        let reverted = sm.revert(&snapshot.id).unwrap();

        assert_eq!(reverted.len(), 100);
        assert_eq!(reverted.get("file_0.txt").unwrap(), "content_0");
        assert_eq!(reverted.get("file_99.txt").unwrap(), "content_99");
    }

    #[test]
    fn test_snapshot_get_nonexistent_id() {
        let sm = SnapshotManager::new();
        let result = sm.get("nonexistent-id");
        assert!(result.is_none());
    }

    #[test]
    fn test_snapshot_revert_nonexistent_id() {
        let sm = SnapshotManager::new();
        let result = sm.revert("nonexistent-id");
        assert!(result.is_none());
    }

    #[test]
    fn test_snapshot_revert_returns_correct_files() {
        let mut sm = SnapshotManager::new();
        let mut files1 = HashMap::new();
        files1.insert("file1.txt".to_string(), "content1".to_string());

        let mut files2 = HashMap::new();
        files2.insert("file2.txt".to_string(), "content2".to_string());

        let snapshot1 = sm.create("First".to_string(), files1);
        let snapshot2 = sm.create("Second".to_string(), files2);

        let reverted1 = sm.revert(&snapshot1.id).unwrap();
        let reverted2 = sm.revert(&snapshot2.id).unwrap();

        assert_eq!(reverted1.get("file1.txt").unwrap(), "content1");
        assert!(reverted1.get("file2.txt").is_none());

        assert_eq!(reverted2.get("file2.txt").unwrap(), "content2");
        assert!(reverted2.get("file1.txt").is_none());
    }

    #[test]
    fn test_snapshot_uuid_format() {
        let mut sm = SnapshotManager::new();
        let files: HashMap<String, String> = HashMap::new();
        let snapshot = sm.create("Test".to_string(), files);

        // UUID format: 8-4-4-4-12 = 36 characters with hyphens
        assert_eq!(snapshot.id.len(), 36);
        assert!(snapshot
            .id
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-'));
    }

    #[test]
    fn test_snapshot_with_large_content() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        // 1MB of content
        let large_content = "x".repeat(1024 * 1024);
        files.insert("large_file.txt".to_string(), large_content.clone());

        let snapshot = sm.create("Large content".to_string(), files);
        let reverted = sm.revert(&snapshot.id).unwrap();

        assert_eq!(reverted.get("large_file.txt").unwrap().len(), 1024 * 1024);
    }

    #[test]
    fn test_snapshot_with_binary_like_content() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        // Content with null bytes (binary-like)
        let binary_content = "hello\x00world\x00\x00end".to_string();
        files.insert("binary.bin".to_string(), binary_content.clone());

        let snapshot = sm.create("Binary content".to_string(), files);
        let reverted = sm.revert(&snapshot.id).unwrap();

        assert_eq!(reverted.get("binary.bin").unwrap(), &binary_content);
    }

    #[test]
    fn test_snapshot_created_at_is_set() {
        let mut sm = SnapshotManager::new();
        let files: HashMap<String, String> = HashMap::new();
        let before = chrono::Utc::now();
        let snapshot = sm.create("Test".to_string(), files);
        let after = chrono::Utc::now();

        assert!(snapshot.created_at >= before);
        assert!(snapshot.created_at <= after);
    }

    #[test]
    fn test_snapshot_list_ordering() {
        let mut sm = SnapshotManager::new();
        let files: HashMap<String, String> = HashMap::new();

        let snap1 = sm.create("First".to_string(), files.clone());
        let snap2 = sm.create("Second".to_string(), files.clone());
        let snap3 = sm.create("Third".to_string(), files.clone());

        let list = sm.list();
        assert_eq!(list.len(), 3);
        // Most recent should be last
        assert_eq!(list[0].id, snap1.id);
        assert_eq!(list[1].id, snap2.id);
        assert_eq!(list[2].id, snap3.id);
    }

    #[test]
    fn test_snapshot_multiple_snapshots_independent() {
        let mut sm = SnapshotManager::new();
        let mut files1 = HashMap::new();
        files1.insert("file1.txt".to_string(), "content1".to_string());

        let mut files2 = HashMap::new();
        files2.insert("file2.txt".to_string(), "content2".to_string());

        let snapshot1 = sm.create("First".to_string(), files1.clone());
        let snapshot2 = sm.create("Second".to_string(), files2);

        // Modify the original hashmap - should not affect stored snapshots
        let mut files1_modified = files1;
        files1_modified.insert("file1.txt".to_string(), "modified".to_string());

        let reverted1 = sm.revert(&snapshot1.id).unwrap();
        let reverted2 = sm.revert(&snapshot2.id).unwrap();

        // Snapshots should be independent of subsequent modifications
        assert_eq!(reverted1.get("file1.txt").unwrap(), "content1");
        assert_eq!(reverted2.get("file2.txt").unwrap(), "content2");
    }

    #[test]
    fn test_snapshot_with_empty_string_key_and_value() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert("".to_string(), "".to_string());

        let snapshot = sm.create("Empty key/value".to_string(), files);
        let reverted = sm.revert(&snapshot.id).unwrap();

        assert!(reverted.contains_key(""));
        assert_eq!(reverted.get("").unwrap(), "");
    }

    #[test]
    fn test_snapshot_description_preserved() {
        let mut sm = SnapshotManager::new();
        let files: HashMap<String, String> = HashMap::new();

        let descriptions = vec![
            "Simple description",
            "",
            "Description with\nnewline",
            "Description with    tabs\t",
            "Unicode: äöü",
        ];

        for desc in descriptions {
            let snapshot = sm.create(desc.to_string(), files.clone());
            let found = sm.get(&snapshot.id).unwrap();
            assert_eq!(found.description, desc);
        }
    }
}
