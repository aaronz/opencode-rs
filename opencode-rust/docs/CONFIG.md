# OpenCode RS Configuration Manual

Complete reference for configuring opencode-rs via `opencode.json` (or `.jsonc`, `.json5`), `tui.json`, environment variables, and CLI arguments.

## Config File Locations

OpenCode loads configuration from multiple sources in priority order (highest to lowest):

| Location | Scope | Notes |
|----------|-------|-------|
| `~/.config/opencode-rs/config.json` | Global | User-wide defaults |
| `~/.config/opencode-rs/opencode.json` | Global | Alias for above |
| `./.opencode-rs/config.json` | Project | Project-specific overrides |
| `./.opencode-rs/opencode.json` | Project | Alias for above |
| `OPENCODE_*` env vars | Session | CLI overrides |
| CLI arguments | Command | Highest priority |

Supported file extensions: `.json`, `.jsonc`, `.json5`

### TUI-Specific Config

TUI settings (`theme`, `keybinds`, `scroll_speed`, etc.) should be in a separate `tui.json` file:

- `~/.config/opencode-rs/tui.json` (global)
- `./.opencode-rs/tui.json` (project)

TUI fields in `opencode.json` will emit a warning and should be migrated to `tui.json`.

---

## Config Loading Precedence

Configuration is merged in this order (later sources override earlier):

1. Default values (built-in)
2. Global config (`~/.config/opencode-rs/config.json`)
3. Project config (`./.opencode-rs/config.json`)
4. `.opencode-rs/` directory contents (agents, commands, skills, plugins)
5. `tui.json` (merged separately for TUI settings)
6. Environment variables (`OPENCODE_*`)
7. CLI arguments (highest priority)

---

## Environment Variables

### General Settings

| Variable | Description | Example |
|----------|-------------|---------|
| `OPENCODE_CONFIG_DIR` | Override config directory | `/etc/opencode` |
| `OPENCODE_MODEL` | Default model | `openai/gpt-4o` |
| `OPENCODE_SMALL_MODEL` | Small/fast model for indexing | `openai/gpt-4o-mini` |
| `OPENCODE_API_KEY` | API key for default provider | `sk-...` |
| `OPENCODE_PROVIDER` | Default provider name | `openai` |
| `OPENCODE_TEMPERATURE` | Sampling temperature | `0.7` |
| `OPENCODE_MAX_TOKENS` | Max tokens per response | `4096` |
| `OPENCODE_USERNAME` | Username for sessions | `alice` |
| `OPENCODE_DEFAULT_AGENT` | Default agent name | `build` |
| `OPENCODE_LOG_LEVEL` | Log level | `debug`, `info`, `warn`, `error` |
| `OPENCODE_AUTO_SHARE` | Auto-share sessions | `true`, `false` |
| `OPENCODE_DISABLE_AUTOUPDATE` | Disable auto-update | `true`, `false` |
| `OPENCODE_SERVER_PASSWORD` | Server password | `secret` |

### Provider API Keys

Automatically loaded into provider config:

| Variable | Provider |
|----------|----------|
| `OPENAI_API_KEY` | OpenAI |
| `ANTHROPIC_API_KEY` | Anthropic |
| `GOOGLE_API_KEY` | Google (Gemini) |
| `AZURE_OPENAI_API_KEY` | Azure OpenAI |
| `OLLAMA_HOST` | Ollama |
| `AWS_ACCESS_KEY_ID` | AWS (Bedrock) |
| `COHERE_API_KEY` | Cohere |
| `MISTRAL_API_KEY` | Mistral |
| `PERPLEXITY_API_KEY` | Perplexity |
| `GROQ_API_KEY` | Groq |

### GitHub Integration

| Variable | Description |
|----------|-------------|
| `OPENCODE_GITHUB_API_URL` | GitHub Enterprise API URL |
| `OPENCODE_REMOTE_CONFIG_TOKEN` | Bearer token for remote config |

### Experimental Features

