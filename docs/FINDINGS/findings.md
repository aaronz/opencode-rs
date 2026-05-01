# Findings: opencode-rs Refactoring Analysis

## Requirements (from Refactor.md)
Refactor opencode-rs according to Rust ecosystem best practices for AI-coding-agent-friendly engineering:

- High cohesion inside each module
- Low coupling between modules
- Clear crate/module boundaries
- Explicit ownership of responsibilities
- Minimal circular dependencies
- Clean public APIs between subsystems
- Better separation between domain logic, infrastructure, UI, CLI, TUI, provider integration, config, logging, persistence, and testing utilities

---

## Repository Structure Overview

### Current Crate Layout (24 crates in workspace)
```
opencode-rust/
├── Cargo.toml (workspace root)
├── crates/
│   ├── acp/          # Agent Communication Protocol
│   ├── agent/        # Agent implementations
│   ├── auth/         # Authentication
│   ├── cli/          # CLI binary + commands
│   ├── config/       # Configuration
│   ├── control-plane/# Control plane client
│   ├── core/         # Core types and abstractions (⚠️ PROBLEMATIC)
│   ├── file/         # File operations
│   ├── format/       # Formatting utilities
│   ├── git/          # Git operations
│   ├── llm/          # LLM provider integrations
│   ├── logging/      # Logging infrastructure
│   ├── lsp/          # LSP integration
│   ├── mcp/          # MCP protocol
│   ├── permission/   # Permission system
│   ├── plugin/       # Plugin system
│   ├── sdk/          # SDK for external consumers
│   ├── server/       # HTTP server
│   ├── storage/      # Database/storage
│   ├── tools/        # Tool implementations
│   ├── tui/          # Terminal UI
│   └── util/         # Utilities
├── integration_tests/
├── opencode-benches/
└── ratatui-testing/  # TUI testing framework
```

---

## Phase 1 Analysis Findings

### 1. CRATE/MODULE LAYOUT ISSUES

#### Problem: `core` crate is a catch-all (64 files!)

The `crates/core/src/` directory has 64 files covering vastly different concerns:

| Category | Files |
|----------|-------|
| Session | session.rs (2461 lines!), session_sharing, session_state, snapshot |
| Project | project.rs (2203 lines!), filesystem, directory |
| Skill | skill.rs (1485 lines!), skill_integration |
| Tool | tool.rs (1088 lines!), tool_config |
| Command | command.rs (1196 lines!) |
| Execution | executor.rs, effect.rs, processor.rs |
| Storage | storage.rs, checkpoint.rs, crash_recovery.rs |
| Auth/Cred | credential_store.rs (561 lines), account.rs |
| Observability | token_counter.rs (489 lines!), flag.rs (774 lines) |
| MCP | mcp.rs, part.rs |
| ACP | acp.rs |
| Misc | context.rs, env.rs, global.rs, id.rs, ide.rs, installation.rs, instance.rs, instructions.rs, message.rs, paths.rs, permission.rs, plugin.rs, prompt.rs, pty.rs, question.rs, revert.rs, server.rs, share.rs, shell.rs, status.rs, summary.rs, sync.rs, util.rs, watcher.rs, worktree.rs |

**Assessment**: Core is trying to be everything - domain logic, infrastructure, and UI. This violates single responsibility.

#### Problem: CLI crate has 42 command files

`crates/cli/src/cmd/` has 42 .rs files with commands:
- Some large: session.rs (1250), config.rs (1111), providers.rs (921), github.rs (767), acp.rs (762)
- Mixed responsibilities: workspace management, model management, MCP auth, web, desktop, etc.

#### Problem: LLM crate is provider-heavy (39 files)

`crates/llm/src/` has ~39 files with many provider implementations:
- Provider files: anthropic.rs, openai.rs, google.rs, azure.rs, bedrock.rs, etc.
- abstraction layers: provider.rs, provider_abstraction.rs, provider_adapter.rs, provider_registry.rs
- Model selection: model_selection.rs (442 lines), models.rs (430 lines)

### 2. OVERLY LARGE FILES

