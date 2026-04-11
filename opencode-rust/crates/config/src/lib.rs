pub use opencode_core::config::Config;

#[cfg(test)]
mod tools_alias_regression {
    use opencode_core::config::Config;
    use std::collections::HashMap;

    fn assert_tools_alias_old_key_still_works() {
        let config_json = serde_json::json!({
            "model": "openai/gpt-4o",
            "tools": {
                "bash": false,
                "read": true,
                "edit": false
            }
        });

        let config: Config = serde_json::from_value(config_json).unwrap();

        #[allow(deprecated)]
        {
            assert!(config.tools.is_some());
            let tools = config.tools.as_ref().unwrap();
            assert_eq!(tools.get("bash"), Some(&false));
            assert_eq!(tools.get("read"), Some(&true));
            assert_eq!(tools.get("edit"), Some(&false));
        }
    }

    #[test]
    fn tools_alias_regression_old_key_still_works() {
        assert_tools_alias_old_key_still_works();
    }

    fn assert_tools_alias_agent_level_works() {
        let config_json = serde_json::json!({
            "model": "openai/gpt-4o",
            "agent": {
                "plan": {
                    "model": "anthropic/claude-3-5",
                    "tools": {
                        "grep": false,
                        "glob": true
                    }
                }
            }
        });

        let config: Config = serde_json::from_value(config_json).unwrap();

        #[allow(deprecated)]
        {
            let agent_config = config.agent.as_ref().unwrap().get_agent("plan").unwrap();
            assert!(agent_config.tools.is_some());
            let tools = agent_config.tools.as_ref().unwrap();
            assert_eq!(tools.get("grep"), Some(&false));
            assert_eq!(tools.get("glob"), Some(&true));
        }
    }

    #[test]
    fn tools_alias_regression_agent_level_works() {
        assert_tools_alias_agent_level_works();
    }

    #[test]
    fn tools_alias_regression_permission_takes_precedence() {
        let config_json = serde_json::json!({
            "model": "openai/gpt-4o",
            "tools": {
                "bash": false,
                "read": true
            },
            "permission": {
                "bash": "allow",
                "read": "deny"
            }
        });

        let config: Config = serde_json::from_value(config_json).unwrap();

        assert!(config.permission.is_some());

        #[allow(deprecated)]
        let tools_disabled = config.get_disabled_tools();

        assert!(
            !tools_disabled.contains("bash"),
            "bash should be allowed via permission"
        );
        assert!(
            tools_disabled.contains("read"),
            "read should be denied via permission"
        );
    }

    #[test]
    fn tools_alias_regression_permission_only_when_no_tools() {
        let config_json = serde_json::json!({
            "model": "openai/gpt-4o",
            "permission": {
                "bash": "deny",
                "read": "allow"
            }
        });

        let config: Config = serde_json::from_value(config_json).unwrap();

        assert!(config.permission.is_some());

        #[allow(deprecated)]
        let tools_disabled = config.get_disabled_tools();

        assert!(
            tools_disabled.contains("bash"),
            "bash should be disabled via permission"
        );
        assert!(
            !tools_disabled.contains("read"),
            "read should not be disabled"
        );
    }

    #[test]
    fn tools_alias_regression_top_level_over_agent_level() {
        let config_json = serde_json::json!({
            "model": "openai/gpt-4o",
            "tools": {
                "bash": false,
                "write": true
            },
            "agent": {
                "default_agent": "build",
                "agents": {
                    "build": {
                        "tools": {
                            "bash": true,
                            "read": false
                        }
                    }
                }
            }
        });

        let config: Config = serde_json::from_value(config_json).unwrap();

        #[allow(deprecated)]
        let tools_disabled = config.get_disabled_tools();

        assert!(
            tools_disabled.contains("bash"),
            "bash should be disabled (top-level override)"
        );
        assert!(
            !tools_disabled.contains("write"),
            "write should not be disabled"
        );
        assert!(
            tools_disabled.contains("read"),
            "read should be disabled (default agent-level)"
        );
    }

    #[test]
    fn tools_alias_regression_round_trip_serialization() {
        let config = Config {
            model: Some("openai/gpt-4o".to_string()),
            #[allow(deprecated)]
            tools: Some(HashMap::from([
                ("bash".to_string(), false),
                ("read".to_string(), true),
            ])),
            ..Default::default()
        };

        let json = serde_json::to_value(&config).unwrap();
        let deserialized: Config = serde_json::from_value(json).unwrap();

        #[allow(deprecated)]
        {
            assert!(deserialized.tools.is_some());
            let tools = deserialized.tools.as_ref().unwrap();
            assert_eq!(tools.get("bash"), Some(&false));
            assert_eq!(tools.get("read"), Some(&true));
        }
    }
}
