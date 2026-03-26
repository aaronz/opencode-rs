# Design: Port TypeScript OpenCode to Rust

## Architecture Overview

The Rust port will maintain the same modular architecture as the TypeScript version, with each module implemented as a separate crate or module within the existing crate structure.

### Crate Structure

```
rust-opencode-port/
├── crates/
│   ├── core/          # Core types and utilities
│   ├── cli/           # CLI commands
│   ├── llm/           # LLM providers
│   ├── tools/         # Tool implementations
│   ├── tui/           # Terminal UI
│   ├── agent/         # Agent system
│   └── lsp/           # Language Server Protocol
```

## Core Modules Design

### 1. Configuration System

**Current**: Basic TOML config with 5 fields
**Target**: Multi-layer configuration with JSONC support

```rust
// config.rs
pub struct Config {
    pub provider: ProviderConfig,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    // New fields
    pub instructions: Vec<String>,
    pub plugin: Vec<String>,
    pub tui: TuiConfig,
    pub mcp: McpConfig,
    pub providers: HashMap<String, ProviderConfig>,
}

pub enum ConfigLayer {
    Remote,      // .well-known/opencode
    Global,      // ~/.config/opencode/opencode.json
    Custom,      // OPENCODE_CONFIG
    Project,     // opencode.json
    OpencodeDir, // .opencode/
    Inline,      // OPENCODE_CONFIG_CONTENT
    Managed,     // Enterprise managed
}
```

### 2. Session Management

**Current**: JSON file storage
**Target**: SQLite with full session lifecycle

```rust
// session.rs
pub struct Session {
    pub id: Uuid,
    pub slug: String,
    pub project_id: String,
    pub workspace_id: Option<String>,
    pub directory: String,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub version: i32,
    pub summary: Option<SessionSummary>,
    pub share: Option<ShareInfo>,
    pub revert: Option<RevertInfo>,
    pub permission: Option<String>,
    pub time: SessionTime,
}

pub struct SessionTime {
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub compacting: Option<DateTime<Utc>>,
    pub archived: Option<DateTime<Utc>>,
}
```

### 3. LLM Providers

**Current**: 3 providers (OpenAI, Anthropic, Ollama)
**Target**: 18+ providers

```rust
// provider.rs
#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError>;
    async fn stream_chat(
        &self,
        messages: &[ChatMessage],
    ) -> Result<mpsc::Receiver<Result<StreamChunk, OpenCodeError>>, OpenCodeError>;
}

// New providers to implement
pub struct AzureProvider { ... }
pub struct GoogleProvider { ... }
pub struct VertexProvider { ... }
pub struct BedrockProvider { ... }
pub struct OpenRouterProvider { ... }
pub struct CopilotProvider { ... }
pub struct XaiProvider { ... }
pub struct MistralProvider { ... }
pub struct GroqProvider { ... }
pub struct DeepInfraProvider { ... }
pub struct CerebrasProvider { ... }
pub struct CohereProvider { ... }
pub struct GatewayProvider { ... }
pub struct TogetherAiProvider { ... }
pub struct PerplexityProvider { ... }
pub struct VercelProvider { ... }
pub struct GitLabProvider { ... }
```

### 4. CLI Commands

**Current**: 2 commands (session, list)
**Target**: 23 commands

```rust
// cli.rs
#[derive(Subcommand)]
pub enum Commands {
    Run(RunArgs),
    Generate(GenerateArgs),
    Debug(DebugArgs),
    Account(AccountArgs),
    Providers(ProvidersArgs),
    Agent(AgentArgs),
    Upgrade(UpgradeArgs),
    Uninstall(UninstallArgs),
    Serve(ServeArgs),
    Web(WebArgs),
    Models(ModelsArgs),
    Stats(StatsArgs),
    Export(ExportArgs),
    Import(ImportArgs),
    Github(GithubArgs),
    Pr(PrArgs),
    Session(SessionArgs),
    Db(DbArgs),
    Acp(AcpArgs),
    Mcp(McpArgs),
    TuiThread(TuiThreadArgs),
    Attach(AttachArgs),
    WorkspaceServe(WorkspaceServeArgs),
}
```

### 5. Tools

**Current**: 23 tools
**Target**: 26 tools

```rust
// New tools to implement
pub struct GlobTool { ... }
pub struct PlanTool { ... }
pub struct SchemaTool { ... }
pub struct InvalidTool { ... }
pub struct ExternalDirectoryTool { ... }
pub struct TruncationDirTool { ... }
```

### 6. MCP Support

**Current**: Basic MCP
**Target**: Full MCP with OAuth

```rust
// mcp.rs
pub struct McpManager {
    pub auth: McpAuth,
    pub oauth_provider: OAuthProvider,
    pub oauth_callback: OAuthCallback,
}

pub struct McpAuth { ... }
pub struct OAuthProvider { ... }
pub struct OAuthCallback { ... }
```

## Implementation Strategy

### Phase 1: Core Infrastructure

1. **Enhance Config System**
   - Add JSONC parsing support
   - Implement multi-layer loading
   - Add environment variable overrides
   - Add managed config support

2. **Implement SQLite Storage**
   - Add rusqlite dependency
   - Create session schema
   - Implement CRUD operations
   - Add migration support

3. **Add Authentication System**
   - Implement auth module
   - Add OAuth support
   - Add token management

4. **Implement Environment Management**
   - Add env module
   - Add platform-specific paths
   - Add variable expansion

### Phase 2: LLM Providers

For each provider:
1. Add provider struct
2. Implement Provider trait
3. Add authentication
4. Add streaming support
5. Add error handling
6. Add tests

### Phase 3: CLI Commands

For each command:
1. Add command struct
2. Add argument parsing
3. Implement command logic
4. Add error handling
5. Add tests

### Phase 4: Tools & Features

For each tool:
1. Add tool struct
2. Implement Tool trait
3. Add schema validation
4. Add error handling
5. Add tests

### Phase 5: Polish & Testing

1. Add integration tests
2. Performance optimization
3. Documentation
4. Release preparation

## Dependencies

### New Dependencies

```toml
[dependencies]
# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# Configuration
jsonc-parser = "0.24"

# HTTP
reqwest = { version = "0.12", features = ["json", "stream"] }

# OAuth
oauth2 = "4.4"

# Async
tokio = { version = "1.45", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# CLI
clap = { version = "4.5", features = ["derive"] }

# TUI
ratatui = "0.28"
crossterm = "0.28"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Glob
glob = "0.3"
globset = "0.4"

# UUID
uuid = { version = "1.10", features = ["v4"] }

# Chrono
chrono = { version = "0.4", features = ["serde"] }

# Async trait
async-trait = "0.1"
```

## Testing Strategy

### Unit Tests
- Test each module independently
- Mock external dependencies
- Aim for 80%+ coverage

### Integration Tests
- Test CLI commands end-to-end
- Test tool integrations
- Test provider connections

### Performance Tests
- Compare with TypeScript version
- Identify bottlenecks
- Optimize critical paths

## Migration Path

### For Users
1. Install Rust version alongside TypeScript
2. Migrate configuration
3. Test workflows
4. Switch to Rust version

### For Developers
1. Port code module by module
2. Maintain TypeScript version during transition
3. Deprecate TypeScript version after parity

## Risks

### Technical Risks
- API incompatibilities between TypeScript and Rust
- Missing Rust libraries for some features
- Performance differences

### Mitigations
- Document API differences
- Implement missing libraries
- Profile and optimize
