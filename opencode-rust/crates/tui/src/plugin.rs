use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::plugin_api::{
    CommandContext, CommandResult, PluginCommand, PluginCommandError, PluginCommandRegistry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginLifecycleState {
    Registered,
    Activating,
    Active,
    Deactivating,
    Inactive,
}

#[derive(Debug, Clone)]
pub struct TuiPluginEntry {
    pub id: String,
    pub source: String,
    pub spec: String,
    pub enabled: bool,
    pub active: bool,
    pub state: PluginLifecycleState,
}

pub struct TuiPluginManager {
    plugins: RwLock<HashMap<String, TuiPluginEntry>>,
    master_enabled: RwLock<bool>,
    command_registry: Arc<PluginCommandRegistry>,
}

impl TuiPluginManager {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            master_enabled: RwLock::new(true),
            command_registry: Arc::new(PluginCommandRegistry::new()),
        }
    }

    pub fn set_master_enabled(&self, enabled: bool) {
        *self.master_enabled.write().unwrap() = enabled;
    }

    pub fn is_master_enabled(&self) -> bool {
        *self.master_enabled.read().unwrap()
    }

    pub fn register_plugin(
        &self,
        id: String,
        source: String,
        spec: String,
        enabled: bool,
    ) -> Result<(), TuiPluginError> {
        let mut plugins = self.plugins.write().unwrap();
        if plugins.contains_key(&id) {
            return Err(TuiPluginError::DuplicatePlugin(id));
        }
        plugins.insert(
            id.clone(),
            TuiPluginEntry {
                id,
                source,
                spec,
                enabled,
                active: false,
                state: PluginLifecycleState::Registered,
            },
        );
        Ok(())
    }

    pub fn unregister_plugin(&self, id: &str) -> Result<(), TuiPluginError> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(entry) = plugins.get(id) {
            if entry.active {
                return Err(TuiPluginError::PluginActive(id.to_string()));
            }
        }
        plugins
            .remove(id)
            .ok_or(TuiPluginError::NotFound(id.to_string()))?;
        Ok(())
    }

    pub fn activate(&self, id: &str) -> Result<(), TuiPluginError> {
        if !self.is_master_enabled() {
            return Err(TuiPluginError::MasterSwitchDisabled);
        }

        let mut plugins = self.plugins.write().unwrap();
        let entry = plugins
            .get_mut(id)
            .ok_or_else(|| TuiPluginError::NotFound(id.to_string()))?;

        if entry.active {
            return Err(TuiPluginError::AlreadyActive(id.to_string()));
        }

        if !entry.enabled {
            return Err(TuiPluginError::PluginDisabled(id.to_string()));
        }

        entry.state = PluginLifecycleState::Activating;
        entry.active = true;
        entry.state = PluginLifecycleState::Active;
        Ok(())
    }

    pub fn deactivate(&self, id: &str) -> Result<(), TuiPluginError> {
        let mut plugins = self.plugins.write().unwrap();
        let entry = plugins
            .get_mut(id)
            .ok_or_else(|| TuiPluginError::NotFound(id.to_string()))?;

        if !entry.active {
            return Err(TuiPluginError::NotActive(id.to_string()));
        }

        entry.state = PluginLifecycleState::Deactivating;
        entry.active = false;
        entry.state = PluginLifecycleState::Inactive;
        Ok(())
    }

    pub fn get_plugin(&self, id: &str) -> Option<TuiPluginEntry> {
        self.plugins.read().unwrap().get(id).cloned()
    }

    pub fn list_plugins(&self) -> Vec<TuiPluginEntry> {
        self.plugins.read().unwrap().values().cloned().collect()
    }

    pub fn set_plugin_enabled(&self, id: &str, enabled: bool) -> Result<(), TuiPluginError> {
        let mut plugins = self.plugins.write().unwrap();
        let entry = plugins
            .get_mut(id)
            .ok_or_else(|| TuiPluginError::NotFound(id.to_string()))?;

        if entry.active && !enabled {
            entry.enabled = false;
            entry.state = PluginLifecycleState::Deactivating;
            entry.active = false;
            entry.state = PluginLifecycleState::Inactive;
        } else {
            entry.enabled = enabled;
        }
        Ok(())
    }

    pub fn is_plugin_active(&self, id: &str) -> bool {
        self.plugins
            .read()
            .unwrap()
            .get(id)
            .map(|e| e.active)
            .unwrap_or(false)
    }

    pub fn command_registry(&self) -> Arc<PluginCommandRegistry> {
        Arc::clone(&self.command_registry)
    }

    pub fn register_plugin_command<C: PluginCommand + 'static>(
        &self,
        plugin_id: &str,
        command: C,
    ) -> Result<(), PluginCommandError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginCommandError::PluginNotFound(plugin_id.to_string()));
        }
        self.command_registry.register_command(plugin_id, command)
    }

    pub fn unregister_plugin_commands(&self, plugin_id: &str) {
        self.command_registry.unregister_plugin_commands(plugin_id);
    }

    pub fn execute_plugin_command(
        &self,
        plugin_id: &str,
        command_name: &str,
        ctx: &CommandContext,
    ) -> Result<CommandResult, PluginCommandError> {
        self.command_registry.execute(plugin_id, command_name, ctx)
    }

    pub fn list_plugin_commands(&self) -> Vec<crate::plugin_api::RegisteredCommand> {
        self.command_registry.list_commands()
    }

    pub fn clear(&self) {
        self.plugins.write().unwrap().clear();
        self.command_registry.clear();
    }
}

