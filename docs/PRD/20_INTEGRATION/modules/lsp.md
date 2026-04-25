# lsp.md — LSP (Language Server Protocol) Module

## Module Overview

- **Crate**: `opencode-lsp`
- **Source**: `crates/lsp/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Manages LSP server lifecycle (detection, launching, crash recovery), aggregates diagnostics, provides `LspTool` for code navigation, and supports custom/experimental LSP servers.

---

## Crate Layout

```
crates/lsp/src/
├── lib.rs              ← Public re-exports
├── aggregator.rs       ← DiagnosticAggregator
├── builtin.rs          ← BuiltInRegistry, BuiltInServer, BundledConfig, DetectionResult, PathIndicator
├── client.rs           ← LspClient (JSON-RPC LSP client)
├── custom.rs           ← CustomLspServer, CustomRegistry, CustomServerConfig, RegisterError
├── error.rs             ← LspError, CrashCause, FailureHandlingConfig, ProtocolViolationType, UnhealthyReason
├── experimental.rs      ← ExperimentalLspTool, ExperimentalLspToolArgs
├── language.rs         ← Language enum (Rust, TypeScript, Python, etc.)
├── launch.rs           ← LaunchConfig
├── manager.rs           ← LspManager (orchestrates all LSP servers)
├── mock.rs             ← MockLspServer (for testing)
├── server.rs           ← Internal LSP server wrappers
└── types.rs            ← Diagnostic, Location, Severity, Symbol
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1.45", features = ["full"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

opencode-core = { path = "../core" }
```

**Public exports**:
```rust
pub use aggregator::DiagnosticAggregator;
pub use builtin::{BuiltInRegistry, BuiltInServer, BundledConfig, DetectionResult, PathIndicator};
pub use client::LspClient;
pub use custom::{
    CustomLspServer, CustomRegistry, CustomServerConfig, RegisterError, ServerCapabilities,
};
pub use error::{
    CrashCause, FailureHandlingConfig, LspError, ProtocolViolationType, UnhealthyReason,
};
pub use experimental::{ExperimentalLspTool, ExperimentalLspToolArgs};
pub use language::Language;
pub use launch::LaunchConfig;
pub use manager::LspManager;
pub use mock::MockLspServer;
pub use types::{Diagnostic, Location, Severity, Symbol};
```

---

## Core Types

### Language

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    CSharp,
    Ruby,
    PHP,
    Swift,
    Kotlin,
    Scala,
    Html,
    Css,
    Json,
    Yaml,
    Toml,
    Markdown,
    Sql,
    Unknown,
}
```

### LspManager

```rust
pub struct LspManager {
    servers: RwLock<HashMap<Language, Arc<dyn LspServer>>>,
    builtin_registry: BuiltInRegistry,
    custom_registry: CustomRegistry,
    diagnostic_aggregator: DiagnosticAggregator,
}

impl LspManager {
    pub fn new() -> Self;
    pub async fn start_for_project(&self, project_path: &Path) -> Result<(), LspError>;
    pub async fn stop(&self, language: Language) -> Result<(), LspError>;
    pub async fn get_diagnostics(&self, path: &Path) -> Result<Vec<Diagnostic>, LspError>;
    pub async fn symbol_search(&self, query: &str, language: Language) -> Result<Vec<Symbol>, LspError>;
    pub async fn goto_definition(&self, path: &Path, line: u32, col: u32) -> Result<Option<Location>, LspError>;
    pub async fn find_references(&self, path: &Path, line: u32, col: u32) -> Result<Vec<Location>, LspError>;
    pub async fn hover(&self, path: &Path, line: u32, col: u32) -> Result<Option<String>, LspError>;
}

pub trait LspServer: Send + Sync {
    fn language(&self) -> Language;
    fn is_running(&self) -> bool;
    async fn initialize(&self, project_path: &Path) -> Result<(), LspError>;
    async fn shutdown(&self) -> Result<(), LspError>;
    async fn did_change(&self, path: &Path, content: &str) -> Result<(), LspError>;
    async fn diagnostics(&self) -> Result<Vec<Diagnostic>, LspError>;
}
```

### Built-in LSP Servers

```rust
pub struct BuiltInRegistry;

impl BuiltInRegistry {
    pub fn detect_for_project(project_path: &Path) -> Vec<DetectionResult>;
    pub fn launch_server(config: &BundledConfig, project_path: &Path) -> Result<Arc<dyn LspServer>, LspError>;
}

pub struct BundledConfig {
    pub language: Language,
    pub command: Vec<String>,
    pub args: Option<Vec<String>>,
    pub extensions: Option<Vec<String>>,
}

pub struct DetectionResult {
    pub language: Language,
    pub confidence: f32,  // 0.0-1.0
    pub indicator: PathIndicator,
}

pub enum PathIndicator {
    /// LSP server detected via lockfile (e.g., Cargo.toml for Rust)
    Lockfile(String),
    /// Detected via file extension (e.g., .py for Python)
    Extension(String),
    /// Detected via LSP server binary in PATH
    Binary(String),
}
```

### Custom LSP Servers

```rust
pub struct CustomRegistry {
    servers: RwLock<HashMap<String, CustomLspServer>>,
}

impl CustomRegistry {
    pub fn register(&self, name: String, config: CustomServerConfig) -> Result<(), RegisterError>;
    pub fn unregister(&self, name: &str);
    pub fn get(&self, name: &str) -> Option<CustomLspServer>;
    pub fn list(&self) -> Vec<String>;
}

pub struct CustomServerConfig {
    pub name: String,
    pub command: Vec<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub initialization_options: Option<serde_json::Value>,
}

pub struct CustomLspServer {
    pub config: CustomServerConfig,
    pub process: Child,
    pub stdin: ChildStdin,
    pub stdout: BufReader<ChildStdout>,
}

pub struct ServerCapabilities {
    pub diagnostics: bool,
    pub goto_definition: bool,
    pub find_references: bool,
    pub hover: bool,
    pub completion: bool,
    pub symbol_search: bool,
}
```

