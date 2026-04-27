# Config Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix P0/P1 config compatibility gaps between opencode-rs and official opencode

**Architecture:** Add case-insensitive LogLevel parsing, migrate deprecated `mode` field to `agent`, extend plugin support for tuples, add `shell` field, and normalize ShareMode. Changes are additive migrations with deprecation warnings.

**Tech Stack:** Rust, serde, thiserror

---

## File Structure

- **Modify:** `opencode-rust/crates/config/src/lib.rs` - Add LogLevel case handling, shell field, mode migration, plugin tuple enum
- **Modify:** `opencode-rust/crates/config/src/builtin_config.schema.json` - Add shell field, update LogLevel enum
- **Modify:** `opencode-rust/crates/config/src/schema.rs` - Update KNOWN_CONFIG_FIELDS
- **Test:** `opencode-rust/crates/config/src/lib.rs` - Add tests in existing test module

---

## Task 1: Fix LogLevel Case Sensitivity

**Files:**
- Modify: `opencode-rust/crates/config/src/lib.rs:157-175`
- Modify: `opencode-rust/crates/config/src/builtin_config.schema.json`
- Test: `opencode-rust/crates/config/src/lib.rs` (existing test module)

- [ ] **Step 1: Write failing test for uppercase LogLevel**

Add to test module in lib.rs:

```rust
#[test]
fn test_loglevel_uppercase_parses() {
    let json = r#"{"logLevel": "DEBUG"}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.log_level, Some(LogLevel::Debug));
}

#[test]
fn test_loglevel_uppercase_warn_parses() {
    let json = r#"{"logLevel": "WARN"}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.log_level, Some(LogLevel::Warn));
}

#[test]
fn test_loglevel_uppercase_error_parses() {
    let json = r#"{"logLevel": "ERROR"}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.log_level, Some(LogLevel::Error));
}

#[test]
fn test_loglevel_lowercase_still_works() {
    let json = r#"{"logLevel": "debug"}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.log_level, Some(LogLevel::Debug));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- test_loglevel_uppercase
```

Expected: 3 failing tests (uppercase WARN, ERROR not matching)

- [ ] **Step 3: Implement case-insensitive LogLevel deserialization**

Replace the LogLevel enum and add a custom deserializer. The key is implementing `Deserialize` that handles case-insensitive matching:

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LogLevelVisitor;
        impl<'de> Visitor<'de> for LogLevelVisitor {
            type Value = LogLevel;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("trace, debug, info, warn, error (case insensitive)")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value.to_lowercase().as_str() {
                    "trace" => Ok(LogLevel::Trace),
                    "debug" => Ok(LogLevel::Debug),
                    "info" => Ok(LogLevel::Info),
                    "warn" | "warning" => Ok(LogLevel::Warn),
                    "error" => Ok(LogLevel::Error),
                    s if s.eq_ignore_ascii_case("DEBUG") => Ok(LogLevel::Debug),
                    s if s.eq_ignore_ascii_case("INFO") => Ok(LogLevel::Info),
                    s if s.eq_ignore_ascii_case("WARN") | s.eq_ignore_ascii_case("WARNING") => Ok(LogLevel::Warn),
                    s if s.eq_ignore_ascii_case("ERROR") => Ok(LogLevel::Error),
                    _ => Err(serde::de::Error::custom("invalid log level")),
                }
            }
        }
        deserializer.deserialize_str(LogLevelVisitor)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- test_loglevel
