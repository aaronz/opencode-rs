## Context

The original OpenCode (TypeScript) is a mature AI coding agent with:
- 10,580+ commits, 131k GitHub stars
- Built on TypeScript (56%), MDX (40%), minimal Rust (0.5%)
- Client/server architecture with TUI frontend
- Multi-provider LLM support (OpenAI, Anthropic, Google, local)
- Comprehensive tool system (file ops, grep, web search, git, LSP)

This Rust port aims to achieve feature parity while improving:
- Binary distribution (single .exe, no Node.js runtime)
- Startup performance and memory efficiency
- Type safety and maintainability

## Goals / Non-Goals

**Goals:**
1. Achieve feature parity with OpenCode TypeScript version
2. Provide CLI-first experience matching original TUI
3. Support multiple LLM providers (OpenAI, Anthropic, Google, Ollama)
4. Implement core tool system (file, grep, web search, git)
5. Create extensible plugin architecture
6. Enable streaming responses for real-time feedback

**Non-Goals:**
1. Desktop GUI application (future consideration)
2. Browser-based interface (separate project)
3. Real-time collaboration features (v2)
4. Cloud/saas infrastructure (separate services)

## Decisions

### 1. Architecture: Monolithic Binary
**Decision**: Single Rust binary with internal crate modules rather than microservices.

**Rationale**: 
- Simpler deployment (one executable)
- Lower latency for tool calls
- Original OpenCode uses client/server but not strictly necessary for CLI tool
- Easier to maintain and distribute

**Alternative Considered**: Separate CLI and server processes like original.
*Rejected*: Adds complexity without clear benefit for CLI-first use case.

### 2. Async Runtime: Tokio
**Decision**: Use Tokio as the async runtime.

**Rationale**:
- Industry standard for Rust async
- Rich ecosystem (tokio-util, etc.)
- Well-documented and maintained
- Compatible with most async crates

**Alternative Considered**: async-std, smol
*Rejected*: Smaller ecosystem, less community support.

### 3. TUI Framework: Ratatui
**Decision**: Use Ratatui for terminal UI.

**Rationale**:
- Actively maintained fork of tui-rs
- Good widget ecosystem
- Supports modern terminal features
- Similar API to original's frontend

**Alternative Considered**: cursive, treemacs
*Rejected*: Cursive is more imperative, treemacs is Emacs-specific.

### 4. LLM Provider Abstraction: Trait-Based
**Decision**: Define a Provider trait for pluggable LLM backends.

**Rationale**:
- Rust's trait system provides compile-time polymorphism
- Easy to add new providers without modifying core code
- Enables dynamic provider selection at runtime via config

**Implementation**:
```rust
trait LLMProvider {
    async fn chat(&self, messages: &[Message]) -> Result<Response, Error>;
    async fn stream_chat(&self, messages: &[Message]) -> Result<StreamResponse, Error>;
}
```

### 5. Tool System: Async Trait Methods
**Decision**: Define tools as async trait methods with typed inputs/outputs.

**Rationale**:
- Type-safe tool definitions
- Easy to compose and chain tools
- Clear separation of concerns
- Enables tool introspection and discovery

### 6. Configuration: TOML Format
**Decision**: Use TOML for configuration files.

**Rationale**:
- Native Rust support (toml crate)
- More idiomatic than YAML for Rust projects
- Clearer syntax, less error-prone

**Alternative Considered**: JSON, YAML
*Rejected*: JSON less readable for config, YAML requires additional dependency.

### 7. Error Handling: Thiserror + Anyhow
**Decision**: Use thiserror for library errors, anyhow for application errors.

**Rationale**:
- thiserror provides clean error enum derivation
- anyhow enables flexible error handling in CLI
- Standard Rust error handling best practices

### 8. Logging: Tracing
**Decision**: Use tracing for structured logging.

**Rationale**:
- Async-friendly logging
- Structured log events with spans
- Compatible with OpenTelemetry
- Industry standard for Rust

### 9. Serialization: Serde
**Decision**: Use serde with JSON as primary format.

**Rationale**:
- Mature, well-tested
- Supports JSON, YAML, TOML
- Zero-copy parsing where possible

### 10. LSP Integration: Custom Client
**Decision**: Implement lightweight LSP client instead of depending on existing crates.

**Rationale**:
- Need only specific LSP features (diagnostics, goto definition, find references)
- Full LSP crates are heavy
- Easier to customize for OpenCode's needs

## Risks / Trade-offs

### [Risk] Feature Parity Gap
**Mitigation**: 
- Start with MVP (core chat + basic tools)
- Prioritize tools by usage frequency
- Document feature gaps in README

### [Risk] Performance Regression
**Mitigation**:
- Benchmark critical paths
- Profile memory usage
- Optimize hot paths (token streaming, tool execution)

### [Risk] LLM Provider API Changes
**Mitigation**:
- Abstract provider behind traits
- Version API responses
- Maintain compatibility layer

### [Risk] TUI Complexity
**Mitigation**:
- Use established Ratatui widgets
- Start with simple UI, iterate
- Test on multiple terminal emulators

### [Risk] Config Migration
**Mitigation**:
- Provide migration script
- Support both YAML and TOML initially
- Document config changes clearly

### [Risk] Community Adoption
**Mitigation**:
- Publish to crates.io
- Create comprehensive documentation
- Engage with Rust community early

## Migration Plan

### Phase 1: Core Foundation (Weeks 1-2)
1. Set up Cargo workspace
2. Implement logging and error handling
3. Create basic CLI structure with clap
4. Set up configuration system (TOML)

### Phase 2: LLM Integration (Weeks 3-4)
1. Implement Provider trait
2. Add OpenAI provider (GPT-4, GPT-4o)
3. Add Anthropic provider (Claude)
4. Add Ollama provider (local models)
5. Implement streaming responses

### Phase 3: Tool System (Weeks 5-6)
1. Define tool trait and registry
2. Implement file operations (read, write, edit, glob, grep)
3. Implement web search tool
4. Implement git tools
5. Add tool result formatting

### Phase 4: Agent System (Weeks 7-8)
1. Implement agent trait
2. Create build agent (default, full access)
3. Create plan agent (read-only)
4. Implement message handling and context management

### Phase 5: TUI (Weeks 9-10)
1. Set up Ratatui application
2. Create message view (user/assistant)
3. Create input area with history
4. Add status bar and tool output panel

### Phase 6: Polish & Release (Weeks 11-12)
1. Add session management
2. Configuration validation
3. Error handling improvements
4. Documentation and examples
5. Release v0.1.0

## Open Questions

1. **Provider Priority**: Which provider should be default? (OpenAI GPT-4o most capable, Anthropic Claude often better for coding)

2. **Config Location**: Should config be `~/.config/opencode-rs/` or `~/.opencode/` for backward compatibility?

3. **Session Format**: Store sessions as JSON or SQLite? (JSON simpler, SQLite better for large histories)

4. **Tool Sandboxing**: Should tools run in isolated processes for security? (Future consideration)

5. **Plugin System**: Should plugins be Rust crates or external processes? (Crates more type-safe, processes more flexible)

6. **Versioning**: How to handle version compatibility with original OpenCode config format?
