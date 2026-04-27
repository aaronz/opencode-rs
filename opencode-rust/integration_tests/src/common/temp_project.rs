use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Temporary project helper for integration tests.
/// Creates a temporary directory with optional files and auto-cleans on drop.
pub struct TempProject {
    temp_dir: TempDir,
}

impl TempProject {
    /// Creates an empty temporary project directory.
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp directory"),
        }
    }

    /// Creates a temporary project with the specified files.
    /// Each tuple is (relative_path, content).
    pub fn with_files(files: &[(&str, &str)]) -> Self {
        let project = Self::new();
        for (path, content) in files {
            project.create_file(path, content);
        }
        project
    }

    /// Returns the path to the temporary project root.
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Creates a file with the given content at the specified relative path.
    /// Creates parent directories as needed.
    pub fn create_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let full_path = self.temp_dir.path().join(relative_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        std::fs::write(&full_path, content).expect("Failed to write file");
        full_path
    }

    /// Reads the content of a file at the specified relative path.
    pub fn read_file(&self, relative_path: &str) -> String {
        let full_path = self.temp_dir.path().join(relative_path);
        std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Failed to read file {}: {}", full_path.display(), e))
    }

    /// Asserts that a file exists at the specified relative path.
    pub fn assert_file_exists(&self, relative_path: &str) {
        let full_path = self.temp_dir.path().join(relative_path);
        assert!(
            full_path.exists(),
            "File should exist: {}",
            full_path.display()
        );
    }

    /// Asserts that a file does NOT exist at the specified relative path.
    #[allow(dead_code)]
    pub fn assert_file_not_exists(&self, relative_path: &str) {
        let full_path = self.temp_dir.path().join(relative_path);
        assert!(
            !full_path.exists(),
            "File should NOT exist: {}",
            full_path.display()
        );
    }

    /// Asserts that a file exists and has the expected content.
    pub fn assert_file_contents(&self, relative_path: &str, expected: &str) {
        let actual = self.read_file(relative_path);
        assert_eq!(
            actual, expected,
            "File contents mismatch for {}:\nExpected:\n{}\nActual:\n{}",
            relative_path, expected, actual
        );
    }

    /// Returns the number of files in the project (recursively).
    pub fn file_count(&self) -> usize {
        walkdir::WalkDir::new(self.temp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count()
    }
}

impl Default for TempProject {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_project_new() {
        let project = TempProject::new();
        assert!(project.path().exists());
    }

    #[test]
    fn test_temp_project_with_files() {
        let project =
            TempProject::with_files(&[("src/main.rs", "fn main() {}"), ("README.md", "# Test")]);

        project.assert_file_exists("src/main.rs");
        project.assert_file_exists("README.md");
        project.assert_file_contents("src/main.rs", "fn main() {}");
        project.assert_file_contents("README.md", "# Test");
        assert_eq!(project.file_count(), 2);
    }

    #[test]
    fn test_temp_project_create_file() {
        let project = TempProject::new();
        project.create_file("nested/deep/file.txt", "content");
        project.assert_file_contents("nested/deep/file.txt", "content");
    }

    #[test]
    fn test_temp_project_cleanup_on_drop() {
        let path;
        {
            let project = TempProject::with_files(&[("test.txt", "data")]);
            path = project.path().to_path_buf();
            assert!(path.exists());
        }
        // After drop, the temp directory is cleaned up by tempfile
        assert!(!path.exists());
    }
}