impl Default for TuiPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TuiPluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("plugin already registered: {0}")]
    DuplicatePlugin(String),
    #[error("plugin already active: {0}")]
    AlreadyActive(String),
    #[error("plugin not active: {0}")]
    NotActive(String),
    #[error("plugin disabled: {0}")]
    PluginDisabled(String),
    #[error("plugin is currently active and cannot be unregistered: {0}")]
    PluginActive(String),
    #[error("master plugin switch is disabled")]
    MasterSwitchDisabled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_new() {
        let manager = TuiPluginManager::new();
        assert!(manager.list_plugins().is_empty());
        assert!(manager.is_master_enabled());
    }

    #[test]
    fn test_register_plugin() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let plugins = manager.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "test.plugin");
        assert!(!plugins[0].active);
    }

    #[test]
    fn test_register_duplicate_plugin() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let result = manager.register_plugin(
            "test.plugin".to_string(),
            "npm:test.plugin".to_string(),
            "@test/plugin@1.0.0".to_string(),
            true,
        );
        assert!(matches!(result, Err(TuiPluginError::DuplicatePlugin(_))));
    }

    #[test]
    fn test_activate_plugin() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.activate("test.plugin").unwrap();
        assert!(manager.is_plugin_active("test.plugin"));

        let entry = manager.get_plugin("test.plugin").unwrap();
        assert!(entry.active);
        assert_eq!(entry.state, PluginLifecycleState::Active);
    }

    #[test]
    fn test_activate_disabled_plugin() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                false,
            )
            .unwrap();

        let result = manager.activate("test.plugin");
        assert!(matches!(result, Err(TuiPluginError::PluginDisabled(_))));
    }

    #[test]
    fn test_activate_already_active() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.activate("test.plugin").unwrap();
        let result = manager.activate("test.plugin");
        assert!(matches!(result, Err(TuiPluginError::AlreadyActive(_))));
    }

    #[test]
    fn test_deactivate_plugin() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.activate("test.plugin").unwrap();
        manager.deactivate("test.plugin").unwrap();

        assert!(!manager.is_plugin_active("test.plugin"));
        let entry = manager.get_plugin("test.plugin").unwrap();
        assert!(!entry.active);
        assert_eq!(entry.state, PluginLifecycleState::Inactive);
    }

    #[test]
    fn test_deactivate_not_active() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let result = manager.deactivate("test.plugin");
        assert!(matches!(result, Err(TuiPluginError::NotActive(_))));
    }

    #[test]
    fn test_master_switch_disables_all() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.set_master_enabled(false);
        let result = manager.activate("test.plugin");
        assert!(matches!(result, Err(TuiPluginError::MasterSwitchDisabled)));
    }

    #[test]
    fn test_unregister_inactive_plugin() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.unregister_plugin("test.plugin").unwrap();
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_unregister_active_plugin_fails() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.activate("test.plugin").unwrap();
        let result = manager.unregister_plugin("test.plugin");
        assert!(matches!(result, Err(TuiPluginError::PluginActive(_))));
    }

    #[test]
    fn test_set_plugin_enabled_deactivates_if_active() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.activate("test.plugin").unwrap();
        manager.set_plugin_enabled("test.plugin", false).unwrap();

        let entry = manager.get_plugin("test.plugin").unwrap();
        assert!(!entry.enabled);
        assert!(!entry.active);
        assert_eq!(entry.state, PluginLifecycleState::Inactive);
    }

    #[test]
    fn test_activate_not_found() {
        let manager = TuiPluginManager::new();
        let result = manager.activate("nonexistent");
        assert!(matches!(result, Err(TuiPluginError::NotFound(_))));
    }

    #[test]
    fn test_clear_plugins() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        manager.clear();
        assert!(manager.list_plugins().is_empty());
    }
}
