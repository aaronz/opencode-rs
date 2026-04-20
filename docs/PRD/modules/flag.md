# Module: flag

## Overview

The `flag` module in `opencode-core` (`crates/core/src/flag.rs`) manages feature flags, runtime configuration switches, and optional string/number parameters. All flags are read from environment variables.

**Crate**: `opencode-core`  
**Source**: `crates/core/src/flag.rs`  
**Visibility**: `pub(crate)` — internal to `opencode-core`  
**Status**: Fully implemented (462 lines)

---

## Crate Layout

```
crates/core/src/
└── flag.rs
```

---

## Core Types

### `Flag`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Flag {
    pub name: String,
    pub description: String,
    pub default: bool,
    pub value: bool,
}
```

### `FlagManager`

```rust
pub(crate) struct FlagManager {
    flags: HashMap<String, Flag>,           // Boolean feature flags
    string_flags: HashMap<String, Option<String>>,  // String config values
    number_flags: HashMap<String, Option<u64>>,     // Numeric config values
}
```

---

## All Registered Flags

### Boolean Flags (`OPENCODE_*`)

| Env Var | Default | Description |
|---|---|---|
| `OPENCODE_EXPERIMENTAL` | false | Master experimental gate |
| `OPENCODE_DEBUG` | false | Debug mode |
| `OPENCODE_AUTO_SHARE` | false | Auto-share sessions |
| `OPENCODE_DISABLE_AUTOUPDATE` | false | Disable auto-update |
| `OPENCODE_ALWAYS_NOTIFY_UPDATE` | false | Always notify about updates |
| `OPENCODE_DISABLE_PRUNE` | false | Disable pruning |
| `OPENCODE_DISABLE_TERMINAL_TITLE` | false | Disable terminal title |
| `OPENCODE_DISABLE_DEFAULT_PLUGINS` | false | Disable default plugins |
| `OPENCODE_DISABLE_LSP_DOWNLOAD` | false | Disable LSP server download |
| `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` | false | Enable experimental models |
| `OPENCODE_DISABLE_AUTOCOMPACT` | false | Disable auto-compaction |
| `OPENCODE_DISABLE_MODELS_FETCH` | false | Disable model catalog fetch |
| `OPENCODE_DISABLE_CLAUDE_CODE` | false | Disable Claude Code features |
| `OPENCODE_ENABLE_QUESTION_TOOL` | false | Enable question tool |
| `OPENCODE_EXPERIMENTAL_FILEWATCHER` | false | Enable file watcher |
| `OPENCODE_EXPERIMENTAL_DISABLE_FILEWATCHER` | false | Disable file watcher |
| `OPENCODE_EXPERIMENTAL_ICON_DISCOVERY` | false | Enable icon discovery |
| `OPENCODE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT` | `cfg!(windows)` | Disable copy-on-select |
| `OPENCODE_ENABLE_EXA` | false | Enable Exa web search |
| `OPENCODE_EXPERIMENTAL_OXFMT` | false | Enable oxfmt formatter |
| `OPENCODE_EXPERIMENTAL_LSP_TY` | false | Enable LSP ty |
| `OPENCODE_EXPERIMENTAL_LSP_TOOL` | false | Enable LSP tool |
| `OPENCODE_DISABLE_FILETIME_CHECK` | false | Disable file time check |
| `OPENCODE_EXPERIMENTAL_PLAN_MODE` | false | Enable plan mode |
| `OPENCODE_EXPERIMENTAL_WORKSPACES` | false | Enable workspaces |
| `OPENCODE_EXPERIMENTAL_MARKDOWN` | **true** | Enable markdown rendering |
| `OPENCODE_EXPERIMENTAL_VARIANT_REASONING` | false | Enable variant/reasoning budget |
| `OPENCODE_DISABLE_CHANNEL_DB` | false | Disable channel DB |
| `OPENCODE_SKIP_MIGRATIONS` | false | Skip DB migrations |
| `OPENCODE_STRICT_CONFIG_DEPS` | false | Strict config dependency checking |

### String Flags

| Env Var | Purpose |
|---|---|
| `OPENCODE_GIT_BASH_PATH` | Override git-bash executable path |
| `OPENCODE_CONFIG` | Path to config file override |
| `OPENCODE_CONFIG_CONTENT` | Inline config JSON/TOML |
| `OPENCODE_PERMISSION` | Permission scope override |
| `OPENCODE_FAKE_VCS` | Fake VCS backend for testing |
| `OPENCODE_CLIENT` | Client type identifier (default: "cli") |
| `OPENCODE_SERVER_PASSWORD` | Server auth password |
| `OPENCODE_SERVER_USERNAME` | Server auth username |
| `OPENCODE_MODELS_URL` | Override models catalog URL |
| `OPENCODE_MODELS_PATH` | Local models catalog path |
| `OPENCODE_DB` | Override SQLite DB path |

### Number Flags

| Env Var | Purpose |
|---|---|
| `OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS` | Bash tool timeout (default: 120000) |
| `OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX` | Max output tokens |

---

## Key Implementations

```rust
impl FlagManager {
    pub fn new() -> Self { /* registers all flags above */ }

    /// Get a boolean flag value.
    pub fn get(&self, name: &str) -> Option<bool> {
        self.flags.get(name).map(|f| f.value)
    }

