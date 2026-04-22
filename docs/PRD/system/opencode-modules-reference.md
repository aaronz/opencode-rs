# OpenCode Modules Architecture

A comprehensive reference guide to the modules in the OpenCode AI coding agent.

## Overview

OpenCode is organized into **46 modules** providing the complete functionality for an AI-powered coding agent with TUI, server, and agent capabilities.

## Module Index

| Module | Description |
|--------|-------------|
| **account** | User account management and authentication |
| **acp** | Agent Communication Protocol - inter-agent messaging |
| **agent** | Core agent implementation and orchestration |
| **auth** | Authentication and credential management |
| **bus** | Event bus for internal communication |
| **cli** | Command-line interface and UI |
| **config** | Configuration management and schema |
| **control-plane** | Control plane client integration |
| **effect** | Effect-based functional programming utilities |
| **env** | Environment variable handling |
| **file** | File operations and utilities |
| **flag** | Feature flags and runtime configuration |
| **format** | Code formatting integration |
| **git** | Git operations wrapper |
| **global** | Global state and paths |
| **id** | Unique identifier generation |
| **ide** | IDE integration utilities |
| **installation** | Installation and version detection |
| **lsp** | Language Server Protocol integration |
| **mcp** | Model Context Protocol implementation |
| **npm** | NPM package management |
| **permission** | Permission and security model |
| **plugin** | Plugin system and hooks |
| **project** | Project detection and management |
| **provider** | AI model provider abstraction (models.dev) |
| **pty** | Pseudo-terminal emulation |
| **question** | Interactive question/answer handling |
| **server** | HTTP server and API routes |
| **session** | Conversation session management |
| **share** | Session sharing functionality |
| **shell** | Shell command execution |
| **skill** | Skill loading and management |
| **snapshot** | State snapshot utilities |
| **storage** | Persistent storage (SQLite) |
| **sync** | Synchronization utilities |
| **tool** | Tool implementations (read, write, edit, etc.) |
| **util** | General utilities and helpers |
| **v2** | Version 2 APIs and migrations |
| **worktree** | Git worktree management |

---

## Core Modules

### agent

Core AI agent implementation that orchestrates the interaction between tools, providers, and session management.

**Key Files:**
- `index.ts` - Agent entry point
- `*.ts` - Agent implementation modules

**Purpose:** Manages the main agent loop, tool execution, and LLM interaction.

---

### session

Conversation session management including message handling, prompt engineering, and context management.

**Key Files:**
- `session.ts` - Main session class
- `message.ts` - Message handling
- `message-v2.ts` - New message format
- `prompt.ts` - Prompt generation (72KB - extensive prompt engineering)
- `processor.ts` - Message processing
- `compaction.ts` - Session compaction for context management
- `llm.ts` - LLM interface abstraction
- `summary.ts` - Conversation summarization
- `retry.ts` - Retry logic for failed operations

**Purpose:** Manages conversation context, message history, and prompt construction for LLM interactions.

---

### tool

Implements all tools available to the agent (like filesystem operations, search, etc.).

**Key Tools:**
| Tool | Description |
|------|-------------|
| `bash` | Execute bash commands |
| `read` | Read file contents |
| `write` | Write/edit file contents |
| `edit` | In-place file editing |
| `grep` | Search file contents |
| `glob` | Find files by pattern |
| `lsp` | Language Server Protocol queries |
| `codesearch` | Code search functionality |
| `webfetch` | Fetch web content |
| `websearch` | Web search |
| `mcp-exa` | MCP Exa search integration |
| `multiedit` | Multiple edit operations |
| `plan` | Plan mode operations |
| `task` | Task management |
| `todowrite` | Todo list management |
| `question` | Interactive questions |
| `skill` | Skill loading |
| `apply_patch` | Apply patches |
| `truncate` | File truncation |
| `external-directory` | External directory access |

**Purpose:** Provides all capabilities the agent can use to interact with the filesystem, run commands, and search.

---

### provider

AI model provider abstraction layer, integrating with models.dev for dynamic model discovery.

**Key Files:**
- `provider.ts` - Main provider service (64KB - extensive)
- `models.ts` - models.dev API client and caching
- `transform.ts` - Message/model transformation
- `schema.ts` - Provider and model schemas
- `auth.ts` - Provider authentication
- `error.ts` - Provider-specific errors

**Supported Providers:**
- Anthropic, OpenAI, Google, Amazon Bedrock, Azure, GitHub Copilot
- Groq, DeepInfra, Cerebras, Cohere, TogetherAI, Perplexity, XAI, Mistral
- Cloudflare Workers AI, Cloudflare AI Gateway, GitLab, SAP AI Core
- OpenRouter, Vercel, NVIDIA, Kilo, ZenMux, and more (75+ total)

**Purpose:** Unified interface to 75+ AI providers with dynamic model discovery, pricing, and capability data from models.dev.

---

### cli

Command-line interface implementation.

**CLI Commands:**
| Command | Description |
|---------|-------------|
| `run` | Run agent in current directory |
| `models` | List available AI models |
| `providers` | List AI providers |
| `agent` | Agent management |
| `serve` | Start API server |
| `web` | Start web interface |
| `mcp` | MCP server management |
| `acp` | Agent Communication Protocol |
| `tui` | Terminal UI (attach, thread) |
| `session` | Session management |
| `db` | Database operations |
| `github` | GitHub integration |
| `pr` | Pull request operations |
| `export` | Export session/data |
| `import` | Import session/data |
| `plug` | Plugin management |
| `stats` | Usage statistics |
| `debug` | Debug utilities |
| `upgrade` | Self-upgrade |
| `uninstall` | Uninstall |
| `generate` | Code generation |

