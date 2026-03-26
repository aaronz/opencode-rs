use crate::language::Language;
use crate::types::Diagnostic;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct LspServer {
    language: Language,
    root: PathBuf,
    diagnostics: HashMap<String, Vec<Diagnostic>>,
}

impl LspServer {
    pub fn new(language: Language, root: PathBuf) -> Self {
        Self {
            language,
            root,
            diagnostics: HashMap::new(),
        }
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn update_diagnostics(&mut self, uri: String, diags: Vec<Diagnostic>) {
        self.diagnostics.insert(uri, diags);
    }

    pub fn get_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        self.diagnostics.get(uri).cloned().unwrap_or_default()
    }

    pub fn all_diagnostics(&self) -> &HashMap<String, Vec<Diagnostic>> {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self, uri: &str) {
        self.diagnostics.remove(uri);
    }
}
