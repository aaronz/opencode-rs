# OpenCode RS Modules Architecture

> **Note**: This document describes the Rust implementation (opencode-rs). For the TypeScript/original implementation, see the [anomalyco/opencode](https://github.com/anomalyco/opencode) repository.

A comprehensive reference guide to the modules in the OpenCode Rust AI coding agent.

## Overview

OpenCode RS is organized into **30+ crates** providing complete functionality for an AI-powered coding agent with TUI, server, and agent capabilities.

## Crate Index

| Crate | Description |
|-------|-------------|
| **opencode-core** | Core entities: Session, Message, Part, Tool, Error |
| **opencode-agent** | Agent implementations: Build, Plan, Explore, Debug, etc. |
| **opencode-tools** | 30+ built-in tools: read, write, edit, grep, glob, bash |
| **opencode-llm** | Multi-provider LLM abstraction: OpenAI, Anthropic, Ollama, etc. |
| **opencode-config** | Configuration from TOML, env vars, keychain |
| **opencode-storage** | SQLite persistence via rusqlite |
| **opencode-tui** | Ratatui-based terminal UI |
| **opencode-server** | Actix-web HTTP server |
| **opencode-lsp** | Language Server Protocol integration |
| **opencode-mcp** | Model Context Protocol implementation |
| **opencode-plugin** | WASM plugin system |
| **opencode-permission** | Permission and security model |
| **opencode-auth** | Authentication and credentials |
| **opencode-git** | Git operations wrapper |
| **opencode-cli** | CLI commands and argument parsing |

---

## Core Crates

### opencode-core

Core entities and utilities.

**Source**: `crates/core/src/lib.rs`

**Key Modules:**
- `session.rs` - Session management (fork, share, undo/redo, compaction)
- `message.rs` - Message and Role types
- `context.rs` - LLM prompt context building
- `compaction.rs` - Session compaction logic
- `error.rs` - Error types with code ranges (1xxx-9xxx)
- `tool.rs` - Tool trait definition

**Purpose:** Foundation crate used by all other crates.

---

### opencode-agent

Agent implementations and runtime.

**Source**: `crates/agent/src/lib.rs`

**Key Modules:**
- `agent.rs` - Agent trait, AgentType enum, ToolCall
- `runtime.rs` - AgentRuntime for execution
- `build_agent.rs` - BuildAgent (full access)
- `plan_agent.rs` - PlanAgent (read-only)
- `explore_agent.rs` - ExploreAgent (code search)
- `debug_agent.rs` - DebugAgent
- `refactor_agent.rs` - RefactorAgent
- `review_agent.rs` - ReviewAgent
- `delegation.rs` - Task delegation for subagents
- `events.rs` - Event emission system

**Agent Types:**
| Type | Access | Tools | Subagent |
|------|--------|-------|----------|
| Build | Full | All | Yes |
| Plan | Read-only | None | No |
| Explore | Read-only | Search | No |
| Debug | Diagnostic | Yes | No |
| Refactor | Read/write | Refactor | No |
| Review | Read-only | Review | No |

---

### opencode-tools

Built-in tools for agent execution.

**Source**: `crates/tools/src/lib.rs`

**Tool Implementations:**
| Tool | Source | Description |
|------|--------|-------------|
| `Read` | `read.rs` | File content reading |
| `Write` | `write.rs` | File creation/overwrite |
| `Edit` | `edit.rs` | In-place editing |
| `Bash` | `bash.rs` | Shell command execution |
| `Glob` | `glob.rs` | Pattern-based file finding |
| `Grep` | `grep_tool.rs` | Content search |
| `Lsp` | `lsp_tool.rs` | LSP queries |
| `CodeSearch` | `codesearch.rs` | Code search |
| `MultiEdit` | `multiedit.rs` | Batch edits |
| `WebFetch` | `webfetch.rs` | HTTP fetch |
| `WebSearch` | `web_search.rs` | Web search |
| `Git` | `git_tools.rs` | Git operations |
| `Skill` | `skill.rs` | Skill loading |
| `Todowrite` | `todowrite.rs` | Todo management |
| `Task` | `task.rs` | Task management |
| `Question` | `question.rs` | Interactive Q&A |

**Registry**: `ToolRegistry` in `registry.rs` - async, RwLock-based, priority-ordered

---

### opencode-llm

Multi-provider LLM abstraction.

**Source**: `crates/llm/src/lib.rs`

**Supported Providers (20+):**
- OpenAI (GPT-4, GPT-4o, GPT-3.5)
- Anthropic (Claude 3.5, Claude 3)
- Ollama (local models)
- Azure OpenAI
- Google (Gemini)
- AWS Bedrock
- OpenRouter
- Groq, DeepInfra, Cerebras, Cohere, Mistral
- TogetherAI, Perplexity, XAI
- And more...

**Key Types:**
- `Provider` trait - unified interface
- `Model` - specific model instance
- `ChatMessage` - message format
- `BudgetTracker` - token budget management
- `ProviderRegistry` - dynamic provider registration

---

### opencode-config

Configuration management.

**Source**: `crates/config/src/lib.rs`

**Config Sources:**
1. `OPENCODE_CONFIG` env var (JSON/JSONC)
2. `~/.config/opencode-rs/config.toml`
3. Project `.opencode/config.toml`
4. Remote config server
5. Keychain for secrets

**Key Types:**
- `Config` - main config struct
- `ServerConfig` - server settings
- `ProviderConfig` - provider settings
- `SkillsConfig` - skill configuration

---

### opencode-storage

SQLite persistence.

**Source**: `crates/storage/src/lib.rs`

**Uses**: `rusqlite` with connection pooling

**Purpose:** Stores sessions, messages, settings persistently.

---

### opencode-tui

Terminal UI using Ratatui.

**Source**: `crates/tui/src/lib.rs`

**Components:**
- Dialog system
- Input handling
- Rendering widgets
- Theme support

---

### opencode-server

HTTP API server.

**Source**: `crates/server/src/lib.rs`

**Framework**: `actix-web`

**Routes:**
- ACP protocol endpoints
- Session management API
- Provider configuration API

---

## Supporting Crates

| Crate | Description |
|-------|-------------|
| **opencode-git** | Git operations: commit, diff, status, log |
| **opencode-lsp** | LSP client for code intelligence |
| **opencode-mcp** | MCP protocol implementation |
| **opencode-plugin** | WASM plugin runtime (wasmi) |
| **opencode-permission** | Filesystem permission model |
| **opencode-auth** | API key and OAuth management |
| **opencode-bus** | Event bus for pub/sub |
| **opencode-sync** | Synchronization utilities |
| **opencode-share** | Session sharing functionality |
| **opencode-snapshot** | State snapshots |
| **opencode-worktree** | Git worktree management |
| **opencode-patch** | Patch application |
| **opencode-format** | Code formatting integration |

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Entry                           │
│                    (crates/cli/src/)                        │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                      Core Crates                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │opencode-core│  │opencode-agent│  │opencode-tools│         │
│  │  Session    │  │   Agent     │  │    Tools    │         │
│  │  Message    │  │   Runtime   │  │  Registry   │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                  │
│         └────────────────┴────────────────┘                  │
│                          │                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │opencode-llm │  │opencode-config│ │opencode-storage│      │
│  │  Provider   │  │    Config    │  │   SQLite    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    Server / TUI Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │opencode-tui │  │opencode-server│ │opencode-lsp │         │
│  │   Ratatui   │  │   Actix-web  │  │    LSP      │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Flow

1. **CLI Input** → `crates/cli/src/` parses commands
2. **Session Init** → `opencode-core` creates/loads session
3. **Agent Loop** → `opencode-agent` processes messages
4. **Tool Execution** → `opencode-tools` performs actions
5. **LLM Call** → `opencode-llm` → provider → AI API
6. **Response** → `opencode-agent` handles response
7. **Storage** → `opencode-storage` persists data
8. **Output** → `opencode-tui` renders or JSON response

---

## Error Code Ranges

| Range | Category |
|-------|----------|
| 1xxx | Authentication errors |
| 2xxx | Authorization errors |
| 3xxx | Provider errors |
| 4xxx | Tool errors |
| 5xxx | Session errors |
| 6xxx | Config errors |
| 7xxx | Validation errors |
| 9xxx | Internal errors |

**Reference**: `crates/core/src/error.rs`

---

## Reference

- **Repository**: https://github.com/anomalyco/opencode-rs
- **Original**: https://github.com/anomalyco/opencode (TypeScript)
