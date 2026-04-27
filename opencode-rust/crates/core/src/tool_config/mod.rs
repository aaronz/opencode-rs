mod types;
pub use types::ToolConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn merge_gives_top_level_priority_over_agent_level() {
        let top_level = HashMap::from([("bash".to_string(), false), ("read".to_string(), true)]);
        let agent_level = HashMap::from([
            ("bash".to_string(), true),
            ("read".to_string(), false),
            ("write".to_string(), false),
        ]);

        let config = ToolConfig::merge(Some(&top_level), Some(&agent_level));

        assert!(config.is_disabled("bash"));
        assert!(!config.is_disabled("read"));
        assert!(config.is_disabled("write"));
    }
}