### LspClient

```rust
pub struct LspClient {
    // JSON-RPC LSP client wrapping an LSP server process
}

impl LspClient {
    pub fn new(process: Child, writer: Stdin, reader: BufReader<Stdout>) -> Self;
    pub async fn initialize(&self, project_path: &Path) -> Result<ServerCapabilities, LspError>;
    pub async fn shutdown(&self) -> Result<(), LspError>;
    pub async fn text_document_did_open(&self, path: &Path, language_id: &str) -> Result<(), LspError>;
    pub async fn text_document_did_change(&self, path: &Path, content: &str) -> Result<(), LspError>;
    pub async fn text_document_did_save(&self, path: &Path) -> Result<(), LspError>;
    pub async fn text_document_did_close(&self, path: &Path) -> Result<(), LspError>;
    pub async fn initialize_result(&self) -> Result<ServerCapabilities, LspError>;
    pub async fn shutdown_request(&self) -> Result<(), LspError>;
}
```

### DiagnosticAggregator

```rust
pub struct DiagnosticAggregator {
    // Aggregates diagnostics from multiple LSP servers
}

impl DiagnosticAggregator {
    pub fn new() -> Self;
    pub fn add_diagnostics(&self, language: Language, path: &Path, diags: Vec<Diagnostic>);
    pub fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic>;
    pub fn clear(&self);
}
```

### LSP Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Severity,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
    pub related_information: Option<Vec<DiagnosticRelated>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub location: Location,
}
```

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("server not found for language: {0:?}")]
    ServerNotFound(Language),
    #[error("server crashed: {0}")]
    ServerCrash(CrashCause),
    #[error("protocol violation: {0}")]
    ProtocolViolation(ProtocolViolationType),
    #[error("server unhealthy: {0}")]
    Unhealthy(UnhealthyReason),
    #[error("timeout waiting for server")]
    Timeout,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub enum CrashCause {
    Oom,
    Signal(i32),
    ExitCode(i32),
    Output(String),
}

#[derive(Debug, Clone)]
pub enum FailureHandlingConfig {
    Restart,
    Fallback,
    Disable,
}

#[derive(Debug, Clone, Copy)]
pub enum UnhealthyReason {
    NoResponse,
    TooManyRestarts,
    MemoryPressure,
}
```

### Experimental LSP Tool

```rust
pub struct ExperimentalLspTool {
    manager: Arc<LspManager>,
}

impl ExperimentalLspTool {
    pub fn new(manager: Arc<LspManager>) -> Self;
    pub fn args_from_config(config: &serde_json::Value) -> Result<ExperimentalLspToolArgs, LspError>;
}

pub struct ExperimentalLspToolArgs {
    pub subcommand: LspSubcommand,
}

pub enum LspSubcommand {
    Start { project_path: Option<String>, language: Option<String> },
    Stop { language: String },
    Restart { language: String },
    Status,
    Diagnostics { path: String },
    GotoDefinition { path: String, line: u32, col: u32 },
    FindReferences { path: String, line: u32, col: u32 },
    SymbolSearch { query: String, language: String },
}
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-lsp` |
|---|---|
| `opencode-tools` | `LspTool` registered as a tool |
| `opencode-server` | `LspManager` for project diagnostics |
| `opencode-tui` | `LspTool` for code navigation in UI |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_serialization() {
        let lang = Language::Rust;
        let json = serde_json::to_string(&lang).unwrap();
        assert_eq!(json, "\"rust\"");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Information);
    }

    #[test]
    fn test_diagnostic_serialization() {
        let diag = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 10 },
            },
            severity: Severity::Error,
            code: Some("E0001".to_string()),
            source: Some("rustc".to_string()),
            message: "expected item".to_string(),
            related_information: None,
        };
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("\"message\":\"expected item\""));
    }

    #[tokio::test]
    async fn test_lsp_manager_starts_for_rust_project() {
        let manager = LspManager::new();
        let result = manager.start_for_project(Path::new("/tmp/rust-project")).await;
        // Depends on whether rust-analyzer is available
    }

    #[tokio::test]
    async fn test_mock_lsp_server() {
        let mock = MockLspServer::new();
        mock.initialize().await.unwrap();
        mock.send_diagnostics(vec![]).await;
        mock.shutdown().await.unwrap();
    }
}
```

---

## Usage Example

```rust
use opencode_lsp::{LspManager, Language, DiagnosticAggregator};

async fn lsp_example() -> Result<(), LspError> {
    let manager = LspManager::new();
    
    // Start LSP server for a project
    manager.start_for_project(Path::new("/path/to/project")).await?;
    
    // Get diagnostics for a file
    let diags = manager.get_diagnostics(Path::new("/path/to/project/src/main.rs")).await?;
    for diag in diags {
        println!("{:?} at {:?}: {}", diag.severity, diag.range, diag.message);
    }
    
    // Search for symbols
    let symbols = manager.symbol_search("main", Language::Rust).await?;
    for sym in symbols {
        println!("Found {} at {:?}", sym.name, sym.location);
    }
    
    // Goto definition
    if let Some(loc) = manager.goto_definition(Path::new("/path/to/project/src/main.rs"), 10, 5).await? {
        println!("Definition at {}:{:?}", loc.uri, loc.range);
    }
    
    Ok(())
}
```
