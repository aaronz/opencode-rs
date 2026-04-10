//! Built-in LSP server detection and bundling mechanism.
//!
//! This module provides automatic detection and launching of language servers
//! that are bundled with the application.

use crate::language::Language;
use crate::launch::LaunchConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents a built-in LSP server that can be bundled with the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuiltInServer {
    /// Unique identifier for the server (e.g., "rust-analyzer", "gopls")
    pub id: String,
    /// Display name (e.g., "Rust Analyzer", "Go Language Server")
    pub name: String,
    /// Languages this server supports
    pub languages: Vec<Language>,
    /// Primary command to invoke
    pub command: String,
    /// Additional arguments
    pub default_args: Vec<String>,
    /// File patterns that indicate this server should be used
    pub indicators: Vec<PathIndicator>,
    /// If true, any indicator matching is sufficient; if false, all must match
    pub match_any: bool,
    /// Whether this server is typically bundled/embedded
    pub is_bundled: bool,
}

/// Indicates a file or directory that suggests a particular language server.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PathIndicator {
    /// A file that must exist (e.g., "Cargo.toml", "go.mod")
    FileExists(String),
    /// A directory that must exist
    DirExists(String),
    /// A file with specific content (checks for substring)
    FileContains { path: String, content: String },
}

impl PathIndicator {
    /// Check if this indicator matches the given root path.
    pub fn matches(&self, root: &Path) -> bool {
        match self {
            PathIndicator::FileExists(name) => root.join(name).exists(),
            PathIndicator::DirExists(name) => root.join(name).is_dir(),
            PathIndicator::FileContains { path, content } => {
                let full_path = root.join(path);
                if let Ok(contents) = std::fs::read_to_string(&full_path) {
                    contents.contains(content)
                } else {
                    false
                }
            }
        }
    }
}

/// Registry of all built-in LSP servers.
pub struct BuiltInRegistry {
    servers: Vec<BuiltInServer>,
}

impl Default for BuiltInRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltInRegistry {
    /// Create a new registry with all known built-in servers.
    pub fn new() -> Self {
        let servers = vec![
            BuiltInServer {
                id: "rust-analyzer".to_string(),
                name: "Rust Analyzer".to_string(),
                languages: vec![Language::Rust],
                command: "rust-analyzer".to_string(),
                default_args: vec![],
                indicators: vec![PathIndicator::FileExists("Cargo.toml".to_string())],
                match_any: false,
                is_bundled: true,
            },
            BuiltInServer {
                id: "typescript-language-server".to_string(),
                name: "TypeScript Language Server".to_string(),
                languages: vec![Language::TypeScript, Language::JavaScript],
                command: "typescript-language-server".to_string(),
                default_args: vec!["--stdio".to_string()],
                indicators: vec![
                    PathIndicator::FileExists("tsconfig.json".to_string()),
                    PathIndicator::FileExists("package.json".to_string()),
                ],
                match_any: false,
                is_bundled: true,
            },
            BuiltInServer {
                id: "javascript-language-server".to_string(),
                name: "JavaScript Language Server".to_string(),
                languages: vec![Language::JavaScript],
                command: "javascript-language-server".to_string(),
                default_args: vec!["--stdio".to_string()],
                indicators: vec![PathIndicator::FileExists("package.json".to_string())],
                match_any: false,
                is_bundled: true,
            },
            BuiltInServer {
                id: "pylsp".to_string(),
                name: "Python Language Server".to_string(),
                languages: vec![Language::Python],
                command: "pylsp".to_string(),
                default_args: vec![],
                indicators: vec![
                    PathIndicator::FileExists("pyproject.toml".to_string()),
                    PathIndicator::FileExists("setup.py".to_string()),
                    PathIndicator::FileExists("requirements.txt".to_string()),
                ],
                match_any: true,
                is_bundled: true,
            },
            BuiltInServer {
                id: "gopls".to_string(),
                name: "Go Language Server".to_string(),
                languages: vec![Language::Go],
                command: "gopls".to_string(),
                default_args: vec![],
                indicators: vec![PathIndicator::FileExists("go.mod".to_string())],
                match_any: false,
                is_bundled: true,
            },
        ];

        Self { servers }
    }

    /// Get all registered built-in servers.
    pub fn servers(&self) -> &[BuiltInServer] {
        &self.servers
    }

    /// Find a built-in server by its ID.
    pub fn get(&self, id: &str) -> Option<&BuiltInServer> {
        self.servers.iter().find(|s| s.id == id)
    }

    /// Detect which built-in servers should be used for a project at the given root.
    pub fn detect_for_root(&self, root: &Path) -> Vec<&BuiltInServer> {
        self.servers
            .iter()
            .filter(|server| {
                if server.indicators.is_empty() {
                    return false;
                }
                if server.match_any {
                    server
                        .indicators
                        .iter()
                        .any(|indicator| indicator.matches(root))
                } else {
                    server
                        .indicators
                        .iter()
                        .all(|indicator| indicator.matches(root))
                }
            })
            .collect()
    }

    /// Detect servers for a specific language.
    pub fn detect_for_language(&self, language: &Language) -> Vec<&BuiltInServer> {
        self.servers
            .iter()
            .filter(|server| server.languages.contains(language))
            .collect()
    }

    /// Check if a specific built-in server is available (command exists).
    pub fn is_available(&self, id: &str) -> bool {
        if let Some(server) = self.get(id) {
            which::which(&server.command).is_ok()
        } else {
            false
        }
    }