| Variable | Description |
|----------|-------------|
| `OPENCODE_EXPERIMENTAL` | Comma-separated flags: `batch_tool`, `open_telemetry`, `continue_loop_on_deny`, `disable_paste_summary` |
| `OPENCODE_ENABLE_EXA` | Enable Exa web search | `true`, `false` |

---

## Variable Expansion

Config values support variable expansion using `{env:VAR}`, `{file:path}`, and `{keychain:name}` syntax.

### Environment Variables

```json
{
  "provider": {
    "openai": {
      "api_key": "{env:OPENAI_API_KEY}"
    }
  }
}
```

### File Contents

```json
{
  "instructions": ["{file:./system-prompt.txt}"]
}
```

File paths are resolved relative to the config file location.

### Keychain Secrets

```json
{
  "provider": {
    "openai": {
      "api_key": "{keychain:openai-api-key}"
    }
  }
}
```

Supported keychain backends: macOS Keychain, Linux libsecret, Windows Credential Manager.

### Circular Reference Detection

The config parser detects circular references and will error:

```json
{
  "a": "{env:B}",
  "b": "{env:A}"
}
```

Error: `Circular environment variable reference detected: {env:A -> env:B -> env:A}`

---

## Config Schema Reference

### Top-Level Config (`Config`)

```json
{
  "$schema": "./opencode.schema.json",
  "log_level": "info",
  "server": { ... },
  "command": { ... },
  "skills": { ... },
  "watcher": { ... },
  "plugin": ["path/to/plugin.wasm"],
  "snapshot": true,
  "share": "manual",
  "autoshare": false,
  "autoupdate": true,
  "disabled_providers": [],
  "enabled_providers": [],
  "model": "openai/gpt-4o",
  "small_model": "openai/gpt-4o-mini",
  "default_agent": "default",
  "username": "alice",
  "agent": { ... },
  "provider": { ... },
  "mcp": { ... },
  "formatter": { ... },
  "lsp": { ... },
  "instructions": ["You are a helpful coding assistant."],
  "agents_md": { ... },
  "permission": { ... },
  "enterprise": { ... },
  "compaction": { ... },
  "experimental": { ... },
  "github": { ... },
  "api_key": "sk-...",
  "temperature": 0.7,
  "max_tokens": 4096,
  "hidden_models": []
}
```

| Field | Type | Description |
|-------|------|-------------|
| `$schema` | `string?` | JSON schema URL for validation |
| `log_level` | `LogLevel?` | Logging level: `trace`, `debug`, `info`, `warn`, `error` |
| `server` | `ServerConfig?` | Server settings (port, hostname, CORS, desktop, ACP) |
| `command` | `Record<string, CommandConfig>?` | Custom command templates |
| `skills` | `SkillsConfig?` | Skills paths and URLs |
| `watcher` | `WatcherConfig?` | File watcher settings |
| `plugin` | `string[]?` | Plugin paths to load |
| `snapshot` | `boolean?` | Enable session snapshots |
| `share` | `ShareMode?` | Session sharing: `manual`, `auto`, `disabled`, `read_only`, `collaborative`, `controlled` |
| `autoshare` | `boolean?` | Auto-share completed sessions |
| `autoupdate` | `AutoUpdate?` | Auto-update behavior: `true`, `false`, or `"notify"` |
| `disabled_providers` | `string[]?` | Disabled provider IDs |
| `enabled_providers` | `string[]?` | Enabled provider IDs (if set, only these are enabled) |
| `model` | `string?` | Default model (format: `provider/model`) |
| `small_model` | `string?` | Small model for fast tasks |
| `default_agent` | `string?` | Default agent name |
| `username` | `string?` | Username for session metadata |
| `agent` | `AgentMapConfig?` | Per-agent configuration |
| `provider` | `Record<string, ProviderConfig>?` | Provider configurations |
| `mcp` | `Record<string, McpConfig>?` | MCP server configurations |
| `formatter` | `FormatterConfig?` | Code formatter settings |
| `lsp` | `LspConfig?` | LSP server configurations |
| `instructions` | `string[]?` | System instructions |
| `agents_md` | `AgentsMdConfig?` | Agents.md scanning settings |
| `permission` | `PermissionConfig?` | Permission rules |
| `enterprise` | `EnterpriseConfig?` | Enterprise settings |
| `compaction` | `CompactionConfig?` | Session compaction settings |
| `experimental` | `ExperimentalConfig?` | Experimental feature flags |
| `github` | `GitHubConfig?` | GitHub integration |
| `api_key` | `string?` | API key (deprecated, use provider config) |
| `temperature` | `number?` | Sampling temperature (0.0-2.0) |
| `max_tokens` | `number?` | Max tokens per response |
| `hidden_models` | `string[]?` | Models to hide from picker |

