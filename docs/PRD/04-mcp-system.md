# PRD: MCP (Model Context Protocol) System

## Overview

This document describes OpenCode's MCP integration, supporting both local and remote MCP servers.

It is not the canonical config-schema authority. The `mcp` top-level config key is owned by [Configuration System](./06-configuration-system.md); this document covers MCP-specific runtime behavior and integration semantics.

---

## MCP Server Configuration

### Configuration Schema

```json
{
  "mcp": {
    "server-name": {
      "type": "local" | "remote",
      "command?: string[]",        // local only
      "url?: string",             // remote only
      "enabled": boolean,
      "environment?: object",      // local only
      "headers?: object",         // remote only
      "oauth?: object | false",
      "timeout?: number           // milliseconds
    }
  }
}
```

### Local MCP Server

```json
{
  "mcp": {
    "my-local-server": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-everything"],
      "environment": {
        "MY_ENV_VAR": "value"
      },
      "timeout": 5000
    }
  }
}
```

### Remote MCP Server

```json
{
  "mcp": {
    "my-remote-server": {
      "type": "remote",
      "url": "https://mcp.example.com/mcp",
      "headers": {
        "Authorization": "Bearer $API_KEY"
      },
      "timeout": 10000
    }
  }
}
```

---

## OAuth Support

### Automatic OAuth Flow

OpenCode automatically handles OAuth for supported servers:
1. Detects 401 response
2. Initiates OAuth flow
3. Uses dynamic client registration (RFC 7591) when supported
4. Securely stores tokens for subsequent requests

### OAuth Configuration

OAuth configuration is scoped per MCP server entry rather than as a standalone top-level config block:

```json
{
  "mcp": {
    "my-remote-server": {
      "type": "remote",
      "url": "https://mcp.example.com/mcp",
      "oauth": {
        "clientId": "{env:MY_CLIENT_ID}",
        "clientSecret": "{env:MY_CLIENT_SECRET}",
        "scope": "tools:read tools:execute"
      }
    }
  }
}
```

### Disable OAuth

```json
{
  "mcp": {
    "my-remote-server": {
      "oauth": false
    }
  }
}
```

### OAuth CLI Commands

```bash
opencode mcp auth <server-name>    # Authenticate specific server
opencode mcp auth list             # List auth status
opencode mcp debug <server-name>   # Debug OAuth issues
opencode mcp logout <server-name> # Remove credentials
```

---

## Tool Integration

### Tool Naming

MCP tools are exposed with the format `<servername>_<toolname>`:
- Server `sentry` with tool `list_issues` → `sentry_list_issues`

### Permission Control

Permission rules use glob patterns (see [Configuration System](./06-configuration-system.md)):

```json
{
  "permission": {
    "mcp_sentry_*": "deny"        // disable all sentry tools
  }
}
```

### Per-Agent Configuration

```json
{
  "permission": {
    "mcp_*": "deny"               // disable globally
  },
  "agent": {
    "my-agent": {
      "permission": {
        "mcp_github_*": "allow"   // enable for specific agent
      }
    }
  }
}
```

---

## Built-in MCP Server Examples

### Sentry

```json
{
  "mcp": {
    "sentry": {
      "type": "remote",
      "url": "https://mcp.sentry.dev/mcp",
      "oauth": {}
    }
  }
}
```

### Context7

```json
{
  "mcp": {
    "context7": {
      "type": "remote",
      "url": "https://mcp.context7.com/mcp"
    }
  }
}
```

### Vercel Grep

```json
{
  "mcp": {
    "gh_grep": {
      "type": "remote",
      "url": "https://mcp.grep.app"
    }
  }
}
```

---

## Configuration Ownership

MCP configuration is owned by the MCP subsystem with schema referenced in [Configuration System](./06-configuration-system.md). The main config holds `mcp` as top-level key; MCP runtime semantics are defined here.

---

## Context Usage Warning

MCP servers consume context space. Each tool definition and its schema adds to the prompt size. Users should carefully select which MCP servers to enable.

---

## Transport

### Local Servers

Use stdio transport with JSON-RPC protocol:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {}
}
```

### Remote Servers

Use HTTP+SSE transport:
- POST requests for tool calls
- SSE stream for tool responses

Default timeout: 5000ms (5 seconds)

```json
{
  "timeout": 10000  // 10 seconds for slow servers
}
```

---

## Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | Config ownership, `mcp` key schema, `permission` rules |
| [07-server-api.md](./07-server-api.md) | MCP API endpoints (if exposed via HTTP) |
