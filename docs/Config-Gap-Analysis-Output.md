# Config Gap Analysis: opencode-rs vs Official opencode

## 1. Executive Summary

**Compatibility Status: PARTIALLY COMPATIBLE**

opencode-rs shares the core configuration architecture with opencode but diverges in several significant areas:

- **Shared**: ~70% of field names, basic structure, permission model
- **Incompatible**: LogLevel case, ShareMode enum values, Agent/Mode structure, Provider model
- **Missing**: `shell` field, certain agent modes (`subagent`, `primary`, `all`), `logLevel` upper case
- **Extra**: `provider` objects, `mcp`, `formatter`, `lsp`, `tui`, `compaction`, `experimental`, `github`, `enterprise` configs

The most critical gaps are **P0** (blocks core compatibility): LogLevel case sensitivity and the `mode` vs `agent` structural difference.

---

## 2. Current opencode-rs Config Architecture

### Main Files/Modules

```
crates/config/src/
├── lib.rs              # Main Config struct, ServerConfig, AgentConfig, etc.
├── schema.rs           # Schema validation, KNOWN_CONFIG_FIELDS
├── merge.rs            # Config merging logic
├── jsonc.rs            # JSONC parsing (comments)
├── directory_scanner.rs # .opencode-rs/ directory scanning
├── remote_cache.rs     # Remote config caching
├── secret_storage.rs   # Keychain integration
└── builtin_config.schema.json  # JSON schema (669 lines)
```

### Data Structures

- `Config` - Top-level (35+ fields)
- `ServerConfig`, `DesktopConfig`, `AcpConfig`
- `AgentMapConfig`, `AgentConfig`
- `ProviderConfig`, `ProviderOptions`, `ModelConfig`, `VariantConfig`
- `McpConfig` (Local/Remote/Simple)
- `PermissionConfig`, `PermissionRule`, `PermissionAction`
- `FormatterConfig`, `LspConfig`
- `TuiConfig`, `ThemeConfig`, `KeybindConfig`

### Load/Save Flow

1. `Config::load()` reads file (JSON/JSONC/JSON5)
2. `Config::parse_json_content()` handles JSONC comments
3. `expand_variables()` resolves `{env:}`, `{file:}`, `{keychain:}`
4. `apply_env_overrides()` applies environment variables
5. `merge_opencode_directory_into_config()` adds from `.opencode-rs/` directory
6. TUI config merged separately from `tui.json`

### Validation

- `validate_unknown_fields()` warns on unknown fields
- `validate_json_schema()` performs JSON Schema validation (fetches remote or uses builtin)
- Deprecation warnings for `mode`, `tools`, `theme`, `keybinds`

### Config Path Strategy

| Location | Purpose |
|----------|---------|
| `~/.config/opencode-rs/config.json` | Global user config |
| `./.opencode-rs/config.json` | Project config |
| `tui.json` (same locations) | TUI-specific settings |

### Known Risks

1. **No `shell` field** - opencode supports `shell` to set default shell
2. **LogLevel case mismatch** - opencode uses uppercase
3. **ShareMode extended** - opencode-rs has extra enum values
4. **Agent structure** - `mode.build`/`mode.plan` vs `agent.agents`

---

## 3. Official opencode Schema Overview

### Key Configuration Domains

1. **Core Settings**: `shell`, `logLevel`, `snapshot`, `share`, `autoupdate`
2. **Server**: `port`, `hostname`, `mdns`, `cors`
3. **Providers**: No explicit provider config objects - uses `OPENAI_API_KEY`, etc. env vars
4. **Agents**: Uses `mode` with `build`/`plan` as primary, plus custom agents
5. **Permissions**: Comprehensive `PermissionConfig` with per-tool rules
6. **Commands**: Custom command templates
7. **Skills**: Path and URL-based skill loading
8. **Plugins**: Array of plugin paths with optional config objects

---

## 4. Field-by-Field Compatibility Matrix

