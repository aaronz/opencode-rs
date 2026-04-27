use std::path::PathBuf;
use std::sync::Mutex;

mod types;

pub use types::PathOverride;

static PATH_OVERRIDE: Mutex<Option<PathOverride>> = Mutex::new(None);

pub fn override_paths(override_: PathOverride) {
    *PATH_OVERRIDE.lock().unwrap() = Some(override_);
}

pub fn clear_path_override() {
    *PATH_OVERRIDE.lock().unwrap() = None;
}

fn get_override() -> Option<PathOverride> {
    PATH_OVERRIDE.lock().unwrap().clone()
}

pub struct Paths;

impl Paths {
    fn project_dirs() -> Option<directories::ProjectDirs> {
        directories::ProjectDirs::from("ai", "opencode", "opencode-rs")
    }

    fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME").ok().map(PathBuf::from)
    }

    pub fn config_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.config_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_CONFIG_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::project_dirs()
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                Self::home_dir()
                    .unwrap_or_default()
                    .join(".config/opencode-rs")
            })
    }

    pub fn data_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.data_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_DATA_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::project_dirs()
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| {
                Self::home_dir()
                    .unwrap_or_default()
                    .join(".local/share/opencode-rs")
            })
    }

    pub fn cache_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.cache_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_CACHE_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::project_dirs()
            .map(|d| d.cache_dir().to_path_buf())
            .unwrap_or_else(|| {
                Self::home_dir()
                    .unwrap_or_default()
                    .join(".cache/opencode-rs")
            })
    }

    pub fn log_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.log_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_LOG_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::config_dir().join("logs")
    }

    pub fn log_file() -> PathBuf {
        Self::log_dir().join("opencode-rs.log")
    }

    pub fn schema_cache_dir() -> PathBuf {
        Self::config_dir().join("schemas")
    }

    pub fn secrets_path() -> PathBuf {
        Self::data_dir().join("secrets.json")
    }

    pub fn credentials_path() -> PathBuf {
        Self::data_dir().join("credentials.enc.json")
    }

    pub fn credentials_key_path() -> PathBuf {
        Self::data_dir().join("credentials.key")
    }

    pub fn oauth_sessions_path() -> PathBuf {
        Self::data_dir().join("oauth_sessions.json")
    }

    pub fn crash_dump_dir() -> PathBuf {
        Self::config_dir().join("crashes")
    }

    pub fn project_local_dir() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        for ancestor in cwd.ancestors() {
            let opencode_rs_dir = ancestor.join(".opencode-rs");
            if opencode_rs_dir.is_dir() {
                return Some(opencode_rs_dir);
            }
        }
        None
    }

    pub fn find_project_local_dir() -> Option<PathBuf> {
        Self::project_local_dir()
    }

    pub fn ensure_project_local_dir() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        let opencode_rs_dir = cwd.join(".opencode-rs");
        if std::fs::create_dir_all(&opencode_rs_dir).is_ok() {
            Some(opencode_rs_dir)
        } else {
            None
        }
    }

    pub fn project_tools_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("tools"))
    }

    pub fn project_skills_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("skills"))
    }

    pub fn project_workflows_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("workflows"))
    }

    pub fn project_plugins_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("plugins"))
    }

    pub fn project_commands_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("commands"))
    }

    pub fn project_agents_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("agents"))
    }

    pub fn project_themes_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("themes"))
    }

    pub fn project_modes_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("modes"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_config_dir_contains_opencode_rs() {
        let config_dir = Paths::config_dir();
        let config_str = config_dir.to_string_lossy();
        assert!(
            config_str.contains("opencode-rs"),
            "config_dir should contain 'opencode-rs', got: {}",
            config_str
        );
        assert!(
            !config_str.contains("/opencode/"),
            "config_dir should NOT contain '/opencode/' (original project), got: {}",
            config_str
        );
    }

    #[test]
    fn test_paths_log_file_contains_opencode_rs() {
        let log_file = Paths::log_file();
        let log_str = log_file.to_string_lossy();
        assert!(
            log_str.contains("opencode-rs"),
            "log_file should contain 'opencode-rs', got: {}",
            log_str
        );
    }

    #[test]
    fn test_paths_schema_cache_uses_opencode_rs() {
        let schema_dir = Paths::schema_cache_dir();
        let schema_str = schema_dir.to_string_lossy();
        assert!(
            schema_str.contains("opencode-rs"),
            "schema_cache_dir should contain 'opencode-rs', got: {}",
            schema_str
        );
    }

    #[test]
    fn test_paths_data_dir_uses_opencode_rs() {
        let data_dir = Paths::data_dir();
        let data_str = data_dir.to_string_lossy();
        assert!(
            data_str.contains("opencode-rs"),
            "data_dir should contain 'opencode-rs', got: {}",
            data_str
        );
    }

    #[test]
    fn test_paths_cache_dir_uses_opencode_rs() {
        let cache_dir = Paths::cache_dir();
        let cache_str = cache_dir.to_string_lossy();
        assert!(
            cache_str.contains("opencode-rs"),
            "cache_dir should contain 'opencode-rs', got: {}",
            cache_str
        );
    }

    #[test]
    fn test_paths_secrets_path_uses_opencode_rs() {
        let secrets = Paths::secrets_path();
        let secrets_str = secrets.to_string_lossy();
        assert!(
            secrets_str.contains("opencode-rs"),
            "secrets_path should contain 'opencode-rs', got: {}",
            secrets_str
        );
    }

    #[test]
    fn test_paths_log_dir_is_under_config_dir() {
        let log_dir = Paths::log_dir();
        let config_dir = Paths::config_dir();
        if config_dir.to_string_lossy().contains(".config/opencode-rs") {
            assert!(
                log_dir.starts_with(&config_dir),
                "log_dir should be under config_dir when using .config fallback. log_dir: {}, config_dir: {}",
                log_dir.display(),
                config_dir.display()
            );
        } else {
            assert!(
                log_dir.ends_with("logs"),
                "log_dir should end with 'logs'. log_dir: {}",
                log_dir.display()
            );
        }
    }

    #[test]
    fn test_paths_credentials_path_uses_opencode_rs() {
        let creds = Paths::credentials_path();
        let creds_str = creds.to_string_lossy();
        assert!(
            creds_str.contains("opencode-rs"),
            "credentials_path should contain 'opencode-rs', got: {}",
            creds_str
        );
    }

    #[test]
    fn test_paths_oauth_sessions_path_uses_opencode_rs() {
        let oauth = Paths::oauth_sessions_path();
        let oauth_str = oauth.to_string_lossy();
        assert!(
            oauth_str.contains("opencode-rs"),
            "oauth_sessions_path should contain 'opencode-rs', got: {}",
            oauth_str
        );
    }

    #[test]
    fn test_paths_crash_dump_dir_uses_opencode_rs() {
        let crash = Paths::crash_dump_dir();
        let crash_str = crash.to_string_lossy();
        assert!(
            crash_str.contains("opencode-rs"),
            "crash_dump_dir should contain 'opencode-rs', got: {}",
            crash_str
        );
    }

    #[test]
    fn test_path_override_works() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        override_paths(PathOverride {
            config_dir: Some(temp_path.clone()),
            data_dir: None,
            cache_dir: None,
            log_dir: None,
        });

        assert_eq!(Paths::config_dir(), temp_path);

        clear_path_override();
    }
}