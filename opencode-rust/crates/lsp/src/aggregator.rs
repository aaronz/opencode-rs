use crate::types::{Diagnostic, Severity};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct DiagnosticAggregator {
    diagnostics_by_file: HashMap<PathBuf, Vec<Diagnostic>>,
}

impl DiagnosticAggregator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest(&mut self, path: &Path, diagnostics: Vec<Diagnostic>) {
        let mut dedupe = HashSet::new();
        let mut merged = Vec::new();

        if let Some(existing) = self.diagnostics_by_file.get(path) {
            for diagnostic in existing {
                let key = (
                    diagnostic.severity,
                    diagnostic.message.clone(),
                    diagnostic.range.start.line,
                    diagnostic.range.start.character,
                    diagnostic.range.end.line,
                    diagnostic.range.end.character,
                );
                if dedupe.insert(key) {
                    merged.push(diagnostic.clone());
                }
            }
        }

        for mut diagnostic in diagnostics {
            diagnostic.file_path = Some(path.display().to_string());
            let key = (
                diagnostic.severity,
                diagnostic.message.clone(),
                diagnostic.range.start.line,
                diagnostic.range.start.character,
                diagnostic.range.end.line,
                diagnostic.range.end.character,
            );
            if dedupe.insert(key) {
                merged.push(diagnostic);
            }
        }

        self.diagnostics_by_file.insert(path.to_path_buf(), merged);
    }

    pub fn clear_for_file(&mut self, path: &Path) {
        self.diagnostics_by_file.remove(path);
    }

    pub fn get_diagnostics_for_file(&self, path: &Path) -> Vec<Diagnostic> {
        self.diagnostics_by_file
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_total_diagnostic_count(&self) -> usize {
        self.diagnostics_by_file.values().map(Vec::len).sum()
    }

    pub fn get_diagnostics_summary(&self) -> HashMap<Severity, usize> {
        let mut summary = HashMap::new();
        for diagnostics in self.diagnostics_by_file.values() {
            for diagnostic in diagnostics {
                *summary.entry(diagnostic.severity).or_insert(0) += 1;
            }
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Position, Range};

    fn sample_diagnostic(message: &str, severity: Severity) -> Diagnostic {
        Diagnostic {
            severity,
            message: message.to_string(),
            range: Range {
                start: Position {
                    line: 1,
                    character: 1,
                },
                end: Position {
                    line: 1,
                    character: 4,
                },
            },
            source: Some("rust-analyzer".to_string()),
            file_path: None,
        }
    }

    #[test]
    fn deduplicates_diagnostics_per_file() {
        let mut agg = DiagnosticAggregator::new();
        let path = PathBuf::from("src/main.rs");

        let d1 = sample_diagnostic("oops", Severity::Error);
        let d2 = sample_diagnostic("oops", Severity::Error);
        agg.ingest(&path, vec![d1, d2]);

        assert_eq!(agg.get_diagnostics_for_file(&path).len(), 1);
    }

    #[test]
    fn returns_summary_counts() {
        let mut agg = DiagnosticAggregator::new();
        agg.ingest(
            &PathBuf::from("src/main.rs"),
            vec![
                sample_diagnostic("err", Severity::Error),
                sample_diagnostic("warn", Severity::Warning),
            ],
        );

        assert_eq!(agg.get_total_diagnostic_count(), 2);
        let summary = agg.get_diagnostics_summary();
        assert_eq!(summary.get(&Severity::Error), Some(&1));
        assert_eq!(summary.get(&Severity::Warning), Some(&1));
    }
}
