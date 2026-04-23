use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

fn truthy(key: &str) -> bool {
    env::var(key)
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Flag {
    pub name: String,
    pub description: String,
    pub default: bool,
    pub value: bool,
}

#[allow(dead_code)]
pub(crate) struct FlagManager {
    flags: HashMap<String, Flag>,
    string_flags: HashMap<String, Option<String>>,
    number_flags: HashMap<String, Option<u64>>,
}

#[allow(dead_code)]
impl FlagManager {
    pub fn new() -> Self {
        let mut flags = HashMap::new();
        let mut string_flags = HashMap::new();
        let mut number_flags = HashMap::new();

        // Basic flags
        flags.insert(
            "OPENCODE_EXPERIMENTAL".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL".to_string(),
                description: "Enable experimental features".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DEBUG".to_string(),
            Flag {
                name: "OPENCODE_DEBUG".to_string(),
                description: "Enable debug mode".to_string(),
                default: false,
                value: false,
            },
        );

        // Client type flags
        flags.insert(
            "OPENCODE_AUTO_SHARE".to_string(),
            Flag {
                name: "OPENCODE_AUTO_SHARE".to_string(),
                description: "Auto-share sessions".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_AUTOUPDATE".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_AUTOUPDATE".to_string(),
                description: "Disable auto-update".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_ALWAYS_NOTIFY_UPDATE".to_string(),
            Flag {
                name: "OPENCODE_ALWAYS_NOTIFY_UPDATE".to_string(),
                description: "Always notify about updates".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_PRUNE".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_PRUNE".to_string(),
                description: "Disable pruning".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_TERMINAL_TITLE".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_TERMINAL_TITLE".to_string(),
                description: "Disable terminal title".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_DEFAULT_PLUGINS".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_DEFAULT_PLUGINS".to_string(),
                description: "Disable default plugins".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_LSP_DOWNLOAD".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_LSP_DOWNLOAD".to_string(),
                description: "Disable LSP download".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_ENABLE_EXPERIMENTAL_MODELS".to_string(),
            Flag {
                name: "OPENCODE_ENABLE_EXPERIMENTAL_MODELS".to_string(),
                description: "Enable experimental models".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_AUTOCOMPACT".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_AUTOCOMPACT".to_string(),
                description: "Disable auto-compaction".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_MODELS_FETCH".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_MODELS_FETCH".to_string(),
                description: "Disable models fetch".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_CLAUDE_CODE".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_CLAUDE_CODE".to_string(),
                description: "Disable Claude Code features".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_ENABLE_QUESTION_TOOL".to_string(),
            Flag {
                name: "OPENCODE_ENABLE_QUESTION_TOOL".to_string(),
                description: "Enable question tool".to_string(),
                default: false,
                value: false,
            },
        );

        // Experimental flags
        flags.insert(
            "OPENCODE_EXPERIMENTAL_FILEWATCHER".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_FILEWATCHER".to_string(),
                description: "Enable experimental file watcher".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_DISABLE_FILEWATCHER".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_DISABLE_FILEWATCHER".to_string(),
                description: "Disable file watcher".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_ICON_DISCOVERY".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_ICON_DISCOVERY".to_string(),
                description: "Enable icon discovery".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT".to_string(),
                description: "Disable copy on select".to_string(),
                default: cfg!(windows),
                value: cfg!(windows),
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_EXA".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_EXA".to_string(),
                description: "Enable Exa web search (experimental)".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_ENABLE_EXA".to_string(),
            Flag {
                name: "OPENCODE_ENABLE_EXA".to_string(),
                description: "Enable Exa web search".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_OXFMT".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_OXFMT".to_string(),
                description: "Enable oxfmt".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_LSP_TY".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_LSP_TY".to_string(),
                description: "Enable LSP ty".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_LSP_TOOL".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_LSP_TOOL".to_string(),
                description: "Enable LSP tool".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_FILETIME_CHECK".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_FILETIME_CHECK".to_string(),
                description: "Disable file time check".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_PLAN_MODE".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_PLAN_MODE".to_string(),
                description: "Enable plan mode".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_WORKSPACES".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_WORKSPACES".to_string(),
                description: "Enable workspaces".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_MARKDOWN".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_MARKDOWN".to_string(),
                description: "Enable markdown".to_string(),
                default: true,
                value: true,
            },
        );

        flags.insert(
            "OPENCODE_EXPERIMENTAL_VARIANT_REASONING".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL_VARIANT_REASONING".to_string(),
                description: "Enable experimental variant/reasoning budget support".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DISABLE_CHANNEL_DB".to_string(),
            Flag {
                name: "OPENCODE_DISABLE_CHANNEL_DB".to_string(),
                description: "Disable channel DB".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_SKIP_MIGRATIONS".to_string(),
            Flag {
                name: "OPENCODE_SKIP_MIGRATIONS".to_string(),
                description: "Skip migrations".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_STRICT_CONFIG_DEPS".to_string(),
            Flag {
                name: "OPENCODE_STRICT_CONFIG_DEPS".to_string(),
                description: "Strict config deps".to_string(),
                default: false,
                value: false,
            },
        );

        // String flags
        string_flags.insert("OPENCODE_GIT_BASH_PATH".to_string(), None);
        string_flags.insert("OPENCODE_CONFIG".to_string(), None);
        string_flags.insert("OPENCODE_CONFIG_CONTENT".to_string(), None);
        string_flags.insert("OPENCODE_PERMISSION".to_string(), None);
        string_flags.insert("OPENCODE_FAKE_VCS".to_string(), None);
        string_flags.insert("OPENCODE_CLIENT".to_string(), None);
        string_flags.insert("OPENCODE_SERVER_PASSWORD".to_string(), None);
        string_flags.insert("OPENCODE_SERVER_USERNAME".to_string(), None);
        string_flags.insert("OPENCODE_MODELS_URL".to_string(), None);
        string_flags.insert("OPENCODE_MODELS_PATH".to_string(), None);
        string_flags.insert("OPENCODE_DB".to_string(), None);

        // Number flags
        number_flags.insert(
            "OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS".to_string(),
            None,
        );
        number_flags.insert("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX".to_string(), None);

        Self {
            flags,
            string_flags,
            number_flags,
        }
    }

    pub fn get(&self, name: &str) -> Option<bool> {
        self.flags.get(name).map(|f| f.value)
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        self.string_flags.get(name).and_then(|v| v.clone())
    }

    pub fn get_number(&self, name: &str) -> Option<u64> {
        self.number_flags.get(name).and_then(|v| *v)
    }

    pub fn set(&mut self, name: &str, value: bool) {
        if let Some(flag) = self.flags.get_mut(name) {
            flag.value = value;
        }
    }

    pub fn set_string(&mut self, name: &str, value: Option<String>) {
        if let Some(v) = self.string_flags.get_mut(name) {
            *v = value;
        }
    }

    pub fn set_number(&mut self, name: &str, value: Option<u64>) {
        if let Some(v) = self.number_flags.get_mut(name) {
            *v = value;
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.get(name).unwrap_or(false)
    }

    pub fn load_from_env(&mut self) {
        for (name, flag) in self.flags.iter_mut() {
            flag.value = truthy(name);
        }

        for (name, value) in self.string_flags.iter_mut() {
            if let Ok(val) = env::var(name) {
                *value = Some(val);
            }
        }

        for (name, value) in self.number_flags.iter_mut() {
            if let Ok(val) = env::var(name) {
                if let Ok(parsed) = val.parse::<u64>() {
                    if parsed > 0 {
                        *value = Some(parsed);
                    }
                }
            }
        }
    }

    // Convenience methods that match TypeScript API
    pub fn opencode_auto_share(&self) -> bool {
        self.get("OPENCODE_AUTO_SHARE").unwrap_or(false)
    }

    pub fn opencode_client(&self) -> String {
        self.get_string("OPENCODE_CLIENT")
            .unwrap_or_else(|| "cli".to_string())
    }

    pub fn opencode_enable_question_tool(&self) -> bool {
        self.get("OPENCODE_ENABLE_QUESTION_TOOL").unwrap_or(false)
    }

    pub fn opencode_experimental(&self) -> bool {
        self.get("OPENCODE_EXPERIMENTAL").unwrap_or(false)
    }

    pub fn opencode_enable_exa(&self) -> bool {
        self.get("OPENCODE_ENABLE_EXA").unwrap_or(false)
            || self.opencode_experimental()
            || self.get("OPENCODE_EXPERIMENTAL_EXA").unwrap_or(false)
    }

    pub fn opencode_experimental_plan_mode(&self) -> bool {
        self.opencode_experimental() || self.get("OPENCODE_EXPERIMENTAL_PLAN_MODE").unwrap_or(false)
    }

    pub fn opencode_experimental_lsp_tool(&self) -> bool {
        self.opencode_experimental() || self.get("OPENCODE_EXPERIMENTAL_LSP_TOOL").unwrap_or(false)
    }

    pub fn opencode_experimental_variant_reasoning(&self) -> bool {
        self.opencode_experimental()
            || self
                .get("OPENCODE_EXPERIMENTAL_VARIANT_REASONING")
                .unwrap_or(false)
    }

    pub fn opencode_experimental_bash_timeout_ms(&self) -> Option<u64> {
        self.get_number("OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS")
            .or(Some(120000)) // Default 2 minutes
    }

    pub fn all_flags(&self) -> &HashMap<String, Flag> {
        &self.flags
    }

    pub fn all_string_flags(&self) -> &HashMap<String, Option<String>> {
        &self.string_flags
    }

    pub fn all_number_flags(&self) -> &HashMap<String, Option<u64>> {
        &self.number_flags
    }
}

impl Default for FlagManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_has_all_boolean_flags() {
        let fm = FlagManager::new();
        let expected_flags = [
            "OPENCODE_EXPERIMENTAL",
            "OPENCODE_DEBUG",
            "OPENCODE_AUTO_SHARE",
            "OPENCODE_DISABLE_AUTOUPDATE",
            "OPENCODE_ALWAYS_NOTIFY_UPDATE",
            "OPENCODE_DISABLE_PRUNE",
            "OPENCODE_DISABLE_TERMINAL_TITLE",
            "OPENCODE_DISABLE_DEFAULT_PLUGINS",
            "OPENCODE_DISABLE_LSP_DOWNLOAD",
            "OPENCODE_ENABLE_EXPERIMENTAL_MODELS",
            "OPENCODE_DISABLE_AUTOCOMPACT",
            "OPENCODE_DISABLE_MODELS_FETCH",
            "OPENCODE_DISABLE_CLAUDE_CODE",
            "OPENCODE_ENABLE_QUESTION_TOOL",
            "OPENCODE_EXPERIMENTAL_FILEWATCHER",
            "OPENCODE_EXPERIMENTAL_DISABLE_FILEWATCHER",
            "OPENCODE_EXPERIMENTAL_ICON_DISCOVERY",
            "OPENCODE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT",
            "OPENCODE_EXPERIMENTAL_EXA",
            "OPENCODE_ENABLE_EXA",
            "OPENCODE_EXPERIMENTAL_OXFMT",
            "OPENCODE_EXPERIMENTAL_LSP_TY",
            "OPENCODE_EXPERIMENTAL_LSP_TOOL",
            "OPENCODE_DISABLE_FILETIME_CHECK",
            "OPENCODE_EXPERIMENTAL_PLAN_MODE",
            "OPENCODE_EXPERIMENTAL_WORKSPACES",
            "OPENCODE_EXPERIMENTAL_MARKDOWN",
            "OPENCODE_EXPERIMENTAL_VARIANT_REASONING",
            "OPENCODE_DISABLE_CHANNEL_DB",
            "OPENCODE_SKIP_MIGRATIONS",
            "OPENCODE_STRICT_CONFIG_DEPS",
        ];
        assert_eq!(expected_flags.len(), 31);
        for flag in expected_flags {
            assert!(fm.get(flag).is_some(), "Missing flag: {}", flag);
        }
    }

    #[test]
    fn default_values_are_false_except_markdown() {
        let fm = FlagManager::new();
        assert!(!fm.is_enabled("OPENCODE_EXPERIMENTAL"));
        assert!(!fm.is_enabled("OPENCODE_DEBUG"));
        assert!(fm.is_enabled("OPENCODE_EXPERIMENTAL_MARKDOWN"));
    }

    #[test]
    fn set_overrides_value() {
        let mut fm = FlagManager::new();
        assert!(!fm.is_enabled("OPENCODE_DEBUG"));
        fm.set("OPENCODE_DEBUG", true);
        assert!(fm.is_enabled("OPENCODE_DEBUG"));
    }

    #[test]
    fn unknown_flag_returns_false() {
        let fm = FlagManager::new();
        assert!(!fm.is_enabled("NONEXISTENT_FLAG"));
    }

    #[test]
    fn exa_enabled_when_experimental_is_true() {
        let mut fm = FlagManager::new();
        fm.set("OPENCODE_EXPERIMENTAL", true);
        assert!(fm.opencode_enable_exa());
    }

    #[test]
    fn plan_mode_enabled_when_experimental_is_true() {
        let mut fm = FlagManager::new();
        fm.set("OPENCODE_EXPERIMENTAL", true);
        assert!(fm.opencode_experimental_plan_mode());
    }

    #[test]
    fn opencode_client_defaults_to_cli() {
        let fm = FlagManager::new();
        assert_eq!(fm.opencode_client(), "cli");
    }

    #[test]
    fn bash_timeout_has_default() {
        let fm = FlagManager::new();
        assert_eq!(fm.opencode_experimental_bash_timeout_ms(), Some(120_000));
    }

    #[test]
    fn string_flag_returns_none_when_not_set() {
        let fm = FlagManager::new();
        assert!(fm.get_string("OPENCODE_CONFIG").is_none());
    }

    #[test]
    fn number_flag_returns_none_when_not_set() {
        let fm = FlagManager::new();
        assert!(fm
            .get_number("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX")
            .is_none());
    }

    #[test]
    fn opencode_experimental_exa_flag_exists() {
        let fm = FlagManager::new();
        assert!(fm.get("OPENCODE_EXPERIMENTAL_EXA").is_some());
    }

    #[test]
    fn opencode_experimental_exa_default_is_false() {
        let fm = FlagManager::new();
        assert_eq!(fm.get("OPENCODE_EXPERIMENTAL_EXA"), Some(false));
    }

    #[test]
    fn opencode_enable_exa_returns_correct_value_based_on_flag_state() {
        let mut fm = FlagManager::new();
        assert!(!fm.opencode_enable_exa());
        fm.set("OPENCODE_EXPERIMENTAL_EXA", true);
        assert!(fm.opencode_enable_exa());
    }

    #[test]
    fn opencode_enable_exa_works_when_experimental_is_true() {
        let mut fm = FlagManager::new();
        fm.set("OPENCODE_EXPERIMENTAL", true);
        assert!(fm.opencode_enable_exa());
    }

    struct EnvVarGuard {
        vars: Vec<(String, Option<String>, String)>,
    }

    impl EnvVarGuard {
        fn new() -> Self {
            Self { vars: Vec::new() }
        }
        fn set(&mut self, key: &str, value: &str) {
            let prev = std::env::var(key).ok();
            std::env::set_var(key, value);
            self.vars.push((key.to_string(), prev, value.to_string()));
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            for (key, old_value, set_value) in self.vars.drain(..) {
                let current = std::env::var(&key).ok();
                if current == Some(set_value) {
                    match old_value {
                        Some(v) => std::env::set_var(&key, v),
                        None => std::env::remove_var(&key),
                    }
                }
            }
        }
    }

    #[test]
    fn load_from_env_uses_truthy_helper() {
        let mut guard = EnvVarGuard::new();
        guard.set("OPENCODE_DEBUG", "1");
        guard.set("OPENCODE_EXPERIMENTAL", "true");
        guard.set("OPENCODE_DISABLE_AUTOUPDATE", "TRUE");

        let mut fm = FlagManager::new();
        fm.load_from_env();

        assert!(fm.is_enabled("OPENCODE_DEBUG"));
        assert!(fm.is_enabled("OPENCODE_EXPERIMENTAL"));
        assert!(fm.is_enabled("OPENCODE_DISABLE_AUTOUPDATE"));
    }

    #[test]
    fn load_from_env_boolean_flags_from_environment() {
        let mut guard = EnvVarGuard::new();
        guard.set("OPENCODE_EXPERIMENTAL", "true");

        let mut fm = FlagManager::new();
        assert!(!fm.is_enabled("OPENCODE_EXPERIMENTAL"));
        fm.load_from_env();
        assert!(fm.is_enabled("OPENCODE_EXPERIMENTAL"));
    }

    #[test]
    fn set_string_updates_string_flag_value_when_flag_exists() {
        let mut fm = FlagManager::new();
        assert!(fm.get_string("OPENCODE_CONFIG").is_none());
        fm.set_string("OPENCODE_CONFIG", Some("test_value".to_string()));
        assert_eq!(
            fm.get_string("OPENCODE_CONFIG"),
            Some("test_value".to_string())
        );
    }

    #[test]
    fn set_string_is_no_op_when_flag_does_not_exist() {
        let mut fm = FlagManager::new();
        fm.set_string("NONEXISTENT_FLAG", Some("value".to_string()));
    }

    #[test]
    fn set_number_updates_number_flag_value_when_flag_exists() {
        let mut fm = FlagManager::new();
        assert!(fm
            .get_number("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX")
            .is_none());
        fm.set_number("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX", Some(5000));
        assert_eq!(
            fm.get_number("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX"),
            Some(5000)
        );
    }

    #[test]
    fn set_number_is_no_op_when_flag_does_not_exist() {
        let mut fm = FlagManager::new();
        fm.set_number("NONEXISTENT_FLAG", Some(100));
    }

    #[test]
    fn all_flags_returns_all_boolean_flags() {
        let fm = FlagManager::new();
        let flags = fm.all_flags();
        assert!(flags.contains_key("OPENCODE_EXPERIMENTAL"));
        assert!(flags.contains_key("OPENCODE_DEBUG"));
        assert!(flags.contains_key("OPENCODE_EXPERIMENTAL_MARKDOWN"));
        assert_eq!(flags.len(), 31);
    }

    #[test]
    fn all_string_flags_returns_all_string_flags() {
        let fm = FlagManager::new();
        let string_flags = fm.all_string_flags();
        assert!(string_flags.contains_key("OPENCODE_CLIENT"));
        assert!(string_flags.contains_key("OPENCODE_CONFIG"));
        assert!(string_flags.contains_key("OPENCODE_DB"));
        assert_eq!(string_flags.len(), 11);
    }

    #[test]
    fn all_number_flags_returns_all_number_flags() {
        let fm = FlagManager::new();
        let number_flags = fm.all_number_flags();
        assert!(number_flags.contains_key("OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS"));
        assert!(number_flags.contains_key("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX"));
        assert_eq!(number_flags.len(), 2);
    }
}
