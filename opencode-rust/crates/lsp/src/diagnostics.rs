use serde::{Deserialize, Serialize};

pub use crate::types::{Diagnostic, Position, Range, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl From<i32> for DiagnosticSeverity {
    fn from(value: i32) -> Self {
        match value {
            1 => DiagnosticSeverity::Error,
            2 => DiagnosticSeverity::Warning,
            3 => DiagnosticSeverity::Info,
            4 => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Warning,
        }
    }
}

impl From<Severity> for DiagnosticSeverity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Error => DiagnosticSeverity::Error,
            Severity::Warning => DiagnosticSeverity::Warning,
            Severity::Information => DiagnosticSeverity::Info,
            Severity::Hint => DiagnosticSeverity::Hint,
        }
    }
}

impl From<DiagnosticSeverity> for Severity {
    fn from(severity: DiagnosticSeverity) -> Self {
        match severity {
            DiagnosticSeverity::Error => Severity::Error,
            DiagnosticSeverity::Warning => Severity::Warning,
            DiagnosticSeverity::Info => Severity::Information,
            DiagnosticSeverity::Hint => Severity::Hint,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn handle_publish_diagnostics(params: &PublishDiagnosticsParams) -> Vec<Diagnostic> {
    params.diagnostics.clone()
}

pub fn filter_diagnostics_by_severity(
    diagnostics: &[Diagnostic],
    severity: Severity,
) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| d.severity == severity)
        .cloned()
        .collect()
}

pub fn filter_diagnostics_by_file(diagnostics: &[Diagnostic], file_path: &str) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| d.file_path.as_deref() == Some(file_path))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_struct_has_required_fields() {
        let diagnostic = Diagnostic {
            severity: Severity::Error,
            message: "test error".to_string(),
            range: Range {
                start: Position {
                    line: 1,
                    character: 2,
                },
                end: Position {
                    line: 1,
                    character: 10,
                },
            },
            source: Some("test".to_string()),
            file_path: Some("/test/file.rs".to_string()),
        };

        assert_eq!(diagnostic.severity, Severity::Error);
        assert_eq!(diagnostic.message, "test error");
        assert_eq!(diagnostic.range.start.line, 1);
        assert_eq!(diagnostic.range.start.character, 2);
        assert_eq!(diagnostic.range.end.line, 1);
        assert_eq!(diagnostic.range.end.character, 10);
        assert_eq!(diagnostic.source, Some("test".to_string()));
        assert_eq!(diagnostic.file_path, Some("/test/file.rs".to_string()));
    }

    #[test]
    fn test_diagnostic_severity_conversion() {
        let severity_i32: Severity = 1i32.into();
        assert_eq!(severity_i32, Severity::Error);

        let severity_i32: Severity = 2i32.into();
        assert_eq!(severity_i32, Severity::Warning);

        let severity_i32: Severity = 3i32.into();
        assert_eq!(severity_i32, Severity::Information);

        let severity_i32: Severity = 4i32.into();
        assert_eq!(severity_i32, Severity::Hint);

        let severity_i32: Severity = 99i32.into();
        assert_eq!(severity_i32, Severity::Warning);
    }

    #[test]
    fn test_diagnostic_severity_from_lsp_level() {
        let ds: DiagnosticSeverity = 1i32.into();
        assert_eq!(ds, DiagnosticSeverity::Error);

        let ds: DiagnosticSeverity = 2i32.into();
        assert_eq!(ds, DiagnosticSeverity::Warning);

        let ds: DiagnosticSeverity = 3i32.into();
        assert_eq!(ds, DiagnosticSeverity::Info);

        let ds: DiagnosticSeverity = 4i32.into();
        assert_eq!(ds, DiagnosticSeverity::Hint);
    }

    #[test]
    fn test_diagnostic_severity_round_trip() {
        let original = DiagnosticSeverity::Error;
        let severity: Severity = original.into();
        let back: DiagnosticSeverity = severity.into();
        assert_eq!(back, DiagnosticSeverity::Error);

        let original = DiagnosticSeverity::Warning;
        let severity: Severity = original.into();
        let back: DiagnosticSeverity = severity.into();
        assert_eq!(back, DiagnosticSeverity::Warning);

        let original = DiagnosticSeverity::Info;
        let severity: Severity = original.into();
        let back: DiagnosticSeverity = severity.into();
        assert_eq!(back, DiagnosticSeverity::Info);

        let original = DiagnosticSeverity::Hint;
        let severity: Severity = original.into();
        let back: DiagnosticSeverity = severity.into();
        assert_eq!(back, DiagnosticSeverity::Hint);
    }

    #[test]
    fn test_publish_diagnostics_handler_returns_diagnostics() {
        let diagnostics = vec![
            Diagnostic {
                severity: Severity::Error,
                message: "error 1".to_string(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                source: None,
                file_path: None,
            },
            Diagnostic {
                severity: Severity::Warning,
                message: "warning 1".to_string(),
                range: Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 1,
                        character: 5,
                    },
                },
                source: None,
                file_path: None,
            },
        ];

        let params = PublishDiagnosticsParams {
            uri: "file:///test.rs".to_string(),
            diagnostics: diagnostics.clone(),
        };

        let result = handle_publish_diagnostics(&params);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].message, "error 1");
        assert_eq!(result[1].message, "warning 1");
    }

    #[test]
    fn test_publish_diagnostics_handler_returns_copy() {
        let params = PublishDiagnosticsParams {
            uri: "file:///test.rs".to_string(),
            diagnostics: vec![],
        };

        let mut result = handle_publish_diagnostics(&params);
        result.push(Diagnostic {
            severity: Severity::Error,
            message: "added later".to_string(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            source: None,
            file_path: None,
        });

        assert_eq!(result.len(), 1);
        assert_eq!(params.diagnostics.len(), 0);
    }

    #[test]
    fn test_filter_diagnostics_by_severity_filters_correctly() {
        let diagnostics = vec![
            Diagnostic {
                severity: Severity::Error,
                message: "error 1".to_string(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                source: None,
                file_path: None,
            },
            Diagnostic {
                severity: Severity::Warning,
                message: "warning 1".to_string(),
                range: Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 1,
                        character: 5,
                    },
                },
                source: None,
                file_path: None,
            },
            Diagnostic {
                severity: Severity::Error,
                message: "error 2".to_string(),
                range: Range {
                    start: Position {
                        line: 2,
                        character: 0,
                    },
                    end: Position {
                        line: 2,
                        character: 5,
                    },
                },
                source: None,
                file_path: None,
            },
        ];

        let errors = filter_diagnostics_by_severity(&diagnostics, Severity::Error);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].message, "error 1");
        assert_eq!(errors[1].message, "error 2");

        let warnings = filter_diagnostics_by_severity(&diagnostics, Severity::Warning);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "warning 1");
    }

    #[test]
    fn test_filter_diagnostics_by_file_filters_correctly() {
        let diagnostics = vec![
            Diagnostic {
                severity: Severity::Error,
                message: "error in file1".to_string(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                source: None,
                file_path: Some("/path/file1.rs".to_string()),
            },
            Diagnostic {
                severity: Severity::Error,
                message: "error in file2".to_string(),
                range: Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 1,
                        character: 5,
                    },
                },
                source: None,
                file_path: Some("/path/file2.rs".to_string()),
            },
            Diagnostic {
                severity: Severity::Warning,
                message: "warning in file1".to_string(),
                range: Range {
                    start: Position {
                        line: 2,
                        character: 0,
                    },
                    end: Position {
                        line: 2,
                        character: 5,
                    },
                },
                source: None,
                file_path: Some("/path/file1.rs".to_string()),
            },
        ];

        let file1_diags = filter_diagnostics_by_file(&diagnostics, "/path/file1.rs");
        assert_eq!(file1_diags.len(), 2);
        assert_eq!(file1_diags[0].message, "error in file1");
        assert_eq!(file1_diags[1].message, "warning in file1");

        let file2_diags = filter_diagnostics_by_file(&diagnostics, "/path/file2.rs");
        assert_eq!(file2_diags.len(), 1);
        assert_eq!(file2_diags[0].message, "error in file2");

        let no_match = filter_diagnostics_by_file(&diagnostics, "/path/file3.rs");
        assert_eq!(no_match.len(), 0);
    }

    #[test]
    fn test_empty_diagnostics_list() {
        let diagnostics: Vec<Diagnostic> = vec![];
        let params = PublishDiagnosticsParams {
            uri: "file:///test.rs".to_string(),
            diagnostics,
        };

        let result = handle_publish_diagnostics(&params);
        assert!(result.is_empty());

        let filtered = filter_diagnostics_by_severity(&result, Severity::Error);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_position_uses_line_and_column() {
        let pos = Position {
            line: 10,
            character: 5,
        };
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_range_uses_start_and_end() {
        let range = Range {
            start: Position {
                line: 1,
                character: 2,
            },
            end: Position {
                line: 3,
                character: 4,
            },
        };
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.character, 2);
        assert_eq!(range.end.line, 3);
        assert_eq!(range.end.character, 4);
    }

    #[test]
    fn test_diagnostic_clone() {
        let original = Diagnostic {
            severity: Severity::Error,
            message: "test".to_string(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            source: Some("test".to_string()),
            file_path: Some("/test.rs".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(original.severity, cloned.severity);
        assert_eq!(original.message, cloned.message);
        assert_eq!(original.range.start.line, cloned.range.start.line);
    }
}