**Purpose:** Entry point and command routing for all CLI operations.

---

### server

HTTP server and API routes.

**Key Files:**
- `server.ts` - Main server implementation
- `routes/` - API route handlers
- `proxy.ts` - Proxy functionality
- `middleware.ts` - Server middleware
- `workspace.ts` - Workspace management
- `mdns.ts` - mDNS discovery
- `fence.ts` - Security fencing

**Purpose:** Provides HTTP API for desktop/web modes and remote agent access.

---

### config

Configuration management with schema validation.

**Key Files:**
- `index.ts` - Config entry point
- Schema definitions for all config options
- `model-id.ts` - Model ID parsing
- `provider.ts` - Provider configuration

**Purpose:** Handles `opencode.json` configuration, environment variables, and defaults.

---

### storage

Persistent storage using SQLite with Drizzle ORM.

**Key Files:**
- Database schema and migrations
- `index.ts` - Storage entry point

**Purpose:** Stores sessions, messages, settings, and other persistent data.

---

### plugin

Plugin system for extensibility.

**Purpose:** Allows external plugins to add providers, tools, and hooks.

---

### lsp

Language Server Protocol integration for code intelligence.

**Purpose:** Provides go-to-definition, find-references, symbol search, and rename capabilities.

---

### mcp

Model Context Protocol implementation.

**Purpose:** Integrates with MCP servers for extended capabilities.

---

### auth

Authentication and credential management for API providers.

**Purpose:** Manages API keys, OAuth tokens, and secure credential storage.

---

## Supporting Modules

| Module | Description |
|--------|-------------|
| **util** | General utilities (logging, errors, helpers) |
| **effect** | Functional Effect-based error handling |
| **flag** | Feature flags for runtime toggles |
| **global** | Global paths and state |
| **file** | File system utilities |
| **git** | Git operations wrapper |
| **ide** | IDE detection and integration |
| **npm** | NPM package operations |
| **pty** | PTY for terminal emulation |
| **sync** | Data synchronization |
| **v2** | Version 2 API migrations |
| **worktree** | Git worktree operations |
| **bus** | Event bus for pub/sub |
| **acp** | Agent Communication Protocol |
| **control-plane** | Control plane client |
| **share** | Session sharing |
| **shell** | Shell execution |
| **skill** | Skill loading system |
| **snapshot** | State snapshots |
| **account** | User account management |
| **env** | Environment variables |
| **format** | Code formatting |
| **id** | ID generation |
| **installation** | Installation info |
| **permission** | Permission model |
| **question** | Q&A handling |

---

## CLI Commands

```
opencode run [directory]     # Run agent
opencode models [provider]    # List models
opencode providers           # List providers
opencode serve               # Start API server
opencode web                 # Start web UI
opencode mcp                 # MCP server
opencode acp                 # ACP commands
opencode tui attach|thread   # TUI operations
opencode session             # Session management
opencode db                  # Database ops
opencode github              # GitHub integration
opencode pr                  # Pull requests
opencode export|import       # Data export/import
opencode plug                # Plugin management
opencode stats               # Usage stats
opencode debug               # Debug tools
opencode upgrade             # Self-upgrade
opencode uninstall           # Uninstall
```

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Entry                           │
│                     (index.ts, yargs)                       │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                      Core Modules                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │  agent  │  │ session │  │  tool   │  │provider │         │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘         │
│       │            │            │            │              │
│       └────────────┴────────────┴────────────┘              │
│                         │                                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │  config │  │ storage │  │  lsp    │  │   mcp   │         │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘         │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    Server / TUI Layer                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ server  │  │   tui   │  │  pty    │  │  auth   │         │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘         │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Flow

1. **CLI Input** → `cli/cmd/run.ts` or other commands
2. **Session Init** → `session/session.ts` creates/loads session
3. **Agent Loop** → `agent/` processes messages
4. **Tool Execution** → `tool/*.ts` performs actions
5. **LLM Call** → `session/llm.ts` → `provider/` → AI provider
6. **Response** → `session/processor.ts` handles response
7. **Storage** → `storage/` persists data
8. **Output** → TUI or JSON response

---

## Configuration

OpenCode uses `opencode.json` for configuration:

```json
{
  "provider": {
    "openai": { "options": {} },
    "anthropic": { "options": {} }
  },
  "enabled_providers": [],
  "disabled_providers": [],
  "agent": {},
  "lsp": {}
}
```

---

## Dependencies

Key external dependencies:
- **yargs** - CLI argument parsing
- **ai** (Vercel) - AI SDK for model interactions
- **drizzle-orm** - Database ORM
- **@modelcontextprotocol/sdk** - MCP integration
- **vscode-languageserver** - LSP implementation
- **ratatui** - TUI rendering

---

## Reference

- **Repository**: https://github.com/anomalyco/opencode
- **Models.dev**: https://models.dev (model registry)
- **AI SDK**: https://sdk.vercel.ai