| Official Field | Official Type | opencode-rs | Gap Type | Severity |
|----------------|---------------|-------------|----------|----------|
| `shell` | `string` | **MISSING** | Missing | P2 |
| `logLevel` | `enum: DEBUG\|INFO\|WARN\|ERROR` | `log_level: trace\|debug\|info\|warn\|error` | Wrong case + extra trace | P0 |
| `server.port` | `integer` | `Option<u16>` | None | - |
| `server.hostname` | `string` | `Option<String>` | None | - |
| `server.mdns` | `boolean` | `Option<bool>` | None | - |
| `server.mdnsDomain` | `string` | `mdns_domain` (snake_case) | Minor | P3 |
| `server.cors` | `string[]` | `Option<Vec<String>>` | None | - |
| `server.additionalProperties` | `false` | **ALLOWS EXTRAS** | Extra incompatible | P1 |
| `command` | `object` | `Option<HashMap<String, CommandConfig>>` | None | - |
| `skills.paths` | `string[]` | `Option<Vec<String>>` | None | - |
| `skills.urls` | `string[] (uris)` | `Option<Vec<String>>` | None | - |
| `watcher.ignore` | `string[]` | `Option<Vec<String>>` | None | - |
| `snapshot` | `boolean` | `Option<bool>` | None | - |
| `plugin` | `array (string\|[string,object])` | `Option<Vec<String>>` | Partially supported | P1 |
| `share` | `enum: manual\|auto\|disabled` | `enum: manual\|auto\|disabled\|read_only\|collaborative\|controlled` | Extra values | P1 |
| `autoshare` | `boolean` | `Option<bool>` (deprecated) | None | - |
| `autoupdate` | `boolean\|"notify"` | `AutoUpdate::Bool(bool)\|Notify(String)` | Same semantics | - |
| `disabled_providers` | `string[]` | `Option<Vec<String>>` | None | - |
| `enabled_providers` | `string[]` | `Option<Vec<String>>` | None | - |
| `model` | `string (provider/model)` | `Option<String>` | None | - |
| `small_model` | `string (provider/model)` | `Option<String>` | None | - |
| `default_agent` | `string` | `Option<String>` | None | - |
| `username` | `string` | `Option<String>` | None | - |
| `mode` | **deprecated**, `build`/`plan` primary | `agent` with `agents` map | Different structure | P0 |
| `mode.*.model` | `string` | `AgentConfig.model: Option<String>` | None | - |
| `mode.*.variant` | `string` | `AgentConfig.variant: Option<String>` | None | - |
| `mode.*.temperature` | `number` | `Option<f32>` | None | - |
| `mode.*.top_p` | `number` | `Option<f32>` | None | - |
| `mode.*.prompt` | `string` | `Option<String>` | None | - |
| `mode.*.tools` | **deprecated**, `object` | Uses `permission` | Migrated | - |
| `mode.*.disable` | `boolean` | `Option<bool>` | None | - |
| `mode.*.description` | `string` | `Option<String>` | None | - |
| `mode.*.mode` | `enum: subagent\|primary\|all` | **MISSING** | Missing | P2 |
| `mode.*.hidden` | `boolean` | `Option<bool>` | None | - |
| `mode.*.options` | `object` | `Option<HashMap>` | None | - |
| `mode.*.color` | `hex\|theme` | `Option<String>` | Partially | P2 |
| `mode.*.steps` | `integer` | `Option<u32>` | None | - |
| `mode.*.maxSteps` | **deprecated** | Uses `steps` | Migrated | - |
| `mode.*.permission` | `PermissionConfig` | `Option<PermissionConfig>` | Same | - |
| `permission` | See PermissionConfig | `Option<PermissionConfig>` | Same | - |
| **MISSING** | - | `provider` object config | Extra | P1 |
| **MISSING** | - | `mcp` servers | Extra | P2 |
| **MISSING** | - | `formatter` config | Extra | P2 |
| **MISSING** | - | `lsp` config | Extra | P2 |
| **MISSING** | - | `instructions` | Extra | P3 |
| **MISSING** | - | `agents_md` config | Extra | P3 |
| **MISSING** | - | `enterprise` config | Extra | P2 |
| **MISSING** | - | `compaction` config | Extra | P2 |
| **MISSING** | - | `experimental` config | Extra | P3 |
| **MISSING** | - | `github` config | Extra | P2 |
| **MISSING** | - | `tui` config | Extra | P2 |
| **MISSING** | - | `hidden_models` | Extra | P3 |
| **MISSING** | - | `api_key` (top-level) | Extra deprecated | P3 |
| **MISSING** | - | `temperature` (top-level) | Extra deprecated | P3 |
| **MISSING** | - | `max_tokens` (top-level) | Extra deprecated | P3 |

