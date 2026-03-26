use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    pub version: String,
    pub install_path: PathBuf,
    pub data_path: PathBuf,
    pub config_path: PathBuf,
}

pub struct InstallationManager {
    info: InstallationInfo,
}

impl InstallationManager {
    pub fn new() -> Self {
        Self {
            info: InstallationInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                install_path: PathBuf::from("/usr/local/bin"),
                data_path: directories::ProjectDirs::from("com", "opencode", "rs")
                    .map(|d| d.data_dir().to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("~/.local/share/opencode-rs")),
                config_path: directories::ProjectDirs::from("com", "opencode", "rs")
                    .map(|d| d.config_dir().to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("~/.config/opencode-rs")),
            },
        }
    }

    pub fn version(&self) -> &str {
        &self.info.version
    }

    pub fn info(&self) -> &InstallationInfo {
        &self.info
    }
}

impl Default for InstallationManager {
    fn default() -> Self {
        Self::new()
    }
}
