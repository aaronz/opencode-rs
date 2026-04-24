use serde::{Deserialize, Serialize};

use crate::types::{Location, Position, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencesParams {
    pub uri: String,
    pub position: Position,
    pub context: ReferencesContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferencesContext {
    pub include_declaration: bool,
}

impl Default for ReferencesContext {
    fn default() -> Self {
        Self {
            include_declaration: true,
        }
    }
}

impl ReferencesContext {
    pub fn new(include_declaration: bool) -> Self {
        Self {
            include_declaration,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencesResult {
    pub locations: Vec<Location>,
}

impl ReferencesResult {
    pub fn new(locations: Vec<Location>) -> Self {
        Self { locations }
    }

    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.locations.len()
    }
}

pub fn find_references_in_document(
    source: &str,
    word: &str,
    uri: &str,
    include_declaration: bool,
) -> Vec<Location> {
    let lines: Vec<&str> = source.lines().collect();
    let mut locations = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let mut col = 0;
        while let Some(pos) = line[col..].find(word) {
            let abs_col = col + pos;
            locations.push(Location {
                uri: uri.to_string(),
                range: Range {
                    start: Position {
                        line: line_idx as u32,
                        character: abs_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (abs_col + word.len()) as u32,
                    },
                },
            });
            col += pos + 1;
        }
    }

    if !include_declaration {
        locations = locations.into_iter().skip(1).collect();
    }

    locations
}

pub fn find_references_workspace(
    sources: &[(String, String)],
    word: &str,
    include_declaration: bool,
) -> Vec<Location> {
    let mut all_locations = Vec::new();

    for (uri, source) in sources {
        let locations = find_references_in_document(source, word, uri, true);
        all_locations.extend(locations);
    }

    if !include_declaration && !all_locations.is_empty() {
        all_locations.remove(0);
    }

    all_locations
}

pub fn filter_references_by_file(locations: &[Location], uri: &str) -> Vec<Location> {
    locations
        .iter()
        .filter(|loc| loc.uri == uri)
        .cloned()
        .collect()
}

pub fn get_declaration_location(source: &str, word: &str, uri: &str) -> Option<Location> {
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        if line.contains(&format!("fn {}", word))
            || line.contains(&format!("pub fn {}", word))
            || line.contains(&format!("struct {}", word))
            || line.contains(&format!("pub struct {}", word))
            || line.contains(&format!("enum {}", word))
            || line.contains(&format!("pub enum {}", word))
            || line.contains(&format!("trait {}", word))
            || line.contains(&format!("pub trait {}", word))
            || line.contains(&format!("const {}", word))
            || line.contains(&format!("pub const {}", word))
            || line.contains(&format!("static {}", word))
            || line.contains(&format!("pub static {}", word))
        {
            let start_col = line.find(word).unwrap_or(0);
            return Some(Location {
                uri: uri.to_string(),
                range: Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + word.len()) as u32,
                    },
                },
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_references_params_default_context() {
        let params = ReferencesParams {
            uri: "file:///test.rs".to_string(),
            position: Position {
                line: 0,
                character: 0,
            },
            context: ReferencesContext::default(),
        };

        assert_eq!(params.uri, "file:///test.rs");
        assert!(params.context.include_declaration);
    }

    #[test]
    fn test_references_context_new() {
        let context = ReferencesContext::new(false);
        assert!(!context.include_declaration);
    }

    #[test]
    fn test_references_result_new_and_properties() {
        let locations = vec![
            Location {
                uri: "file:///test.rs".to_string(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 3,
                    },
                },
            },
            Location {
                uri: "file:///test.rs".to_string(),
                range: Range {
                    start: Position {
                        line: 1,
                        character: 5,
                    },
                    end: Position {
                        line: 1,
                        character: 8,
                    },
                },
            },
        ];

        let result = ReferencesResult::new(locations);
        assert_eq!(result.len(), 2);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_references_result_empty() {
        let result = ReferencesResult::new(vec![]);
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_references_in_document_returns_all_locations() {
        let source = "fn my_func() {}\n\
fn bar() {\n\
    my_func();\n\
}\n\
fn baz() {\n\
    my_func();\n\
}";

        let locations = find_references_in_document(source, "my_func", "file:///test.rs", true);

        assert_eq!(locations.len(), 3);
        assert_eq!(locations[0].uri, "file:///test.rs");
        assert_eq!(locations[0].range.start.line, 0);
        assert_eq!(locations[1].range.start.line, 2);
        assert_eq!(locations[2].range.start.line, 5);
    }

    #[test]
    fn test_find_references_in_document_includes_declaration() {
        let source = "fn my_func() {}\n\
fn bar() {\n\
    my_func();\n\
}";

        let with_decl = find_references_in_document(source, "my_func", "file:///test.rs", true);
        let without_decl = find_references_in_document(source, "my_func", "file:///test.rs", false);

        assert_eq!(with_decl.len(), 2);
        assert_eq!(without_decl.len(), 1);
        assert_eq!(without_decl[0].range.start.line, 2);
    }

    #[test]
    fn test_find_references_in_document_no_matches() {
        let source = r#"fn bar() {}
fn baz() {
    qux();
}"#;

        let locations = find_references_in_document(source, "my_foo", "file:///test.rs", true);
        assert!(locations.is_empty());
    }

    #[test]
    fn test_find_references_workspace_multiple_files() {
        let sources = vec![
            (
                "file:///test1.rs".to_string(),
                "fn my_func() {}\n\
fn bar() { my_func(); }"
                    .to_string(),
            ),
            (
                "file:///test2.rs".to_string(),
                "fn baz() { my_func(); }".to_string(),
            ),
        ];

        let locations = find_references_workspace(&sources, "my_func", true);

        assert_eq!(locations.len(), 3);
    }

    #[test]
    fn test_find_references_workspace_excludes_declaration() {
        let sources = vec![
            (
                "file:///test1.rs".to_string(),
                "fn my_func() {}\n\
fn bar() { my_func(); }"
                    .to_string(),
            ),
            (
                "file:///test2.rs".to_string(),
                "fn baz() { my_func(); }".to_string(),
            ),
        ];

        let locations = find_references_workspace(&sources, "my_func", false);

        assert_eq!(locations.len(), 2);
    }

    #[test]
    fn test_filter_references_by_file() {
        let locations = vec![
            Location {
                uri: "file:///test1.rs".to_string(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 3,
                    },
                },
            },
            Location {
                uri: "file:///test2.rs".to_string(),
                range: Range {
                    start: Position {
                        line: 1,
                        character: 5,
                    },
                    end: Position {
                        line: 1,
                        character: 8,
                    },
                },
            },
            Location {
                uri: "file:///test1.rs".to_string(),
                range: Range {
                    start: Position {
                        line: 2,
                        character: 10,
                    },
                    end: Position {
                        line: 2,
                        character: 13,
                    },
                },
            },
        ];

        let filtered = filter_references_by_file(&locations, "file:///test1.rs");
        assert_eq!(filtered.len(), 2);

        let filtered2 = filter_references_by_file(&locations, "file:///test2.rs");
        assert_eq!(filtered2.len(), 1);
    }

    #[test]
    fn test_filter_references_by_file_no_match() {
        let locations = vec![Location {
            uri: "file:///test1.rs".to_string(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 3,
                },
            },
        }];

        let filtered = filter_references_by_file(&locations, "file:///test999.rs");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_get_declaration_location_function() {
        let source = r#"fn my_function() {}
fn other() {
    my_function();
}"#;

        let decl = get_declaration_location(source, "my_function", "file:///test.rs");
        assert!(decl.is_some());
        let decl = decl.unwrap();
        assert_eq!(decl.uri, "file:///test.rs");
        assert_eq!(decl.range.start.line, 0);
    }

    #[test]
    fn test_get_declaration_location_struct() {
        let source = r#"struct MyStruct {
    field: i32,
}
impl MyStruct {
    fn new() -> Self {}
}"#;

        let decl = get_declaration_location(source, "MyStruct", "file:///test.rs");
        assert!(decl.is_some());
        let decl = decl.unwrap();
        assert_eq!(decl.range.start.line, 0);
    }

    #[test]
    fn test_get_declaration_location_not_found() {
        let source = r#"fn other() {}"#;

        let decl = get_declaration_location(source, "foo", "file:///test.rs");
        assert!(decl.is_none());
    }

    #[test]
    fn test_references_params_with_explicit_context() {
        let params = ReferencesParams {
            uri: "file:///test.rs".to_string(),
            position: Position {
                line: 5,
                character: 10,
            },
            context: ReferencesContext::new(false),
        };

        assert_eq!(params.position.line, 5);
        assert_eq!(params.position.character, 10);
        assert!(!params.context.include_declaration);
    }

    #[test]
    fn test_location_with_full_range() {
        let location = Location {
            uri: "file:///test.rs".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 5,
                },
                end: Position {
                    line: 10,
                    character: 15,
                },
            },
        };

        assert_eq!(location.range.start.line, 10);
        assert_eq!(location.range.end.character, 15);
    }
}
