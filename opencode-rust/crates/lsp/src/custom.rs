//! Custom LSP server registration API.
//!
//! This module provides mechanisms for external LSP servers to register
//! with the system, including their capabilities and configuration.

use crate::language::Language;
use crate::launch::LaunchConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Represents the capabilities of a custom LSP server.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Whether the server supports text document synchronization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<bool>,

    /// Whether the server supports hover requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_provider: Option<bool>,

    /// Whether the server supports completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_provider: Option<bool>,

    /// Whether the server supports goto definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_provider: Option<bool>,

    /// Whether the server supports find references
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references_provider: Option<bool>,

    /// Whether the server supports code actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action_provider: Option<bool>,

    /// Custom capabilities not covered by the standard fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl ServerCapabilities {
    /// Create a new empty ServerCapabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this capability set includes hover support
    pub fn supports_hover(&self) -> bool {
        self.hover_provider.unwrap_or(false)
    }

    /// Check if this capability set includes completion support
    pub fn supports_completion(&self) -> bool {
        self.completion_provider.unwrap_or(false)
    }

    /// Check if this capability set includes definition support
    pub fn supports_definition(&self) -> bool {
        self.definition_provider.unwrap_or(false)
    }

    /// Check if this capability set includes references support
    pub fn supports_references(&self) -> bool {
        self.references_provider.unwrap_or(false)
    }

    /// Check if this capability set includes code action support
    pub fn supports_code_action(&self) -> bool {
        self.code_action_provider.unwrap_or(false)
    }
}