| File | Lines | Issue |
|------|-------|-------|
| core/src/session.rs | 2461 | Session management is too large |
| core/src/project.rs | 2203 | Project management is too large |
| core/src/skill.rs | 1485 | Skill system is too large |
| core/src/command.rs | 1196 | Command system is too large |
| core/src/tool.rs | 1088 | Tool system is too large |
| core/src/compaction.rs | 1024 | Compaction logic |
| cli/src/cmd/session.rs | 1250 | Session CLI is too large |
| cli/src/cmd/config.rs | 1111 | Config CLI is too large |
| cli/src/cmd/providers.rs | 921 | Providers CLI is too large |
| tools/src/registry.rs | 2640 | Tool registry is very large |
| tools/src/lsp_tool.rs | 1660 | LSP tool is large |
| tools/src/discovery.rs | 835 | Discovery is large |

### 3. MIXED RESPONSIBILITIES

**In `core` crate:**
- Domain logic (Session, Project, Skill, Tool, Command)
- Infrastructure (Storage, CrashRecovery, Checkpoint)
- Cross-cutting (Logging aliases, Observability)
- UI-adjacent (Shell, Pty, Question)

**In `cli` crate:**
- Binary entry point (main.rs)
- CLI framework (clap commands)
- Output formatting (output/ directory)
- TUI integration (webview.rs)

### 4. DIRECTORY STRUCTURE OBSERVATIONS

**Good:**
- Separate crates for TUI, CLI, Server, LSP, MCP, Agent
- Testing framework in ratatui-testing/
- Integration tests separated

**Could Improve:**
- `core` is monolithic
- `tools` contains both low-level tools (read, write) and high-level (lsp, discovery)
- `logging` crate has TUI-specific code (tui/ subdirectory)
- `server` has routes/ subdirectory which is good

### 5. DEPENDENCY STRUCTURE (Observed)

CLI depends on almost everything:
```
opencode-cli -> core, config, llm, git, plugin, server, storage, tui, mcp, auth, control-plane, acp, permission, tools, agent, util
```

Core dependencies:
```
opencode-core -> config, permission, storage (infrastructure)
```

TUI dependencies:
```
opencode-tui -> core, llm, auth, mcp, lsp
```

### 6. PUBLIC API ASSESSMENT

**core/lib.rs exports:**
- 270 lines of re-exports and public declarations
- Many `pub(crate)` internals exposed
- Mix of domain types (Session, Skill, Tool) and infrastructure (Storage, Checkpoint)

### 7. IDENTIFIED RISK AREAS

1. **Circular dependencies**: Potential between core modules (session <-> storage, skill <-> tool)
2. **Global state**: `global.rs` suggests singleton patterns (but is actually DI container)
3. **Large refactor needed**: Moving code from core to separate crates is high-risk
4. **Test infrastructure**: Need to ensure tests still pass after restructuring

---

## Recommended Refactoring Plan

### Step 1: Extract from `core` into new crates

Create these new crates from core modules:
- `opencode-session` - Session, session_sharing, session_state, snapshot
- `opencode-project` - Project, filesystem, directory
- `opencode-skill` - Skill, skill_integration
- `opencode-tool` - Tool, tool_config
- `opencode-command` - Command system

Keep in `core` (truly core):
- Error types
- ID types
- Message types
- Basic re-exports for compatibility

### Step 2: Restructure `tools` crate

Split into:
- `opencode-tool-core` - Base Tool trait, ToolCall, ToolResult
- `opencode-tool-registry` - ToolRegistry
- `opencode-tool-impl/` - Individual tool implementations as sub-modules or separate crates

### Step 3: Clarify `logging` crate

Remove any TUI-specific code from logging crate, or move to TUI.

### Step 4: Document Architecture

Create `ARCHITECTURE.md` explaining:
- Crate boundaries
- Dependency direction rules
- How to add new tools, commands, providers
- Testing strategy

---

## Completed Refactoring Work

### 1. Created ARCHITECTURE.md
**Status:** ✅ Complete
**File:** `opencode-rust/docs/ARCHITECTURE.md`

Documents:
- Crate structure and responsibilities
- Dependency direction rules
- Large files that need splitting
- Adding new features guide
- CI pipeline overview
- Known issues

### 2. Fixed Clippy Warnings in CLI
**Status:** ✅ Complete
**File:** `crates/cli/src/cmd/files.rs`

Fixed 2 collapsible else-if warnings:
- Lines 149-161: collapsed nested `else { if files.is_empty() }`
- Lines 187-196: collapsed nested `else { if results.is_empty() }`

---

## Resources
- Existing Cargo.toml workspace structure
- AGENTS.md has some architectural guidance
- CONTRIBUTING.md has plugin development docs