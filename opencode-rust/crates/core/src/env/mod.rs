mod types;

pub(crate) use types::EnvManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_get_set() {
        let env = EnvManager::new();

        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());

        env.set("TEST_VAR".to_string(), "test_value".to_string());
        assert_eq!(env.get("TEST_VAR"), Some("test_value".to_string()));

        env.remove("TEST_VAR");
        assert_eq!(env.get("TEST_VAR"), None);
    }

    #[test]
    fn test_env_all() {
        let env = EnvManager::new();
        let all = env.all();

        assert!(!all.is_empty());
        assert!(all.contains_key("PATH") || all.contains_key("HOME"));
    }

    #[test]
    fn test_env_env_guard() {
        let env = EnvManager::new();

        let guard = env.env();
        assert!(!guard.is_empty());
    }

    #[test]
    fn test_env_manager_new() {
        let env = EnvManager::new();
        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());
    }

    #[test]
    fn test_env_manager_default() {
        let env = EnvManager::default();
        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());
    }

    #[test]
    fn test_env_set_overwrites() {
        let env = EnvManager::new();

        env.set("TEST_VAR".to_string(), "value1".to_string());
        assert_eq!(env.get("TEST_VAR"), Some("value1".to_string()));

        env.set("TEST_VAR".to_string(), "value2".to_string());
        assert_eq!(env.get("TEST_VAR"), Some("value2".to_string()));
    }

    #[test]
    fn test_env_remove_nonexistent() {
        let env = EnvManager::new();
        env.remove("NONEXISTENT_VAR_12345");
        assert_eq!(env.get("NONEXISTENT_VAR_12345"), None);
    }

    #[test]
    fn test_env_instance_isolation() {
        let env = EnvManager::new();

        env.set(
            "ISOLATION_TEST_VAR".to_string(),
            "isolated_value".to_string(),
        );

        assert_eq!(
            env.get("ISOLATION_TEST_VAR"),
            Some("isolated_value".to_string())
        );
        assert!(std::env::var("ISOLATION_TEST_VAR").is_err());

        env.remove("ISOLATION_TEST_VAR");
    }

    #[test]
    fn test_env_empty_instance_returns_none() {
        let env = EnvManager::new();
        assert_eq!(env.get("__DOES_NOT_EXIST__"), None);
        assert_eq!(env.get("__ANOTHER_MISSING__"), None);
    }
}
