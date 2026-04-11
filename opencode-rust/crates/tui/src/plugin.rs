use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::plugin_api::{
    CommandContext, CommandResult, DialogRequest, DialogResult, PluginCommand, PluginCommandError,
    PluginCommandRegistry, PluginDialogError, PluginDialogRegistry, PluginDispose,
    PluginDisposeError, PluginDisposeRegistry, PluginEvent, PluginEventData, PluginEventError,
    PluginEventRegistry, PluginRoute, PluginRouteError, PluginRouteRegistry, PluginStateError,
    PluginStateRegistry, PluginTheme, PluginThemeError, PluginThemeRegistry, RegisteredEvent,
    RegisteredTheme, RouteContext, RouteParams,
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
    route_registry: Arc<PluginRouteRegistry>,
    theme_registry: Arc<PluginThemeRegistry>,
    event_registry: Arc<PluginEventRegistry>,
    state_registry: Arc<PluginStateRegistry>,
    dispose_registry: Arc<PluginDisposeRegistry>,
    dialog_registry: Arc<PluginDialogRegistry>,
}

impl TuiPluginManager {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            master_enabled: RwLock::new(true),
            command_registry: Arc::new(PluginCommandRegistry::new()),
            route_registry: Arc::new(PluginRouteRegistry::new()),
            theme_registry: Arc::new(PluginThemeRegistry::new()),
            event_registry: Arc::new(PluginEventRegistry::new()),
            state_registry: Arc::new(PluginStateRegistry::with_default_dir()),
            dispose_registry: Arc::new(PluginDisposeRegistry::new()),
            dialog_registry: Arc::new(PluginDialogRegistry::new()),
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
        let plugin_id = id.to_string();
        let should_dispose = {
            let mut plugins = self.plugins.write().unwrap();
            let entry = plugins
                .get_mut(&plugin_id)
                .ok_or_else(|| TuiPluginError::NotFound(plugin_id.clone()))?;

            if !entry.active {
                return Err(TuiPluginError::NotActive(plugin_id));
            }

            entry.state = PluginLifecycleState::Deactivating;
            entry.active = false;
            entry.state = PluginLifecycleState::Inactive;
            true
        };

        if should_dispose {
            if let Err(e) = self.dispose_registry.dispose_plugin(&plugin_id) {
                tracing::warn!("dispose hook failed for plugin {}: {}", plugin_id, e);
            }
        }

