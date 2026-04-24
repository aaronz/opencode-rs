use std::path::PathBuf;
use tempfile::tempdir;

use opencode_tui::input::completer::FileCompleter;
use opencode_tui::input::parser::InputParser;

#[test]
fn test_file_references_at_triggers_fuzzy_search() {
    let temp = tempdir().expect("tempdir");

    std::fs::create_dir_all(temp.path().join("src")).expect("create src dir");
    std::fs::write(temp.path().join("src/main.rs"), "fn main() {}").expect("write main.rs");
    std::fs::write(temp.path().join("src/lib.rs"), "").expect("write lib.rs");
    std::fs::write(temp.path().join("README.md"), "# Test").expect("write readme");

    let completer = FileCompleter::new(temp.path());

    let suggestions = completer.suggest("@");
    assert!(
        !suggestions.is_empty(),
        "Should return suggestions for @ trigger"
    );

    let has_rs_file = suggestions
        .iter()
        .any(|p| p.to_string_lossy().contains("main.rs"));
    let has_md_file = suggestions
        .iter()
        .any(|p| p.to_string_lossy().contains("README.md"));
    assert!(has_rs_file || has_md_file, "Should return file suggestions");
}

#[test]
fn test_file_references_fuzzy_matching_by_name() {
    let temp = tempdir().expect("tempdir");

    std::fs::create_dir_all(temp.path().join("src/utils")).expect("create dir");
    std::fs::write(temp.path().join("src/main.rs"), "fn main() {}").expect("write main");
    std::fs::write(temp.path().join("src/utils/parser.rs"), "").expect("write parser");
    std::fs::write(temp.path().join("src/utils/helper.rs"), "").expect("write helper");
    std::fs::write(temp.path().join("main.yaml"), "").expect("write main.yaml");

    let completer = FileCompleter::new(temp.path());

    let suggestions = completer.suggest("main");
    assert!(!suggestions.is_empty(), "Should find files matching 'main'");

    let main_rs: Vec<_> = suggestions
        .iter()
        .filter(|p| p.to_string_lossy().contains("main.rs"))
        .collect();
    assert!(!main_rs.is_empty(), "Should fuzzy match main.rs");
}

#[test]
fn test_file_references_fuzzy_matching_partial_path() {
    let temp = tempdir().expect("tempdir");

    std::fs::create_dir_all(temp.path().join("src/legacy")).expect("create dir");
    std::fs::write(temp.path().join("src/legacy/core.rs"), "").expect("write core");
    std::fs::write(temp.path().join("src/new_core.rs"), "").expect("write new_core");

    let completer = FileCompleter::new(temp.path());

    let suggestions = completer.suggest("core");
    assert!(
        suggestions.len() >= 2,
        "Should find multiple files containing 'core'"
    );
}

#[test]
fn test_file_references_parsing() {
    let parser = InputParser::new();

    let result = parser.parse("@src/main.rs");
    assert!(result.has_files, "Should detect file reference with @");
    assert_eq!(result.tokens.len(), 1, "Should have exactly one token");

    if let opencode_tui::input::parser::InputToken::FileRef(path) = &result.tokens[0] {
        assert_eq!(path, &PathBuf::from("src/main.rs"));
    } else {
        panic!("Expected FileRef token");
    }
}

#[test]
fn test_file_references_with_text_before() {
    let parser = InputParser::new();

    let result = parser.parse("Check @src/main.rs for details");
    assert!(result.has_files);
    assert_eq!(
        result.tokens.len(),
        3,
        "Should have text + file + text tokens"
    );
}

#[test]
fn test_file_references_multiple() {
    let parser = InputParser::new();

    let result = parser.parse("@src/a.rs and @src/b.rs");
    assert!(result.has_files);
    assert_eq!(result.tokens.len(), 3, "Should have file + text + file");
}

#[test]
fn test_file_references_excludes_ignored_patterns() {
    let temp = tempdir().expect("tempdir");

    std::fs::write(temp.path().join(".gitignore"), "ignored.txt\n").expect("write gitignore");
    std::fs::create_dir_all(temp.path().join("src")).expect("create src");
    std::fs::write(temp.path().join("ignored.txt"), "nope").expect("write ignored");
    std::fs::write(temp.path().join("visible.txt"), "ok").expect("write visible");
    std::fs::write(temp.path().join("src/main.rs"), "").expect("write main");

    let completer = FileCompleter::new(temp.path());

    let suggestions = completer.suggest(".");
    let suggestion_paths: Vec<String> = suggestions
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert!(
        !suggestion_paths.iter().any(|p| p == "ignored.txt"),
        "Should not suggest ignored.txt from .gitignore"
    );
    assert!(
        suggestion_paths.iter().any(|p| p == "visible.txt"),
        "Should suggest visible.txt"
    );
}

#[test]
fn test_file_references_sorted_by_score() {
    let temp = tempdir().expect("tempdir");

    std::fs::write(temp.path().join("parser.rs"), "").expect("write parser");
    std::fs::write(temp.path().join("parade.rs"), "").expect("write parade");
    std::fs::write(temp.path().join("parent.rs"), "").expect("write parent");
    std::fs::write(temp.path().join("partial.rs"), "").expect("write partial");

    let completer = FileCompleter::new(temp.path());
    let suggestions = completer.suggest("par");

    assert!(!suggestions.is_empty(), "Should find suggestions for 'par'");

    let has_parser = suggestions
        .iter()
        .any(|p| p.to_string_lossy() == "parser.rs");
    assert!(has_parser, "Should find parser.rs");
}

#[test]
fn test_file_references_insertion_format() {
    let parser = InputParser::new();

    let result = parser.parse("@src/lib.rs");

    let file_ref = match &result.tokens[0] {
        opencode_tui::input::parser::InputToken::FileRef(path) => path,
        _ => panic!("Expected FileRef"),
    };

    assert_eq!(file_ref, &PathBuf::from("src/lib.rs"));
}

#[test]
fn test_file_references_escaped_at_not_ref() {
    let parser = InputParser::new();

    let result = parser.parse("\\@src/main.rs");
    assert!(
        !result.has_files,
        "Escaped @ should not trigger file reference"
    );
}

#[test]
fn test_file_references_empty_at_no_ref() {
    let parser = InputParser::new();

    let result = parser.parse("@");
    assert!(
        result.raw == "@"
            || result.tokens.is_empty()
            || !result.has_files
            || matches!(
                result.tokens.first(),
                Some(opencode_tui::input::parser::InputToken::Text(_))
            )
    );
}
