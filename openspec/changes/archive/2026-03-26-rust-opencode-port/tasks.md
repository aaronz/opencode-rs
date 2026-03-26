## 1. Project Setup

- [x] 1.1 Initialize Cargo workspace structure
- [x] 1.2 Create main crate with clap for CLI argument parsing
- [x] 1.3 Set up logging with tracing crate
- [x] 1.4 Configure error handling with anyhow and thiserror
- [x] 1.5 Add dependencies: tokio, serde, reqwest, ratatui, etc.
- [x] 1.6 Create basic project structure (src/main.rs, lib.rs)

## 2. Configuration System

- [x] 2.1 Implement configuration struct with serde
- [x] 2.2 Create default configuration
- [x] 2.3 Implement config file loading from ~/.config/opencode-rs/config.toml
- [x] 2.4 Add environment variable override support
- [x] 2.5 Add --config flag for custom config path

## 3. LLM Provider Abstraction

- [x] 3.1 Define LLM Provider trait
- [x] 3.2 Create Message and Response structs
- [x] 3.3 Implement streaming response support
- [x] 3.4 Implement OpenAI provider with reqwest
- [x] 3.5 Implement Anthropic provider
- [x] 3.6 Implement Ollama provider for local models
- [x] 3.7 Add provider configuration and switching

## 4. Tool System

- [x] 4.1 Define Tool trait
- [x] 4.2 Create ToolRegistry for managing tools
- [x] 4.3 Implement FileRead tool
- [x] 4.4 Implement FileWrite tool
- [x] 4.5 Implement Glob tool using walkdir
- [x] 4.6 Implement Grep tool using regex
- [x] 4.7 Implement WebSearch tool using search API
- [x] 4.8 Implement GitStatus tool
- [x] 4.9 Implement GitDiff tool
- [x] 4.10 Add tool execution parallelization with tokio

## 5. Agent System

- [x] 5.1 Define Agent trait
- [x] 5.2 Implement BuildAgent (default, full access)
- [x] 5.3 Implement PlanAgent (read-only mode)
- [x] 5.4 Implement General subagent for complex searches
- [x] 5.5 Add agent switching mechanism
- [x] 5.6 Implement message handling and context management

## 6. Session Management

- [x] 6.1 Create Session struct with unique ID
- [x] 6.2 Implement Message history storage
- [x] 6.3 Add session persistence to disk (JSON)
- [x] 6.4 Implement session loading by ID
- [x] 6.5 Create session list functionality
- [x] 6.6 Add session deletion capability
- [x] 6.7 Implement context window management

## 7. LSP Client

- [x] 7.1 Implement LSP client using JSON-RPC over stdio
- [x] 7.2 Add language server detection (rust-analyzer, pyright, etc.)
- [x] 7.3 Implement diagnostics display
- [x] 7.4 Add go to definition functionality
- [x] 7.5 Implement find references
- [x] 7.6 Add code completion support

## 8. Terminal UI (TUI)

- [x] 8.1 Set up Ratatui application
- [x] 8.2 Implement main layout (messages, input, status)
- [x] 8.3 Create message display with markdown rendering
- [x] 8.4 Implement streaming response display
- [x] 8.5 Add input area with history
- [x] 8.6 Implement command palette (Ctrl+P)
- [x] 8.7 Add tool output panel
- [x] 8.8 Implement status bar (agent, provider)
- [x] 8.9 Add keyboard shortcuts (Tab for agent switch, arrows for history)
- [x] 8.10 Handle window resize events

## 9. Integration & Polish

- [x] 9.1 Integrate all components (CLI -> Agent -> LLM -> Tools -> TUI)
- [x] 9.2 Add proper error messages and user feedback
- [x] 9.3 Implement graceful shutdown
- [x] 9.4 Add progress indicators for long operations
- [x] 9.5 Test on various terminal emulators
- [x] 9.6 Optimize performance and memory usage

## 10. Documentation & Release

- [x] 10.1 Create README.md
- [x] 10.2 Write configuration documentation
- [x] 10.3 Add usage examples
- [x] 10.4 Create Cargo.toml for publishing to crates.io
- [x] 10.5 Add CI/CD for building releases
- [x] 10.6 Release v0.1.0
