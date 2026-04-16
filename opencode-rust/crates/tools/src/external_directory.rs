use std::path::PathBuf;

pub fn assert_external_directory(target: &str) -> bool {
    if target.is_empty() {
        return false;
    }

    let target_path = PathBuf::from(target);

    if let Ok(current_dir) = std::env::current_dir() {
        if target_path.starts_with(&current_dir) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_external_directory_empty() {
        assert!(!assert_external_directory(""));
    }

    #[test]
    fn test_assert_external_directory_nonexistent() {
        let result = assert_external_directory("/nonexistent/path");
        assert!(result);
    }

    #[test]
    fn test_assert_external_directory_current_dir() {
        let current = std::env::current_dir().unwrap();
        assert!(!assert_external_directory(current.to_str().unwrap()));
    }

    #[test]
    fn test_assert_external_directory_subdirectory() {
        let current = std::env::current_dir().unwrap();
        let subdir = current.join("subdir");
        assert!(!assert_external_directory(subdir.to_str().unwrap()));
    }

    #[test]
    fn test_assert_external_directory_parent() {
        let current = std::env::current_dir().unwrap();
        let parent = current.parent().unwrap();
        assert!(assert_external_directory(parent.to_str().unwrap()));
    }
}