        Ok(())
    }

    pub fn get_plugin(&self, id: &str) -> Option<TuiPluginEntry> {
        self.plugins.read().unwrap().get(id).cloned()
    }

    pub fn list_plugins(&self) -> Vec<TuiPluginEntry> {
        self.plugins.read().unwrap().values().cloned().collect()
    }

    pub fn set_plugin_enabled(&self, id: &str, enabled: bool) -> Result<(), TuiPluginError> {
        let plugin_id = id.to_string();
        let should_dispose = {
            let mut plugins = self.plugins.write().unwrap();
            let entry = plugins
                .get_mut(&plugin_id)
                .ok_or_else(|| TuiPluginError::NotFound(plugin_id.clone()))?;

            if entry.active && !enabled {
                entry.enabled = false;
                entry.state = PluginLifecycleState::Deactivating;
                entry.active = false;
                entry.state = PluginLifecycleState::Inactive;
                true
            } else {
                entry.enabled = enabled;
                false
            }
        };

        if should_dispose {
            if let Err(e) = self.dispose_registry.dispose_plugin(&plugin_id) {
                tracing::warn!("dispose hook failed for plugin {}: {}", plugin_id, e);
            }
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

    pub fn route_registry(&self) -> Arc<PluginRouteRegistry> {
        Arc::clone(&self.route_registry)
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

    pub fn register_plugin_route<R: PluginRoute + 'static>(
        &self,
        plugin_id: &str,
        route: R,
    ) -> Result<(), PluginRouteError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginRouteError::PluginNotFound(plugin_id.to_string()));
        }
        self.route_registry.register_route(plugin_id, route)
    }

    pub fn unregister_plugin_routes(&self, plugin_id: &str) {
        self.route_registry.unregister_plugin_routes(plugin_id);
    }

    pub fn execute_plugin_route(
        &self,
        plugin_id: &str,
        route_name: &str,
        params: RouteParams,
    ) -> Result<crate::plugin_api::RouteResult, PluginRouteError> {
        let ctx = RouteContext {
            plugin_id: plugin_id.to_string(),
            route_name: route_name.to_string(),
            params,
        };
        self.route_registry.execute(plugin_id, route_name, &ctx)
    }

    pub fn list_plugin_routes(&self) -> Vec<crate::plugin_api::RegisteredRoute> {
        self.route_registry.list_routes()
    }

    pub fn theme_registry(&self) -> Arc<PluginThemeRegistry> {
        Arc::clone(&self.theme_registry)
    }

    pub fn register_plugin_theme(
        &self,
        plugin_id: &str,
        theme: PluginTheme,
    ) -> Result<(), PluginThemeError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginThemeError::PluginNotFound(plugin_id.to_string()));
        }
        self.theme_registry.register_theme(plugin_id, theme)
    }

    pub fn unregister_plugin_themes(&self, plugin_id: &str) {
        self.theme_registry.unregister_plugin_themes(plugin_id);
    }

    pub fn list_plugin_themes(&self) -> Vec<RegisteredTheme> {
        self.theme_registry.list_themes()
    }

    pub fn list_themes_for_plugin(&self, plugin_id: &str) -> Vec<RegisteredTheme> {
        self.theme_registry.list_themes_for_plugin(plugin_id)
    }

    pub fn get_plugin_theme(&self, plugin_id: &str, name: &str) -> Option<PluginTheme> {
        self.theme_registry.get_theme(plugin_id, name)
    }

    pub fn get_all_plugin_themes(&self) -> Vec<PluginTheme> {
        self.theme_registry.get_all_themes()
    }

    pub fn event_registry(&self) -> Arc<PluginEventRegistry> {
        Arc::clone(&self.event_registry)
    }

    pub fn subscribe_to_event<E: PluginEvent + 'static>(
        &self,
        plugin_id: &str,
        event: E,
    ) -> Result<(), PluginEventError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginEventError::PluginNotFound(plugin_id.to_string()));
        }
        self.event_registry.subscribe(plugin_id, event)
    }

    pub fn unsubscribe_plugin_events(&self, plugin_id: &str) {
        self.event_registry.unsubscribe_plugin(plugin_id);
    }

    pub fn emit_event(&self, data: PluginEventData) -> Vec<Result<(), PluginEventError>> {
        self.event_registry.emit(&data)
    }

    pub fn list_event_subscriptions(&self) -> Vec<RegisteredEvent> {
        self.event_registry.list_subscriptions()
    }

    pub fn list_event_subscriptions_for_plugin(&self, plugin_id: &str) -> Vec<RegisteredEvent> {
        self.event_registry.list_subscriptions_for_plugin(plugin_id)
    }

    pub fn state_registry(&self) -> Arc<PluginStateRegistry> {
        Arc::clone(&self.state_registry)
    }

    pub fn save_plugin_state(
        &self,
        plugin_id: &str,
        state: serde_json::Value,
    ) -> Result<(), PluginStateError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginStateError::PluginNotFound(plugin_id.to_string()));
        }
        self.state_registry.save_state(plugin_id, state)
    }

    pub fn load_plugin_state(
        &self,
        plugin_id: &str,
    ) -> Result<Option<serde_json::Value>, PluginStateError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginStateError::PluginNotFound(plugin_id.to_string()));
        }
        self.state_registry.load_state(plugin_id)
    }

    pub fn delete_plugin_state(&self, plugin_id: &str) -> Result<(), PluginStateError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginStateError::PluginNotFound(plugin_id.to_string()));
        }
        self.state_registry.delete_state(plugin_id)
    }

    pub fn dispose_registry(&self) -> Arc<PluginDisposeRegistry> {
        Arc::clone(&self.dispose_registry)
    }

    pub fn register_plugin_dispose<D: PluginDispose + 'static>(
        &self,
        plugin_id: &str,
        disposer: D,
    ) -> Result<(), PluginDisposeError> {
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginDisposeError::PluginNotFound(plugin_id.to_string()));
        }
        self.dispose_registry.register_disposer(plugin_id, disposer)
    }

    pub fn unregister_plugin_dispose(&self, plugin_id: &str) {
        self.dispose_registry.unregister_disposer(plugin_id);
    }

    pub fn has_plugin_dispose(&self, plugin_id: &str) -> bool {
        self.dispose_registry.has_disposer(plugin_id)
    }

    pub fn dialog_registry(&self) -> Arc<PluginDialogRegistry> {
        Arc::clone(&self.dialog_registry)
    }

    pub fn request_plugin_dialog(
        &self,
        request: DialogRequest,
    ) -> Result<DialogResult, PluginDialogError> {
        self.dialog_registry.request_dialog(request)
    }

    pub fn get_pending_dialogs(&self) -> Vec<DialogRequest> {
        self.dialog_registry.get_pending_dialogs()
    }

    pub fn has_active_dialog(&self, plugin_id: &str) -> bool {
        self.dialog_registry.has_active_dialog(plugin_id)
    }

    pub fn complete_plugin_dialog(
        &self,
        plugin_id: &str,
        result: DialogResult,
    ) -> Result<(), PluginDialogError> {
        self.dialog_registry.complete_dialog(plugin_id, result)
    }

    pub fn cancel_plugin_dialog(&self, plugin_id: &str) -> Result<(), PluginDialogError> {
        self.dialog_registry.cancel_dialog(plugin_id)
    }

    pub fn clear(&self) {
        let active_plugin_ids: Vec<String> = {
            let plugins = self.plugins.read().unwrap();
            plugins
                .values()
                .filter(|p| p.active)
                .map(|p| p.id.clone())
                .collect()
        };

        for plugin_id in active_plugin_ids {
            if let Err(e) = self.dispose_registry.dispose_plugin(&plugin_id) {
                tracing::warn!(
                    "dispose hook failed for plugin {} during clear: {}",
                    plugin_id,
                    e
                );
            }
        }

        self.plugins.write().unwrap().clear();
        self.command_registry.clear();
        self.route_registry.clear();
        self.theme_registry.clear();
        self.event_registry.clear();
        let _ = self.state_registry.clear_all_states();
        self.dispose_registry.clear();
        self.dialog_registry.clear();
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

    struct TestRoute {
        name: String,
        should_succeed: bool,
    }

    impl TestRoute {
        fn new(name: &str, should_succeed: bool) -> Self {
            Self {
                name: name.to_string(),
                should_succeed,
            }
        }
    }

    impl PluginRoute for TestRoute {
        fn name(&self) -> &str {
            &self.name
        }

        fn render(&self, _ctx: &RouteContext) -> crate::plugin_api::RouteResult {
            if self.should_succeed {
                crate::plugin_api::RouteResult::success("Route rendered")
            } else {
                crate::plugin_api::RouteResult::error("Route render failed")
            }
        }
    }

    #[test]
    fn test_plugin_routes_register() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "route.plugin".to_string(),
                "npm:route.plugin".to_string(),
                "@route/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let route = TestRoute::new("demo", true);
        manager
            .register_plugin_route("route.plugin", route)
            .unwrap();

        let routes = manager.list_plugin_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].name, "demo");
        assert_eq!(routes[0].plugin_id, "route.plugin");
    }

    #[test]
    fn test_plugin_routes_register_nonexistent_plugin() {
        let manager = TuiPluginManager::new();
        let route = TestRoute::new("demo", true);

        let result = manager.register_plugin_route("nonexistent", route);
        assert!(matches!(result, Err(PluginRouteError::PluginNotFound(_))));
    }

    #[test]
    fn test_plugin_routes_execute() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "exec.plugin".to_string(),
                "npm:exec.plugin".to_string(),
                "@exec/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let route = TestRoute::new("exec-route", true);
        manager.register_plugin_route("exec.plugin", route).unwrap();

        let params = RouteParams::new().with_param("sessionID", "abc123");
        let result = manager.execute_plugin_route("exec.plugin", "exec-route", params);
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[test]
    fn test_plugin_routes_execute_not_found() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "test.plugin".to_string(),
                "npm:test.plugin".to_string(),
                "@test/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let params = RouteParams::new();
        let result = manager.execute_plugin_route("test.plugin", "nonexistent", params);
        assert!(matches!(result, Err(PluginRouteError::RouteNotFound(_))));
    }

    #[test]
    fn test_plugin_routes_unregister() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "unreg.plugin".to_string(),
                "npm:unreg.plugin".to_string(),
                "@unreg/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let route1 = TestRoute::new("route1", true);
        let route2 = TestRoute::new("route2", true);
        manager
            .register_plugin_route("unreg.plugin", route1)
            .unwrap();
        manager
            .register_plugin_route("unreg.plugin", route2)
            .unwrap();

        assert_eq!(manager.list_plugin_routes().len(), 2);

        manager.unregister_plugin_routes("unreg.plugin");
        assert!(manager.list_plugin_routes().is_empty());
    }

    #[test]
    fn test_plugin_routes_tied_to_plugin_lifecycle() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "lifecycle.plugin".to_string(),
                "npm:lifecycle.plugin".to_string(),
                "@lifecycle/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let route = TestRoute::new("lifecycle-route", true);
        manager
            .register_plugin_route("lifecycle.plugin", route)
            .unwrap();
        assert_eq!(manager.list_plugin_routes().len(), 1);

        manager.activate("lifecycle.plugin").unwrap();
        assert!(manager.is_plugin_active("lifecycle.plugin"));
        assert_eq!(manager.list_plugin_routes().len(), 1);

        manager.deactivate("lifecycle.plugin").unwrap();
        assert!(!manager.is_plugin_active("lifecycle.plugin"));
        assert_eq!(manager.list_plugin_routes().len(), 1);
    }

    #[test]
    fn test_plugin_routes_clear() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "clear.plugin".to_string(),
                "npm:clear.plugin".to_string(),
                "@clear/plugin@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let route = TestRoute::new("clear-route", true);
        manager
            .register_plugin_route("clear.plugin", route)
            .unwrap();

        manager.clear();
        assert!(manager.list_plugin_routes().is_empty());
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_routes_multiple_plugins() {
        let manager = TuiPluginManager::new();
        manager
            .register_plugin(
                "plugin1".to_string(),
                "npm:plugin1".to_string(),
                "@plugin1@1.0.0".to_string(),
                true,
            )
            .unwrap();
        manager
            .register_plugin(
                "plugin2".to_string(),
                "npm:plugin2".to_string(),
                "@plugin2@1.0.0".to_string(),
                true,
            )
            .unwrap();

        let route1 = TestRoute::new("shared-name", true);
        let route2 = TestRoute::new("shared-name", true);
        manager.register_plugin_route("plugin1", route1).unwrap();
        manager.register_plugin_route("plugin2", route2).unwrap();

        assert_eq!(manager.list_plugin_routes().len(), 2);
    }
}
