use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolConfig {
    disabled_tools: HashSet<String>,
}

impl ToolConfig {
    pub fn merge(
        top_level: Option<&HashMap<String, bool>>,
        agent_level: Option<&HashMap<String, bool>>,
    ) -> Self {
        let mut merged = agent_level.cloned().unwrap_or_default();

        if let Some(top_level) = top_level {
            for (name, enabled) in top_level {
                merged.insert(name.clone(), *enabled);
            }
        }

        let disabled_tools = merged
            .into_iter()
            .filter_map(|(name, enabled)| (!enabled).then_some(name))
            .collect();

        Self { disabled_tools }
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled_tools.contains(name)
    }

    pub fn disabled_tools(&self) -> &HashSet<String> {
        &self.disabled_tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
