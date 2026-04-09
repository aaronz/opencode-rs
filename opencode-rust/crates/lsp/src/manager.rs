use crate::aggregator::DiagnosticAggregator;
use crate::client::LspClient;
use crate::language::Language;
use crate::types::{Diagnostic, Severity};
use futures::future::join_all;
use opencode_core::OpenCodeError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct LspManager {
    root: PathBuf,
    clients: HashMap<Language, LspClient>,
    file_to_language: HashMap<PathBuf, Language>,
    aggregator: DiagnosticAggregator,
}

impl LspManager {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            clients: HashMap::new(),
            file_to_language: HashMap::new(),
            aggregator: DiagnosticAggregator::new(),
        }
    }

    pub async fn start_for_file(&mut self, path: &Path) -> Result<(), OpenCodeError> {
        let language = Language::detect(path);
        if language == Language::Unknown {
            return Ok(());
        }

        if !self.clients.contains_key(&language) {
            let mut client = LspClient::new();
            if let Some(command) = language.server_command() {
                client.start(command, &self.root).await?;
                self.clients.insert(language.clone(), client);
            }
        }

        self.file_to_language
            .insert(path.to_path_buf(), language.clone());

        if let Some(client) = self.clients.get_mut(&language) {
            let diagnostics = client
                .get_diagnostics(path.to_string_lossy().as_ref())
                .await?;
            self.aggregator.ingest(path, diagnostics);
        }

        Ok(())
    }

    pub async fn start_for_files_parallel(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<(), OpenCodeError> {
        let mut starts = Vec::new();
        for path in paths {
            let language = Language::detect(path);
            if language == Language::Unknown || self.clients.contains_key(&language) {
                self.file_to_language.insert(path.clone(), language);
                continue;
            }
            if let Some(command) = language.server_command() {
                let root = self.root.clone();
                let language_clone = language.clone();
                let command = command.to_string();
                starts.push(async move {
                    let mut client = LspClient::new();
                    client.start(&command, &root).await?;
                    Ok::<(Language, LspClient), OpenCodeError>((language_clone, client))
                });
            }
        }

        for result in join_all(starts).await {
            let (language, client) = result?;
            self.clients.entry(language).or_insert(client);
        }

        for path in paths {
            self.file_to_language
                .insert(path.clone(), Language::detect(path));
        }

        Ok(())
    }

    pub async fn stop_for_file(&mut self, path: &Path) -> Result<(), OpenCodeError> {
        let Some(language) = self.file_to_language.remove(path) else {
            return Ok(());
        };

        self.aggregator.clear_for_file(path);

        let still_used = self.file_to_language.values().any(|lang| *lang == language);
        if !still_used {
            if let Some(mut client) = self.clients.remove(&language) {
                client.shutdown().await?;
            }
        }

        Ok(())
    }

    pub fn get_diagnostics_for_file(&self, path: &Path) -> Vec<Diagnostic> {
        self.aggregator.get_diagnostics_for_file(path)
    }

    pub fn record_diagnostics(&mut self, path: &Path, diagnostics: Vec<Diagnostic>) {
        self.aggregator.ingest(path, diagnostics);
    }

    pub fn get_total_diagnostic_count(&self) -> usize {
        self.aggregator.get_total_diagnostic_count()
    }

    pub fn get_diagnostics_summary(&self) -> HashMap<Severity, usize> {
        self.aggregator.get_diagnostics_summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unknown_extension_noop() {
        let root = std::env::current_dir().unwrap();
        let mut manager = LspManager::new(root);
        manager
            .start_for_file(Path::new("README.unknown_ext"))
            .await
            .unwrap();
        assert_eq!(manager.get_total_diagnostic_count(), 0);
    }
}
