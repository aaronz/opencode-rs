#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub struct AppFileSystem;

impl AppFileSystem {
    pub fn is_dir(path: &str) -> bool {
        Path::new(path).is_dir()
    }

    pub fn is_file(path: &str) -> bool {
        Path::new(path).is_file()
    }

    pub fn exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    pub fn read_json(path: &str) -> Result<serde_json::Value, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_json(
        path: &str,
        data: &serde_json::Value,
        mode: Option<u32>,
    ) -> Result<(), String> {
        let content = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        std::fs::write(path, content).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(m) = mode {
            std::fs::set_permissions(path, PermissionsExt::from_mode(m)).ok();
        }
        Ok(())
    }

    pub fn ensure_dir(path: &str) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())
    }

    pub fn write_with_dirs(path: &str, content: &str, mode: Option<u32>) -> Result<(), String> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, content).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(m) = mode {
            std::fs::set_permissions(path, PermissionsExt::from_mode(m)).ok();
        }
        Ok(())
    }

    pub fn find_up(target: &str, start: &str, stop: Option<&str>) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = PathBuf::from(start);
        let stop = stop.map(PathBuf::from);

        loop {
            let search = current.join(target);
            if search.exists() {
                result.push(search.to_string_lossy().to_string());
            }
            if stop.as_ref() == Some(&current) {
                break;
            }
            if !current.pop() {
                break;
            }
        }
        result
    }

    pub fn up(targets: &[String], start: &str, stop: Option<&str>) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = PathBuf::from(start);
        let stop = stop.map(PathBuf::from);

        loop {
            for target in targets {
                let search = current.join(target);
                if search.exists() {
                    result.push(search.to_string_lossy().to_string());
                }
            }
            if stop.as_ref() == Some(&current) {
                break;
            }
            if !current.pop() {
                break;
            }
        }
        result
    }

    pub fn normalize_path(p: &str) -> std::path::PathBuf {
        PathBuf::from(p)
    }

    pub fn overlaps(a: &str, b: &str) -> bool {
        let a_path = Path::new(a);
        let b_path = Path::new(b);
        a_path.starts_with(b_path) || b_path.starts_with(a_path)
    }

    pub fn contains(parent: &str, child: &str) -> bool {
        let parent_path = Path::new(parent);
        let child_path = Path::new(child);
        child_path.starts_with(parent_path) && child_path != parent_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_dir() {
        assert!(AppFileSystem::is_dir("/tmp"));
        assert!(!AppFileSystem::is_file(
            "/tmp/nonexistent_file_for_test_12345"
        ));
    }

    #[test]
    fn test_find_up() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .to_string_lossy()
                .to_string()
        });
        let result = AppFileSystem::find_up("Cargo.toml", &manifest_dir, None);
        assert!(
            !result.is_empty(),
            "Should find Cargo.toml from manifest dir: {}",
            manifest_dir
        );
    }

    #[test]
    fn test_read_json_nonexistent_file() {
        let result = AppFileSystem::read_json("/nonexistent/path/file.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such file"));
    }

    #[test]
    fn test_read_json_invalid_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("invalid.json");
        fs::write(&path, "not valid json {").unwrap();

        let result = AppFileSystem::read_json(path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_write_json_creates_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("output.json");
        let data = serde_json::json!({"key": "value"});

        let result = AppFileSystem::write_json(path.to_str().unwrap(), &data, None);
        assert!(result.is_ok());

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("key"));
        assert!(content.contains("value"));
    }

    #[test]
    fn test_write_json_to_readonly_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o444);
            fs::set_permissions(&readonly_dir, perms).unwrap();
        }

        let path = readonly_dir.join("file.json");
        let data = serde_json::json!({"key": "value"});
        let result = AppFileSystem::write_json(path.to_str().unwrap(), &data, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_dir_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("nested/deeply");

        let result = AppFileSystem::ensure_dir(path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_ensure_dir_readonly_parent_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let parent = temp_dir.path().join("readonly");
        fs::create_dir(&parent).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&parent).unwrap().permissions();
            perms.set_mode(0o444);
            fs::set_permissions(&parent, perms).unwrap();
        }

        let child = parent.join("cannot_create");
        let result = AppFileSystem::ensure_dir(child.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_write_with_dirs_creates_parent_directories() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("a/b/c/file.txt");

        let result = AppFileSystem::write_with_dirs(path.to_str().unwrap(), "content", None);
        assert!(result.is_ok());
        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "content");
    }

    #[test]
    fn test_up_finds_multiple_targets() {
        let temp_dir = tempfile::tempdir().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(temp_dir.path().join("target1.txt"), "1").unwrap();
        fs::write(subdir.join("target2.txt"), "2").unwrap();

        let result = AppFileSystem::up(
            &["target1.txt".to_string(), "target2.txt".to_string()],
            subdir.to_str().unwrap(),
            None,
        );
        assert!(!result.is_empty());
    }

    #[test]
    fn test_up_stops_at_boundary() {
        let temp_dir = tempfile::tempdir().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let result = AppFileSystem::up(
            &["*.txt".to_string()],
            subdir.to_str().unwrap(),
            Some(temp_dir.path().to_str().unwrap()),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn test_normalize_path() {
        let path = AppFileSystem::normalize_path("/foo/bar");
        assert_eq!(path, PathBuf::from("/foo/bar"));
    }

    #[test]
    fn test_overlaps_same() {
        assert!(AppFileSystem::overlaps("/foo", "/foo"));
    }

    #[test]
    fn test_overlaps_parent_child() {
        assert!(AppFileSystem::overlaps("/foo", "/foo/bar"));
        assert!(AppFileSystem::overlaps("/foo/bar", "/foo"));
    }

    #[test]
    fn test_overlaps_sibling() {
        assert!(!AppFileSystem::overlaps("/foo", "/bar"));
    }

    #[test]
    fn test_contains_direct_child() {
        assert!(AppFileSystem::contains("/foo", "/foo/bar"));
        assert!(AppFileSystem::contains("/foo", "/foo/bar/baz"));
    }

    #[test]
    fn test_contains_sibling_not_contained() {
        assert!(!AppFileSystem::contains("/foo", "/bar"));
        assert!(!AppFileSystem::contains("/foo", "/foo sibling/bar"));
    }

    #[test]
    fn test_is_file_nonexistent() {
        assert!(!AppFileSystem::is_file("/nonexistent/file/path"));
    }

    #[test]
    fn test_exists_nonexistent() {
        assert!(!AppFileSystem::exists("/nonexistent/path"));
    }

    #[test]
    fn test_exists_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        assert!(AppFileSystem::exists(file_path.to_str().unwrap()));
    }

    #[test]
    fn test_exists_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        assert!(AppFileSystem::exists(temp_dir.path().to_str().unwrap()));
    }
}