```

Expected: 4 tests passing

- [ ] **Step 5: Update builtin_config.schema.json**

Update the LogLevel enum to accept uppercase:

```json
"log_level": {
  "type": "string",
  "enum": ["trace", "debug", "info", "warn", "error", "DEBUG", "INFO", "WARN", "ERROR"],
  "description": "Log level (case insensitive)"
},
"logLevel": {
  "type": "string",
  "enum": ["trace", "debug", "info", "warn", "error", "DEBUG", "INFO", "WARN", "ERROR"],
  "description": "Log level (legacy camelCase, case insensitive)"
}
```

- [ ] **Step 6: Build and run all config tests**

```bash
cd opencode-rust && cargo test -p opencode-config --lib
```

- [ ] **Step 7: Commit**

```bash
git add opencode-rust/crates/config/src/lib.rs opencode-rust/crates/config/src/builtin_config.schema.json
git commit -m "fix(config): make LogLevel case-insensitive to match official opencode"
```

---

## Task 2: Add shell Field

**Files:**
- Modify: `opencode-rust/crates/config/src/lib.rs:44-50`
- Modify: `opencode-rust/crates/config/src/builtin_config.schema.json`

- [ ] **Step 1: Write failing test for shell field**

Add to test module:

```rust
#[test]
fn test_shell_field_parses() {
    let json = r#"{"shell": "/bin/zsh"}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.shell, Some("/bin/zsh".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- test_shell_field
```

Expected: FAIL - shell field doesn't exist

- [ ] **Step 3: Add shell field to Config struct**

Add after line 49 (after log_level field):

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub shell: Option<String>,
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- test_shell_field
```

Expected: PASS

- [ ] **Step 5: Update builtin_config.schema.json**

Add to properties:

```json
"shell": {
  "type": "string",
  "description": "Default shell to use for terminal and bash tool"
}
```

- [ ] **Step 6: Update KNOWN_CONFIG_FIELDS in schema.rs**

Add `"shell"` to the KNOWN_CONFIG_FIELDS array.

- [ ] **Step 7: Build and run all config tests**

```bash
cd opencode-rust && cargo test -p opencode-config --lib
```

- [ ] **Step 8: Commit**

```bash
git add opencode-rust/crates/config/src/lib.rs opencode-rust/crates/config/src/builtin_config.schema.json opencode-rust/crates/config/src/schema.rs
git commit -m "feat(config): add shell field for default shell configuration"
```

---

## Task 3: Support mode-to-agent Migration

**Files:**
- Modify: `opencode-rust/crates/config/src/lib.rs`
- Modify: `opencode-rust/crates/config/src/builtin_config.schema.json`

**Note:** This requires understanding the official opencode `mode` structure vs opencode-rs `agent` structure. The official uses `mode.build` and `mode.plan` as primary agents. We need to parse this and convert it to `agent.agents`.

- [ ] **Step 1: Write failing test for mode migration**

Add to test module:

```rust
#[test]
fn test_mode_build_migrates_to_agent() {
    let json = r#"{
        "mode": {
            "build": {
                "model": "openai/gpt-4o",
                "temperature": 0.2
            }
        }
    }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    // The mode field is deprecated but should be parsed
    // Check that migration path exists
    assert!(config.agent.is_some() || json.contains("mode"));
}

#[test]
fn test_mode_plan_migrates_to_agent() {
    let json = r#"{
        "mode": {
            "plan": {
                "model": "anthropic/claude-3",
                "description": "Planning agent"
            }
        }
    }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert!(json.contains("mode") || json.contains("plan"));
}
```

Actually, since `mode` is NOT currently parsed at all, we need to first implement the parsing. Let me reconsider this approach.

**Simplified approach:** For now, add `mode` as an alias that gets silently ignored with a deprecation warning. This allows config files to load without error. True migration can happen in a future phase.

- [ ] **Step 1: Write test that mode is ignored (with warning)**

Add to test module:

```rust
#[test]
fn test_mode_field_parsed_without_error() {
    // The mode field should be parsed but produce a deprecation warning
    // For now, just ensure it doesn't cause a parse error
    let json = r#"{
        "mode": {
            "build": {
                "model": "openai/gpt-4o"
            }
        }
    }"#;
    let result: Result<Config, _> = serde_json::from_str(json);
    // Should not fail - mode is deprecated but supported for compatibility
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("mode"));
}
```

Actually this approach is flawed - serde will fail if we don't define mode at all. Let me implement a proper approach:

- [ ] **Step 1: Define ModeConfig struct for deprecated migration**

Add near AgentMapConfig:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<AgentConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<AgentConfig>,
}
```

- [ ] **Step 2: Add mode field to Config with skip_serializing**

Add after `agent` field:

```rust
#[serde(skip_serializing, alias = "mode")]
pub mode: Option<ModeConfig>,
```

Note: `skip_serializing` means it won't be saved back, `alias` allows deserializing from `mode` field.

- [ ] **Step 3: Write test for mode deserialization**

```rust
#[test]
fn test_mode_build_deserializes() {
    let json = r#"{
        "mode": {
            "build": {
                "model": "openai/gpt-4o",
                "temperature": 0.2
            }
        }
    }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    // mode.build should be accessible
    assert!(config.mode.is_some());
}
```

- [ ] **Step 4: Run tests**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- test_mode
```

- [ ] **Step 5: Add migration logic from mode to agent**

Add a method on Config:

