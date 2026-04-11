use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

pub const VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiVersion {
    V1,
    V2,
}

impl ApiVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "v1",
            ApiVersion::V2 => "v2",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub plugin_id: String,
    pub app_state: CommandContextState,
}

#[derive(Debug, Clone)]
pub struct CommandContextState {
    pub messages: Vec<CommandMessage>,
    pub work_mode: String,
}

#[derive(Debug, Clone)]
pub struct CommandMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct RouteParams {
    pub params: HashMap<String, String>,
}

impl RouteParams {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }
}

impl Default for RouteParams {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RouteContext {
    pub plugin_id: String,
    pub route_name: String,
    pub params: RouteParams,
}

#[derive(Debug, Clone)]
pub struct RouteResult {
    pub success: bool,
    pub message: String,
}

impl RouteResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegisteredRoute {
    pub plugin_id: String,
    pub name: String,
}

pub trait PluginRoute: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, ctx: &RouteContext) -> RouteResult;
}

pub struct PluginRouteRegistry {
    routes: RwLock<HashMap<String, RegisteredRoute>>,
    handlers: RwLock<HashMap<String, Box<dyn PluginRoute>>>,
}

impl PluginRouteRegistry {
    pub fn new() -> Self {
        Self {
            routes: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_route<R: PluginRoute + 'static>(
        &self,
        plugin_id: &str,
        route: R,
    ) -> Result<(), PluginRouteError> {
        let name = route.name().to_string();
        let full_name = format!("{}:{}", plugin_id, name);

        let mut routes = self.routes.write().unwrap();
        if routes.contains_key(&full_name) {
            return Err(PluginRouteError::RouteAlreadyRegistered(full_name));
        }

        let registered = RegisteredRoute {
            plugin_id: plugin_id.to_string(),
            name: name.clone(),
        };

        routes.insert(full_name.clone(), registered);

        let mut handlers = self.handlers.write().unwrap();
        handlers.insert(full_name, Box::new(route));

        Ok(())
    }

    pub fn unregister_plugin_routes(&self, plugin_id: &str) {
        let prefix = format!("{}:", plugin_id);

        let mut routes = self.routes.write().unwrap();
        let keys_to_remove: Vec<String> = routes
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            routes.remove(&key);
        }

        let mut handlers = self.handlers.write().unwrap();
        let handler_keys_to_remove: Vec<String> = handlers
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in handler_keys_to_remove {
            handlers.remove(&key);
        }
    }

    pub fn get_route(&self, plugin_id: &str, name: &str) -> Option<RegisteredRoute> {
        let full_name = format!("{}:{}", plugin_id, name);
        self.routes.read().unwrap().get(&full_name).cloned()
    }

    pub fn list_routes(&self) -> Vec<RegisteredRoute> {
        self.routes.read().unwrap().values().cloned().collect()
    }

    pub fn execute(
        &self,
        plugin_id: &str,
        name: &str,
        ctx: &RouteContext,
    ) -> Result<RouteResult, PluginRouteError> {
        let full_name = format!("{}:{}", plugin_id, name);
        let handlers = self.handlers.read().unwrap();
        let handler = handlers
            .get(&full_name)
            .ok_or_else(|| PluginRouteError::RouteNotFound(full_name.clone()))?;

        Ok(handler.render(ctx))
    }

    pub fn clear(&self) {
        self.routes.write().unwrap().clear();
        self.handlers.write().unwrap().clear();
    }
}

impl Default for PluginRouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginRouteError {
    #[error("route not found: {0}")]
    RouteNotFound(String),
    #[error("route already registered: {0}")]
    RouteAlreadyRegistered(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("render error: {0}")]
    RenderError(String),
}

pub trait PluginCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn aliases(&self) -> Vec<String>;
    fn execute(&self, ctx: &CommandContext) -> CommandResult;
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

impl CommandResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegisteredCommand {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
}

pub struct PluginCommandRegistry {
    commands: RwLock<HashMap<String, RegisteredCommand>>,
    executors: RwLock<HashMap<String, Box<dyn PluginCommand>>>,
}

