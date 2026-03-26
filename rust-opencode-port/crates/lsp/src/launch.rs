use crate::language::Language;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    pub command: String,
    pub args: Vec<String>,
    pub root: PathBuf,
    pub language: String,
    pub initialization_options: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
}

impl LaunchConfig {
    pub fn for_language(language: &Language, root: PathBuf) -> Option<Self> {
        let command = language.server_command()?.to_string();

        Some(Self {
            command,
            args: Vec::new(),
            root,
            language: language.name().to_lowercase(),
            initialization_options: None,
            settings: None,
        })
    }

    pub fn from_root(root: PathBuf) -> Vec<Self> {
        let languages = Language::detect_from_root(&root);
        languages
            .iter()
            .filter_map(|lang| Self::for_language(lang, root.clone()))
            .collect()
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_initialization_options(mut self, options: serde_json::Value) -> Self {
        self.initialization_options = Some(options);
        self
    }

    pub fn with_settings(mut self, settings: serde_json::Value) -> Self {
        self.settings = Some(settings);
        self
    }
}
