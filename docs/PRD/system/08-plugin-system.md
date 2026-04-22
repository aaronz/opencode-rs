# PRD: Plugin System (Server/Runtime)

## Scope

This document defines the **server/runtime plugin system** for OpenCode. It covers plugin loading, hooks, custom tool registration, and plugin context for extensions that run in the OpenCode server/runtime process.

**This document does NOT define TUI plugin APIs.** For TUI extensions (commands, routes, dialogs, themes, slots), see [TUI Plugin API](./15-tui-plugin-api.md).

**Configuration ownership**: Server/runtime plugin configuration is owned by the main config (`opencode.json`), as defined in [Configuration System](./06-configuration-system.md). TUI plugin configuration belongs in `tui.json` and is owned by [TUI System](./09-tui-system.md).

---

## Plugin Targets

Plugins may expose one or more package entrypoints, but each concrete module target is exclusive about where it runs:

| Target | Description |
|--------|-------------|
| `server` | Runs in the OpenCode server/runtime process |
| `tui` | Runs in the TUI process (see [TUI Plugin API](./15-tui-plugin-api.md)) |

A package may provide both `./server` and `./tui` entrypoints, but the module loaded for a given target should remain target-specific.

---

## Plugin Loading

### Sources

1. **npm packages** - configured in main `opencode.json` under `plugin`
2. **Local files** - `.opencode/plugins/` or `~/.config/opencode/plugins/`

### npm Plugin Config

```json
{
  "plugin": ["opencode-helicone-session", "@my-org/custom-plugin"]
}
```

### Local Plugin Structure

```
.opencode/
  plugins/
    my-plugin.ts
  package.json  (for dependencies)
```

---

## Plugin Definition

### Basic Plugin

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const MyPlugin: Plugin = async ({ project, client, $, directory, worktree }) => {
  return {
    // hooks here
  }
}
```

### Plugin Context

```typescript
context: {
  project: Project           // current project
  directory: string          // working directory
  worktree: string           // git worktree root
  client: OpenCodeClient      // SDK client
  $: BunShellAPI             // shell execution
}
```

---

## Event Hooks

### Command Events
- `command.executed`

### File Events
- `file.edited`
- `file.watcher.updated`

### Installation Events
- `installation.updated`

### LSP Events
- `lsp.client.diagnostics`
- `lsp.updated`

### Message Events
- `message.part.removed`
- `message.part.updated`
- `message.removed`
- `message.updated`

### Permission Events
- `permission.asked`
- `permission.replied`

### Server Events
- `server.connected`

### Session Events
- `session.created`
- `session.compacted`
- `session.deleted`
- `session.diff`
- `session.error`
- `session.idle`
- `session.status`
- `session.updated`

### Todo Events
- `todo.updated`

### Shell Events
- `shell.env`

### Tool Events
- `tool.execute.after`
- `tool.execute.before`

### Experimental
- `experimental.session.compacting`

---

## Hook Examples

### Tool Execute Hook

```typescript
export const EnvProtection: Plugin = async () => {
  return {
    "tool.execute.before": async (input, output) => {
      if (input.tool === "read" && input.args.filePath.includes(".env")) {
        throw new Error("Do not read .env files")
      }
    },
  }
}
```

### Shell Env Hook

```typescript
export const InjectEnvPlugin: Plugin = async () => {
  return {
    "shell.env": async (input, output) => {
      output.env.MY_API_KEY = "secret"
      output.env.PROJECT_ROOT = input.cwd
    },
  }
}
```

### Session Events

```typescript
export const NotificationPlugin: Plugin = async ({ $ }) => {
  return {
    event: async ({ event }) => {
      if (event.type === "session.idle") {
        await $`osascript -e 'display notification "Session done" with title "opencode"'`
      }
    },
  }
}
```

---

## Custom Tools

Plugins can add custom tools that extend the agent's capabilities:

```typescript
import { type Plugin, tool } from "@opencode-ai/plugin"

export const CustomToolsPlugin: Plugin = async (ctx) => {
  return {
    tool: {
      mytool: tool({
        description: "My custom tool",
        args: {
          foo: tool.schema.string()
        },
        async execute(args, context) {
          return `Hello ${args.foo}`
        },
      }),
    },
  }
}
```

---

## Compaction Hook

```typescript
export const CompactionPlugin: Plugin = async () => {
  return {
    "experimental.session.compacting": async (input, output) => {
      output.context.push(`
## Custom Context
- Current task status
- Important decisions
      `)
    },
  }
}
```

---

## Plugin Dependencies

Local plugins can specify dependencies in `.opencode/package.json`:

```json
{
  "dependencies": {
    "shescape": "^2.1.0"
  }
}
```

Then import in plugin:

```typescript
import { escape } from "shescape"
```

---

## Logging

```typescript
export const MyPlugin = async ({ client }) => {
  await client.app.log({
    body: {
      service: "my-plugin",
      level: "info",
      message: "Plugin initialized",
      extra: { foo: "bar" }
    }
  })
}
```

Log levels: `debug`, `info`, `warn`, `error`

---

## Loading Order and Precedence

Plugins load in this order:

1. Global config `~/.config/opencode/opencode.json`
2. Project config `opencode.json`
3. Global plugins `~/.config/opencode/plugins/`
4. Project plugins `.opencode/plugins/`

Duplicate npm packages (same name+version) load only once.

**Configuration**: Server/runtime plugin configuration is in the main `opencode.json` `plugin` key. See [Configuration System](./06-configuration-system.md) for details.

---

## Cross-References

| Topic | Document |
|-------|----------|
| Configuration ownership | [Configuration System](./06-configuration-system.md) |
| TUI plugin API (commands, routes, dialogs, slots, themes) | [TUI Plugin API](./15-tui-plugin-api.md) |
| TUI system and tui.json schema | [TUI System](./09-tui-system.md) |
