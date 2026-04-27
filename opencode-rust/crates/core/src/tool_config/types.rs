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