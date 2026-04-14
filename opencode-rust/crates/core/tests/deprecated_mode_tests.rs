use opencode_core::config::Config;

#[cfg(test)]
mod deprecated_mode_tests {
    use super::*;

    #[test]
    fn deprecated_mode_field_still_parses() {
        let json_content = r#"{
            "model": "openai/gpt-4o",
            "mode": "agent"
        }"#;
        let config: Config = serde_json::from_str(json_content).unwrap();
        assert_eq!(config.model, Some("openai/gpt-4o".to_string()));
    }

    #[test]
    fn deprecated_mode_agent_mode_field_still_parses() {
        let json_content = r#"{
            "model": "openai/gpt-4o",
            "agent": {
                "build": {
                    "mode": "agent"
                }
            }
        }"#;
        let config: Config = serde_json::from_str(json_content).unwrap();
        assert_eq!(config.model, Some("openai/gpt-4o".to_string()));
    }
}
