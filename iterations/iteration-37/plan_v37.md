# Implementation Plan: Module `global` (Iteration 37)

**Crate**: `opencode-core`
**Source**: `crates/core/src/global.rs`
**Status**: Active Development
**Last Updated**: 2026-04-21

---

## 1. Overview

The `global` module provides the top-level `GlobalState` struct that owns the `Config`, `EventBus`, and active `Session`. It serves as the central dependency injection container for CLI/TUI runtime paths.

---

## 2. Implementation Status

| Requirement | ID | Status | Notes |
|-------------|----|--------|-------|
| Core structure | FR-370 | ✅ Implemented | 20 lines, minimal implementation |
| Constructor | FR-371 | ✅ Implemented | `new(config: Config) -> Self` |
| Subscriber count method | FR-372 | ⬜ Missing | Needs to be added |
| Public export | FR-373 | ⬜ Missing | Currently `pub(crate)`, needs `pub` |
| Test module | FR-374 | ⬜ Missing | 6 tests specified, none implemented |
| Doc comments | - | ⬜ Missing | Module and struct need documentation |

---

## 3. P0 Tasks (Blocking - Must Fix)

### P0.1: Fix GlobalState visibility in lib.rs
- **File**: `opencode-rust/crates/core/src/lib.rs:178`
- **Current**: `pub(crate) use global::GlobalState;`
- **Change to**: `pub use global::GlobalState;`
- **Rationale**: External crates cannot use `GlobalState` with `pub(crate)`, violating design intent

### P0.2: Add comprehensive test module to global.rs
- **File**: `opencode-rust/crates/core/src/global.rs`
- **Add**: `#[cfg(test)] mod tests` with 6 test cases as specified in PRD FR-374

---

## 4. P1 Tasks (High Priority)

### P1.1: Add subscriber_count() method
- **File**: `opencode-rust/crates/core/src/global.rs`
- **Method**: `pub fn subscriber_count(&self) -> usize { self.event_bus.subscriber_count() }`
- **Verified**: `EventBus::subscriber_count()` exists at `bus.rs:171`

---

## 5. P2 Tasks (Medium Priority - Optional for Iteration 37)

### P2.1: Add documentation comments
- Module-level doc comment explaining purpose
- Struct-level doc comment for `GlobalState`
- Method-level doc comments for `new()` and `subscriber_count()`

### P2.2: Extension pattern fields (Future)
- Per PRD Section 3, add placeholder fields when needed:
  - `tool_registry: Option<Arc<opencode_tools::ToolRegistry>>`
  - `plugin_manager: Option<Arc<opencode_lsp::LspManager>>`

---

## 6. Implementation Details

### Test Cases (FR-374)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn new_global_state_has_no_session() {
        let state = GlobalState::new(Config::default());
        assert!(state.current_session.is_none());
    }

    #[test]
    fn event_bus_is_initialized() {
        let state = GlobalState::new(Config::default());
        assert_eq!(state.event_bus.subscriber_count(), 0);
    }

    #[test]
    fn config_is_accessible() {
        let config = Config::default();
        let state = GlobalState::new(config.clone());
        let _ = &state.config;
    }

    #[test]
    fn can_set_current_session() {
        let mut state = GlobalState::new(Config::default());
        state.current_session = Some(Session::new());
        assert!(state.current_session.is_some());
    }

    #[test]
    fn event_bus_is_arc_clonable() {
        let state = GlobalState::new(Config::default());
        let bus_clone = Arc::clone(&state.event_bus);
        let mut rx = bus_clone.subscribe();
        state.event_bus.publish(crate::bus::InternalEvent::ConfigUpdated);
        assert!(rx.try_recv().is_ok());
    }
}
```

---

## 7. Verification Checklist

- [ ] `GlobalState` is publicly exported from `opencode-core`
- [ ] `GlobalState::new(config)` creates instance with `current_session = None`
- [ ] `GlobalState::subscriber_count()` returns event bus subscriber count
- [ ] All 6 test cases pass (FR-374.1 through FR-374.5)
- [ ] Module has doc comments explaining purpose and usage
- [ ] `EventBus::subscriber_count()` is accessible via `GlobalState`

---

## 8. Dependencies

- `Session::new()` - verified at session.rs:137
- `EventBus::subscriber_count()` - verified at bus.rs:171
- `EventBus::subscribe()` - used in tests
- `EventBus::publish()` - used in tests
- `InternalEvent::ConfigUpdated` - used in tests

---

## 9. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `GlobalState` API breaking change | Low | Medium | Add tests to verify behavior |
| Test dependency on internal events | Low | Low | Use documented public API |
| Session::new() API change | Low | Low | Tests are isolated |

---

## 10. Next Steps

1. Edit `lib.rs:178` - change `pub(crate)` to `pub`
2. Edit `global.rs` - add `subscriber_count()` method
3. Edit `global.rs` - add `#[cfg(test)] mod tests` with all 6 tests
4. Run `cargo test -p opencode-core` to verify
5. Run `cargo clippy --all -- -D warnings` to ensure code quality

---

*Document Version: 37*
*Next Action: Execute P0 tasks first*