/// Configuration for a custom LSP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomServerConfig {
    /// Command and arguments to start the server
    pub command: Vec<String>,
    /// Languages this server handles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
    /// File extensions this server handles (e.g., [".rs", ".toml"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
    /// Root path for the server (if different from project root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<PathBuf>,
    /// Environment variables for the server process
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    /// Server capabilities (optional - can be auto-detected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ServerCapabilities>,
    /// Custom initialization options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialization_options: Option<serde_json::Value>,
    /// Custom settings for the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
    /// Whether this server is enabled (default true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl CustomServerConfig {
    /// Create a new custom server config with the given command
    pub fn new(command: Vec<String>) -> Self {
        Self {
            command,
            languages: None,
            extensions: None,
            root: None,
            env: None,
            capabilities: None,
            initialization_options: None,
            settings: None,
            enabled: true,
        }
    }

    /// Set the languages this server handles
    pub fn with_languages(mut self, languages: Vec<String>) -> Self {
        self.languages = Some(languages);
        self
    }

    /// Set the file extensions this server handles
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Set the root path for the server
    pub fn with_root(mut self, root: PathBuf) -> Self {
        self.root = Some(root);
        self
    }

    /// Set the server capabilities
    pub fn with_capabilities(mut self, capabilities: ServerCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Check if this server is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Represents a custom LSP server that has been registered with the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLspServer {
    /// Unique identifier for this server
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// The command configuration
    pub config: CustomServerConfig,
    /// The server capabilities
    pub capabilities: ServerCapabilities,
    /// Whether this server is currently active
    #[serde(default)]
    pub active: bool,
}

impl CustomLspServer {
    /// Create a new custom LSP server
    pub fn new(id: String, name: String, config: CustomServerConfig) -> Self {
        let capabilities = config.capabilities.clone().unwrap_or_default();
        Self {
            id,
            name,
            config,
            capabilities,
            active: false,
        }
    }

    /// Get the command as a single string (for spawning)
    pub fn command_string(&self) -> String {
        self.config.command.join(" ")
    }

    /// Check if this server handles the given language
    pub fn handles_language(&self, language: &Language) -> bool {
        if let Some(ref languages) = self.config.languages {
            let lang_name = language.name();
            languages.iter().any(|l| l.eq_ignore_ascii_case(lang_name))
        } else {
            // If no languages specified, check by extension
            !language.file_extensions().is_empty()
        }
    }

    /// Check if this server handles the given file extension
    pub fn handles_extension(&self, ext: &str) -> bool {
        if let Some(ref extensions) = self.config.extensions {
            extensions
                .iter()
                .any(|e| e == ext || e == &format!(".{}", ext))
        } else {
            false
        }
    }

    /// Activate this server (mark as active)
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate this server (mark as inactive)
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Registry for custom LSP servers.
///
/// This provides thread-safe registration and management of external LSP servers.
#[derive(Debug, Clone, Default)]
pub struct CustomRegistry {
    servers: Arc<RwLock<HashMap<String, CustomLspServer>>>,
}

impl CustomRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new custom LSP server.
    ///
    /// Returns an error if a server with the same ID is already registered.
    pub fn register(&self, server: CustomLspServer) -> Result<(), RegisterError> {
        let mut servers = self
            .servers
            .write()
            .map_err(|_| RegisterError::LockPoisoned)?;
        let id = server.id.clone();
        if servers.contains_key(&id) {
            return Err(RegisterError::AlreadyExists(id));
        }
        servers.insert(id, server);
        Ok(())
    }

    /// Unregister a custom LSP server by ID.
    ///
    /// Returns the server if it was registered.
    pub fn unregister(&self, id: &str) -> Result<CustomLspServer, RegisterError> {
        let mut servers = self
            .servers
            .write()
            .map_err(|_| RegisterError::LockPoisoned)?;
        servers
            .remove(id)
            .ok_or_else(|| RegisterError::NotFound(id.to_string()))
    }

    /// Get a server by ID.
    pub fn get(&self, id: &str) -> Result<Option<CustomLspServer>, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers.get(id).cloned())
    }

    /// Get all registered servers.
    pub fn all(&self) -> Result<Vec<CustomLspServer>, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers.values().cloned().collect())
    }

    /// Get all enabled servers.
    pub fn enabled(&self) -> Result<Vec<CustomLspServer>, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers
            .values()
            .filter(|s| s.config.is_enabled())
            .cloned()
            .collect())
    }

    /// Get servers that handle a specific language.
    pub fn for_language(&self, language: &Language) -> Result<Vec<CustomLspServer>, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers
            .values()
            .filter(|s| s.config.is_enabled() && s.handles_language(language))
            .cloned()
            .collect())
    }

    /// Get servers that handle a specific file extension.
    pub fn for_extension(&self, ext: &str) -> Result<Vec<CustomLspServer>, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers
            .values()
            .filter(|s| s.config.is_enabled() && s.handles_extension(ext))
            .cloned()
            .collect())
    }

    /// Update a registered server.
    pub fn update(&self, server: CustomLspServer) -> Result<(), RegisterError> {
        let mut servers = self
            .servers
            .write()
            .map_err(|_| RegisterError::LockPoisoned)?;
        let id = server.id.clone();
        if !servers.contains_key(&id) {
            return Err(RegisterError::NotFound(id));
        }
        servers.insert(id, server);
        Ok(())
    }

    /// Activate a server by ID.
    pub fn activate(&self, id: &str) -> Result<(), RegisterError> {
        let mut servers = self
            .servers
            .write()
            .map_err(|_| RegisterError::LockPoisoned)?;
        if let Some(server) = servers.get_mut(id) {
            server.activate();
            Ok(())
        } else {
            Err(RegisterError::NotFound(id.to_string()))
        }
    }

    /// Deactivate a server by ID.
    pub fn deactivate(&self, id: &str) -> Result<(), RegisterError> {
        let mut servers = self
            .servers
            .write()
            .map_err(|_| RegisterError::LockPoisoned)?;
        if let Some(server) = servers.get_mut(id) {
            server.deactivate();
            Ok(())
        } else {
            Err(RegisterError::NotFound(id.to_string()))
        }
    }

    /// Check if a server with the given ID is registered.
    pub fn contains(&self, id: &str) -> Result<bool, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers.contains_key(id))
    }

    /// Get the count of registered servers.
    pub fn len(&self) -> Result<usize, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers.len())
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> Result<bool, RegisterError> {
        let servers = self
            .servers
            .read()
            .map_err(|_| RegisterError::LockPoisoned)?;
        Ok(servers.is_empty())
    }

    /// Clear all registered servers.
    pub fn clear(&self) -> Result<(), RegisterError> {
        let mut servers = self
            .servers
            .write()
            .map_err(|_| RegisterError::LockPoisoned)?;
        servers.clear();
        Ok(())
    }

    /// Generate launch configurations for all enabled servers.
    pub fn generate_launch_configs(
        &self,
        root: &PathBuf,
    ) -> Result<Vec<LaunchConfig>, RegisterError> {
        let servers = self.enabled()?;
        Ok(servers
            .iter()
            .filter_map(|server| {
                let language = server
                    .config
                    .languages
                    .as_ref()
                    .and_then(|l| l.first())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());

                let cmd_str = server.command_string();
                if cmd_str.is_empty() {
                    return None;
                }

                Some(LaunchConfig {
                    command: cmd_str,
                    args: Vec::new(),
                    root: server.config.root.clone().unwrap_or_else(|| root.clone()),
                    language,
                    initialization_options: server.config.initialization_options.clone(),
                    settings: server.config.settings.clone(),
                })
            })
            .collect())
    }
}

