# PRD: TUI Plugin API

## Overview

This document is the **canonical authority** for TUI plugin behavior in the PRD set. It covers TUI-specific plugin configuration, runtime lifecycle, and the JavaScript API surface available to TUI plugins.

This document **does not** cover:

- Server/runtime plugin hooks, events, and custom tools (see [Plugin System](./08-plugin-system.md))
- Main config schema or cross-subsystem config ownership (see [Configuration System](./06-configuration-system.md))

---

## Scope

TUI plugins extend the terminal UI through:

- Custom routes and views
- Commands and keybindings
- Sidebar slots and dialogs
- Theme installation and switching
- TUI-specific events

---

## TUI Config Ownership

**`tui.json`** is the sole owner of TUI plugin configuration. The main `opencode.json` config does **not** manage TUI plugins.

Location: `~/.config/opencode/tui.json` or project root `tui.json`

```json
{
  "$schema": "https://opencode.ai/tui.json",
  "theme": "smoke-theme",
  "plugin": ["@acme/opencode-plugin@1.2.3", ["./plugins/demo.tsx", { "label": "demo" }]],
  "plugin_enabled": {
    "acme.demo": false
  }
}
```

### Config Keys

| Key | Type | Description |
|-----|------|-------------|
| `plugin` | `string[] \| [string, object][]` | Plugin specs (see [Plugin Entry Formats](#plugin-entry-formats)) |
| `plugin_enabled` | `Record<string, boolean>` | Runtime enable/disable state per plugin ID |

### Plugin Entry Formats

TUI plugins support two entry formats:

- **npm spec**: `"@acme/opencode-plugin@1.2.3"`
- **file + options**: `["path/to/plugin.tsx", { label: "demo" }]`

### `plugin_enabled` Semantics

`plugin_enabled` is keyed by **plugin ID**, not plugin spec. It controls whether a plugin's TUI module is activated at runtime.

Required behavior:

- Plugins are enabled by default unless explicitly overridden
- `plugin_enabled` is merged across config layers
- Runtime enable/disable state may also be persisted in TUI KV/runtime state and override config on startup
- Disabling a plugin in `plugin_enabled` does **not** uninstall it; it only prevents TUI activation

---

## Plugin Package Shape

TUI plugins are npm packages that export a `tui` entry point:

```json
{
  "name": "@acme/opencode-plugin",
  "type": "module",
  "exports": {
    "./server": { "import": "./dist/server.js" },
    "./tui": { "import": "./dist/tui.js" }
  },
  "engines": {
    "opencode": "^1.0.0"
  }
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `exports.tui` | Yes | TUI entry point |
| `exports.server` | No | Server/runtime entry point |
| `engines.opencode` | Yes | Minimum OpenCode version |

---

## Plugin Identity and Deduplication

### Identity

Each TUI plugin has a unique runtime `id` string (e.g., `"acme.demo"`).

ID rules:

- **file plugins** must export a non-empty `id`
- **npm plugins** may use an exported `id`; if omitted, package identity may be used as fallback
- TUI runtime identity is by resolved plugin ID, not by the raw input spec string alone

### Deduplication

Deduplication occurs before activation:

- Duplicate npm plugins are deduped by package identity
- Duplicate file plugins are deduped by exact resolved file spec
- Effective precedence follows config merge/loader rules; this document does not redefine those merge rules beyond requiring deterministic dedupe

---

## TUI Plugin Module

```tsx
import type { TuiPlugin, TuiPluginModule } from "@opencode-ai/plugin/tui"

const tui: TuiPlugin = async (api, options, meta) => {
  api.command.register(() => [
    {
      title: "Demo",
      value: "demo.open",
      onSelect: () => api.route.navigate("demo"),
    },
  ])

  api.route.register([
    {
      name: "demo",
      render: () => (
        <box>
          <text>demo</text>
        </box>
      ),
    },
  ])
}

const plugin: TuiPluginModule & { id: string } = {
  id: "acme.demo",
  tui,
}

export default plugin
```

### TUI Plugin Type

```typescript
type TuiPlugin = (
  api: TuiPluginAPI,
  options: unknown,
  meta: TuiPluginMeta
) => Promise<void> | void

type TuiPluginModule = {
  id: string
  tui: TuiPlugin
}
```

### Plugin Metadata

```typescript
meta: {
  state: "first" | "updated" | "same",
  id: string,
  source: string,       // npm:<name> or file:<path>
  spec: string,         // Full spec string
  target: "tui",
  // npm-only:
  requested?: string,   // Requested version range
  version?: string,     // Resolved version
  // file-only:
  modified?: number,    // File mtime
  // persisted:
  first_time?: number,
  last_time?: number,
  time_changed?: number,
  load_count?: number,
  fingerprint?: string
}
```

---

## TUI Plugin API

### Top-level API

```typescript
api.app.version

api.command.register(cb) / .trigger(value) / .show()
api.route.register(routes) / .navigate(name, params?)
api.route.current  // { name, params? }

api.ui.Dialog / DialogAlert / DialogConfirm / DialogPrompt / DialogSelect
api.ui.Slot / Prompt
api.ui.toast(message, variant?)
api.ui.dialog.replace(render, onClose?)
api.ui.dialog.clear()
api.ui.dialog.setSize("medium" | "large" | "xlarge")

api.keybind.match / .print / .create

api.tuiConfig

api.kv.get(key) / .set(key, value) / .ready

api.state.ready / .config / .provider / .path.*
api.state.workspace.list() / .get(id)
api.state.session.count() / .diff(id) / .todo(id) / .messages(id) / .status(id) / .permission(id) / .question(id)
api.state.part(messageID)
api.state.lsp() / .mcp()

api.theme.current / .selected / .has(name) / .set(name) / .mode() / .install(path) / .ready

api.client
api.scopedClient(workspaceID?)
api.workspace.current() / .set(workspaceID?)

api.event.on(type, handler)  // Returns unsubscribe

api.renderer

api.slots.register(plugin)

api.plugins.list()   // TUI plugin list
api.plugins.activate(id)
api.plugins.deactivate(id)

api.lifecycle.signal      // AbortSignal
api.lifecycle.onDispose(fn)  // Register cleanup, returns unregister
```

---

## Commands

Register command palette entries:

```typescript
api.command.register(() => [
  {
    title: "Demo",              // Display name
    value: "demo.open",        // Command ID
    description?: "...",
    category?: "...",
    keybind?: "ctrl+d",
    suggested?: true,
    hidden?: false,
    enabled?: true,
    slash?: { name: "demo", aliases?: ["d"] },
    onSelect: () => { }
  }
])
```

Returns an **unregister function** that removes the command.

Trigger a command programmatically:

```typescript
api.command.trigger("demo.open")
api.command.show()  // Open command palette
```

---

## Routes

Register custom routes/views:

```typescript
api.route.register([
  {
    name: "demo",              // Route name (reserved: home, session)
    render: () => <JSX>
  }
])
```

Navigate:

```typescript
api.route.navigate("demo")
api.route.navigate("session", { sessionID: "..." })

api.route.current  // { name: "home" | "session" | string, params?: {...} }
```

**Reserved route names**: `home`, `session`. Plugins cannot override these.

---

## Dialogs

Dialog components available via `api.ui`:

```typescript
api.ui.Dialog              // Base wrapper
api.ui.DialogAlert         // Alert component
api.ui.DialogConfirm       // Confirm dialog
api.ui.DialogPrompt        // Prompt input
api.ui.DialogSelect        // Selection dialog
api.ui.Slot                // Host/plugin slots
api.ui.Prompt               // Host prompt component
```

Dialog helpers:

```typescript
api.ui.toast(message, variant?)  // variant: "info" | "success" | "warning" | "error"

api.ui.dialog.replace(render, onClose?)  // Replace current dialog
api.ui.dialog.clear()                     // Close current dialog
api.ui.dialog.setSize("medium" | "large" | "xlarge")
```

---

## State

### KV Store

Shared TUI key-value storage:

```typescript
api.kv.ready    // Promise<true>
api.kv.get(key) // Promise<unknown | null>
api.kv.set(key, value) // Promise<void>
```

KV storage persists across sessions. Plugin authors should use plugin-specific key prefixes to avoid collisions.

### Live State

Read current TUI/application state:

```typescript
api.state.ready
api.state.config
api.state.provider
api.state.path.state
api.state.path.config
api.state.path.worktree
api.state.path.directory
api.state.vcs?.branch
api.state.workspace.list()
api.state.workspace.get(id)
api.state.session.count()
api.state.session.diff(sessionID)
api.state.session.todo(sessionID)
api.state.session.messages(sessionID)
api.state.session.status(sessionID)
api.state.session.permission(sessionID)
api.state.session.question(sessionID)
api.state.part(messageID)
api.state.lsp()
api.state.mcp()
```

---

## Theme

```typescript
api.theme.current    // Current theme tokens
api.theme.selected   // Selected theme name
api.theme.has(name)  // Check if theme installed
api.theme.set(name)  // Switch theme, returns boolean (false if not found)
api.theme.mode()     // "dark" | "light"
api.theme.ready     // Promise<true>
```

### Installing Themes

```typescript
api.theme.install("relative/to/plugin/root/theme.json")
```

Theme files are JSON with color definitions:

```json
{
  "defs": {
    "nord0": "#2E3440"
  },
  "theme": {
    "primary": { "dark": "nord8", "light": "nord10" },
    "text": { "dark": "nord4", "light": "nord0" },
    "background": { "dark": "nord0", "light": "nord6" }
  }
}
```

Installed themes are persisted by the host runtime. Theme installation and selection should follow host-managed persistence rather than ad hoc plugin-owned storage.

---

## TUI Plugins Runtime API

Runtime plugin management within the TUI:

```typescript
api.plugins.list()
```

Returns:

```typescript
{
  id: string,
  source: string,      // npm:<name> or file:<path>
  spec: string,
  target: "tui",
  enabled: boolean,    // Persisted desired state
  active: boolean     // true if initialized
}[]
```

```typescript
api.plugins.activate(id)     // Enable + initialize plugin in TUI runtime
api.plugins.deactivate(id)   // Disable + dispose plugin in TUI runtime
```

**Note**: These operate on TUI plugin state only. Server/runtime plugins are managed separately (see [Plugin System](./08-plugin-system.md)).

---

## Events

Subscribe to TUI events:

```typescript
api.event.on(type, handler)
// Returns unsubscribe function
```

TUI-specific event types:

- TUI lifecycle events
- Route change events
- Session events scoped to TUI
- Theme change events

---

## Lifecycle

### AbortSignal

```typescript
api.lifecycle.signal  // AbortSignal
```

Plugins should respect the abort signal for async operations. The host aborts the signal before running cleanup handlers.

### Disposal

Register cleanup callbacks:

```typescript
api.lifecycle.onDispose(fn)
// Returns unregister function
```

Cleanup is called when:

- Plugin is deactivated via `api.plugins.deactivate()`
- OpenCode TUI shuts down

**Cleanup should include**:

- Unregistering commands, routes, slots
- Canceling pending async work
- Releasing resources

```typescript
api.lifecycle.onDispose(() => {
  unregisterCommands()
  unregisterRoutes()
  unregisterSlots()
})
```

---

## Slots

Plugins can register slot renderers for host UI positions:

```typescript
api.slots.register(plugin)
```

**Host slot names**:

| Slot | Props |
|------|-------|
| `app` | — |
| `home_logo` | — |
| `home_prompt` | `{ workspace_id?, ref? }` |
| `home_prompt_right` | `{ workspace_id? }` |
| `session_prompt` | `{ session_id, visible?, disabled?, on_submit?, ref? }` |
| `session_prompt_right` | `{ session_id }` |
| `home_bottom` | — |
| `home_footer` | — |
| `sidebar_title` | `{ session_id, title, share_url? }` |
| `sidebar_content` | `{ session_id }` |
| `sidebar_footer` | `{ session_id }` |

---

## Built-in Plugins

| ID | Description |
|----|-------------|
| `internal:home-tips` | Home view tips |
| `internal:sidebar-context` | Context sidebar |
| `internal:sidebar-mcp` | MCP sidebar |
| `internal:sidebar-lsp` | LSP sidebar |
| `internal:sidebar-todo` | Todo sidebar |
| `internal:sidebar-files` | Files sidebar |
| `internal:sidebar-footer` | Sidebar footer |
| `internal:plugin-manager` | Plugin manager UI |

**Sidebar order**: context 100, mcp 200, lsp 300, todo 400, files 500

---

## Cleanup Guarantees

When a plugin is deactivated or the TUI shuts down:

1. The host aborts the plugin lifecycle signal
2. Host-tracked registrations such as commands, routes, event subscriptions, and slots are disposed
3. Registered `onDispose` callbacks run in reverse cleanup order
4. Cleanup is awaited by the host runtime
5. Cleanup failures are logged and should not block overall shutdown

The host may enforce a bounded cleanup budget per plugin. Plugin code must therefore avoid long-running disposal work.

---

## Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | Config authority, `tui.json` ownership, `plugin_enabled` semantics |
| [Plugin System](./08-plugin-system.md) | Server/runtime plugin hooks, events, custom tools |
| [TUI System](./09-tui-system.md) | `tui.json` schema, themes, keybinds, general TUI features |