    /// Get a string flag value.
    pub fn get_string(&self, name: &str) -> Option<String> {
        self.string_flags.get(name).and_then(|v| v.clone())
    }

    /// Get a number flag value.
    pub fn get_number(&self, name: &str) -> Option<u64> {
        self.number_flags.get(name).and_then(|v| *v)
    }

    /// Programmatically set a boolean flag (for testing).
    pub fn set(&mut self, name: &str, value: bool) { ... }

    /// Returns flag value or false if not registered.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.get(name).unwrap_or(false)
    }

    /// Load all flag values from environment variables.
    /// Call once at startup.
    pub fn load_from_env(&mut self) {
        // Boolean flags: "1" or "true" (case-insensitive)
        // String flags: raw value
        // Number flags: parsed u64, must be > 0
    }
}
```

### Convenience Methods

These mirror the TypeScript API for common flags:

```rust
impl FlagManager {
    pub fn opencode_auto_share(&self) -> bool { ... }
    pub fn opencode_client(&self) -> String { ... }         // default: "cli"
    pub fn opencode_enable_question_tool(&self) -> bool { ... }
    pub fn opencode_experimental(&self) -> bool { ... }

    /// Exa enabled if OPENCODE_ENABLE_EXA OR OPENCODE_EXPERIMENTAL OR OPENCODE_EXPERIMENTAL_EXA
    pub fn opencode_enable_exa(&self) -> bool { ... }

    /// Plan mode if OPENCODE_EXPERIMENTAL OR OPENCODE_EXPERIMENTAL_PLAN_MODE
    pub fn opencode_experimental_plan_mode(&self) -> bool { ... }

    /// LSP tool if OPENCODE_EXPERIMENTAL OR OPENCODE_EXPERIMENTAL_LSP_TOOL
    pub fn opencode_experimental_lsp_tool(&self) -> bool { ... }

    /// Variant reasoning if OPENCODE_EXPERIMENTAL OR OPENCODE_EXPERIMENTAL_VARIANT_REASONING
    pub fn opencode_experimental_variant_reasoning(&self) -> bool { ... }

    /// Bash timeout: OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS or default 120000ms
    pub fn opencode_experimental_bash_timeout_ms(&self) -> Option<u64> { ... }
}
```

**Truthy check** (private helper):
```rust
fn truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}
```

---

## Usage Pattern

```rust
use opencode_core::flag::FlagManager;

let mut flags = FlagManager::new();
flags.load_from_env();  // Call once at startup

// Check features
if flags.opencode_experimental() {
    println!("Running in experimental mode");
}

if flags.is_enabled("OPENCODE_DISABLE_AUTOCOMPACT") {
    // skip compaction
}

// Get string config
let db_path = flags.get_string("OPENCODE_DB");
let client_type = flags.opencode_client(); // "cli", "tui", "desktop"

// Get bash timeout
let timeout = flags.opencode_experimental_bash_timeout_ms().unwrap_or(120_000);
```

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_has_all_boolean_flags() {
        let fm = FlagManager::new();
        assert!(fm.get("OPENCODE_EXPERIMENTAL").is_some());
        assert!(fm.get("OPENCODE_DEBUG").is_some());
    }

    #[test]
    fn default_values_are_false_except_markdown() {
        let fm = FlagManager::new();
        assert!(!fm.is_enabled("OPENCODE_EXPERIMENTAL"));
        assert!(fm.is_enabled("OPENCODE_EXPERIMENTAL_MARKDOWN")); // default true
    }

    #[test]
    fn set_overrides_value() {
        let mut fm = FlagManager::new();
        assert!(!fm.is_enabled("OPENCODE_DEBUG"));
        fm.set("OPENCODE_DEBUG", true);
        assert!(fm.is_enabled("OPENCODE_DEBUG"));
    }

    #[test]
    fn unknown_flag_returns_false() {
        let fm = FlagManager::new();
        assert!(!fm.is_enabled("NONEXISTENT_FLAG"));
    }

    #[test]
    fn opencode_client_defaults_to_cli() {
        let fm = FlagManager::new();
        assert_eq!(fm.opencode_client(), "cli");
    }

    #[test]
    fn bash_timeout_has_default() {
        let fm = FlagManager::new();
        // Default is 120000 ms
        assert_eq!(fm.opencode_experimental_bash_timeout_ms(), Some(120_000));
    }

    #[test]
    fn exa_enabled_when_experimental_is_true() {
        let mut fm = FlagManager::new();
        fm.set("OPENCODE_EXPERIMENTAL", true);
        assert!(fm.opencode_enable_exa());
    }

    #[test]
    fn plan_mode_enabled_when_experimental_is_true() {
        let mut fm = FlagManager::new();
        fm.set("OPENCODE_EXPERIMENTAL", true);
        assert!(fm.opencode_experimental_plan_mode());
    }

    #[test]
    fn string_flag_returns_none_when_not_set() {
        let fm = FlagManager::new();
        assert!(fm.get_string("OPENCODE_CONFIG").is_none());
    }

    #[test]
    fn number_flag_returns_none_when_not_set() {
        let fm = FlagManager::new();
        // Without env var, number flags are None
        assert!(fm.get_number("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX").is_none());
    }
}
```
