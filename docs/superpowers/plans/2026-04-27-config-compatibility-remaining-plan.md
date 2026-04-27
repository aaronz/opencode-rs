# Config Gap Implementation Plan - Remaining Gaps

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix remaining P1/P2 config compatibility gaps with official opencode

**Architecture:** These are smaller targeted fixes - ShareMode enum handling, Server additionalProperties, and Agent mode values.

---

## Task 1: Handle ShareMode Extended Enum

**Gap**: opencode-rs has `manual|auto|disabled|read_only|collaborative|controlled` but official only has `manual|auto|disabled`

**Files:**
- Modify: `opencode-rust/crates/config/src/lib.rs`
- Modify: `opencode-rust/crates/config/src/builtin_config.schema.json`

**Approach**: Accept official values silently, warn when extended values are used (but still parse them).

- [ ] **Step 1: Write test for ShareMode official values**

```rust
#[test]
fn test_share_official_values_parse() {
    for json in &["manual", "auto", "disabled"] {
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.share.is_some());
    }
}
```

- [ ] **Step 2: Write test for extended values (warn but parse)**

```rust
#[test]
fn test_share_extended_values_warn_but_parse() {
    // Extended values should be accepted but logged
    let config: Config = serde_json::from_str(r#""read_only""#).unwrap();
    assert_eq!(config.share, Some(ShareMode::ReadOnly));
}
```

---

## Task 2: Server additionalProperties

**Gap**: Official `server` has `additionalProperties: false` but opencode-rs allows extra fields

**Files:**
- Modify: `opencode-rust/crates/config/src/lib.rs`
- Modify: `opencode-rust/crates/config/src/builtin_config.schema.json`

**Approach**: Two options:
1. Add `#[serde(flatten)] extra: Option<HashMap<String, serde_json::Value>>` to ServerConfig and log warning on unknown fields
2. Or document this as intentional divergence

- [ ] **Step 1: Investigate current ServerConfig handling**

Check if ServerConfig currently rejects unknown fields or allows them silently.

- [ ] **Step 2: If needed, add warning for unknown server fields**

Add logging when extra fields are present in server config.

---

## Task 3: Add Agent Mode Values

**Gap**: Official agents have `mode: "subagent" | "primary" | "all"` but opencode-rs doesn't implement this

**Files:**
- Modify: `opencode-rust/crates/config/src/lib.rs`
- Modify: `opencode-rust/crates/config/src/builtin_config.schema.json`

**Approach**: Add `AgentMode` enum and `mode` field to AgentConfig.

- [ ] **Step 1: Write test for agent mode deserialization**

```rust
#[test]
fn test_agent_mode_subagent() {
    let json = r#"{
        "agent": {
            "agents": {
                "helper": {
                    "model": "openai/gpt-4o",
                    "mode": "subagent"
                }
            }
        }
    }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    // mode field should deserialize
}
```

- [ ] **Step 2: Add AgentMode enum**

```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum AgentMode {
    Subagent,
    Primary,
    All,
}
```

- [ ] **Step 3: Add mode field to AgentConfig**

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub mode: Option<AgentMode>,
```

---

## Summary

| Task | Priority | Files | Notes |
|------|----------|-------|-------|
| ShareMode extended | P1 | lib.rs, schema.json | Warn on extended values |
| Server additionalProperties | P1 | lib.rs, schema.json | Investigate first |
| Agent mode values | P2 | lib.rs, schema.json | Add enum + field |

**Plan complete and saved to `docs/superpowers/plans/2026-04-27-config-compatibility-remaining-plan.md`**
