# Tasks: Port TypeScript OpenCode to Rust

## Phase 1: Core Infrastructure (Week 1-2)

### 1.1 Enhance Configuration System
- [ ] Add JSONC parsing support
- [ ] Implement multi-layer config loading
- [ ] Add environment variable overrides
- [ ] Add managed config support
- [ ] Add TUI config
- [ ] Add MCP config
- [ ] Add provider-specific config
- [ ] Write config tests

### 1.2 Implement SQLite Storage
- [ ] Add rusqlite dependency
- [ ] Create session schema
- [ ] Create message schema
- [ ] Create project schema
- [ ] Implement CRUD operations
- [ ] Add migration support
- [ ] Write storage tests

### 1.3 Add Authentication System
- [ ] Create auth module
- [ ] Implement OAuth flow
- [ ] Add token storage
- [ ] Add token refresh
- [ ] Write auth tests

### 1.4 Implement Environment Management
- [ ] Create env module
- [ ] Add platform-specific paths
- [ ] Add variable expansion
- [ ] Write env tests

### 1.5 Add File Operations
- [ ] Create file module
- [ ] Implement filesystem abstraction
- [ ] Add path utilities
- [ ] Write file tests

### 1.6 Add Git Integration
- [ ] Create git module
- [ ] Implement git operations
- [ ] Add diff support
- [ ] Write git tests

## Phase 2: LLM Providers (Week 3-4)

### 2.1 Add Azure Provider
- [ ] Create Azure provider struct
- [ ] Implement Provider trait
- [ ] Add Azure authentication
- [ ] Add streaming support
- [ ] Write Azure tests

### 2.2 Add Google Provider
- [ ] Create Google provider struct
- [ ] Implement Provider trait
- [ ] Add Google authentication
- [ ] Add streaming support
- [ ] Write Google tests

### 2.3 Add Vertex Provider
- [ ] Create Vertex provider struct
- [ ] Implement Provider trait
- [ ] Add Vertex authentication
- [ ] Add streaming support
- [ ] Write Vertex tests

### 2.4 Add Bedrock Provider
- [ ] Create Bedrock provider struct
- [ ] Implement Provider trait
- [ ] Add AWS authentication
- [ ] Add streaming support
- [ ] Write Bedrock tests

### 2.5 Add OpenRouter Provider
- [ ] Create OpenRouter provider struct
- [ ] Implement Provider trait
- [ ] Add OpenRouter authentication
- [ ] Add streaming support
- [ ] Write OpenRouter tests

### 2.6 Add Copilot Provider
- [ ] Create Copilot provider struct
- [ ] Implement Provider trait
- [ ] Add Copilot authentication
- [ ] Add streaming support
- [ ] Write Copilot tests

### 2.7 Add Remaining Providers
- [ ] Add Xai provider
- [ ] Add Mistral provider
- [ ] Add Groq provider
- [ ] Add DeepInfra provider
- [ ] Add Cerebras provider
- [ ] Add Cohere provider
- [ ] Add Gateway provider
- [ ] Add TogetherAI provider
- [ ] Add Perplexity provider
- [ ] Add Vercel provider
- [ ] Add GitLab provider
- [ ] Write tests for all providers

## Phase 3: CLI Commands (Week 5-6)

### 3.1 Implement Run Command
- [ ] Add RunCommand struct
- [ ] Add argument parsing
- [ ] Implement command logic
- [ ] Add error handling
- [ ] Write run tests

### 3.2 Implement Generate Command
- [ ] Add GenerateCommand struct
- [ ] Add argument parsing
- [ ] Implement command logic
- [ ] Add error handling
- [ ] Write generate tests

### 3.3 Implement Serve Command
- [ ] Add ServeCommand struct
- [ ] Add argument parsing
- [ ] Implement HTTP server
- [ ] Add WebSocket support
- [ ] Write serve tests

### 3.4 Implement Debug Commands
- [ ] Add DebugCommand struct
- [ ] Implement config debug
- [ ] Implement file debug
- [ ] Implement lsp debug
- [ ] Implement agent debug
- [ ] Implement snapshot debug
- [ ] Implement skill debug
- [ ] Write debug tests