```rust
impl Config {
    pub fn migrate_mode_to_agent(&mut self) {
        if let Some(mode) = &self.mode {
            let agent_config = self.agent.get_or_insert_with(AgentMapConfig::default);
            if let Some(build) = &mode.build {
                if !agent_config.agents.contains_key("build") {
                    agent_config.agents.insert("build".to_string(), build.clone());
                }
            }
            if let Some(plan) = &mode.plan {
                if !agent_config.agents.contains_key("plan") {
                    agent_config.agents.insert("plan".to_string(), plan.clone());
                }
            }
        }
    }
}
```

Then call this during config loading after parsing.

- [ ] **Step 6: Update builtin_config.schema.json to include mode**

Add to properties:

```json
"mode": {
  "type": "object",
  "description": "@deprecated Use 'agent' field instead",
  "properties": {
    "build": { "$ref": "#/definitions/AgentConfig" },
    "plan": { "$ref": "#/definitions/AgentConfig" }
  },
  "additionalProperties": {}
}
```

- [ ] **Step 7: Build and run tests**

```bash
cd opencode-rust && cargo test -p opencode-config --lib
```

- [ ] **Step 8: Commit**

```bash
git add opencode-rust/crates/config/src/lib.rs opencode-rust/crates/config/src/builtin_config.schema.json
git commit -m "feat(config): add deprecated mode field for opencode compatibility"
```

---

## Task 4: Extend Plugin Tuple Support

**Files:**
- Modify: `opencode-rust/crates/config/src/lib.rs`

**Note:** Official opencode supports plugins as both strings and `[path, config]` tuples.

- [ ] **Step 1: Define PluginConfig enum for tuple/string support**

Add near other config types:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PluginConfig {
    Simple(String),
    WithConfig {
        path: String,
        #[serde(default)]
        config: serde_json::Value,
    },
}
```

- [ ] **Step 2: Update plugin field type in Config**

Change from:
```rust
pub plugin: Option<Vec<String>>,
```

To:
```rust
pub plugin: Option<Vec<PluginConfig>>,
```

- [ ] **Step 3: Write test for plugin tuple format**

```rust
#[test]
fn test_plugin_tuple_parses() {
    let json = r#"{
        "plugin": [
            "/path/to/plugin.wasm",
            ["/path/to/plugin2.wasm", {"option": "value"}]
        ]
    }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.plugin.as_ref().unwrap().len(), 2);
    match &config.plugin.as_ref().unwrap()[1] {
        PluginConfig::WithConfig { path, config: _ } => {
            assert_eq!(path, "/path/to/plugin2.wasm");
        }
        _ => panic!("Expected tuple format"),
    }
}
```

- [ ] **Step 4: Run test to verify it fails**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- test_plugin_tuple
```

Expected: FAIL

- [ ] **Step 5: Implement PluginConfig enum**

(Already done in Step 1)

- [ ] **Step 6: Run test to verify it passes**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- test_plugin_tuple
```

Expected: PASS

- [ ] **Step 7: Update builtin_config.schema.json**

```json
"plugin": {
  "type": "array",
  "items": {
    "anyOf": [
      { "type": "string" },
      {
        "type": "array",
        "prefixItems": [
          { "type": "string" },
          { "type": "object", "additionalProperties": {} }
        ]
      }
    ]
  }
}
```

- [ ] **Step 8: Build and run all config tests**

```bash
cd opencode-rust && cargo test -p opencode-config --lib
```

- [ ] **Step 9: Commit**

```bash
git add opencode-rust/crates/config/src/lib.rs opencode-rust/crates/config/src/builtin_config.schema.json
git commit -m "feat(config): support plugin tuple format [path, config] for opencode compatibility"
```

---

## Task 5: Run Full Verification

- [ ] **Step 1: Run clippy on config crate**

```bash
cd opencode-rust && cargo clippy -p opencode-config -- -D warnings
```

- [ ] **Step 2: Run full test suite**

```bash
cd opencode-rust && cargo test -p opencode-config --lib -- --nocapture
```

- [ ] **Step 3: Build entire project**

```bash
cd opencode-rust && cargo build --all-features
```

---

## Summary

| Task | Changes | Priority |
|------|---------|----------|
| 1 | LogLevel case-insensitive | P0 |
| 2 | shell field | P2 |
| 3 | mode-to-agent migration | P0 |
| 4 | Plugin tuple format | P1 |
| 5 | Full verification | - |

**Plan complete and saved to `docs/superpowers/plans/2026-04-27-config-compatibility-plan.md`**

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