---

## 5. Major Capability Gaps

### P0: Blocks Core Compatibility

#### 1. LogLevel Case Mismatch

**Problem**: opencode uses uppercase `DEBUG|INFO|WARN|ERROR` but opencode-rs uses lowercase `trace|debug|info|warn|error`

**Current opencode-rs**:
```rust
pub enum LogLevel {
    Trace,  // extra
    Debug,
    Info,
    Warn,
    Error,
}
```

**Official**: `DEBUG`, `INFO`, `WARN`, `ERROR` (no Trace)

**Impact**: Config files with uppercase log levels will fail validation or be ignored.

**Fix**: Accept both cases, or normalize on parse.

---

#### 2. Agent/Mode Structure Difference

**Problem**: Official opencode uses `mode` with `build`/`plan` as primary agents:
```json
"mode": {
  "build": { "model": "..." },
  "plan": { "model": "..." }
}
```

opencode-rs uses `agent` with `agents` map:
```json
"agent": {
  "agents": { "build": { "model": "..." } },
  "defaultAgent": "build"
}
```

**Impact**: Config migration incompatibility, user confusion.

**Fix**: Support both formats during migration, convert `mode.build`/`mode.plan` to `agent.agents.build`/`agent.agents.plan`.

---

### P1: Major Feature Gaps

#### 3. Server AdditionalProperties

**Official**: `server` has `additionalProperties: false` - no extra fields allowed

**opencode-rs**: `ServerConfig` allows extra fields via `flatten` on extra

**Impact**: Could accept config that opencode would reject.

---

#### 4. Plugin Format

**Official**: Plugins can be `string` or `[string, object]` pairs:
```json
"plugin": [
  "/path/to/plugin.wasm",
  ["/path/to/plugin2.wasm", { "option": "value" }]
]
```

**opencode-rs**: Only `Vec<String>` supported.

**Fix**: Extend to support tuple format.

---

#### 5. ShareMode Extended Enum

**Official**: `manual | auto | disabled`

**opencode-rs**: `manual | auto | disabled | read_only | collaborative | controlled`

**Impact**: Extra values are opencode-rs-specific extensions, not in official schema.

**Fix**: Accept official values, ignore extended values (with warning).

---

#### 6. Provider Configuration

**Official**: No explicit provider config - relies on environment variables and model strings.

**opencode-rs**: Full `ProviderConfig` objects with api_key, base_url, timeout, etc.

**Impact**: opencode-rs has richer provider config. This is an extension, not a gap per se, but could cause confusion.

---

### P2: Important But Not Blocking

#### 7. Missing `shell` Field

**Official**: `shell: "/bin/zsh"` sets default shell

**opencode-rs**: Not implemented

**Impact**: Users can't configure default shell in config.

---

#### 8. Missing Agent `mode` Values

**Official**: Agent has `mode: "subagent" | "primary" | "all"`

**opencode-rs**: Not implemented

**Impact**: Agent role distinctions not supported.

---

#### 9. Color Field Format

**Official**: Supports both hex (`#FF5733`) and theme colors (`primary`, `secondary`, etc.)

**opencode-rs**: Only `Option<String>` - no validation

---

#### 10. Extra opencode-rs Fields

These are opencode-rs extensions, not gaps:
- `mcp` - MCP server config
- `formatter` - Code formatter config
- `lsp` - LSP server config
- `enterprise` - Enterprise config
- `compaction` - Session compaction
- `github` - GitHub integration
- `tui` - TUI configuration

---

