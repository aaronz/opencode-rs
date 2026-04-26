use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct EnvVarConfig {
    pub auto_share: Option<bool>,
    pub config_path: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub disable_autoupdate: Option<bool>,
    pub enable_exa: Option<bool>,
    pub server_password: Option<String>,
}

impl EnvVarConfig {
    pub fn parse() -> Self {
        Self {
            auto_share: parse_bool("OPENCODE_AUTO_SHARE"),
            config_path: parse_path("OPENCODE_CONFIG"),
            config_dir: parse_path("OPENCODE_CONFIG_DIR"),
            disable_autoupdate: parse_bool("OPENCODE_DISABLE_AUTOUPDATE"),
            enable_exa: parse_bool("OPENCODE_ENABLE_EXA"),
            server_password: parse_string("OPENCODE_SERVER_PASSWORD"),
        }
    }

    #[allow(dead_code)]
    pub fn has_any_override(&self) -> bool {
        self.auto_share.is_some()
            || self.config_path.is_some()
            || self.config_dir.is_some()
            || self.disable_autoupdate.is_some()
            || self.enable_exa.is_some()
            || self.server_password.is_some()
    }
}

fn parse_bool(name: &str) -> Option<bool> {
    env::var(name).ok().map(|v| {
        let lower = v.to_lowercase();
        lower == "true" || lower == "1" || lower == "yes" || lower == "on"
    })
}

