# Task List: Module `global` (Iteration 37)

**Crate**: `opencode-core`
**Source**: `crates/core/src/global.rs`
**Status**: Active Development
**Last Updated**: 2026-04-21

---

## Task Summary

| Priority | Task | Status | Estimated Effort |
|----------|------|--------|-----------------|
| P0 | Fix GlobalState visibility in lib.rs | ✅ Done | 2 min |
| P0 | Add comprehensive test module to global.rs | ✅ Done | 15 min |
| P1 | Add subscriber_count() method | ✅ Done | 5 min |
| P2 | Add documentation comments | ⬜ Pending | 10 min |

---

## P0 Tasks (Blocking)

### P0.1: Fix GlobalState visibility in lib.rs
**File**: `opencode-rust/crates/core/src/lib.rs`
**Line**: 178
**Current Code**:
```rust
pub(crate) use global::GlobalState;
```
**Change To**:
```rust
pub use global::GlobalState;
```
**Acceptance Criteria**:
- [ ] External crates can import `use opencode_core::GlobalState;`
- [ ] No other code changes required

**Verification**:
```bash
cargo build -p opencode-core
```

---

### P0.2: Add comprehensive test module to global.rs
**File**: `opencode-rust/crates/core/src/global.rs`
**Add after line 20**:

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
        // Bus should have 0 subscribers initially
        assert_eq!(state.event_bus.subscriber_count(), 0);
    }

    #[test]
    fn config_is_accessible() {
        let mut config = Config::default();
        let state = GlobalState::new(config);
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
        // Both point to same bus
        let mut rx = bus_clone.subscribe();
        state.event_bus.publish(crate::bus::InternalEvent::ConfigUpdated);
        assert!(rx.try_recv().is_ok());
    }
}
```

**Acceptance Criteria**:
- [ ] All 5 tests pass individually
- [ ] All tests run with `cargo test -p opencode-core`
- [ ] Tests cover all FR-374 requirements

**Verification**:
```bash
cargo test -p opencode-core -- --nocapture
```

---

## P1 Tasks (High Priority)

### P1.1: Add subscriber_count() method
**File**: `opencode-rust/crates/core/src/global.rs`
**Add after line 19**:

```rust
/// Returns the number of subscribers currently registered with the event bus.
pub fn subscriber_count(&self) -> usize {
    self.event_bus.subscriber_count()
}
```

**Acceptance Criteria**:
- [ ] Method returns correct subscriber count
- [ ] Method is callable from tests
- [ ] Follows existing code style

**Verification**:
```bash
cargo build -p opencode-core
```

---

## P2 Tasks (Medium Priority)

### P2.1: Add documentation comments
**File**: `opencode-rust/crates/core/src/global.rs`

Add module-level doc (before line 1):
```rust
//! `global` - Global state container for CLI/TUI runtime.
//!
//! Provides the top-level `GlobalState` struct that owns `Config`, `EventBus`,
//! and optional active `Session`. Used as central DI container for single-session
//! CLI/TUI runtime paths.
//!
//! # Example
//! ```
//! use opencode_core::{Config, GlobalState};
//!
//! let config = Config::load().unwrap_or_default();
//! let global = GlobalState::new(config);
//! ```
```

Add struct doc (before line 6):
```rust
/// Global state container for CLI/TUI runtime.
/// Owns Config, EventBus, and optional active Session.
pub struct GlobalState {
```

**Acceptance Criteria**:
- [ ] `cargo doc --no-deps -p opencode-core` generates documentation
- [ ] Module docs appear in generated docs

---

## Execution Order

1. P0.1: Fix visibility (lib.rs:178)
2. P1.1: Add subscriber_count() method
3. P0.2: Add test module (includes P1.1 verification)
4. P2.1: Add documentation (optional for iteration 37)

---

## Verification Commands

```bash
# Build check
cargo build -p opencode-core

# Run tests
cargo test -p opencode-core

# Clippy check
cargo clippy -p opencode-core -- -D warnings

# Documentation check
cargo doc --no-deps -p opencode-core
```

---

## Notes

- **Dependencies verified**:
  - `Session::new()` exists at session.rs:137
  - `EventBus::subscriber_count()` exists at bus.rs:171
  - `EventBus::subscribe()` exists
  - `EventBus::publish()` exists
  - `InternalEvent::ConfigUpdated` exists

- **Test isolation**: Each test is independent and can run in parallel
- **No breaking changes**: This is additive only (new method, new tests, new visibility)

---

*Task List Version: 37*
*Priority: P0 tasks must complete before iteration 38*