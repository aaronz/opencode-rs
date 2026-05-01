use std::path::PathBuf;

pub trait PathResolver: Send + Sync {
    fn user_config_dir(&self) -> PathBuf;
    fn user_state_dir(&self) -> PathBuf;
    fn user_log_dir(&self) -> PathBuf;
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
}
