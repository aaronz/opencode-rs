# PRD: Configuration System

> **User Documentation**: [config.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/config.mdx)
>
> This document describes the configuration system from an implementation perspective. For user-facing configuration guides and examples, see the user docs linked above.

## Overview

OpenCode uses a layered configuration system with precedence-based merging. This document is the **canonical authority** for configuration ownership in the PRD set. Other subsystem documents may reference configuration, but they must not redefine config boundaries or conflict with this document.

---

## Scope

This document governs:

- **Main config** (`opencode.json`, environment variables, CLI flags)
- **Config precedence** across all config sources
- **Variable expansion** syntax
- **Auth/secret storage** paths and formats
- **Cross-subsystem config ownership** boundaries

This document **does not** govern (these are owned by respective subsystems):

- `tui.json` schema and TUI-specific settings (see [TUI System](./09-tui-system.md))
- Plugin internal structure and hooks (see [Plugin System](./08-plugin-system.md))
- LSP server configuration details (see [LSP System](./05-lsp-system.md))
- MCP server runtime behavior and integration details (see [MCP System](./04-mcp-system.md))
- Tool implementation specifics (see [Tools System](./03-tools-system.md))
- Permission rule evaluation semantics (see [Agent System](./02-agent-system.md))

---

## Configuration Precedence

Configuration is merged in this order (later overrides earlier):

1. **Remote config** (`.well-known/opencode`) - Organization defaults
2. **Global config** (`~/.config/opencode/opencode.json`) - User preferences
3. **Custom config** (`OPENCODE_CONFIG` env var)
4. **Project config** (`opencode.json` in project root)
5. **`.opencode` directory** - agents/, commands/, modes/, plugins/, skills/, tools/, themes/
6. **Inline config** (`OPENCODE_CONFIG_CONTENT` env var)

---

## Config File Formats

OpenCode supports:
- JSON (strict JSON)
- JSONC (JSON with comments)

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  // comments are allowed in JSONC
  "model": "anthropic/claude-sonnet-4-5"
}
```

---

## Schema Reference

### Canonical Main Config Schema

```typescript
type Config = {
  $schema?: string

  // Model
  model?: string                    // format: provider/model-id
  small_model?: string              // for lightweight tasks (titles)
  provider?: {
    [providerID: string]: ProviderOptions
  }

  // Agents
  agent?: {
    [agentName: string]: AgentConfig
  }
  default_agent?: string

  // Commands
  command?: {
    [commandName: string]: CommandConfig
  }

  // Permissions (normative)
  permission?: {
    [toolName: string]: "allow" | "ask" | "deny" | PermissionRule
  }

  // Server/runtime plugins only. TUI plugin config belongs in tui.json.
  plugin?: string[]

  // MCP (see MCP documentation for schema)
  mcp?: Record<string, unknown>

  // LSP (see LSP System for schema)
  lsp?: false | Record<string, unknown>

  // Features
  share?: "manual" | "auto" | "disabled"
  autoupdate?: boolean | "notify"
  compaction?: {
    auto?: boolean
    prune?: boolean
    reserved?: number
  }
  watcher?: {
    ignore?: string[]  // glob patterns
  }

  // Instructions
  instructions?: string[]  // file paths or URLs

  // Provider filters
  disabled_providers?: string[]
  enabled_providers?: string[]

  // Experimental
  experimental?: Record<string, unknown>
}
```

### Ownership Notes

- **`permission`**: Normative configuration for tool access. Values: `allow`, `ask`, `deny`, or `PermissionRule` with glob patterns.
- **`tools`**: Deprecated alias for `permission`. Existing configs using `tools` are converted to `permission` semantics at load time. New configs should use `permission`.
- **`theme` / `keybinds` / TUI plugin state**: TUI-specific settings belong in `tui.json`, not here. See [TUI System](./09-tui-system.md) and [TUI Plugin API](./15-tui-plugin-api.md).
- **`server`**: Server configuration is managed by the server subsystem. See [Server API](./07-server-api.md).

### PermissionRule Type

```typescript
type PermissionRule = {
  "*"?: "allow" | "ask" | "deny"
  [pattern: string]: "allow" | "ask" | "deny"
}
```

---

## TUI Configuration (tui.json)

**Ownership**: TUI subsystem owns `tui.json` schema and settings.

Location: `~/.config/opencode/tui.json` or project root `tui.json`

For schema and TUI-specific settings (theme, keybinds, scroll behavior, diff style, TUI plugins, and `plugin_enabled`), see [TUI System](./09-tui-system.md) and [TUI Plugin API](./15-tui-plugin-api.md).

The main `opencode.json` config does **not** own TUI settings. TUI reads its own `tui.json`.

---

## Plugin Configuration

**Ownership**: Main config owns **server/runtime plugin** configuration. TUI plugin configuration is owned by `tui.json`. Plugin internals are owned by [Plugin System](./08-plugin-system.md) and [TUI Plugin API](./15-tui-plugin-api.md).

### Config Keys (Main Config)

```json
{
  "plugin": ["opencode-helicone-session", "@my-org/custom-plugin"]
}
```

### Plugin Internals

For plugin structure, hooks, custom tools, and event handling, see [Plugin System](./08-plugin-system.md).

---

## Variables

### Environment Variables

```json
{
  "model": "{env:OPENCODE_MODEL}",
  "provider": {
    "anthropic": {
      "options": {
        "apiKey": "{env:ANTHROPIC_API_KEY}"
      }
    }
  }
}
```

### File Content

```json
{
  "instructions": ["{file:./custom-instructions.md}"]
}
```

---

## AGENTS.md / Rules

`AGENTS.md` files contain project-specific instructions for the agent.

### Location Priority

1. Local file from current directory up (AGENTS.md, CLAUDE.md)
2. Global file `~/.config/opencode/AGENTS.md`
3. Claude Code fallback `~/.claude/CLAUDE.md`

### Example AGENTS.md

```markdown
# Project Name

