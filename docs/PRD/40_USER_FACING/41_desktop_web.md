# PRD: Desktop and Web Interface

## Overview

OpenCode provides desktop, web, and editor-facing interfaces alongside the CLI/TUI experience.

This document describes interface shapes and integration patterns. It intentionally avoids treating any single packaging/runtime choice as universally canonical unless backed by the upstream implementation chosen for the Rust port.

---

## Desktop App

### Overview

OpenCode Desktop runs a local server with a web-based UI inside a native desktop shell.

### Architecture

```
┌─────────────────────────────┐
│  Desktop App shell            │
│  ┌─────────────────────────┐ │
│  │  WebView / Webview2     │ │
│  │  (OpenCode Web UI)      │ │
│  └─────────────────────────┘ │
│  ┌─────────────────────────┐ │
│  │  opencode-cli (subsidiary)│ │
│  └─────────────────────────┘ │
└─────────────────────────────┘
```

### Startup Flow

1. Desktop app starts
2. Launches an OpenCode runtime/backend subprocess or embedded service
3. WebView loads CLI's web server
4. UI communicates via localhost HTTP

### Troubleshooting

#### Disable Plugins

If app crashes, disable plugins. Server/runtime plugins are configured in main `opencode.json` under `plugin` (see [Configuration System](./06-configuration-system.md) and [Plugin System](./08-plugin-system.md)). TUI plugins are configured in `tui.json` (see [TUI Plugin API](./15-tui-plugin-api.md)).

Or rename plugin directories temporarily.

#### Clear Cache

```bash
# macOS
rm -rf ~/.cache/opencode

# Linux
rm -rf ~/.cache/opencode

# Windows
del %USERPROFILE%\.cache\opencode
```

#### Server Connection Issues

- Clear desktop default server URL
- Check `OPENCODE_PORT` env var
- Verify `server.port` / `server.hostname` in config

---

## Web Interface

### Overview

Browser-based OpenCode accessible via a web-serving mode.

### Startup

```bash
opencode web [--port 4096] [--hostname 0.0.0.0]
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--port` | random | Port |
| `--hostname` | 127.0.0.1 | Bind address |
| `--mdns` | false | Enable discovery |
| `--mdns-domain` | opencode.local | mDNS name |
| `--cors` | [] | CORS origins |

### Security

```bash
OPENCODE_SERVER_PASSWORD=secret opencode web
```

Authentication details should follow the selected server/auth implementation baseline.

### Attach TUI

```bash
opencode attach http://localhost:4096
```

Shares sessions between web and CLI TUI.

---

## IDE Integration

### VS Code / Cursor / Compatible

1. Install OpenCode CLI
2. Open terminal in IDE
3. Run `opencode` - extension auto-installs

### Features

- `Cmd+Esc` / `Ctrl+Esc`: Open/focus OpenCode session
- `Cmd+Shift+Esc` / `Ctrl+Shift+Esc`: New session
- Context-aware file sharing
- `Cmd+Option+K` / `Alt+Ctrl+K`: Insert file reference

### Manual Install

Search "OpenCode" in VS Code marketplace.

---

## ACP (Agent Client Protocol)

OpenCode may expose ACP-compatible integration for editor clients. Exact wire behavior and route ownership should follow the selected server/API baseline. See [Server API](./07-server-api.md).

For editor integration:

### Zed

```json
{
  "agent_servers": {
    "OpenCode": {
      "command": "opencode",
      "args": ["acp"]
    }
  }
}
```

### JetBrains

Add to `acp.json`:
```json
{
  "agent_servers": {
    "OpenCode": {
      "command": "/path/to/bin/opencode",
      "args": ["acp"]
    }
  }
}
```

### Neovim (Avante.nvim)

```lua
{
  acp_providers = {
    ["opencode"] = {
      command = "opencode",
      args = { "acp" }
    }
  }
}
```

### CodeCompanion.nvim

```lua
require("codecompanion").setup({
  interactions = {
    chat = {
      adapter = { name = "opencode" }
    }
  }
})
```

---

## Windows WSL Setup

### Recommended Setup

1. Install WSL2
2. Install OpenCode in WSL
3. Run from WSL terminal

### Desktop + WSL Server

```bash
# In WSL
opencode serve --hostname 0.0.0.0 --port 4096

# In Desktop app
Connect to http://localhost:4096
```

### Web + WSL

```bash
# In WSL
opencode web --hostname 0.0.0.0

# In Windows browser
http://localhost:4096
```

---

## Notifications

Desktop shows notifications when:
- App not focused
- Notifications enabled in OS settings

---

## Sharing

### Modes

- `manual`: `/share` command
- `auto`: Auto-share new sessions
- `disabled`: No sharing

### Privacy

Shared sessions may be published through an OpenCode-managed sharing surface, depending on deployment mode. Shared data typically includes:
- Full conversation history
- Session metadata

Unshare via `/unshare`.

### Enterprise / Managed Deployments

Enterprise or managed deployments may:
- Disable sharing entirely
- Restrict sharing with SSO or organization policy
- Provide self-hosted or private sharing infrastructure