## 6. Recommended Target Config Model

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    // NEW: Add shell field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,

    // FIXED: Accept both cases, normalize to lowercase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfig>,

    // KEEP: Both mode (deprecated) and agent supported for compatibility
    #[serde(skip_serializing_if = "Option::is_none", alias = "mode")]
    pub agent: Option<AgentMapConfig>,

    // ... rest of fields
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,  // No Trace to match official
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,  // default
        }
    }
}
```

---

## 7. Config Loading and Precedence

Current opencode-rs behavior (correct, should maintain):

1. Default values
2. Global config (`~/.config/opencode-rs/config.json`)
3. Project config (`./.opencode-rs/config.json`)
4. `.opencode-rs/` directory contents (agents, commands, skills)
5. `tui.json` merged
6. Environment variables (`OPENCODE_*`)
7. CLI arguments

**Should add for compatibility**:
- Support `mode` field alongside `agent` during migration
- Normalize `logLevel` (uppercase) to `log_level` (lowercase)

---

## 8. Migration Plan

### Phase 1: Schema Alignment Foundation

1. **LogLevel case normalization** - Accept `DEBUG`, `INFO`, `WARN`, `ERROR` on parse
2. **Add `shell` field** - Simple string field
3. **Agent mode migration** - Convert `mode.build`/`mode.plan` to `agent.agents.build`/`agent.agents.plan`

**Files**: `crates/config/src/lib.rs`

**Tests**:
```rust
#[test]
fn test_loglevel_uppercase_parses() {
    let json = r#"{"logLevel": "DEBUG"}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.log_level, Some(LogLevel::Debug));
}
```

---

### Phase 2: Provider/Model Compatibility

1. Ensure model string format `provider/model` works correctly
2. Support provider API keys via env vars (already done)

---

### Phase 3: Agent/Tool/MCP Compatibility

1. Add `mode` field support (deprecated but parsed)
2. Add agent `mode` values: `subagent`, `primary`, `all`

---

### Phase 4: Plugin Compatibility

1. Extend plugin config to support `[path, config]` tuple format

---

### Phase 5: CLI/TUI Persistence

1. Validate `/connect` flow persists correctly
2. Ensure `opencode --validate` works

---

## 9. Test Strategy

### Unit Tests

```rust
// LogLevel case
#[test]
fn test_loglevel_uppercase() { ... }
#[test]
fn test_loglevel_lowercase() { ... }

// Agent migration
#[test]
fn test_mode_build_migrates_to_agent() { ... }
#[test]
fn test_mode_plan_migrates_to_agent() { ... }

// Share compatibility
#[test]
fn test_share_official_values() { ... }
#[test]
fn test_share_extended_values_ignored() { ... }

// Shell
#[test]
fn test_shell_field_parsed() { ... }
```

### Golden Tests

Parse official opencode config files and verify they work.

---

## 10. Implementation Task Breakdown

```json
{
  "tasks": [
    {
      "id": "CONFIG-GAP-001",
      "title": "Fix LogLevel case sensitivity",
      "priority": "P0",
      "type": "bugfix",
      "description": "Accept both uppercase (DEBUG, INFO, WARN, ERROR) and lowercase (debug, info, warn, error) LogLevel values",
      "files_to_modify": [
        "crates/config/src/lib.rs"
      ],
      "tests": [
        "test_loglevel_uppercase_parses",
        "test_loglevel_lowercase_parses"
      ]
    },
    {
      "id": "CONFIG-GAP-002",
      "title": "Add shell field support",
      "priority": "P2",
      "type": "feature",
      "description": "Add shell field to Config and ServerConfig for default shell selection",
      "files_to_modify": [
        "crates/config/src/lib.rs"
      ],
      "tests": [
        "test_shell_field_parsed"
      ]
    },
    {
      "id": "CONFIG-GAP-003",
      "title": "Support mode-to-agent migration",
      "priority": "P0",
      "type": "migration",
      "description": "Convert legacy mode.build/plan structure to agent.agents structure",
      "files_to_modify": [
        "crates/config/src/lib.rs"
      ],
      "tests": [
        "test_mode_build_migrates_to_agent",
        "test_mode_plan_migrates_to_agent"
      ]
    },
    {
      "id": "CONFIG-GAP-004",
      "title": "Extend plugin tuple support",
      "priority": "P1",
      "type": "feature",
      "description": "Support plugin as [path, config] tuple alongside string",
      "files_to_modify": [
        "crates/config/src/lib.rs"
      ]
    },
    {
      "id": "CONFIG-GAP-005",
      "title": "Add agent mode values",
      "priority": "P2",
      "type": "feature",
      "description": "Add subagent, primary, all to agent mode enum",
      "files_to_modify": [
        "crates/config/src/lib.rs"
      ]
    }
  ]
}
```