### 3.5 Implement Account Commands
- [ ] Add AccountCommand struct
- [ ] Implement login
- [ ] Implement logout
- [ ] Implement status
- [ ] Write account tests

### 3.6 Implement Remaining Commands
- [ ] Add providers command
- [ ] Add agent command
- [ ] Add upgrade command
- [ ] Add uninstall command
- [ ] Add web command
- [ ] Add models command
- [ ] Add stats command
- [ ] Add export command
- [ ] Add import command
- [ ] Add github command
- [ ] Add pr command
- [ ] Add db command
- [ ] Add acp command
- [ ] Add mcp command
- [ ] Add tui-thread command
- [ ] Add attach command
- [ ] Add workspace-serve command
- [ ] Write tests for all commands

## Phase 4: Tools & Features (Week 7-8)

### 4.1 Add Missing Tools
- [ ] Implement glob tool
- [ ] Implement plan tool
- [ ] Implement schema tool
- [ ] Implement invalid tool
- [ ] Implement external-directory tool
- [ ] Implement truncation-dir tool
- [ ] Write tests for all tools

### 4.2 Enhance MCP Support
- [ ] Add MCP auth
- [ ] Add OAuth callback
- [ ] Add OAuth provider
- [ ] Write MCP tests

### 4.3 Add IDE Integration
- [ ] Create IDE module
- [ ] Add VS Code support
- [ ] Add IntelliJ support
- [ ] Write IDE tests

### 4.4 Add Skill System
- [ ] Create skill module
- [ ] Implement skill loading
- [ ] Implement skill execution
- [ ] Write skill tests

### 4.5 Add Plugin System
- [ ] Enhance plugin module
- [ ] Add plugin loading
- [ ] Add plugin execution
- [ ] Write plugin tests

## Phase 5: Polish & Testing (Week 9-10)

### 5.1 Add Integration Tests
- [ ] Test CLI commands end-to-end
- [ ] Test tool integrations
- [ ] Test provider connections
- [ ] Test session management

### 5.2 Performance Optimization
- [ ] Profile critical paths
- [ ] Optimize hot spots
- [ ] Add benchmarks
- [ ] Compare with TypeScript

### 5.3 Documentation
- [ ] Update README
- [ ] Add API documentation
- [ ] Add usage examples
- [ ] Add migration guide

### 5.4 Release Preparation
- [ ] Version bump
- [ ] Changelog
- [ ] Release notes
- [ ] Binary distribution

## Success Criteria

### Functional Requirements
- [ ] All TypeScript CLI commands have Rust equivalents
- [ ] All TypeScript tools have Rust implementations
- [ ] All TypeScript providers have Rust implementations
- [ ] All TypeScript modules have Rust implementations
- [ ] The Rust version can run the same workflows as TypeScript

### Non-Functional Requirements
- [ ] Tests pass for all implemented features
- [ ] Documentation is complete
- [ ] Performance is comparable to TypeScript
- [ ] Code follows Rust best practices

## Dependencies

### External Dependencies
- rusqlite for SQLite
- jsonc-parser for JSONC
- reqwest for HTTP
- oauth2 for OAuth
- tokio for async
- serde for serialization
- clap for CLI
- ratatui for TUI
- glob/globset for glob patterns

### Internal Dependencies
- Core modules must be implemented first
- Providers depend on core modules
- CLI commands depend on providers and tools
- Tools depend on core modules

## Risks

### Technical Risks
- API incompatibilities between TypeScript and Rust
- Missing Rust libraries for some features
- Performance differences

### Mitigations
- Document API differences
- Implement missing libraries
- Profile and optimize

## Timeline

- **Week 1-2**: Core Infrastructure
- **Week 3-4**: LLM Providers
- **Week 5-6**: CLI Commands
- **Week 7-8**: Tools & Features
- **Week 9-10**: Polish & Testing

Total: 10 weeks
