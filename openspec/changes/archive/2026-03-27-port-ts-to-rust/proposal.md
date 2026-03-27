# Proposal: Port TypeScript OpenCode to Rust

## Problem Statement

The current Rust implementation of OpenCode (`rust-opencode-port`) is an early prototype with only ~20% feature parity compared to the mature TypeScript implementation. This creates a significant gap that prevents users from migrating to the Rust version.

### Current State Analysis

**TypeScript OpenCode (Reference)**
- **23 CLI commands** including run, generate, serve, debug, TUI, etc.
- **26 tools** including glob, plan, schema, etc.
- **18+ LLM providers** including Azure, Google, Bedrock, OpenRouter, Copilot, etc.
- **44 core modules** with full implementation
- **18 session management files** with SQLite storage
- **6 config files** with multi-layer configuration
- **4 MCP files** with OAuth support

**Rust OpenCode Port (Current)**
- **2 CLI commands** (session, list)
- **23 tools** (missing glob, plan, schema, etc.)
- **3 LLM providers** (OpenAI, Anthropic, Ollama)
- **37 core modules** (many are stubs)
- **1 session file** (basic JSON storage)
- **1 config file** (basic TOML)
- **1 MCP file** (basic implementation)

## Proposed Solution

Implement a comprehensive porting strategy to achieve complete feature parity between the TypeScript and Rust implementations.

### Goals

1. **Complete CLI Parity**: Port all 23 TypeScript CLI commands to Rust
2. **Complete Tool Parity**: Port all 26 TypeScript tools to Rust
3. **Complete Provider Parity**: Port all 18+ LLM providers to Rust
4. **Complete Module Parity**: Implement all 44 core modules in Rust
5. **Complete Session Parity**: Port session management to SQLite
6. **Complete Config Parity**: Implement multi-layer configuration
7. **Complete MCP Parity**: Implement full MCP with OAuth

### Non-Goals

1. **JavaScript/TypeScript Interop**: This is a pure Rust port, not a bridge
2. **Backward Compatibility**: The Rust version may have different APIs
3. **Performance Optimization**: Focus on feature parity first, optimize later

## Success Criteria

1. All TypeScript CLI commands have Rust equivalents
2. All TypeScript tools have Rust implementations
3. All TypeScript providers have Rust implementations
4. All TypeScript modules have Rust implementations
5. The Rust version can run the same workflows as TypeScript
6. Tests pass for all implemented features

## Risks and Mitigations

### Risk 1: Scope Creep
**Mitigation**: Prioritize core features first, defer nice-to-haves

### Risk 2: API Differences
**Mitigation**: Maintain API compatibility where possible, document differences

### Risk 3: Testing Coverage
**Mitigation**: Write tests as we port, don't defer testing

### Risk 4: Timeline Overrun
**Mitigation**: Break into phases, deliver incrementally

## Timeline

- **Phase 1**: Core Infrastructure (Weeks 1-2)
- **Phase 2**: LLM Providers (Weeks 3-4)
- **Phase 3**: CLI Commands (Weeks 5-6)
- **Phase 4**: Tools & Features (Weeks 7-8)
- **Phase 5**: Polish & Testing (Weeks 9-10)

Total estimated time: 10 weeks
