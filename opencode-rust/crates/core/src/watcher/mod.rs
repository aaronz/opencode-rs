mod types;

pub use types::{FileWatcher, IgnoreMatcher};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    use crate::config::WatcherConfig;

    #[test]
    fn ignore_matcher_uses_defaults_and_custom_patterns() {
        let matcher = IgnoreMatcher::new(Some(&["custom/**".to_string()])).unwrap();

        assert!(matcher.should_ignore(Path::new(".git/config")));
        assert!(matcher.should_ignore(Path::new("node_modules/pkg/index.js")));
        assert!(matcher.should_ignore(Path::new("custom/tmp.txt")));
        assert!(!matcher.should_ignore(Path::new("src/main.rs")));
    }

    #[test]
    fn ignore_matcher_empty_patterns() {
        let matcher = IgnoreMatcher::new(None).unwrap();
        assert!(matcher.should_ignore(Path::new(".git/config")));
        assert!(matcher.should_ignore(Path::new("node_modules/pkg/index.js")));
        assert!(!matcher.should_ignore(Path::new("src/main.rs")));
    }

    #[test]
    fn ignore_matcher_returns_invalid_pattern_error() {
        let result = IgnoreMatcher::new(Some(&["[invalid".to_string()]));
        assert!(result.is_err());
    }

    #[test]
    fn file_watcher_returns_none_for_missing_path() {
        let config = WatcherConfig { ignore: None };
        let result = FileWatcher::start(Path::new("/definitely/missing/path"), &config);
        assert!(result.is_none());
    }

    #[test]
    fn file_watcher_starts_for_valid_directory() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::create_dir_all(temp.path().join("node_modules/pkg")).unwrap();
        fs::write(
            temp.path().join("node_modules/pkg/index.js"),
            "module.exports = {};\n",
        )
        .unwrap();

        let config = WatcherConfig {
            ignore: Some(vec!["src/generated/**".to_string()]),
        };

        let watcher = FileWatcher::start(temp.path(), &config);
        assert!(watcher.is_some());

        let watcher = watcher.unwrap();
        assert!(watcher
            .ignore_matcher
            .should_ignore(Path::new("node_modules/pkg/index.js")));
        assert!(watcher
            .ignore_matcher
            .patterns()
            .contains(&"src/generated/**".to_string()));
    }

    #[test]
    fn file_watcher_with_only_file_no_subdirs() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("file.txt"), "content").unwrap();

        let config = WatcherConfig { ignore: None };
        let result = FileWatcher::start(temp.path(), &config);

        assert!(result.is_some());
    }
}