impl PluginCommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_command<C: PluginCommand + 'static>(
        &self,
        plugin_id: &str,
        command: C,
    ) -> Result<(), PluginCommandError> {
        let name = command.name().to_string();
        let full_name = format!("{}:{}", plugin_id, name);

        let mut commands = self.commands.write().unwrap();
        if commands.contains_key(&full_name) {
            return Err(PluginCommandError::CommandAlreadyRegistered(full_name));
        }

        let registered = RegisteredCommand {
            plugin_id: plugin_id.to_string(),
            name: name.clone(),
            description: command.description().to_string(),
            aliases: command.aliases(),
        };

        commands.insert(full_name.clone(), registered);

        let mut executors = self.executors.write().unwrap();
        executors.insert(full_name, Box::new(command));

        Ok(())
    }

    pub fn unregister_plugin_commands(&self, plugin_id: &str) {
        let prefix = format!("{}:", plugin_id);

        let mut commands = self.commands.write().unwrap();
        let keys_to_remove: Vec<String> = commands
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            commands.remove(&key);
        }

        let mut executors = self.executors.write().unwrap();
        let exec_keys_to_remove: Vec<String> = executors
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in exec_keys_to_remove {
            executors.remove(&key);
        }
    }

    pub fn get_command(&self, plugin_id: &str, name: &str) -> Option<RegisteredCommand> {
        let full_name = format!("{}:{}", plugin_id, name);
        self.commands.read().unwrap().get(&full_name).cloned()
    }

    pub fn get_by_name(&self, name: &str) -> Option<RegisteredCommand> {
        let commands = self.commands.read().unwrap();
        commands
            .values()
            .find(|c| c.name == name || c.aliases.contains(&name.to_string()))
            .cloned()
    }

    pub fn list_commands(&self) -> Vec<RegisteredCommand> {
        self.commands.read().unwrap().values().cloned().collect()
    }

    pub fn execute(
        &self,
        plugin_id: &str,
        name: &str,
        ctx: &CommandContext,
    ) -> Result<CommandResult, PluginCommandError> {
        let full_name = format!("{}:{}", plugin_id, name);
        let executors = self.executors.read().unwrap();
        let executor = executors
            .get(&full_name)
            .ok_or_else(|| PluginCommandError::CommandNotFound(full_name.clone()))?;

        Ok(executor.execute(ctx))
    }

    pub fn execute_by_name(
        &self,
        name: &str,
        ctx: &CommandContext,
    ) -> Result<CommandResult, PluginCommandError> {
        let cmd = self
            .get_by_name(name)
            .ok_or_else(|| PluginCommandError::CommandNotFound(name.to_string()))?;

        self.execute(&cmd.plugin_id, &cmd.name, ctx)
    }

    pub fn clear(&self) {
        self.commands.write().unwrap().clear();
        self.executors.write().unwrap().clear();
    }
}