    /// Get all available built-in servers.
    pub fn available_servers(&self) -> Vec<&BuiltInServer> {
        self.servers
            .iter()
            .filter(|s| which::which(&s.command).is_ok())
            .collect()
    }

    /// Generate launch configurations for detected servers at a root path.
    pub fn generate_launch_configs(&self, root: &Path) -> Vec<LaunchConfig> {
        self.detect_for_root(root)
            .iter()
            .filter_map(|server| {
                Some(LaunchConfig {
                    command: server.command.clone(),
                    args: server.default_args.clone(),
                    root: root.to_path_buf(),
                    language: server.languages.first()?.name().to_lowercase(),
                    initialization_options: None,
                    settings: None,
                })
            })
            .collect()
    }
}

/// Result of built-in server detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// The server that was detected
    pub server: String,
    /// Whether the server command is available on the system
    pub is_available: bool,
    /// The language detected
    pub language: String,
    /// The command that would be used
    pub command: String,
}

impl BuiltInRegistry {
    /// Perform detection and return detailed results.
    pub fn detect_with_details(&self, root: &Path) -> Vec<DetectionResult> {
        self.detect_for_root(root)
            .iter()
            .map(|server| {
                let is_available = which::which(&server.command).is_ok();
                DetectionResult {
                    server: server.name.clone(),
                    is_available,
                    language: server
                        .languages
                        .first()
                        .map(|l| l.name().to_string())
                        .unwrap_or_default(),
                    command: if server.default_args.is_empty() {
                        server.command.clone()
                    } else {
                        format!("{} {}", server.command, server.default_args.join(" "))
                    },
                }
            })
            .collect()
    }
}

/// Configuration for bundled LSP servers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundledConfig {
    /// Enable or disable built-in server detection.
    pub detection_enabled: bool,
    /// List of server IDs to exclude from auto-detection.
    pub excluded_servers: Vec<String>,
    /// Custom paths for bundled servers.
    pub custom_paths: std::collections::HashMap<String, String>,
}

impl BundledConfig {
    /// Create a default bundled configuration with detection enabled.
    pub fn default_enabled() -> Self {
        Self {
            detection_enabled: true,
            excluded_servers: Vec::new(),
            custom_paths: std::collections::HashMap::new(),
        }
    }

    /// Get the actual command to use for a server, considering custom paths.
    pub fn get_command(&self, server_id: &str, default_command: &str) -> String {
        self.custom_paths
            .get(server_id)
            .cloned()
            .unwrap_or_else(|| default_command.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn detects_rust_server() {
        let dir = create_test_dir();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let registry = BuiltInRegistry::new();
        let detected = registry.detect_for_root(dir.path());

        assert!(detected.iter().any(|s| s.id == "rust-analyzer"));
    }

    #[test]
    fn detects_typescript_server() {
        let dir = create_test_dir();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();

        let registry = BuiltInRegistry::new();
        let detected = registry.detect_for_root(dir.path());

        assert!(detected
            .iter()
            .any(|s| s.id == "typescript-language-server"));
    }

    #[test]
    fn detects_go_server() {
        let dir = create_test_dir();
        fs::write(dir.path().join("go.mod"), "module test").unwrap();

        let registry = BuiltInRegistry::new();
        let detected = registry.detect_for_root(dir.path());

        assert!(detected.iter().any(|s| s.id == "gopls"));
    }

    #[test]
    fn detects_python_server() {
        let dir = create_test_dir();
        fs::write(dir.path().join("pyproject.toml"), "[project]").unwrap();

        let registry = BuiltInRegistry::new();
        let detected = registry.detect_for_root(dir.path());

        assert!(detected.iter().any(|s| s.id == "pylsp"));
    }

    #[test]
    fn generates_launch_configs() {
        let dir = create_test_dir();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let registry = BuiltInRegistry::new();
        let configs = registry.generate_launch_configs(dir.path());

        assert!(!configs.is_empty());
        assert!(configs.iter().any(|c| c.command == "rust-analyzer"));
    }

    #[test]
    fn bundled_config_custom_paths() {
        let mut config = BundledConfig::default();
        config.custom_paths.insert(
            "rust-analyzer".to_string(),
            "/custom/path/rust-analyzer".to_string(),
        );

        assert_eq!(
            config.get_command("rust-analyzer", "rust-analyzer"),
            "/custom/path/rust-analyzer"
        );
    }

    #[test]
    fn builtin_detection_test() {
        let registry = BuiltInRegistry::new();

        assert!(registry.get("rust-analyzer").is_some());
        assert!(registry.get("typescript-language-server").is_some());
        assert!(registry.get("gopls").is_some());
        assert!(registry.get("pylsp").is_some());

        let dir = create_test_dir();
        let results = registry.detect_with_details(dir.path());
        assert!(results.is_empty());

        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        let results = registry.detect_with_details(dir.path());
        assert!(!results.is_empty());
        assert_eq!(results[0].server, "Rust Analyzer");
    }

    #[test]
    fn detection_result_serialization() {
        let result = DetectionResult {
            server: "Rust Analyzer".to_string(),
            is_available: true,
            language: "Rust".to_string(),
            command: "rust-analyzer".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: DetectionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.server, result.server);
        assert_eq!(deserialized.is_available, result.is_available);
    }
}