---

### ServerConfig

```json
{
  "server": {
    "port": 8080,
    "hostname": "0.0.0.0",
    "mdns": true,
    "mdns_domain": "opencode.local",
    "cors": ["http://localhost:3000"],
    "desktop": { ... },
    "acp": { ... },
    "password": "secret"
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `port` | `number?` | Server port |
| `hostname` | `string?` | Server hostname |
| `mdns` | `boolean?` | Enable mDNS discovery |
| `mdns_domain` | `string?` | mDNS domain |
| `cors` | `string[]?` | CORS allowed origins |
| `desktop` | `DesktopConfig?` | Desktop mode settings |
| `acp` | `AcpConfig?` | ACP protocol settings |
| `password` | `string?` | Server password |

### DesktopConfig

```json
{
  "desktop": {
    "enabled": true,
    "auto_open_browser": true,
    "port": 3000,
    "hostname": "127.0.0.1"
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | `boolean?` | Enable desktop mode |
| `auto_open_browser` | `boolean?` | Auto-open browser on startup |
| `port` | `number?` | Desktop web UI port |
| `hostname` | `string?` | Desktop web UI hostname |

### AcpConfig

```json
{
  "acp": {
    "enabled": true,
    "server_id": "local",
    "version": "1.0",
    "session": { ... }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | `boolean?` | Enable ACP protocol |
| `server_id` | `string?` | Server identifier |
| `version` | `string?` | ACP version |
| `session` | `AcpSession?` | Active session (rarely configured manually) |

### AgentMapConfig / AgentConfig

```json
{
  "agent": {
    "defaultAgent": "build",
    "agents": {
      "build": {
        "model": "openai/gpt-4o",
        "variant": "chat",
        "temperature": 0.2,
        "top_p": 0.9,
        "prompt": "You are a code generation agent...",
        "disable": false,
        "description": "Build agent for code generation",
        "hidden": false,
        "options": {},
        "color": "#4A90D9",
        "steps": 100,
        "max_steps": 200,
        "permission": { ... }
      }
    }
  }
}
```

Or as a simple map (legacy format):

```json
{
  "agent": {
    "build": {
      "model": "openai/gpt-4o",
      "temperature": 0.2
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `model` | `string?` | Model for this agent |
| `variant` | `string?` | Model variant |
| `temperature` | `number?` | Sampling temperature |
| `top_p` | `number?` | Nucleus sampling |
| `prompt` | `string?` | System prompt override |
| `disable` | `boolean?` | Disable this agent |
| `description` | `string?` | Agent description |
| `hidden` | `boolean?` | Hide from UI |
| `options` | `Record<string, any>?` | Custom options |
| `color` | `string?` | UI color (hex) |
| `steps` | `number?` | Max steps per session |
| `max_steps` | `number?` | Alias for steps |
| `permission` | `PermissionConfig?` | Permission overrides |

---

### ProviderConfig

```json
{
  "provider": {
    "openai": {
      "id": "openai",
      "name": "OpenAI",
      "whitelist": [],
      "blacklist": [],
      "models": {
        "gpt-4o": { ... }
      },
      "options": {
        "api_key": "{env:OPENAI_API_KEY}",
        "base_url": "https://api.openai.com/v1",
        "enterprise_url": "",
        "set_cache_key": true,
        "timeout": 60000,
        "chunk_timeout": 30000,
        "aws_region": "us-east-1",
        "aws_profile": "default",
        "aws_endpoint": "",
        "headers": {}
      }
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | `string?` | Provider identifier |
| `name` | `string?` | Display name |
| `whitelist` | `string[]?` | Allowed models |
| `blacklist` | `string[]?` | Blocked models |
| `models` | `Record<string, ModelConfig>?` | Model configurations |
| `options` | `ProviderOptions?` | Provider-specific options |

### ProviderOptions

```json
{
  "options": {
    "api_key": "sk-...",
    "base_url": "https://api.openai.com/v1",
    "enterprise_url": "",
    "set_cache_key": true,
    "timeout": 60000,
    "chunk_timeout": 30000,
    "aws_region": "us-east-1",
    "aws_profile": "default",
    "aws_endpoint": "",
    "headers": {
      "X-Custom-Header": "value"
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `api_key` | `string?` | API key |
| `base_url` | `string?` | Base URL for API |
| `enterprise_url` | `string?` | Enterprise URL |
| `set_cache_key` | `boolean?` | Enable caching |
| `timeout` | `number?` | Request timeout (ms) or `{"type": "none"}` |
| `chunk_timeout` | `number?` | Chunk timeout (ms) |
| `aws_region` | `string?` | AWS region |
| `aws_profile` | `string?` | AWS profile |
| `aws_endpoint` | `string?` | Custom AWS endpoint |
| `headers` | `Record<string, string>?` | Custom headers |

### TimeoutConfig

```json
{
  "timeout": 60000
}
```

Or to disable timeout:

```json
{
  "timeout": {
    "type": "none"
  }
}
```

---

### ModelConfig

```json
{
  "models": {
    "gpt-4o": {
      "id": "gpt-4o",
      "name": "GPT-4o",
      "variants": {
        "chat": { ... },
        "preview": { ... }
      },
      "visible": true
    }
  }
}
```

### VariantConfig

```json
{
  "variants": {
    "chat": {
      "disabled": false
    }
  }
}
```

---

### McpConfig

MCP server configuration with three variants:

```json
{
  "mcp": {
    "local-mcp": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "./"],
      "environment": { "KEY": "value" },
      "enabled": true,
      "timeout": 60000,
      "max_tokens": 100000,
      "cost_warning_threshold": 0.1,
      "cost_limit_threshold": 1.0
    },
    "remote-mcp": {
      "type": "remote",
      "url": "https://mcp.example.com/sse",
      "enabled": true,
      "headers": { "Authorization": "Bearer token" },
      "oauth": {
        "type": "config",
        "client_id": "...",
        "client_secret": "...",
        "scope": "read write"
      },
      "timeout": 60000,
      "max_tokens": 100000,
      "cost_warning_threshold": 0.1,
      "cost_limit_threshold": 1.0
    },
    "simple-mcp": {
      "type": "simple",
      "enabled": true
    }
  }
}
```

#### McpLocalConfig

| Field | Type | Description |
|-------|------|-------------|
| `command` | `string[]` | Command and arguments |
| `environment` | `Record<string, string>?` | Environment variables |
| `enabled` | `boolean?` | Enable this server |
| `timeout` | `number?` | Timeout in ms |
| `max_tokens` | `number?` | Max tokens |
| `cost_warning_threshold` | `number?` | Warning threshold (USD) |
| `cost_limit_threshold` | `number?` | Cost limit (USD) |

#### McpRemoteConfig

| Field | Type | Description |
|-------|------|-------------|
| `url` | `string` | Remote server URL |
| `enabled` | `boolean?` | Enable this server |
| `headers` | `Record<string, string>?` | HTTP headers |
| `oauth` | `McpOAuthUnion?` | OAuth configuration |
| `timeout` | `number?` | Timeout in ms |
| `max_tokens` | `number?` | Max tokens |
| `cost_warning_threshold` | `number?` | Warning threshold (USD) |
| `cost_limit_threshold` | `number?` | Cost limit (USD) |

---

### FormatterConfig

```json
{
  "formatter": {
    "rust": {
      "disabled": false,
      "command": ["rustfmt"],
      "environment": {},
      "extensions": ["rs"]
    },
    "disabled": true
  }
}
```

Or to disable all formatters:

```json
{
  "formatter": true
}
```

#### FormatterEntry

| Field | Type | Description |
|-------|------|-------------|
| `disabled` | `boolean?` | Disable this formatter |
| `command` | `string[]?` | Formatter command |
| `environment` | `Record<string, string>?` | Environment variables |
| `extensions` | `string[]?` | File extensions |

---

### LspConfig

```json
{
  "lsp": {
    "rust": {
      "command": ["rust-analyzer"],
      "extensions": ["rs"],
      "disabled": false,
      "env": {},
      "initialization": {}
    },
    "disabled": true
  }
}
```

#### LspEntry

| Field | Type | Description |
|-------|------|-------------|
| `command` | `string[]` | LSP command |
| `extensions` | `string[]?` | File extensions |
| `disabled` | `boolean?` | Disable this server |
| `env` | `Record<string, string>?` | Environment variables |
| `initialization` | `Record<string, any>?` | Initialization options |

---

### PermissionConfig

```json
{
  "permission": {
    "read": "allow",
    "edit": "ask",
    "glob": "allow",
    "grep": "allow",
    "list": "allow",
    "bash": { "action": "deny" },
    "task": "ask",
    "external_directory": "ask",
    "todowrite": "ask",
    "question": "allow",
    "webfetch": "ask",
    "websearch": "deny",
    "codesearch": "ask",
    "lsp": "allow",
    "doom_loop": "deny",
    "skill": "ask"
  }
}
```

Permission actions: `allow`, `ask`, `deny`

Object format for per-path rules:

```json
{
  "bash": {
    "/usr/bin/npm": "allow",
    "/bin/rm": "deny",
    "*": "ask"
  }
}
```

---

### SkillsConfig

```json
{
  "skills": {
    "paths": ["/path/to/skills", "./custom-skills"],
    "urls": ["https://raw.githubusercontent.com/user/skills/main/skill.yaml"]
  }
}
```

### AgentsMdConfig

```json
{
  "agents_md": {
    "enabled": true,
    "stop_at_worktree_root": true,
    "include_hidden": false
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | `boolean?` | Enable agents.md scanning |
| `stop_at_worktree_root` | `boolean?` | Stop at git worktree root |
| `include_hidden` | `boolean?` | Include hidden directories |

---

### WatcherConfig

```json
{
  "watcher": {
    "ignore": ["node_modules", "target", ".git", "*.lock"]
  }
}
```

### CompactionConfig

```json
{
  "compaction": {
    "auto": true,
    "prune": true,
    "reserved": 100,
    "warning_threshold": 0.8,
    "compact_threshold": 0.9,
    "continuation_threshold": 0.7,
    "preserve_recent_messages": 10,
    "preserve_system_messages": true,
    "summary_prefix": "Session Summary"
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `auto` | `boolean?` | Enable auto compaction |
| `prune` | `boolean?` | Prune old sessions |
| `reserved` | `number?` | Reserved message slots |
| `warning_threshold` | `number?` | Warning threshold (0.0-1.0) |
| `compact_threshold` | `number?` | Compaction threshold |
| `continuation_threshold` | `number?` | Continuation threshold |
| `preserve_recent_messages` | `number?` | Recent messages to keep |
| `preserve_system_messages` | `boolean?` | Preserve system messages |
| `summary_prefix` | `string?` | Summary prefix |

---

### ExperimentalConfig

```json
{
  "experimental": {
    "disable_paste_summary": false,
    "batch_tool": true,
    "open_telemetry": false,
    "primary_tools": ["read", "edit", "bash"],
    "continue_loop_on_deny": false,
    "mcp_timeout": 60000,
    "enable_exa": false
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `disable_paste_summary` | `boolean?` | Disable paste summary |
| `batch_tool` | `boolean?` | Enable batch tool calls |
| `open_telemetry` | `boolean?` | Enable OpenTelemetry |
| `primary_tools` | `string[]?` | Primary tools list |
| `continue_loop_on_deny` | `boolean?` | Continue on permission deny |
| `mcp_timeout` | `number?` | MCP timeout (ms) |
| `enable_exa` | `boolean?` | Enable Exa search |

---

### EnterpriseConfig

```json
{
  "enterprise": {
    "url": "https://enterprise.opencode.ai",
    "remote_config_domain": "config.enterprise.com"
  }
}
```

### GitHubConfig

```json
{
  "github": {
    "api_url": "https://api.github.com",
    "installs": [
      {
        "owner": "myorg",
        "repo": "myrepo",
        "branch": "main",
        "workflow_path": ".github/opencode.yml",
        "commit_sha": "abc123",
        "installed_at": "2024-01-01T00:00:00Z"
      }
    ]
  }
}
```

---

### CommandConfig

```json
{
  "command": {
    "review": {
      "template": "Review this code for bugs and style issues:\n\n{context}",
      "description": "Code review command",
      "agent": "review",
      "model": "openai/gpt-4o",
      "subtask": false
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `template` | `string` | Command template |
| `description` | `string?` | Command description |
| `agent` | `string?` | Agent to use |
| `model` | `string?` | Model to use |
| `subtask` | `boolean?` | Run as subtask |

---

### TuiConfig (tui.json)

```json
{
  "$schema": "./tui.schema.json",
  "scroll_speed": 5,
  "scroll_acceleration": {
    "enabled": true,
    "speed": 1.5
  },
  "diff_style": "side-by-side",
  "theme": {
    "name": "catppuccin-mocha",
    "path": "~/themes/catppuccin-mocha.toml",
    "scan_dirs": ["~/.config/opencode/themes"]
  },
  "keybinds": {
    "commands": "ctrl-p",
    "timeline": "ctrl-t",
    "settings": "ctrl-,",
    "models": "ctrl-m",
    "files": "ctrl-f",
    "terminal": "ctrl-`",
    "custom": {
      "my_action": "ctrl-shift-m"
    }
  },
  "plugins": {
    "plugin_enabled": true,
    "plugins": {
      "vim": false
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `scroll_speed` | `number?` | Scroll speed |
| `scroll_acceleration` | `ScrollAccelerationConfig?` | Scroll acceleration |
| `diff_style` | `DiffStyle?` | Diff display: `side-by-side`, `inline`, `unified`, `auto`, `stacked` |
| `theme` | `ThemeConfig?` | Theme settings |
| `keybinds` | `KeybindConfig?` | Keyboard shortcuts |
| `plugins` | `TuiPluginConfig?` | Plugin settings |

#### ScrollAccelerationConfig

```json
{
  "scroll_acceleration": {
    "enabled": true,
    "speed": 1.5
  }
}
```

Or as a legacy number:

```json
{
  "scroll_acceleration": 1.5
}
```

#### ThemeConfig

```json
{
  "theme": {
    "name": "catppuccin-mocha",
    "path": "~/themes/catppuccin-mocha.toml",
    "scan_dirs": ["~/.config/opencode/themes"]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string?` | Theme name |
| `path` | `string?` | Theme file path |
| `scan_dirs` | `string[]?` | Directories to scan for themes |

#### KeybindConfig

```json
{
  "keybinds": {
    "commands": "ctrl-p",
    "timeline": "ctrl-t",
    "settings": "ctrl-,",
    "models": "ctrl-m",
    "files": "ctrl-f",
    "terminal": "ctrl-`",
    "custom": {
      "my_action": "ctrl-shift-m"
    }
  }
}
```

---

## Examples

### Basic Configuration

```json
{
  "model": "openai/gpt-4o",
  "temperature": 0.7,
  "provider": {
    "openai": {
      "options": {
        "api_key": "{env:OPENAI_API_KEY}"
      }
    }
  }
}
```

### Multi-Provider Setup

```json
{
  "model": "openai/gpt-4o",
  "small_model": "anthropic/claude-3-haiku",
  "provider": {
    "openai": {
      "models": {
        "gpt-4o": {
          "variants": {
            "chat": {},
            "preview": {}
          }
        }
      },
      "options": {
        "api_key": "{env:OPENAI_API_KEY}"
      }
    },
    "anthropic": {
      "options": {
        "api_key": "{env:ANTHROPIC_API_KEY}"
      }
    }
  }
}
```

### Agent Configuration

```json
{
  "default_agent": "build",
  "agent": {
    "defaultAgent": "build",
    "agents": {
      "build": {
        "model": "openai/gpt-4o",
        "temperature": 0.2,
        "description": "Code generation agent",
        "max_steps": 100,
        "permission": {
          "bash": "ask",
          "edit": "allow"
        }
      },
      "review": {
        "model": "openai/gpt-4o",
        "temperature": 0.1,
        "description": "Code review agent",
        "permission": {
          "read": "allow",
          "bash": "deny"
        }
      }
    }
  }
}
```

### MCP Servers

```json
{
  "mcp": {
    "filesystem": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "./src"],
      "enabled": true
    },
    "github": {
      "type": "remote",
      "url": "https://api.github.com/mcp",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer {env:GITHUB_TOKEN}"
      }
    }
  }
}
```

### Permissions

```json
{
  "permission": {
    "read": "allow",
    "edit": "ask",
    "glob": "allow",
    "grep": "allow",
    "bash": {
      "/usr/bin/npm": "allow",
      "/usr/bin/pnpm": "allow",
      "/bin/rm": "deny",
      "*": "ask"
    },
    "webfetch": "ask",
    "websearch": "deny"
  }
}
```

### Session Compaction

```json
{
  "compaction": {
    "auto": true,
    "warning_threshold": 0.8,
    "compact_threshold": 0.95,
    "preserve_recent_messages": 5,
    "preserve_system_messages": true,
    "summary_prefix": "[Compacted Session]"
  }
}
```

### Custom Commands

```json
{
  "command": {
    "review": {
      "template": "Review the following code for:\n- Security vulnerabilities\n- Performance issues\n- Code style\n\n```{context}\n```",
      "description": "Comprehensive code review",
      "agent": "review",
      "temperature": 0.1
    },
    "explain": {
      "template": "Explain what this code does in simple terms:\n\n{context}",
      "description": "Explain code",
      "subtask": false
    }
  }
}
```

---

## Deprecated Fields

The following fields are deprecated and will be removed in v4.0:

| Old Field | Replacement | Notes |
|-----------|-------------|-------|
| `mode` | `agent[].permission` | Per-agent permissions |
| `tools` | `permission` | Tool permissions |
| `theme` | `tui.json` | TUI theme |
| `keybinds` | `tui.json` | TUI keybinds |

Migration is automatic, but a warning is logged. See [migration guide](https://docs.opencode.ai/config/migration).

---

## JSON Schema

OpenCode supports JSON Schema validation for configuration files.

### Schema Reference

Add a `$schema` field to your config to enable validation:

```json
{
  "$schema": "./opencode.schema.json",
  "model": "openai/gpt-4o"
}
```

### Built-in Schema

The binary includes a comprehensive built-in schema (`builtin_config.schema.json`) containing:

- All top-level config fields
- All nested structure definitions (ServerConfig, AgentConfig, ProviderConfig, etc.)
- Type validation (string, number, boolean, array, object)
- Enum constraints (LogLevel, ShareMode, PermissionAction, DiffStyle, etc.)
- Numeric ranges (temperature 0-2, port 1-65535, etc.)

### Schema Validation Behavior

1. **Config Load**: Schema validation runs automatically when loading config
2. **Unknown Fields**: Warning logged for unknown fields (non-strict mode)
3. **Type Errors**: Error logged if field type doesn't match schema
4. **CLI Validate**: Use `opencode --validate` to validate config without starting

### Schema Cache

Remote schemas are cached in:

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/opencode-rs/schemas/` |
| Linux | `~/.config/opencode-rs/schemas/` |
| Windows | `%APPDATA%\opencode-rs\schemas\` |

Override cache directory with `OPENCODE_SCHEMA_CACHE_DIR` environment variable.

### Schema URL Resolution

The `$schema` URL is resolved in order:

1. Value of `$schema` field in config file
2. Fetched from remote URL if valid http/https URI
3. Cached locally for offline use
4. Falls back to built-in minimal schema on failure