/// Error type for custom registry operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegisterError {
    /// The registry lock was poisoned (thread safety issue)
    LockPoisoned,
    /// A server with this ID is already registered
    AlreadyExists(String),
    /// No server with this ID was found
    NotFound(String),
}

impl std::fmt::Display for RegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterError::LockPoisoned => write!(f, "Registry lock was poisoned"),
            RegisterError::AlreadyExists(id) => write!(f, "Server '{}' is already registered", id),
            RegisterError::NotFound(id) => write!(f, "Server '{}' not found", id),
        }
    }
}

impl std::error::Error for RegisterError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_server(id: &str) -> CustomLspServer {
        CustomLspServer::new(
            id.to_string(),
            format!("Test Server {}", id),
            CustomServerConfig::new(vec!["test-server".to_string(), "--stdio".to_string()])
                .with_languages(vec!["Rust".to_string(), "TypeScript".to_string()])
                .with_extensions(vec![".rs".to_string(), ".ts".to_string()]),
        )
    }

    #[test]
    fn test_register_and_get() {
        let registry = CustomRegistry::new();
        let server = create_test_server("test1");

        registry.register(server.clone()).unwrap();
        let retrieved = registry.get("test1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test1");
    }

    #[test]
    fn test_register_duplicate() {
        let registry = CustomRegistry::new();
        let server = create_test_server("test1");

        registry.register(server.clone()).unwrap();
        let result = registry.register(server);
        assert!(matches!(result, Err(RegisterError::AlreadyExists(_))));
    }

    #[test]
    fn test_unregister() {
        let registry = CustomRegistry::new();
        let server = create_test_server("test1");

        registry.register(server).unwrap();
        let removed = registry.unregister("test1").unwrap();
        assert_eq!(removed.id, "test1");

        let retrieved = registry.get("test1").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_unregister_not_found() {
        let registry = CustomRegistry::new();
        let result = registry.unregister("nonexistent");
        assert!(matches!(result, Err(RegisterError::NotFound(_))));
    }

    #[test]
    fn test_all_servers() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();
        registry.register(create_test_server("test2")).unwrap();

        let all = registry.all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_enabled_servers() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();

        let mut disabled_server = create_test_server("test2");
        disabled_server.config.enabled = false;
        registry.register(disabled_server).unwrap();

        let enabled = registry.enabled().unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, "test1");
    }

    #[test]
    fn test_for_language() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();

        let rust_servers = registry.for_language(&Language::Rust).unwrap();
        assert_eq!(rust_servers.len(), 1);
        assert_eq!(rust_servers[0].id, "test1");

        let python_servers = registry.for_language(&Language::Python).unwrap();
        assert!(python_servers.is_empty());
    }

    #[test]
    fn test_for_extension() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();

        let rs_servers = registry.for_extension(".rs").unwrap();
        assert_eq!(rs_servers.len(), 1);

        let py_servers = registry.for_extension(".py").unwrap();
        assert!(py_servers.is_empty());
    }

    #[test]
    fn test_activate_deactivate() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();

        assert!(!registry.get("test1").unwrap().unwrap().active);

        registry.activate("test1").unwrap();
        assert!(registry.get("test1").unwrap().unwrap().active);

        registry.deactivate("test1").unwrap();
        assert!(!registry.get("test1").unwrap().unwrap().active);
    }

    #[test]
    fn test_contains() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();

        assert!(registry.contains("test1").unwrap());
        assert!(!registry.contains("test2").unwrap());
    }

    #[test]
    fn test_len_and_is_empty() {
        let registry = CustomRegistry::new();
        assert!(registry.is_empty().unwrap());
        assert_eq!(registry.len().unwrap(), 0);

        registry.register(create_test_server("test1")).unwrap();
        assert!(!registry.is_empty().unwrap());
        assert_eq!(registry.len().unwrap(), 1);
    }

    #[test]
    fn test_clear() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();
        registry.register(create_test_server("test2")).unwrap();

        registry.clear().unwrap();
        assert!(registry.is_empty().unwrap());
    }

    #[test]
    fn test_update() {
        let registry = CustomRegistry::new();
        registry.register(create_test_server("test1")).unwrap();

        let mut updated = create_test_server("test1");
        updated.name = "Updated Name".to_string();

        registry.update(updated).unwrap();
        assert_eq!(registry.get("test1").unwrap().unwrap().name, "Updated Name");
    }

    #[test]
    fn test_update_not_found() {
        let registry = CustomRegistry::new();
        let result = registry.update(create_test_server("nonexistent"));
        assert!(matches!(result, Err(RegisterError::NotFound(_))));
    }

    #[test]
    fn test_capabilities() {
        let caps = ServerCapabilities::new();
        assert!(!caps.supports_hover());
        assert!(!caps.supports_completion());

        let caps_with_hover = ServerCapabilities {
            hover_provider: Some(true),
            ..Default::default()
        };
        assert!(caps_with_hover.supports_hover());
        assert!(!caps_with_hover.supports_completion());
    }

    #[test]
    fn test_custom_server_config() {
        let config = CustomServerConfig::new(vec!["gopls".to_string()])
            .with_languages(vec!["Go".to_string()])
            .with_extensions(vec![".go".to_string()])
            .with_capabilities(ServerCapabilities {
                definition_provider: Some(true),
                ..Default::default()
            });

        assert!(config.is_enabled());
        assert_eq!(config.languages.as_ref().unwrap().len(), 1);
        assert!(config.capabilities.as_ref().unwrap().supports_definition());
    }

    #[test]
    fn test_generate_launch_configs() {
        let registry = CustomRegistry::new();
        let mut server = create_test_server("test1");
        server.config.initialization_options = Some(serde_json::json!({"rootUri": null}));
        registry.register(server).unwrap();

        let root = PathBuf::from("/test/project");
        let configs = registry.generate_launch_configs(&root).unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].command, "test-server --stdio");
    }

    #[test]
    fn test_handles_language() {
        let server = create_test_server("test1");
        assert!(server.handles_language(&Language::Rust));
        assert!(server.handles_language(&Language::TypeScript));
        assert!(!server.handles_language(&Language::Python));
    }

    #[test]
    fn test_handles_extension() {
        let server = create_test_server("test1");
        assert!(server.handles_extension(".rs"));
        assert!(server.handles_extension("rs"));
        assert!(server.handles_extension(".ts"));
        assert!(!server.handles_extension(".py"));
    }

    #[test]
    fn test_register_error_display() {
        assert_eq!(
            RegisterError::AlreadyExists("test".to_string()).to_string(),
            "Server 'test' is already registered"
        );
        assert_eq!(
            RegisterError::NotFound("test".to_string()).to_string(),
            "Server 'test' not found"
        );
    }

    #[test]
    fn custom_registration() {
        let registry = CustomRegistry::new();
        let caps = ServerCapabilities {
            hover_provider: Some(true),
            completion_provider: Some(true),
            definition_provider: Some(true),
            references_provider: Some(true),
            code_action_provider: Some(true),
            text_document_sync: Some(true),
            custom: HashMap::new(),
        };
        let server = CustomLspServer::new(
            "rust-analyzer-custom".to_string(),
            "Rust Analyzer Custom".to_string(),
            CustomServerConfig::new(vec!["rust-analyzer".to_string()])
                .with_languages(vec!["Rust".to_string()])
                .with_extensions(vec![".rs".to_string()])
                .with_capabilities(caps.clone()),
        );
        registry.register(server).unwrap();

        let retrieved = registry.get("rust-analyzer-custom").unwrap().unwrap();
        assert!(retrieved.capabilities.supports_hover());
        assert!(retrieved.capabilities.supports_completion());
        assert!(retrieved.capabilities.supports_definition());
        assert!(retrieved.capabilities.supports_references());
        assert!(retrieved.capabilities.supports_code_action());

        let rust_servers = registry.for_language(&Language::Rust).unwrap();
        assert_eq!(rust_servers.len(), 1);
        assert_eq!(rust_servers[0].id, "rust-analyzer-custom");

        let ts_servers = registry.for_language(&Language::TypeScript).unwrap();
        assert!(ts_servers.is_empty());

        let ext_servers = registry.for_extension(".rs").unwrap();
        assert_eq!(ext_servers.len(), 1);

        let root = PathBuf::from("/test/project");
        let configs = registry.generate_launch_configs(&root).unwrap();
        assert_eq!(configs.len(), 1);
        assert!(configs[0].command.contains("rust-analyzer"));
    }
}