This is a monorepo using bun workspaces.

## Project Structure

- `packages/` - Workspace packages
- `infra/` - Infrastructure definitions

## Code Standards

- Use TypeScript with strict mode
- Run tests before committing
```

---

## Authentication

Credentials stored in `~/.local/share/opencode/auth.json`:

```json
{
  "provider-id": {
    "type": "api",
    "key": "sk-..."
  }
}
```

OAuth tokens stored separately in `~/.local/share/opencode/mcp-auth.json`.

---

## Normalization and Compatibility

### tools → permission Conversion

The `tools` key in config is a **legacy compatibility alias** for `permission`:

```json
// Legacy format (converted internally)
{ "tools": { "bash": false } }

// Equivalent normative format
{ "permission": { "bash": "deny" } }
```

Conversion rules:
- `tools: { <tool>: false }` → `permission: { <tool>: "deny" }`
- `tools: { <tool>: true }` → `permission: { <tool>: "allow" }`

### Pattern Matching

Permission rules support glob patterns:
```json
{
  "permission": {
    "mcp_github_*": "ask",
    "bash": "allow"
  }
}
```

---

## Cross-References

| Subsystem | Document | User Docs | Config Ownership |
|-----------|----------|-----------|------------------|
| Core Config | This document | [config.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/config.mdx) | Schema authority |
| Agent System | [02-agent-system.md](./02-agent-system.md) | [agents.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/agents.mdx) | Agent config structure, permission evaluation |
| Tools System | [03-tools-system.md](./03-tools-system.md) | [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx) | Tool implementation, custom tools |
| LSP System | [05-lsp-system.md](./05-lsp-system.md) | [lsp.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/lsp.mdx) | LSP config schema |
| Plugin System | [08-plugin-system.md](./08-plugin-system.md) | [plugins.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/plugins.mdx) | Server/runtime plugin internals, hooks, events |
| TUI System | [09-tui-system.md](./09-tui-system.md) | [tui.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tui.mdx) | `tui.json` schema, themes, keybinds |
| TUI Plugin API | [15-tui-plugin-api.md](./15-tui-plugin-api.md) | N/A | TUI plugin config, `plugin_enabled`, runtime enable/disable |
| Server API | [07-server-api.md](./07-server-api.md) | [server.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/server.mdx) | Server config, endpoint ownership |
| Skills System | [12-skills-system.md](./12-skills-system.md) | [skills.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/skills.mdx) | Skill loading, permission patterns |
| Config module | [../modules/config.md](../modules/config.md) | N/A | Rust configuration management implementation |

---

## Subsystem Config Boundaries

```
opencode.json (main config)
├── permission/*           ← Normative
├── tools/*                ← Legacy alias → permission
├── plugin/*               ← Server/runtime plugins only
├── mcp/*                  ← Main config ownership, MCP runtime semantics elsewhere
├── lsp/*                  ← Main config ownership, LSP runtime semantics elsewhere
├── server/*               ← Main config ownership, transport/API semantics elsewhere
└── [other top-level keys] ← This document

tui.json
├── theme/*
├── keybinds/*
├── plugin/*               ← TUI plugins
└── plugin_enabled/*       ← TUI plugin enable/disable state
```

Conflict resolution: If this document and another PRD document disagree on config ownership, this document is authoritative.