fn parse_string(name: &str) -> Option<String> {
    let value = env::var(name).ok()?;
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn parse_path(name: &str) -> Option<PathBuf> {
    let value = env::var(name).ok()?;
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bool_truthy_values() {
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("true"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(true));
        });
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("1"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(true));
        });
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("yes"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(true));
        });
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("on"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(true));
        });
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("TRUE"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(true));
        });
    }

    #[test]
    fn test_parse_bool_falsy_values() {
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("false"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(false));
        });
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("0"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(false));
        });
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("no"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(false));
        });
        temp_env::with_var("OPENCODE_TEST_BOOL", Some("off"), || {
            assert_eq!(parse_bool("OPENCODE_TEST_BOOL"), Some(false));
        });
    }

    #[test]
    fn test_parse_bool_not_set() {
        std::env::remove_var("OPENCODE_TEST_BOOL_NOT_SET");
        assert_eq!(parse_bool("OPENCODE_TEST_BOOL_NOT_SET"), None);
    }

    #[test]
    fn test_parse_string_valid() {
        temp_env::with_var("OPENCODE_TEST_STRING", Some("my_password"), || {
            assert_eq!(
                parse_string("OPENCODE_TEST_STRING"),
                Some("my_password".to_string())
            );
        });
    }

    #[test]
    fn test_parse_string_empty() {
        temp_env::with_var("OPENCODE_TEST_STRING", Some(""), || {
            assert_eq!(parse_string("OPENCODE_TEST_STRING"), None);
        });
    }

    #[test]
    fn test_parse_string_not_set() {
        std::env::remove_var("OPENCODE_TEST_STRING_NOT_SET");
        assert_eq!(parse_string("OPENCODE_TEST_STRING_NOT_SET"), None);
    }

    #[test]
    fn test_parse_path_valid() {
        temp_env::with_var("OPENCODE_TEST_PATH", Some("/path/to/config"), || {
            let path = parse_path("OPENCODE_TEST_PATH");
            assert!(path.is_some());
            assert_eq!(path.unwrap(), PathBuf::from("/path/to/config"));
        });
    }

    #[test]
    fn test_parse_path_empty() {
        temp_env::with_var("OPENCODE_TEST_PATH", Some(""), || {
            assert_eq!(parse_path("OPENCODE_TEST_PATH"), None);
        });
    }

    #[test]
    fn test_parse_path_not_set() {
        std::env::remove_var("OPENCODE_TEST_PATH_NOT_SET");
        assert_eq!(parse_path("OPENCODE_TEST_PATH_NOT_SET"), None);
    }

    #[test]
    fn test_env_var_config_parse_all_set() {
        let kvs = vec![
            ("OPENCODE_AUTO_SHARE", Some("true")),
            ("OPENCODE_CONFIG", Some("/custom/config.json")),
            ("OPENCODE_CONFIG_DIR", Some("/custom/dir")),
            ("OPENCODE_DISABLE_AUTOUPDATE", Some("true")),
            ("OPENCODE_ENABLE_EXA", Some("true")),
            ("OPENCODE_SERVER_PASSWORD", Some("secret123")),
        ];
        temp_env::with_vars(kvs, || {
            let config = EnvVarConfig::parse();
            assert_eq!(config.auto_share, Some(true));
            assert_eq!(
                config.config_path,
                Some(PathBuf::from("/custom/config.json"))
            );
            assert_eq!(config.config_dir, Some(PathBuf::from("/custom/dir")));
            assert_eq!(config.disable_autoupdate, Some(true));
            assert_eq!(config.enable_exa, Some(true));
            assert_eq!(config.server_password, Some("secret123".to_string()));
        });
    }

    #[test]
    fn test_env_var_config_parse_none_set() {
        let kvs = vec![
            ("OPENCODE_AUTO_SHARE", None::<&str>),
            ("OPENCODE_CONFIG", None),
            ("OPENCODE_CONFIG_DIR", None),
            ("OPENCODE_DISABLE_AUTOUPDATE", None),
            ("OPENCODE_ENABLE_EXA", None),
            ("OPENCODE_SERVER_PASSWORD", None),
        ];
        temp_env::with_vars(kvs, || {
            let config = EnvVarConfig::parse();
            assert!(config.auto_share.is_none());
            assert!(config.config_path.is_none());
            assert!(config.config_dir.is_none());
            assert!(config.disable_autoupdate.is_none());
            assert!(config.enable_exa.is_none());
            assert!(config.server_password.is_none());
        });
    }

    #[test]
    fn test_has_any_override_with_overrides() {
        temp_env::with_vars(vec![("OPENCODE_AUTO_SHARE", Some("true"))], || {
            let config = EnvVarConfig::parse();
            assert!(config.has_any_override());
        });
    }

    #[test]
    fn test_has_any_override_without_overrides() {
        let config = EnvVarConfig::default();
        assert!(!config.has_any_override());
    }

    #[test]
    fn test_invalid_env_values_are_handled() {
        temp_env::with_vars(
            vec![
                ("OPENCODE_AUTO_SHARE", Some("invalid_bool")),
                ("OPENCODE_CONFIG", Some("")),
            ],
            || {
                let config = EnvVarConfig::parse();
                assert_eq!(config.auto_share, Some(false));
                assert!(config.config_path.is_none());
            },
        );
    }

    #[test]
    fn test_env_config_overrides_take_precedence() {
        use opencode_config::{AutoUpdate, Config};

        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "autoshare": false,
            "autoupdate": true,
            "model": "gpt-4o"
        }"#;
        std::fs::write(&config_path, config_content).unwrap();

        temp_env::with_vars(
            vec![
                ("OPENCODE_AUTO_SHARE", Some("true")),
                ("OPENCODE_DISABLE_AUTOUPDATE", Some("true")),
                ("OPENCODE_ENABLE_EXA", Some("true")),
                ("OPENCODE_SERVER_PASSWORD", Some("env_password")),
                ("OPENCODE_MODEL", None::<&str>),
            ],
            || {
                let config = Config::load(&config_path).unwrap();
                assert_eq!(
                    config.autoshare,
                    Some(true),
                    "OPENCODE_AUTO_SHARE should override config"
                );
                assert_eq!(
                    config.autoupdate,
                    Some(AutoUpdate::Bool(false)),
                    "OPENCODE_DISABLE_AUTOUPDATE should override config"
                );
                assert!(
                    config.experimental.as_ref().and_then(|e| e.enable_exa) == Some(true),
                    "OPENCODE_ENABLE_EXA should override config"
                );
                assert_eq!(
                    config.server.as_ref().and_then(|s| s.password.as_ref()),
                    Some(&"env_password".to_string()),
                    "OPENCODE_SERVER_PASSWORD should override config"
                );
                assert_eq!(
                    config.model,
                    Some("gpt-4o".to_string()),
                    "model from config should be preserved when no env var set"
                );
            },
        );
    }

    #[test]
    fn test_env_config_no_override_preserves_config_values() {
        use opencode_config::{AutoUpdate, Config};

        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "autoshare": true,
            "autoupdate": true,
            "model": "claude-3-5-sonnet"
        }"#;
        std::fs::write(&config_path, config_content).unwrap();

        temp_env::with_vars(
            vec![
                ("OPENCODE_AUTO_SHARE", None::<&str>),
                ("OPENCODE_DISABLE_AUTOUPDATE", None::<&str>),
                ("OPENCODE_ENABLE_EXA", None::<&str>),
                ("OPENCODE_SERVER_PASSWORD", None::<&str>),
                ("OPENCODE_MODEL", Some("gpt-4o")),
            ],
            || {
                let config = Config::load(&config_path).unwrap();
                assert_eq!(
                    config.autoshare,
                    Some(true),
                    "config value should be preserved when no env var set"
                );
                assert_eq!(
                    config.autoupdate,
                    Some(AutoUpdate::Bool(true)),
                    "config value should be preserved when no env var set"
                );
                assert_eq!(
                    config.model,
                    Some("gpt-4o".to_string()),
                    "OPENCODE_MODEL should override config"
                );
            },
        );
    }

    #[test]
    fn test_existing_config_loading_still_works() {
        use opencode_config::Config;

        std::env::remove_var("OPENCODE_AUTO_SHARE");
        std::env::remove_var("OPENCODE_DISABLE_AUTOUPDATE");
        std::env::remove_var("OPENCODE_ENABLE_EXA");
        std::env::remove_var("OPENCODE_SERVER_PASSWORD");
        std::env::remove_var("OPENCODE_MODEL");

        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "logLevel": "debug",
            "server": {
                "port": 3000,
                "hostname": "localhost"
            },
            "model": "gpt-4o",
            "temperature": 0.7
        }"#;
        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load(&config_path).unwrap();
        assert_eq!(config.log_level, Some(opencode_config::LogLevel::Debug));
        assert!(config.server.is_some());
        assert_eq!(config.server.as_ref().unwrap().port, Some(3000));
        assert_eq!(
            config.server.as_ref().unwrap().hostname,
            Some("localhost".to_string())
        );
        assert_eq!(config.model, Some("gpt-4o".to_string()));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_env_vars_with_empty_config_file() {
        use opencode_config::{AutoUpdate, Config};

        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        std::fs::write(&config_path, "{}").unwrap();

        temp_env::with_vars(
            vec![
                ("OPENCODE_AUTO_SHARE", Some("true")),
                ("OPENCODE_DISABLE_AUTOUPDATE", Some("false")),
                ("OPENCODE_ENABLE_EXA", Some("true")),
                ("OPENCODE_SERVER_PASSWORD", Some("secret")),
            ],
            || {
                let config = Config::load(&config_path).unwrap();
                assert_eq!(config.autoshare, Some(true));
                assert_eq!(config.autoupdate, Some(AutoUpdate::Bool(true)));
                assert!(config.experimental.as_ref().and_then(|e| e.enable_exa) == Some(true));
                assert_eq!(
                    config.server.as_ref().and_then(|s| s.password.as_ref()),
                    Some(&"secret".to_string())
                );
            },
        );
    }
}
