use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConfig {
    pub rows: u16,
    pub cols: u16,
    pub env: std::collections::HashMap<String, String>,
}

pub struct PtySession {
    config: PtyConfig,
    cwd: PathBuf,
}

impl PtySession {
    pub fn new(config: PtyConfig, cwd: PathBuf) -> Self {
        Self { config, cwd }
    }

    pub fn config(&self) -> &PtyConfig {
        &self.config
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }
}
