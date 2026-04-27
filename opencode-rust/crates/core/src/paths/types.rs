use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct PathOverride {
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
}