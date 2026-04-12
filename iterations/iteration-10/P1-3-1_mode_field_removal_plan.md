# Plan: Removal of Deprecated `mode` Field

**Task ID:** P1-3-1
**Version:** 1.0
**Generated:** 2026-04-13
**Status:** Planning Complete
**Removal Target:** v4.0

---

## 1. Overview

This document outlines the plan for removing the deprecated `mode` field from the OpenCode configuration schema. The `mode` field was a simplified permission preset system that has been superseded by the more flexible `permission` field.

**Related Technical Debt:**
- TD-003: Deprecated `mode` field (Medium severity)
- TD-004: Deprecated `tools` field (Medium severity) - shares migration infrastructure
- TD-005: Deprecated `theme` field (Low severity)
- TD-006: Deprecated `keybinds` field (Low severity)

**Coordinate with:**
- P0-9-2: Fixed deprecated AgentMode enum usage
- P0-9-3: Fixed deprecated AgentConfig::mode field in config.rs
- P0-9-6: Fixed deprecated AgentConfig::mode field in command.rs

---

## 2. Current State

### 2.1 Deprecated Fields in Config

The following fields are deprecated and will be removed in v4.0:

| Field | Location | Migration Path | Status |
|-------|----------|----------------|--------|
| `mode` | Top-level config | Use `agent[].permission` | Deprecated, warning emitted |
| `agent[].mode` | Agent config | Use `agent[].permission` | Deprecated, warning emitted |
| `tools` | Top-level config | Use `permission` | Deprecated, warning emitted |
| `theme` | Top-level config | Moved to `tui.json` | Deprecated, warning emitted |
| `keybinds` | Top-level config | Moved to `tui.json` | Deprecated, warning emitted |

### 2.2 Deprecation Warning Implementation

Located in `crates/core/src/config.rs:1296-1331`:

```rust
fn check_deprecated_fields(value: &serde_json::Value) {
    let deprecated_fields = [
        ("mode", "Use 'agent[].permission' instead. Will be removed in v4.0."),
        ("tools", "Use 'permission' field instead. Will be removed in v4.0."),
        ("theme", "Theme configuration has moved to 'tui.json'. Will be removed from opencode.json in v4.0."),
        ("keybinds", "Keybinds configuration has moved to 'tui.json'. Will be removed from opencode.json in v4.0."),
    ];
    // ... warning emission logic
}
```

For agent-level `mode`:
```rust
if agent_obj.contains_key("mode") {
    tracing::warn!(
        "Deprecated config field 'agent.{}.mode' detected: \
        Use 'permission' field instead. Will be removed in v4.0.",
        agent_name
    );
}
```

---

## 3. Migration Path: `mode` to `permission`

### 3.1 Permission System Overview

The current `permission` field provides fine-grained control:

```json
{
  "agent": {
    "build": {
      "permission": {
        "read": "allow",
        "edit": "allow",
        "bash": "allow",
        "glob": "allow",
        "grep": "allow",
        "list": "allow",
        "task": "allow",
        "todowrite": "ask",
        "webfetch": "ask",
        "websearch": "ask"
      }
    }
  }
}
```

### 3.2 Mode Preset Mappings

The deprecated `mode` field values should map to `permission` configurations as follows:

| Deprecated Mode | Equivalent Permission Configuration |
|-----------------|-----------------------------------|
| `"full"` or `"build"` | All tools: `allow` |
| `"read_only"` or `"plan"` | read/grep/glob/list: `allow`, edit/bash/task: `deny` |
| `"restricted"` | read/grep/glob/list: `allow`, edit: `ask`, bash/task: `deny` |
| `"ask"` | All tools: `ask` |

### 3.3 Example Migration

**Before (deprecated):**
```json
{
  "agent": {
    "build": {
      "mode": "read_only"
    }
  }
}
```

**After (v4.0+):**
```json
{
  "agent": {
    "build": {
      "permission": {
        "read": "allow",
        "grep": "allow",
        "glob": "allow",
        "list": "allow",
        "edit": "deny",
        "bash": "deny",
        "task": "deny"
      }
    }
  }
}
```

### 3.4 Built-in Agent Defaults

