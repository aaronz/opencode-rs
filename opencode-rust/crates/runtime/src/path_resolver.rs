use std::path::{Path, PathBuf};

pub trait PathResolver: Send + Sync {
    fn user_config_dir(&self) -> PathBuf;
    fn user_state_dir(&self) -> PathBuf;
    fn user_log_dir(&self) -> PathBuf;
    fn project_config_dir(&self, workspace: &Path) -> PathBuf;
    fn project_state_dir(&self, workspace: &Path) -> PathBuf;
}

pub struct DefaultPathResolver;

impl DefaultPathResolver {
    pub fn new() -> Self {
        Self
    }

    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencode-rs")
    }

    fn state_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencode-rs")
    }

    fn log_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencode-rs")
            .join("logs")
    }

    fn project_config_dir_for(workspace: &Path) -> PathBuf {
        workspace.join(".opencode-rs")
    }

    fn project_state_dir_for(workspace: &Path) -> PathBuf {
        workspace.join(".opencode-rs").join("state")
    }
}

impl Default for DefaultPathResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl PathResolver for DefaultPathResolver {
    fn user_config_dir(&self) -> PathBuf {
        Self::config_dir()
    }

    fn user_state_dir(&self) -> PathBuf {
        Self::state_dir()
    }

    fn user_log_dir(&self) -> PathBuf {
        Self::log_dir()
    }

    fn project_config_dir(&self, workspace: &Path) -> PathBuf {
        Self::project_config_dir_for(workspace)
    }

    fn project_state_dir(&self, workspace: &Path) -> PathBuf {
        Self::project_state_dir_for(workspace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path_resolver() {
        let resolver = DefaultPathResolver::new();
        let config_dir = resolver.user_config_dir();
        assert!(config_dir.to_str().unwrap().contains("opencode-rs"));
    }

    #[test]
    fn test_project_config_dir() {
        let resolver = DefaultPathResolver::new();
        let workspace = PathBuf::from("/test/project");
        let config_dir = resolver.project_config_dir(&workspace);
        assert_eq!(config_dir, PathBuf::from("/test/project/.opencode-rs"));
    }

    #[test]
    fn test_project_state_dir() {
        let resolver = DefaultPathResolver::new();
        let workspace = PathBuf::from("/test/project");
        let state_dir = resolver.project_state_dir(&workspace);
        assert_eq!(state_dir, PathBuf::from("/test/project/.opencode-rs/state"));
    }
}
