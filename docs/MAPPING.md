# Documentation Mapping: PRD ↔ User Docs

This document maps OpenCode RS PRD implementation documents to the OpenCode user documentation (zh-cn).

## Source Locations

| Documentation Type | Location |
|--------------------|----------|
| **User Documentation (zh-cn)** | `/Users/aaronzh/Documents/GitHub/opencode/packages/web/src/content/docs/zh-cn/` |
| **PRD Documents (Rust Implementation)** | `/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/` |

---

## System Architecture Mapping

| PRD System Doc | User Doc (zh-cn) | Content Area |
|----------------|------------------|--------------|
| [01-core-architecture.md](./PRD/system/01-core-architecture.md) | (conceptual) | Core entities: Project, Session, Message, Part ownership model |
| [02-agent-system.md](./PRD/system/02-agent-system.md) | [agents.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/agents.mdx) | Agent roles, primary/subagent model, permissions |
| [03-tools-system.md](./PRD/system/03-tools-system.md) | [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx) | Tool categories, execution pipeline, 26 built-in tools |
| [04-mcp-system.md](./PRD/system/04-mcp-system.md) | [mcp-servers.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/mcp-servers.mdx) | MCP protocol, server discovery |
| [05-lsp-system.md](./PRD/system/05-lsp-system.md) | [lsp.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/lsp.mdx) | LSP integration, code intelligence |
| [06-configuration-system.md](./PRD/system/06-configuration-system.md) | [config.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/config.mdx) | Configuration schema, JSON config |
| [07-server-api.md](./PRD/system/07-server-api.md) | [server.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/server.mdx), [web.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/web.mdx) | HTTP REST API, Web interface |
| [08-plugin-system.md](./PRD/system/08-plugin-system.md) | [plugins.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/plugins.mdx) | WASM plugin system |
| [09-tui-system.md](./PRD/system/09-tui-system.md) | [tui.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tui.mdx) | Terminal UI architecture |
| [10-provider-model-system.md](./PRD/system/10-provider-model-system.md) | [providers.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/providers.mdx), [models.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/models.mdx) | 75+ AI providers, model selection |
| [11-formatters.md](./PRD/system/11-formatters.md) | [formatters.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/formatters.mdx) | 25+ code formatters |
| [12-skills-system.md](./PRD/system/12-skills-system.md) | [skills.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/skills.mdx) | Skills system for agent capabilities |
| [13-desktop-web-interface.md](./PRD/system/13-desktop-web-interface.md) | [web.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/web.mdx) | Desktop/web interface |
| [14-github-gitlab-integration.md](./PRD/system/14-github-gitlab-integration.md) | [github.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/github.mdx), [gitlab.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/gitlab.mdx) | GitHub/GitLab automation agents |
| [15-tui-plugin-api.md](./PRD/system/15-tui-plugin-api.md) | (internal) | Dialog rendering, TUI component API |

---

## Module Mapping

### Core Modules (4)

| PRD Module | Crate | User Doc | Description |
|------------|-------|---------|-------------|
| [agent.md](./PRD/modules/agent.md) | `opencode-agent` | [agents.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/agents.mdx) | Agent trait, Build/Plan/Explore agents |
| [session.md](./PRD/modules/session.md) | `opencode-core` | (session concepts) | Session lifecycle, message management |
| [tool.md](./PRD/modules/tool.md) | `opencode-tools` | [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx) | 26 built-in tools implementation |
| [provider.md](./PRD/modules/provider.md) | `opencode-llm` | [providers.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/providers.mdx) | 75+ AI provider abstraction |

### Infrastructure Modules (3)

| PRD Module | Crate | User Doc | Description |
|------------|-------|---------|-------------|
| [cli.md](./PRD/modules/cli.md) | `opencode-cli` | [cli.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx) | 22 CLI commands |
| [server.md](./PRD/modules/server.md) | `opencode-server` | [server.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/server.mdx) | HTTP REST API |
| [storage.md](./PRD/modules/storage.md) | `opencode-storage` | (internal) | SQLite persistence with Drizzle ORM |

### Integration Modules (6)

| PRD Module | Crate | User Doc | Description |
|------------|-------|---------|-------------|
| [lsp.md](./PRD/modules/lsp.md) | `opencode-lsp` | [lsp.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/lsp.mdx) | Language Server Protocol |
| [mcp.md](./PRD/modules/mcp.md) | `opencode-mcp` | [mcp-servers.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/mcp-servers.mdx) | Model Context Protocol |
| [plugin.md](./PRD/modules/plugin.md) | `opencode-plugin` | [plugins.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/plugins.mdx) | WASM plugin system |
| [auth.md](./PRD/modules/auth.md) | `opencode-auth` | [providers.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/providers.mdx) | Authentication (Argon2, JWT, AES-GCM) |
| [project.md](./PRD/modules/project.md) | `opencode-core` | (internal) | Project detection, AGENTS.md |
| [acp.md](./PRD/modules/acp.md) | `opencode-core` | [acp.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/acp.mdx) | Agent Communication Protocol |

### Utility Modules (30)