impl Default for PluginCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginCommandError {
    #[error("command not found: {0}")]
    CommandNotFound(String),
    #[error("command already registered: {0}")]
    CommandAlreadyRegistered(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("execution error: {0}")]
    ExecutionError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub error: String,
    pub warning: String,
    pub success: String,
    pub muted: String,
    pub border: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            primary: "#89b4fa".to_string(),
            secondary: "#cba6f7".to_string(),
            accent: "#f38ba8".to_string(),
            error: "#f38ba8".to_string(),
            warning: "#fab387".to_string(),
            success: "#a6e3a1".to_string(),
            muted: "#6c7086".to_string(),
            border: "#313244".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTheme {
    pub name: String,
    pub colors: ThemeColors,
}

impl PluginTheme {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            colors: ThemeColors::default(),
        }
    }

    pub fn with_colors(mut self, colors: ThemeColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn background(mut self, color: impl Into<String>) -> Self {
        self.colors.background = color.into();
        self
    }

    pub fn foreground(mut self, color: impl Into<String>) -> Self {
        self.colors.foreground = color.into();
        self
    }

    pub fn primary(mut self, color: impl Into<String>) -> Self {
        self.colors.primary = color.into();
        self
    }

    pub fn secondary(mut self, color: impl Into<String>) -> Self {
        self.colors.secondary = color.into();
        self
    }

    pub fn accent(mut self, color: impl Into<String>) -> Self {
        self.colors.accent = color.into();
        self
    }

    pub fn error(mut self, color: impl Into<String>) -> Self {
        self.colors.error = color.into();
        self
    }

    pub fn warning(mut self, color: impl Into<String>) -> Self {
        self.colors.warning = color.into();
        self
    }

    pub fn success(mut self, color: impl Into<String>) -> Self {
        self.colors.success = color.into();
        self
    }

    pub fn muted(mut self, color: impl Into<String>) -> Self {
        self.colors.muted = color.into();
        self
    }

    pub fn border(mut self, color: impl Into<String>) -> Self {
        self.colors.border = color.into();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredTheme {
    pub plugin_id: String,
    pub name: String,
}

pub struct PluginThemeRegistry {
    themes: RwLock<HashMap<String, RegisteredTheme>>,
    theme_defs: RwLock<HashMap<String, PluginTheme>>,
}

impl PluginThemeRegistry {
    pub fn new() -> Self {
        Self {
            themes: RwLock::new(HashMap::new()),
            theme_defs: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_theme(
        &self,
        plugin_id: &str,
        theme: PluginTheme,
    ) -> Result<(), PluginThemeError> {
        let name = theme.name.clone();
        let full_name = format!("{}:{}", plugin_id, &name);

        let mut themes = self.themes.write().unwrap();
        if themes.contains_key(&full_name) {
            return Err(PluginThemeError::ThemeAlreadyRegistered(full_name));
        }

        let registered = RegisteredTheme {
            plugin_id: plugin_id.to_string(),
            name: name.clone(),
        };

        themes.insert(full_name.clone(), registered);

        let mut theme_defs = self.theme_defs.write().unwrap();
        theme_defs.insert(full_name, theme);

        Ok(())
    }

    pub fn unregister_plugin_themes(&self, plugin_id: &str) {
        let prefix = format!("{}:", plugin_id);

        let mut themes = self.themes.write().unwrap();
        let keys_to_remove: Vec<String> = themes
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            themes.remove(&key);
        }

        let mut theme_defs = self.theme_defs.write().unwrap();
        let def_keys_to_remove: Vec<String> = theme_defs
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in def_keys_to_remove {
            theme_defs.remove(&key);
        }
    }

    pub fn get_theme(&self, plugin_id: &str, name: &str) -> Option<PluginTheme> {
        let full_name = format!("{}:{}", plugin_id, name);
        self.theme_defs.read().unwrap().get(&full_name).cloned()
    }

    pub fn list_themes(&self) -> Vec<RegisteredTheme> {
        self.themes.read().unwrap().values().cloned().collect()
    }

    pub fn list_themes_for_plugin(&self, plugin_id: &str) -> Vec<RegisteredTheme> {
        self.themes
            .read()
            .unwrap()
            .values()
            .filter(|t| t.plugin_id == plugin_id)
            .cloned()
            .collect()
    }

    pub fn get_all_themes(&self) -> Vec<PluginTheme> {
        self.theme_defs.read().unwrap().values().cloned().collect()
    }

    pub fn clear(&self) {
        self.themes.write().unwrap().clear();
        self.theme_defs.write().unwrap().clear();
    }
}

impl Default for PluginThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginThemeError {
    #[error("theme not found: {0}")]
    ThemeNotFound(String),
    #[error("theme already registered: {0}")]
    ThemeAlreadyRegistered(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("registration error: {0}")]
    RegistrationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEventData {
    pub event_name: String,
    pub source_plugin: Option<String>,
    pub payload: serde_json::Value,
}

impl PluginEventData {
    pub fn new(event_name: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            event_name: event_name.into(),
            source_plugin: None,
            payload,
        }
    }

    pub fn with_source(mut self, plugin_id: impl Into<String>) -> Self {
        self.source_plugin = Some(plugin_id.into());
        self
    }
}

pub trait PluginEvent: Send + Sync {
    fn event_name(&self) -> &str;
    fn handle(&self, data: &PluginEventData) -> Result<(), PluginEventError>;
}

#[derive(Debug, Clone)]
pub struct RegisteredEvent {
    pub plugin_id: String,
    pub event_name: String,
}

pub struct PluginEventRegistry {
    events: RwLock<HashMap<String, Vec<RegisteredEvent>>>,
    handlers: RwLock<HashMap<String, Vec<Box<dyn PluginEvent>>>>,
}

impl PluginEventRegistry {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub fn subscribe<E: PluginEvent + 'static>(
        &self,
        plugin_id: &str,
        event: E,
    ) -> Result<(), PluginEventError> {
        let event_name = event.event_name().to_string();

        let registered = RegisteredEvent {
            plugin_id: plugin_id.to_string(),
            event_name: event_name.clone(),
        };

        self.events
            .write()
            .unwrap()
            .entry(event_name.clone())
            .or_insert_with(Vec::new)
            .push(registered);

        self.handlers
            .write()
            .unwrap()
            .entry(event_name)
            .or_insert_with(Vec::new)
            .push(Box::new(event));

        Ok(())
    }

    pub fn unsubscribe_plugin(&self, plugin_id: &str) {
        let mut events = self.events.write().unwrap();
        let mut handlers = self.handlers.write().unwrap();

        let event_names: Vec<String> = events.keys().cloned().collect();

        for event_name in event_names {
            let plugin_indices: Vec<usize> = events
                .get(&event_name)
                .map(|v| {
                    v.iter()
                        .enumerate()
                        .filter(|(_, e)| e.plugin_id == plugin_id)
                        .map(|(i, _)| i)
                        .collect()
                })
                .unwrap_or_default();

            for index in plugin_indices.into_iter().rev() {
                if let Some(v) = events.get_mut(&event_name) {
                    v.remove(index);
                }
            }

            if events
                .get(&event_name)
                .map(|v| v.is_empty())
                .unwrap_or(false)
            {
                events.remove(&event_name);
                handlers.remove(&event_name);
            }
        }
    }

    pub fn list_subscriptions(&self) -> Vec<RegisteredEvent> {
        let events = self.events.read().unwrap();
        events.values().flatten().cloned().collect()
    }

    pub fn list_subscriptions_for_plugin(&self, plugin_id: &str) -> Vec<RegisteredEvent> {
        let events = self.events.read().unwrap();
        events
            .values()
            .flatten()
            .filter(|e| e.plugin_id == plugin_id)
            .cloned()
            .collect()
    }

    pub fn emit(&self, data: &PluginEventData) -> Vec<Result<(), PluginEventError>> {
        let handlers = self.handlers.read().unwrap();
        let event_name = &data.event_name;

        let Some(event_handlers) = handlers.get(event_name) else {
            return vec![];
        };

        event_handlers
            .iter()
            .map(|handler| handler.handle(data))
            .collect()
    }

    pub fn clear(&self) {
        self.events.write().unwrap().clear();
        self.handlers.write().unwrap().clear();
    }
}

impl Default for PluginEventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginEventError {
    #[error("event handler error: {0}")]
    HandlerError(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("subscription failed: {0}")]
    SubscriptionFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PluginStateError {
    #[error("state serialization error: {0}")]
    SerializationError(String),
    #[error("state I/O error: {0}")]
    IoError(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("invalid state data: {0}")]
    InvalidData(String),
}

pub trait PluginDispose: Send + Sync {
    fn on_dispose(&self, plugin_id: &str);
}

#[derive(Debug, thiserror::Error)]
pub enum PluginDisposeError {
    #[error("dispose callback error: {0}")]
    CallbackError(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
}

pub struct PluginDisposeRegistry {
    disposers: RwLock<HashMap<String, Arc<dyn PluginDispose>>>,
}

impl PluginDisposeRegistry {
    pub fn new() -> Self {
        Self {
            disposers: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_disposer<D: PluginDispose + 'static>(
        &self,
        plugin_id: &str,
        disposer: D,
    ) -> Result<(), PluginDisposeError> {
        let mut disposers = self.disposers.write().unwrap();
        disposers.insert(plugin_id.to_string(), Arc::new(disposer));
        Ok(())
    }

    pub fn dispose_plugin(&self, plugin_id: &str) -> Result<(), PluginDisposeError> {
        let plugin_id_owned = plugin_id.to_string();
        let disposer = {
            let disposers = self.disposers.read().unwrap();
            disposers.get(plugin_id).cloned()
        };

        if let Some(disposer) = disposer {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                disposer.on_dispose(&plugin_id_owned);
            }));
            if result.is_err() {
                tracing::warn!("dispose hook panicked for plugin: {}", plugin_id);
            }
        }
        Ok(())
    }

    pub fn unregister_disposer(&self, plugin_id: &str) {
        let mut disposers = self.disposers.write().unwrap();
        disposers.remove(plugin_id);
    }

    pub fn has_disposer(&self, plugin_id: &str) -> bool {
        let disposers = self.disposers.read().unwrap();
        disposers.contains_key(plugin_id)
    }

    pub fn clear(&self) {
        self.disposers.write().unwrap().clear();
    }
}

impl Default for PluginDisposeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PluginStateRegistry {
    states: RwLock<HashMap<String, serde_json::Value>>,
    state_dir: std::path::PathBuf,
}

impl PluginStateRegistry {
    pub fn new(state_dir: std::path::PathBuf) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            state_dir,
        }
    }

    pub fn with_default_dir() -> Self {
        let state_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".opencode")
            .join("plugin_states");
        Self::new(state_dir)
    }

    fn state_file_path(&self, plugin_id: &str) -> std::path::PathBuf {
        let safe_filename = plugin_id.replace(['/', '\\', ':', '.'], "_");
        self.state_dir.join(format!("{}.json", safe_filename))
    }

    pub fn save_state(
        &self,
        plugin_id: &str,
        state: serde_json::Value,
    ) -> Result<(), PluginStateError> {
        let states_dir = &self.state_dir;
        if !states_dir.exists() {
            std::fs::create_dir_all(states_dir)
                .map_err(|e| PluginStateError::IoError(e.to_string()))?;
        }

        let file_path = self.state_file_path(plugin_id);
        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| PluginStateError::SerializationError(e.to_string()))?;

        std::fs::write(&file_path, json).map_err(|e| PluginStateError::IoError(e.to_string()))?;

        let mut states = self.states.write().unwrap();
        states.insert(plugin_id.to_string(), state);

        Ok(())
    }

    pub fn load_state(
        &self,
        plugin_id: &str,
    ) -> Result<Option<serde_json::Value>, PluginStateError> {
        {
            let states = self.states.read().unwrap();
            if let Some(state) = states.get(plugin_id) {
                return Ok(Some(state.clone()));
            }
        }

        let file_path = self.state_file_path(plugin_id);
        if !file_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| PluginStateError::IoError(e.to_string()))?;

        let state: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| PluginStateError::InvalidData(e.to_string()))?;

        let mut states = self.states.write().unwrap();
        states.insert(plugin_id.to_string(), state.clone());

        Ok(Some(state))
    }

    pub fn delete_state(&self, plugin_id: &str) -> Result<(), PluginStateError> {
        let file_path = self.state_file_path(plugin_id);
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| PluginStateError::IoError(e.to_string()))?;
        }

        let mut states = self.states.write().unwrap();
        states.remove(plugin_id);

        Ok(())
    }

    pub fn get_state_keys(&self) -> Vec<String> {
        let states = self.states.read().unwrap();
        states.keys().cloned().collect()
    }

    pub fn has_state(&self, plugin_id: &str) -> bool {
        let states = self.states.read().unwrap();
        if states.contains_key(plugin_id) {
            return true;
        }
        drop(states);
        self.state_file_path(plugin_id).exists()
    }

    pub fn clear_all_states(&self) -> Result<(), PluginStateError> {
        let mut states = self.states.write().unwrap();
        states.clear();

        if self.state_dir.exists() {
            for entry in std::fs::read_dir(&self.state_dir)
                .map_err(|e| PluginStateError::IoError(e.to_string()))?
            {
                let entry = entry.map_err(|e| PluginStateError::IoError(e.to_string()))?;
                if entry
                    .path()
                    .extension()
                    .map(|e| e == "json")
                    .unwrap_or(false)
                {
                    std::fs::remove_file(entry.path())
                        .map_err(|e| PluginStateError::IoError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }
}

impl Default for PluginStateRegistry {
    fn default() -> Self {
        Self::with_default_dir()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogType {
    Alert,
    Confirm,
    Prompt,
    Select,
}

#[derive(Debug, Clone)]
pub struct DialogRequest {
    pub plugin_id: String,
    pub dialog_type: DialogType,
    pub title: String,
    pub message: String,
    pub options: Option<Vec<String>>,
    pub default_value: Option<String>,
}

impl DialogRequest {
    pub fn alert(
        plugin_id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            dialog_type: DialogType::Alert,
            title: title.into(),
            message: message.into(),
            options: None,
            default_value: None,
        }
    }

    pub fn confirm(
        plugin_id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            dialog_type: DialogType::Confirm,
            title: title.into(),
            message: message.into(),
            options: None,
            default_value: None,
        }
    }

    pub fn prompt(
        plugin_id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            dialog_type: DialogType::Prompt,
            title: title.into(),
            message: message.into(),
            options: None,
            default_value: Some(placeholder.into()),
        }
    }

    pub fn select(
        plugin_id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        options: Vec<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            dialog_type: DialogType::Select,
            title: title.into(),
            message: message.into(),
            options: Some(options),
            default_value: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveDialog {
    pub request: DialogRequest,
    pub result_tx: std::sync::mpsc::Sender<DialogResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResult {
    Confirmed(String),
    Cancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginDialogError {
    #[error("dialog request failed: {0}")]
    RequestFailed(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("no active dialog for plugin: {0}")]
    NoActiveDialog(String),
    #[error("dialog result channel closed")]
    ChannelClosed,
}

pub struct PluginDialogRegistry {
    pending_dialogs: RwLock<Vec<DialogRequest>>,
    active_dialogs: RwLock<HashMap<String, ActiveDialog>>,
}

impl PluginDialogRegistry {
    pub fn new() -> Self {
        Self {
            pending_dialogs: RwLock::new(Vec::new()),
            active_dialogs: RwLock::new(HashMap::new()),
        }
    }

    pub fn request_dialog(
        &self,
        request: DialogRequest,
    ) -> Result<DialogResult, PluginDialogError> {
        let plugin_id = request.plugin_id.clone();

        let (result_tx, result_rx) = std::sync::mpsc::channel();

        {
            let mut pending = self.pending_dialogs.write().unwrap();
            pending.push(request.clone());
        }

        {
            let mut active = self.active_dialogs.write().unwrap();
            active.insert(plugin_id.clone(), ActiveDialog { request, result_tx });
        }

        result_rx
            .recv()
            .map_err(|_| PluginDialogError::ChannelClosed)
    }

    pub fn get_pending_dialogs(&self) -> Vec<DialogRequest> {
        self.pending_dialogs.read().unwrap().clone()
    }

    pub fn get_active_dialog(&self, plugin_id: &str) -> Option<ActiveDialog> {
        self.active_dialogs.read().unwrap().get(plugin_id).cloned()
    }

    pub fn has_active_dialog(&self, plugin_id: &str) -> bool {
        self.active_dialogs.read().unwrap().contains_key(plugin_id)
    }

    pub fn complete_dialog(
        &self,
        plugin_id: &str,
        result: DialogResult,
    ) -> Result<(), PluginDialogError> {
        let active_dialog = {
            let mut active = self.active_dialogs.write().unwrap();
            active
                .remove(plugin_id)
                .ok_or_else(|| PluginDialogError::NoActiveDialog(plugin_id.to_string()))?
        };

        {
            let mut pending = self.pending_dialogs.write().unwrap();
            pending.retain(|r| r.plugin_id != plugin_id);
        }

        active_dialog
            .result_tx
            .send(result)
            .map_err(|_| PluginDialogError::ChannelClosed)?;

        Ok(())
    }

    pub fn cancel_dialog(&self, plugin_id: &str) -> Result<(), PluginDialogError> {
        self.complete_dialog(plugin_id, DialogResult::Cancelled)
    }

    pub fn clear_pending(&self) {
        let mut pending = self.pending_dialogs.write().unwrap();
        pending.clear();
    }

    pub fn clear_active(&self) {
        let mut active = self.active_dialogs.write().unwrap();
        for (_, dialog) in active.drain() {
            let _ = dialog.result_tx.send(DialogResult::Cancelled);
        }
    }

    pub fn clear(&self) {
        self.clear_active();
        self.clear_pending();
    }
}

impl Default for PluginDialogRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        name: String,
        description: String,
        aliases: Vec<String>,
        should_succeed: bool,
    }

    impl TestCommand {
        fn new(name: &str, description: &str, aliases: Vec<&str>, should_succeed: bool) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                aliases: aliases.iter().map(|s| s.to_string()).collect(),
                should_succeed,
            }
        }
    }

    impl PluginCommand for TestCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn aliases(&self) -> Vec<String> {
            self.aliases.clone()
        }

        fn execute(&self, _ctx: &CommandContext) -> CommandResult {
            if self.should_succeed {
                CommandResult::success("Test command executed")
            } else {
                CommandResult::error("Test command failed")
            }
        }
    }

    #[test]
    fn test_plugin_command_registry_new() {
        let registry = PluginCommandRegistry::new();
        assert!(registry.list_commands().is_empty());
    }

    #[test]
    fn test_register_command() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("test-cmd", "A test command", vec!["tc"], true);

        registry.register_command("test.plugin", cmd).unwrap();

        let cmds = registry.list_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].name, "test-cmd");
        assert_eq!(cmds[0].plugin_id, "test.plugin");
    }

    #[test]
    fn test_register_duplicate_command() {
        let registry = PluginCommandRegistry::new();
        let cmd1 = TestCommand::new("test-cmd", "First", vec![], true);
        let cmd2 = TestCommand::new("test-cmd", "Second", vec![], true);

        registry.register_command("test.plugin", cmd1).unwrap();

        let result = registry.register_command("test.plugin", cmd2);
        assert!(matches!(
            result,
            Err(PluginCommandError::CommandAlreadyRegistered(_))
        ));
    }

    #[test]
    fn test_unregister_plugin_commands() {
        let registry = PluginCommandRegistry::new();
        let cmd1 = TestCommand::new("cmd1", "First command", vec![], true);
        let cmd2 = TestCommand::new("cmd2", "Second command", vec![], true);

        registry.register_command("test.plugin", cmd1).unwrap();
        registry.register_command("test.plugin", cmd2).unwrap();

        assert_eq!(registry.list_commands().len(), 2);

        registry.unregister_plugin_commands("test.plugin");
        assert!(registry.list_commands().is_empty());
    }

    #[test]
    fn test_get_command() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("get-cmd", "Get test", vec!["gc"], true);

        registry.register_command("get.plugin", cmd).unwrap();

        let found = registry.get_command("get.plugin", "get-cmd");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "get-cmd");
    }

    #[test]
    fn test_get_by_name() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("byname", "By name test", vec!["bn"], true);

        registry.register_command("byname.plugin", cmd).unwrap();

        let found = registry.get_by_name("byname");
        assert!(found.is_some());

        let found_by_alias = registry.get_by_name("bn");
        assert!(found_by_alias.is_some());
    }

    #[test]
    fn test_execute_command() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("exec", "Execute test", vec![], true);

        registry.register_command("exec.plugin", cmd).unwrap();

        let ctx = CommandContext {
            plugin_id: "exec.plugin".to_string(),
            app_state: CommandContextState {
                messages: vec![],
                work_mode: "build".to_string(),
            },
        };

        let result = registry.execute("exec.plugin", "exec", &ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Test command executed");
    }

    #[test]
    fn test_execute_by_name() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("byname-exec", "By name exec test", vec!["bne"], true);

        registry.register_command("bne.plugin", cmd).unwrap();

        let ctx = CommandContext {
            plugin_id: "bne.plugin".to_string(),
            app_state: CommandContextState {
                messages: vec![],
                work_mode: "build".to_string(),
            },
        };

        let result = registry.execute_by_name("byname-exec", &ctx).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let registry = PluginCommandRegistry::new();
        let ctx = CommandContext {
            plugin_id: "test".to_string(),
            app_state: CommandContextState {
                messages: vec![],
                work_mode: "build".to_string(),
            },
        };

        let result = registry.execute("test.plugin", "nonexistent", &ctx);
        assert!(matches!(
            result,
            Err(PluginCommandError::CommandNotFound(_))
        ));
    }

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("OK");
        assert!(result.success);
        assert_eq!(result.message, "OK");
    }

    #[test]
    fn test_command_result_error() {
        let result = CommandResult::error("Failed");
        assert!(!result.success);
        assert_eq!(result.message, "Failed");
    }

    #[test]
    fn test_api_version_as_str() {
        assert_eq!(ApiVersion::V1.as_str(), "v1");
        assert_eq!(ApiVersion::V2.as_str(), "v2");
    }

    #[test]
    fn test_multiple_plugins_same_command_name() {
        let registry = PluginCommandRegistry::new();
        let cmd1 = TestCommand::new("cmd", "From plugin1", vec![], true);
        let cmd2 = TestCommand::new("cmd", "From plugin2", vec![], true);

        registry.register_command("plugin1", cmd1).unwrap();

        let result = registry.register_command("plugin2", cmd2);
        assert!(result.is_ok());
        assert_eq!(registry.list_commands().len(), 2);
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

        fn render(&self, _ctx: &RouteContext) -> RouteResult {
            if self.should_succeed {
                RouteResult::success("Route rendered")
            } else {
                RouteResult::error("Route render failed")
            }
        }
    }

    #[test]
    fn test_plugin_route_registry_new() {
        let registry = PluginRouteRegistry::new();
        assert!(registry.list_routes().is_empty());
    }

    #[test]
    fn test_plugin_routes_register_route() {
        let registry = PluginRouteRegistry::new();
        let route = TestRoute::new("demo", true);

        registry.register_route("test.plugin", route).unwrap();

        let routes = registry.list_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].name, "demo");
        assert_eq!(routes[0].plugin_id, "test.plugin");
    }

    #[test]
    fn test_plugin_routes_register_duplicate() {
        let registry = PluginRouteRegistry::new();
        let route1 = TestRoute::new("demo", true);
        let route2 = TestRoute::new("demo", true);

        registry.register_route("test.plugin", route1).unwrap();

        let result = registry.register_route("test.plugin", route2);
        assert!(matches!(
            result,
            Err(PluginRouteError::RouteAlreadyRegistered(_))
        ));
    }

    #[test]
    fn test_plugin_routes_unregister_plugin_routes() {
        let registry = PluginRouteRegistry::new();
        let route1 = TestRoute::new("route1", true);
        let route2 = TestRoute::new("route2", true);

        registry.register_route("test.plugin", route1).unwrap();
        registry.register_route("test.plugin", route2).unwrap();

        assert_eq!(registry.list_routes().len(), 2);

        registry.unregister_plugin_routes("test.plugin");
        assert!(registry.list_routes().is_empty());
    }

    #[test]
    fn test_plugin_routes_get_route() {
        let registry = PluginRouteRegistry::new();
        let route = TestRoute::new("get-route", true);

        registry.register_route("get.plugin", route).unwrap();

        let found = registry.get_route("get.plugin", "get-route");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "get-route");
    }

    #[test]
    fn test_plugin_routes_execute() {
        let registry = PluginRouteRegistry::new();
        let route = TestRoute::new("exec-route", true);

        registry.register_route("exec.plugin", route).unwrap();

        let ctx = RouteContext {
            plugin_id: "exec.plugin".to_string(),
            route_name: "exec-route".to_string(),
            params: RouteParams::new(),
        };

        let result = registry.execute("exec.plugin", "exec-route", &ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Route rendered");
    }

    #[test]
    fn test_plugin_routes_execute_with_params() {
        let registry = PluginRouteRegistry::new();
        let route = TestRoute::new("param-route", true);

        registry.register_route("param.plugin", route).unwrap();

        let ctx = RouteContext {
            plugin_id: "param.plugin".to_string(),
            route_name: "param-route".to_string(),
            params: RouteParams::new().with_param("sessionID", "abc123"),
        };

        let result = registry
            .execute("param.plugin", "param-route", &ctx)
            .unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_plugin_routes_execute_not_found() {
        let registry = PluginRouteRegistry::new();
        let ctx = RouteContext {
            plugin_id: "test.plugin".to_string(),
            route_name: "nonexistent".to_string(),
            params: RouteParams::new(),
        };

        let result = registry.execute("test.plugin", "nonexistent", &ctx);
        assert!(matches!(result, Err(PluginRouteError::RouteNotFound(_))));
    }

    #[test]
    fn test_plugin_routes_multiple_plugins_same_name() {
        let registry = PluginRouteRegistry::new();
        let route1 = TestRoute::new("route", true);
        let route2 = TestRoute::new("route", true);

        registry.register_route("plugin1", route1).unwrap();

        let result = registry.register_route("plugin2", route2);
        assert!(result.is_ok());
        assert_eq!(registry.list_routes().len(), 2);
    }

    #[test]
    fn test_route_params_new() {
        let params = RouteParams::new();
        assert!(params.params.is_empty());
    }

    #[test]
    fn test_route_params_with_param() {
        let params = RouteParams::new().with_param("key", "value");
        assert_eq!(params.get("key"), Some("value"));
    }

    #[test]
    fn test_route_params_get_missing() {
        let params = RouteParams::new();
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_route_result_success() {
        let result = RouteResult::success("OK");
        assert!(result.success);
        assert_eq!(result.message, "OK");
    }

    #[test]
    fn test_route_result_error() {
        let result = RouteResult::error("Failed");
        assert!(!result.success);
        assert_eq!(result.message, "Failed");
    }
}
