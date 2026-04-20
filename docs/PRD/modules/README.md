# OpenCode Modules PRD Index

This directory contains detailed Product Requirement Documents for each module in the OpenCode codebase.

## Purpose

These PRDs serve as implementation guides for LLM code agents to build the Rust equivalent of the OpenCode TypeScript codebase.

## Directory Structure

```
modules/
├── core/
│   ├── agent.md      # Core AI agent implementation
│   ├── session.md    # Conversation session management
│   ├── tool.md       # Tool implementations (26 tools)
│   └── provider.md   # AI provider abstraction (75+ providers)
├── infrastructure/
│   ├── cli.md        # Command-line interface (22 commands)
│   ├── server.md     # HTTP server and API routes
│   └── storage.md    # SQLite database with Drizzle ORM
├── integration/
│   ├── lsp.md        # Language Server Protocol integration
│   ├── mcp.md        # Model Context Protocol implementation
│   ├── plugin.md     # Plugin system for extensibility
│   ├── auth.md       # Authentication and credentials
│   ├── project.md    # Project detection and management
│   └── acp.md        # Agent Communication Protocol
├── utilities/
│   ├── util.md       # General utilities (logging, errors, fs)
│   ├── effect.md      # Effect-based functional programming
│   ├── flag.md       # Feature flags and runtime config
│   ├── global.md     # Global state and paths
│   ├── env.md        # Environment variable handling
│   ├── file.md       # File system utilities
│   ├── git.md        # Git operations wrapper
│   ├── config.md     # Configuration management
│   └── remaining-modules.md  # Other utilities (pty, sync, skill, etc.)
├── opencode-models-dev-integration.md  # models.dev integration PRD
└── opencode-modules-reference.md        # Modules overview reference
```

## Module Categories

### Core Modules (4)
Essential modules that form the heart of the agent:
- `agent` - Orchestrates tool execution and LLM interaction
- `session` - Manages conversation context and prompts
- `tool` - Implements 26 tools (read, write, bash, grep, etc.)
- `provider` - Unified interface to 75+ AI providers

### Infrastructure Modules (3)
Modules that provide system-level functionality:
- `cli` - Entry point and 22 CLI commands
- `server` - HTTP API for remote access
- `storage` - SQLite persistence with Drizzle ORM

### Integration Modules (6)
Modules that integrate with external systems:
- `lsp` - Code intelligence via Language Server Protocol
- `mcp` - Extended capabilities via Model Context Protocol
- `plugin` - Extensibility via external plugins
- `auth` - API key and OAuth credential management
- `project` - Project type detection
- `acp` - Inter-agent communication

### Utility Modules (17)
Supporting modules for common operations:
- `util` - Logging, errors, filesystem
- `effect` - Functional Effect monad
- `flag` - Feature flags
- `global` - Paths and state
- `env` - Environment variables
- `file` - File operations
- `git` - Git operations
- `config` - Configuration
- And more (pty, sync, skill, v2, worktree, bus, etc.)

## Usage

Each PRD contains:
1. **Module Overview** - Name, source path, type, purpose
2. **Functionality** - Core features and capabilities
3. **API Surface** - Key interfaces and types
4. **Data Structures** - Important data types
5. **Dependencies** - External dependencies
6. **Acceptance Criteria** - Success conditions
7. **Rust Implementation Guidance** - Specific recommendations for Rust implementation

## Total Modules: 46

| Category | Count |
|----------|-------|
| Core | 4 |
| Infrastructure | 3 |
| Integration | 6 |
| Utility | 17 |
| Other | 16 |

## Reference

- Original OpenCode TypeScript source: `/packages/opencode/src/`
- models.dev API: `https://models.dev/api.json`
