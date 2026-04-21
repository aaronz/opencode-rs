# Task List: `util` Module (Iteration 41)

**Crate**: `opencode-util` (`opencode-rust/crates/util/`)
**Status**: 🔴 NOT IMPLEMENTED
**Last Updated**: 2026-04-21

---

## Priority Legend

| Priority | Description |
|----------|-------------|
| P0 | Blocking - Must implement first, cannot proceed without |
| P1 | High Priority - Important functionality |
| P2 | Low Priority - Technical debt, nice to have |

---

## P0 Tasks (Blocking)

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P0-1** | Create `crates/util/` directory structure | workspace | - | 🔴 Not Started | - |
| **P0-2** | Update workspace `Cargo.toml` to add `util` crate | workspace | FR-454 | 🔴 Not Started | P0-1 |
| **P0-3** | Create `crates/util/Cargo.toml` | workspace | FR-454 | 🔴 Not Started | P0-1 |
| **P0-4** | Create `crates/util/src/lib.rs` with re-exports | lib.rs | - | 🔴 Not Started | P0-3 |
| **P0-5** | Implement `LogLevel` enum | logging.rs | FR-422 | 🔴 Not Started | P0-4 |
| **P0-6** | Implement `Logger` struct with fluent API | logging.rs | FR-423-428 | 🔴 Not Started | P0-5 |
| **P0-7** | Implement `NamedError` struct | error.rs | FR-431 | 🔴 Not Started | P0-4 |

**P0 Subtotal: 7 tasks**

---

## P1 Tasks (High Priority)

### Error Module

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P1-1** | Implement `From<std::io::Error>` for `NamedError` | error.rs | FR-432 | 🔴 Not Started | P0-7 |
| **P1-2** | Implement `From<reqwest::Error>` for `NamedError` | error.rs | FR-433 | 🔴 Not Started | P0-7 |
| **P1-3** | Implement `WithContext<E>` wrapper | error.rs | FR-434 | 🔴 Not Started | P1-1 |
| **P1-4** | Implement `Context` trait | error.rs | FR-435 | 🔴 Not Started | P1-3 |

### Retry Module

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P1-5** | Implement `RetryConfig` struct | retry.rs | FR-437 | 🔴 Not Started | P0-4 |
| **P1-6** | Implement `retry()` function | retry.rs | FR-438 | 🔴 Not Started | P1-5 |

### Filesystem Module

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P1-7** | Implement `read_to_string()` | fs.rs | FR-441 | 🔴 Not Started | P0-4 |
| **P1-8** | Implement `write()` | fs.rs | FR-442 | 🔴 Not Started | P1-7 |
| **P1-9** | Implement `atomic_write()` | fs.rs | FR-443 | 🔴 Not Started | P1-8 |
| **P1-10** | Implement `ensure_dir()` | fs.rs | FR-444 | 🔴 Not Started | P1-7 |
| **P1-11** | Implement `read_json()` | fs.rs | FR-445 | 🔴 Not Started | P0-7, P1-7 |
| **P1-12** | Implement `write_json()` | fs.rs | FR-446 | 🔴 Not Started | P0-7, P1-8 |

### Logging Module

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P1-13** | Implement `Rotation` struct | logging.rs | FR-423 | 🔴 Not Started | P0-5 |
| **P1-14** | Implement `log_file_path()` | logging.rs | FR-429 | 🔴 Not Started | P0-6 |
| **P1-15** | Add `tracing-appender` dependency | workspace | FR-454 | 🔴 Not Started | P0-2 |

### Tests

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P1-16** | Error tests (8 tests) | error.rs | FR-461.1-461.8 | 🔴 Not Started | P1-1, P1-2, P1-3, P1-4 |
| **P1-17** | Retry tests (5 tests) | retry.rs | FR-461.9-461.13 | 🔴 Not Started | P1-5, P1-6 |
| **P1-18** | FS tests (4 tests) | fs.rs | FR-461.14-461.17 | 🔴 Not Started | P1-9, P1-11, P1-12, P1-10 |

**P1 Subtotal: 18 tasks**

---

## P2 Tasks (Technical Debt)

### Logging Module

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P2-1** | Implement log file rotation with `Rotation` | logging.rs | FR-423 | 🔴 Not Started | P1-13 |
| **P2-2** | Add `json` and `env-filter` features to `tracing-subscriber` | workspace | FR-454 | 🔴 Not Started | P0-2 |

### Helpers Module

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P2-3** | Implement `Lazy<T>` struct | helpers.rs | FR-448 | 🔴 Not Started | P0-4 |
| **P2-4** | Implement `iife!` macro | helpers.rs | FR-449 | 🔴 Not Started | P2-3 |
| **P2-5** | Implement `with_timeout()` | helpers.rs | FR-450 | 🔴 Not Started | P2-3 |
| **P2-6** | Implement `wait_for()` | helpers.rs | FR-451 | 🔴 Not Started | P2-5 |
| **P2-7** | Implement `retry_until()` | helpers.rs | FR-452 | 🔴 Not Started | P2-6 |

### Tests

| Task ID | Task | Module | FR | Status | Dependencies |
|---------|------|--------|-----|--------|--------------|
| **P2-8** | Helper tests (7 tests) | helpers.rs | FR-461.18-461.24 | 🔴 Not Started | P2-3, P2-4, P2-5, P2-6 |
| **P2-9** | Logging tests (1 test) | logging.rs | FR-461.25 | 🔴 Not Started | P0-5 |