| Agent | Default Permission Profile |
|-------|---------------------------|
| `build` | Full access: all tools `allow` |
| `plan` | Read-only: read/grep/glob/list `allow`, edit/bash/task `deny` |
| `general` | Full access for subagent |
| `explore` | Read-only: read/grep/glob/list `allow` |

---

## 4. v4.0 Removal Plan

### Phase 1: Current State (v3.x)
- [x] Emit deprecation warnings at load time
- [x] Document migration path (this document)
- [x] P0-9-2, P0-9-3, P0-9-6: Remove code usage of deprecated mode fields
- [x] Provide backwards compatibility

### Phase 2: v4.0 Preparation
- [ ] Add migration tool to convert old config to new format
- [ ] Update documentation with migration guide
- [ ] Add config version field to track schema changes
- [ ] Emit error instead of warning for invalid mode values

### Phase 3: v4.0 Release
- [ ] Remove `mode` field parsing from schema
- [ ] Remove `check_deprecated_fields` for `mode` and `tools`
- [ ] Keep `theme` and `keybinds` warnings until tui.json migration is complete
- [ ] Update JSON schema

### Phase 4: Post v4.0
- [ ] Remove `theme` and `keybinds` deprecated field handling
- [ ] Clean up any remaining migration code

---

## 5. Compatibility Strategy

### 5.1 Backwards Compatibility
- Configs with `mode` field continue to work with warnings
- `mode` values are translated to equivalent `permission` at load time
- No changes required for configs already using `permission`

### 5.2 Forward Compatibility
- v4.0 configs will not accept `mode` field
- Attempting to use `mode` will produce a clear error message
- Migration tooling available for automatic conversion

---

## 6. Testing Strategy

### 6.1 Unit Tests
- [ ] Test deprecation warning emission
- [ ] Test mode-to-permission translation
- [ ] Test invalid mode value error handling

### 6.2 Integration Tests
- [ ] Test config loading with deprecated `mode` field
- [ ] Test config loading with new `permission` field
- [ ] Test built-in agent permission enforcement

### 6.3 Migration Tests
- [ ] Test migration tool for simple configs
- [ ] Test migration tool for complex configs with multiple agents
- [ ] Test migration preserves original intent

---

## 7. Documentation Updates

### 7.1 Migration Guide
Create a migration guide at `https://docs.opencode.ai/config/migration` covering:
- Overview of deprecated fields
- Step-by-step migration for each field
- Examples of before/after configurations
- Troubleshooting common issues

### 7.2 Configuration Schema Docs
- Remove `mode` field from schema reference
- Add examples of `permission` configuration
- Document built-in agent permission profiles

---

## 8. Implementation Notes

### 8.1 Code Locations
- Deprecation warnings: `crates/core/src/config.rs:1296-1331`
- Permission config: `crates/core/src/config.rs:784-852`
- PermissionAction/Rule: `crates/core/src/config.rs:765-780`

### 8.2 Key Files to Modify in v4.0
1. `crates/core/src/config.rs` - Remove mode field handling
2. `crates/core/src/permission.rs` - Keep as-is (already correct)
3. JSON schema files - Update schema definition

### 8.3 Risks and Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Users with existing `mode` configs break | High | Provide migration tooling, long deprecation cycle |
| Complex permission mappings | Medium | Document common patterns, provide examples |
| Third-party tools using old schema | Medium | Provide schema version detection |

---

## 9. Success Criteria

- [x] Removal plan documented (this document)
- [x] Migration path for mode field to permission field established
- [ ] Deprecation warnings emit correctly for all deprecated fields
- [ ] Migration tooling available before v4.0 release
- [ ] Documentation complete
- [ ] All tests pass
- [ ] v4.0 release removes deprecated fields cleanly

---

## 10. Timeline

| Milestone | Target | Status |
|-----------|--------|--------|
| Plan documented | 2026-04-13 | Complete |
| Code usage removed (P0-9-2/3/6) | 2026-04-13 | Complete |
| v4.0 preparation | Before v4.0 | Pending |
| v4.0 release | TBD | Pending |

---

*Plan generated: 2026-04-13*
*Task: P1-3-1*