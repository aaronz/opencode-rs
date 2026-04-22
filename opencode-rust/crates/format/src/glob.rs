use std::path::Path;

use glob::Pattern;

use opencode_config::FormatterEntry;

pub fn entry_matches_file(entry: &FormatterEntry, file_path: &str) -> bool {
    let Some(patterns) = entry.extensions.as_ref() else {
        return false;
    };

    let path = Path::new(file_path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    patterns.iter().any(|pattern| {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return false;
        }

        if pattern == extension || pattern.trim_start_matches('.') == extension {
            return true;
        }

        Pattern::new(pattern)
            .map(|glob| glob.matches(file_name) || glob.matches(file_path))
            .unwrap_or(false)
    })
}

pub fn matches_patterns(patterns: &[String], file_path: &str) -> bool {
    let path = Path::new(file_path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    patterns.iter().any(|pattern| {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return false;
        }

        if pattern == extension || pattern.trim_start_matches('.') == extension {
            return true;
        }

        Pattern::new(pattern)
            .map(|glob| glob.matches(file_name) || glob.matches(file_path))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_matching_dot_prefix() {
        let patterns = vec![".ts".to_string(), ".tsx".to_string()];
        assert!(matches_patterns(&patterns, "app.ts"));
        assert!(matches_patterns(&patterns, "app.tsx"));
        assert!(
            !matches_patterns(&patterns, "app.js"),
            "should not match js"
        );
    }

    #[test]
    fn test_extension_matching_no_dot_prefix() {
        let patterns = vec!["ts".to_string(), "tsx".to_string()];
        assert!(matches_patterns(&patterns, "app.ts"));
        assert!(matches_patterns(&patterns, "app.tsx"));
        assert!(
            !matches_patterns(&patterns, "app.js"),
            "should not match js"
        );
    }

    #[test]
    fn test_glob_pattern_matching() {
        let patterns = vec!["*.ts".to_string()];
        assert!(matches_patterns(&patterns, "app.ts"));
        assert!(matches_patterns(&patterns, "src/app.ts"));
    }

    #[test]
    fn test_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!matches_patterns(&patterns, "app.ts"));
    }

    #[test]
    fn test_empty_pattern_in_list() {
        let patterns = vec!["ts".to_string(), "".to_string()];
        assert!(matches_patterns(&patterns, "app.ts"));
        assert!(
            !matches_patterns(&patterns, "app.js"),
            "empty pattern should not match"
        );
    }

    #[test]
    fn test_entry_matches_file_typescript() {
        let entry = FormatterEntry {
            disabled: None,
            command: Some(vec!["prettier".to_string()]),
            environment: None,
            extensions: Some(vec!["ts".to_string(), "tsx".to_string()]),
        };
        assert!(entry_matches_file(&entry, "app.ts"));
        assert!(entry_matches_file(&entry, "app.tsx"));
        assert!(!entry_matches_file(&entry, "app.js"));
    }

    #[test]
    fn test_entry_matches_file_with_dot_prefix() {
        let entry = FormatterEntry {
            disabled: None,
            command: Some(vec!["prettier".to_string()]),
            environment: None,
            extensions: Some(vec![".ts".to_string(), ".tsx".to_string()]),
        };
        assert!(entry_matches_file(&entry, "app.ts"));
        assert!(entry_matches_file(&entry, "app.tsx"));
        assert!(!entry_matches_file(&entry, "app.js"));
    }

    #[test]
    fn test_entry_matches_file_no_extensions() {
        let entry = FormatterEntry {
            disabled: None,
            command: Some(vec!["prettier".to_string()]),
            environment: None,
            extensions: None,
        };
        assert!(!entry_matches_file(&entry, "app.ts"));
    }

    #[test]
    fn test_entry_matches_file_with_glob_pattern() {
        let entry = FormatterEntry {
            disabled: None,
            command: Some(vec!["prettier".to_string()]),
            environment: None,
            extensions: Some(vec!["*.ts".to_string()]),
        };
        assert!(entry_matches_file(&entry, "app.ts"));
        assert!(entry_matches_file(&entry, "src/nested/app.ts"));
        assert!(!entry_matches_file(&entry, "app.js"));
    }

    #[test]
    fn test_mixed_patterns() {
        let patterns = vec!["ts".to_string(), "*.js".to_string()];
        assert!(matches_patterns(&patterns, "app.ts"));
        assert!(matches_patterns(&patterns, "app.js"));
        assert!(matches_patterns(&patterns, "src/app.js"));
        assert!(!matches_patterns(&patterns, "app.tsx"));
    }

    #[test]
    fn test_path_with_directory() {
        let patterns = vec!["ts".to_string()];
        assert!(matches_patterns(&patterns, "src/app.ts"));
        assert!(matches_patterns(&patterns, "src/components/Button.ts"));
    }

    #[test]
    fn test_consistency_formatter_hook_vs_formatter() {
        let entry = FormatterEntry {
            disabled: None,
            command: Some(vec!["prettier".to_string()]),
            environment: None,
            extensions: Some(vec!["ts".to_string(), "tsx".to_string()]),
        };

        assert!(entry_matches_file(&entry, "app.ts"));
        assert!(entry_matches_file(&entry, "app.tsx"));
        assert!(entry_matches_file(&entry, "src/app.ts"));
        assert!(!entry_matches_file(&entry, "app.js"));
        assert!(!entry_matches_file(&entry, "app.rb"));
    }
}