**P2 Subtotal: 9 tasks**

---

## Summary

| Priority | Count |
|----------|-------|
| P0 | 7 |
| P1 | 18 |
| P2 | 9 |
| **Total** | **34** |

---

## Detailed Task Specifications

### P0-1: Create Directory Structure
```
crates/util/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── logging.rs
│   ├── error.rs
│   ├── retry.rs
│   ├── fs.rs
│   └── helpers.rs
└── tests/
    └── util_tests.rs
```

### P0-2: Update Workspace Cargo.toml

**File**: `opencode-rust/Cargo.toml`

**Changes**:
1. Add `"crates/util"` to `members` array
2. Add `opencode-util = { path = "crates/util" }` to `[workspace.dependencies]`
3. Add `tracing-appender = "0.2"` to `[workspace.dependencies]`
4. Update `tracing-subscriber` to include features: `features = ["json", "env-filter"]`

### P0-3: Create crates/util/Cargo.toml

```toml
[package]
name = "opencode-util"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["fs", "time", "sync"] }
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
anyhow = "1.0"
walkdir = "2"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

### P0-4: Create crates/util/src/lib.rs

```rust
//! opencode-util - General-purpose utilities for OpenCode RS
//!
//! This crate provides:
//! - Structured logging with tracing
//! - Unified error types with NamedError
//! - Retry utilities with exponential backoff and jitter
//! - Async filesystem helpers
//! - Async helpers (timeout, wait_for)

pub use logging::{Logger, LogLevel, Rotation, log_file_path};
pub use error::{NamedError, WithContext, Context};
pub use retry::{RetryConfig, retry};
pub use fs::{atomic_write, read_json, write_json, ensure_dir};
pub use helpers::{Lazy, with_timeout, wait_for, retry_until};
pub use crate::iife;
```

### P0-5: LogLevel Enum (FR-422)

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
```

### P0-6: Logger Struct (FR-424-428)

```rust
use std::path::PathBuf;

pub struct Logger {
    level: LogLevel,
    file_path: Option<PathBuf>,
    console: bool,
}

impl Logger {
    pub fn with_level(level: LogLevel) -> Self { /* ... */ }
    pub fn with_file(&mut self, path: impl Into<PathBuf>) -> &mut Self { /* ... */ }
    pub fn with_no_console(&mut self) -> &mut Self { /* ... */ }
    pub fn init(&self) -> Result<(), SetLoggerError> { /* ... */ }
}
```

### P0-7: NamedError Struct (FR-431)

```rust
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub struct NamedError {
    pub name: String,
    pub code: Option<String>,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl NamedError {
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self
    pub fn with_code(mut self, code: impl Into<String>) -> Self
    pub fn with_data(mut self, data: serde_json::Value) -> Self
    pub fn kind(&self) -> &str
}

impl std::fmt::Display for NamedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}
```

---

## Test Matrix

| Test ID | Test Name | Module | Priority | Status |
|---------|-----------|--------|----------|--------|
| FR-461.1 | `test_named_error_display` | error | P1 | 🔴 |
| FR-461.2 | `test_named_error_with_code` | error | P1 | 🔴 |
| FR-461.3 | `test_named_error_with_data` | error | P1 | 🔴 |
| FR-461.4 | `test_named_error_serde` | error | P1 | 🔴 |
| FR-461.5 | `test_named_error_from_io_error` | error | P1 | 🔴 |
| FR-461.6 | `test_named_error_from_reqwest_error` | error | P1 | 🔴 |
| FR-461.7 | `test_with_context_display` | error | P1 | 🔴 |
| FR-461.8 | `test_context_trait_extension` | error | P1 | 🔴 |
| FR-461.9 | `test_retry_config_default` | retry | P1 | 🔴 |
| FR-461.10 | `test_retry_succeeds_on_first_attempt` | retry | P1 | 🔴 |
| FR-461.11 | `test_retry_retries_on_failure` | retry | P1 | 🔴 |
| FR-461.12 | `test_retry_returns_err_after_max_attempts` | retry | P1 | 🔴 |
| FR-461.13 | `test_retry_with_jitter` | retry | P1 | 🔴 |
| FR-461.14 | `test_atomic_write` | fs | P1 | 🔴 |
| FR-461.15 | `test_read_json` | fs | P1 | 🔴 |
| FR-461.16 | `test_write_json` | fs | P1 | 🔴 |
| FR-461.17 | `test_ensure_dir` | fs | P1 | 🔴 |
| FR-461.18 | `test_lazy_thread_safe` | helpers | P2 | 🔴 |
| FR-461.19 | `test_lazy_initializes_once` | helpers | P2 | 🔴 |
| FR-461.20 | `test_iife_macro` | helpers | P2 | 🔴 |
| FR-461.21 | `test_with_timeout_fires` | helpers | P2 | 🔴 |
| FR-461.22 | `test_with_timeout_success` | helpers | P2 | 🔴 |
| FR-461.23 | `test_wait_for_success` | helpers | P2 | 🔴 |
| FR-461.24 | `test_wait_for_timeout` | helpers | P2 | 🔴 |
| FR-461.25 | `test_log_level_serde` | logging | P2 | 🔴 |

**Total Tests: 25**

---

*Task list generated based on spec_v41.md*
