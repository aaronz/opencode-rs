use std::collections::HashMap;
use opencode_core::plugin::{Plugin, PluginConfig};

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