| PRD Module | Crate | User Doc | Description |
|------------|-------|---------|-------------|
| [util.md](./PRD/modules/util.md) | `opencode-core` | (internal) | Logging, errors, filesystem helpers |
| [effect.md](./PRD/modules/effect.md) | `opencode-core` | (internal) | Effect monad wrappers |
| [flag.md](./PRD/modules/flag.md) | `opencode-config` | (internal) | Feature flags |
| [global.md](./PRD/modules/global.md) | `opencode-core` | (internal) | Global paths, shared state |
| [env.md](./PRD/modules/env.md) | `opencode-config` | (internal) | Environment variable handling |
| [file.md](./PRD/modules/file.md) | `opencode-tools` | [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx) | File read/write operations |
| [git.md](./PRD/modules/git.md) | `opencode-git` | (bash git commands) | Git operations wrapper |
| [config.md](./PRD/modules/config.md) | `opencode-config` | [config.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/config.mdx) | Configuration management |
| [pty.md](./PRD/modules/pty.md) | `opencode-core` | (internal) | PTY management for shell |
| [sync.md](./PRD/modules/sync.md) | `opencode-core` | (internal) | SSE streaming, state sync |
| [shell.md](./PRD/modules/shell.md) | `opencode-tools` | [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx) | Shell command execution (bash tool) |
| [bus.md](./PRD/modules/bus.md) | `opencode-core` | (internal) | In-process event bus |
| [snapshot.md](./PRD/modules/snapshot.md) | `opencode-core` | (internal) | File snapshot/diff utilities |
| [worktree.md](./PRD/modules/worktree.md) | `opencode-git` | (internal) | Git worktree management |
| [id.md](./PRD/modules/id.md) | `opencode-core` | (internal) | Typed identifier generation |
| [skill.md](./PRD/modules/skill.md) | `opencode-core` | [skills.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/skills.mdx) | Skills registry |
| [account.md](./PRD/modules/account.md) | (control-plane) | (enterprise) | User account management |
| [ide.md](./PRD/modules/ide.md) | `opencode-core` | [ide.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/ide.mdx) | IDE/editor integration |
| [share.md](./PRD/modules/share.md) | `opencode-core` | [share.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/share.mdx) | Session sharing |
| [control-plane.md](./PRD/modules/control-plane.md) | `opencode-control-plane` | [enterprise.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/enterprise.mdx) | Control plane API client |
| [installation.md](./PRD/modules/installation.md) | (installer) | [index.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/index.mdx) | Installation management |
| [permission.md](./PRD/modules/permission.md) | `opencode-permission` | [permissions.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/permissions.mdx) | RBAC permissions |
| [question.md](./PRD/modules/question.md) | `opencode-tools` | (question tool) | Interactive prompts |
| [v2.md](./PRD/modules/v2.md) | `opencode-storage` | (internal) | Session V2 schema, streaming events |
| [format.md](./PRD/modules/format.md) | `opencode-tools` | [formatters.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/formatters.mdx) | Code formatter service |
| [npm.md](./PRD/modules/npm.md) | `opencode-tools` | (npm tool) | NPM package manager |
| [patch.md](./PRD/modules/patch.md) | `opencode-tools` | [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx) | Patch application tool |

---

## Documentation Relationship Diagram

```text
OpenCode User Docs (zh-cn)          OpenCode-RS PRD Documents
┌──────────────────────┐           ┌──────────────────────────┐
│ 用户文档 (用户视角)    │           │ PRD 文档 (Rust 实现指南)  │
├──────────────────────┤           ├──────────────────────────┤
│ index.mdx             │◄─────────►│ installation.md         │
│ agents.mdx            │◄─────────►│ agent.md + 02-agent-*   │
│ tools.mdx             │◄─────────►│ tool.md + 03-tools-*    │
│ cli.mdx               │◄─────────►│ cli.md                  │
│ config.mdx            │◄─────────►│ config.md + 06-config-* │
│ providers.mdx         │◄─────────►│ provider.md + 10-prov-* │
│ mcp-servers.mdx       │◄─────────►│ mcp.md + 04-mcp-*       │
│ lsp.mdx               │◄─────────►│ lsp.md + 05-lsp-*       │
│ plugins.mdx           │◄─────────►│ plugin.md + 08-plugin-* │
│ server.mdx            │◄─────────►│ server.md + 07-server-*│
│ web.mdx               │◄─────────►│ 13-desktop-web-*        │
│ permissions.mdx        │◄─────────►│ permission.md           │
│ skills.mdx            │◄─────────►│ skill.md + 12-skills-*  │
│ formatters.mdx        │◄─────────►│ format.md + 11-format-* │
│ github.mdx            │◄─────────►│ 14-github-gitlab-*     │
│ tui.mdx               │◄─────────►│ 09-tui-system.md       │
│ acp.mdx               │◄─────────►│ acp.md                 │
│ share.mdx             │◄─────────►│ share.md                │
│ ide.mdx               │◄─────────►│ ide.md                  │
└──────────────────────┘           └──────────────────────────┘
         │                                      │
         └──────────────┬───────────────────────┘
                        ▼
              ┌──────────────────┐
              │  Unified Index   │
              │  (MAPPING.md)    │
              └──────────────────┘
```

---

## Usage

### For Users
Navigate to the [OpenCode User Documentation](https://github.com/anomalyco/opencode/tree/main/packages/web/src/content/docs/zh-cn) for usage guides, configuration references, and feature documentation.

### For Rust Developers
Navigate to the [PRD Documents](./PRD/) for implementation guides, API references, and Rust-specific architecture details.

### Cross-Reference
Use this mapping document to:
- Find user documentation for a specific PRD module
- Find PRD implementation details for a user-facing feature
- Understand the relationship between user-facing features and Rust implementation

---

## External Resources

| Resource | URL |
|----------|-----|
| OpenCode Main Repo | https://github.com/anomalyco/opencode |
| OpenCode User Docs (en) | https://github.com/anomalyco/opencode/tree/main/packages/web/src/content/docs |
| OpenCode RS Repo | https://github.com/anomalyco/opencode-rs |
| docs.rs API Docs | https://docs.rs/opencode-core/latest/opencode_core/ |