# Implementation Plan: `util` Module (Iteration 41)

**Crate**: `opencode-util` (`opencode-rust/crates/util/`)
**Status**: 🔴 NOT IMPLEMENTED
**Last Updated**: 2026-04-21

---

## 0. Overview

This plan outlines the implementation of the `opencode-util` crate, providing structured logging, unified error types, retry utilities, async filesystem helpers, and async helpers.

**Implementation Strategy**: Bottom-up, P0-first

| Phase | Focus | Priority |
|-------|-------|----------|
| Phase 1 | Create crate structure + dependencies | P0 |
| Phase 2 | Core types: NamedError, LogLevel, Logger | P0 |
| Phase 3 | Error utilities: From impls, WithContext, Context trait | P1 |
| Phase 4 | Retry: RetryConfig + retry() | P1 |
| Phase 5 | FS helpers: atomic_write, read_json, write_json, ensure_dir | P1 |
| Phase 6 | Helpers: Lazy, iife!, with_timeout, wait_for, retry_until | P2 |
| Phase 7 | Tests (25 tests) | P1 |

---

## 1. Phase 1: Crate Structure (P0)

### Task 1.1: Create Directory Structure

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

### Task 1.2: Update Workspace Cargo.toml

**Actions**:
1. Add `"crates/util"` to workspace `members` array
2. Add `opencode-util = { path = "crates/util" }` to workspace.dependencies
3. Add `tracing-appender = "0.2"` to workspace.dependencies
4. Update `tracing-subscriber` to include `features = ["json", "env-filter"]`

**File**: `opencode-rust/Cargo.toml`

### Task 1.3: Create Cargo.toml

**File**: `opencode-rust/crates/util/Cargo.toml`

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

### Task 1.4: Create lib.rs with Re-exports

**File**: `opencode-rust/crates/util/src/lib.rs`

```rust
pub use logging::{Logger, LogLevel, Rotation, log_file_path};
pub use error::{NamedError, WithContext, Context};
pub use retry::{RetryConfig, retry};
pub use fs::{atomic_write, read_json, write_json, ensure_dir, read_to_string, write as fs_write};
pub use helpers::{Lazy, with_timeout, wait_for, retry_until};
pub use crate::iife;
```

---

## 2. Phase 2: Core Types (P0)

### Task 2.1: Implement LogLevel Enum (FR-422)

**File**: `opencode-rust/crates/util/src/logging.rs` (new file)

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

### Task 2.2: Implement Rotation Struct (FR-423)

**File**: `opencode-rust/crates/util/src/logging.rs`

```rust
pub struct Rotation {
    max_size_bytes: usize,
    max_files: usize,
}

impl Rotation {
    pub fn new(max_size_mb: usize, max_files: usize) -> Self {
        Self {
            max_size_bytes: max_size_mb * 1024 * 1024,
            max_files,
        }
    }
}
```

### Task 2.3: Implement Logger Struct (FR-424-428)

**File**: `opencode-rust/crates/util/src/logging.rs`

```rust
use std::path::PathBuf;

pub struct Logger {
    level: LogLevel,
    file_path: Option<PathBuf>,
    console: bool,
}

impl Logger {
    pub fn init(&self) -> Result<(), SetLoggerError> { /* ... */ }
    pub fn with_level(level: LogLevel) -> Self { /* ... */ }
    pub fn with_file(&mut self, path: impl Into<PathBuf>) -> &mut Self { /* ... */ }
    pub fn with_no_console(&mut self) -> &mut Self { /* ... */ }
}
```

### Task 2.4: Implement log_file_path() (FR-429)

**File**: `opencode-rust/crates/util/src/logging.rs`

```rust
pub fn log_file_path() -> PathBuf {
    opencode_core::Global::path_opencode().join("logs").join("opencode.log")
}
```

**Note**: Depends on `opencode-core` being available.

### Task 2.5: Implement NamedError Struct (FR-431)

**File**: `opencode-rust/crates/util/src/error.rs` (new file)

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
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self { /* ... */ }
    pub fn with_code(mut self, code: impl Into<String>) -> Self { /* ... */ }
    pub fn with_data(mut self, data: serde_json::Value) -> Self { /* ... */ }
    pub fn kind(&self) -> &str { &self.name }
}

impl std::fmt::Display for NamedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}
```

---

## 3. Phase 3: Error Utilities (P1)

### Task 3.1: Implement From<std::io::Error> (FR-432)

**File**: `opencode-rust/crates/util/src/error.rs`

```rust
impl From<std::io::Error> for NamedError {
    fn from(e: std::io::Error) -> Self {
        NamedError::new("IOError", e.to_string())
            .with_code(format!("IO_{}", e.kind() as i32))
    }
}
```

### Task 3.2: Implement From<reqwest::Error> (FR-433)

**File**: `opencode-rust/crates/util/src/error.rs`

```rust
impl From<reqwest::Error> for NamedError {
    fn from(e: reqwest::Error) -> Self {
        NamedError::new("HttpError", e.to_string())
            .with_code("HTTP")
    }
}
```

### Task 3.3: Implement WithContext<E> (FR-434)

**File**: `opencode-rust/crates/util/src/error.rs`

```rust
pub struct WithContext<E> {
    context: String,
    inner: E,
}

impl<E: std::error::Error> std::fmt::Display for WithContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} (caused by: {})",
            self.context,
            self.inner,
            self.inner.source().map(|s| s.to_string()).unwrap_or_default()
        )
    }
}

impl<E: std::error::Error> std::error::Error for WithContext<E> {}
```

### Task 3.4: Implement Context Trait (FR-435)

**File**: `opencode-rust/crates/util/src/error.rs`

```rust
pub trait Context<T, E: std::error::Error> {
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>>;
}

impl<T, E: std::error::Error> Context<T, E> for Result<T, E> {
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>> {
        self.map_err(|e| WithContext {
            context: ctx.into(),
            inner: e,
        })
    }
}
```

---

## 4. Phase 4: Retry Module (P1)

### Task 4.1: Implement RetryConfig (FR-437)

**File**: `opencode-rust/crates/util/src/retry.rs` (new file)

```rust
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl RetryConfig {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self { /* ... */ }
    pub fn with_max_delay(mut self, max: Duration) -> Self { /* ... */ }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new(3, Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10))
    }
}
```

### Task 4.2: Implement retry() Function (FR-438)

**File**: `opencode-rust/crates/util/src/retry.rs`

```rust
pub async fn retry<R, E, F, Fut>(
    config: RetryConfig,
    operation: F,
) -> Result<R, (E, u32)>
where
    F: Fn(u32) -> Fut,
    Fut: Future<Output = Result<R, E>>,
    E: std::fmt::Debug,
{ /* ... */ }
```

---

## 5. Phase 5: Filesystem Module (P1)

### Task 5.1: Implement read_to_string() (FR-441)

**File**: `opencode-rust/crates/util/src/fs.rs` (new file)

```rust
pub async fn read_to_string(path: &Path) -> Result<String, std::io::Error> { /* ... */ }
```

### Task 5.2: Implement write() (FR-442)

**File**: `opencode-rust/crates/util/src/fs.rs`

```rust
pub async fn write(path: &Path, contents: &str) -> Result<(), std::io::Error> { /* ... */ }
```

### Task 5.3: Implement atomic_write() (FR-443)

**File**: `opencode-rust/crates/util/src/fs.rs`

```rust
pub async fn atomic_write(path: &Path, contents: &str) -> Result<(), std::io::Error> {
    // 1. Create temp file in same directory
    // 2. Write content
    // 3. Rename to target (atomic)
}
```

### Task 5.4: Implement ensure_dir() (FR-444)

**File**: `opencode-rust/crates/util/src/fs.rs`

```rust
pub async fn ensure_dir(path: &Path) -> Result<(), std::io::Error> { /* ... */ }
```

### Task 5.5: Implement read_json() (FR-445)

**File**: `opencode-rust/crates/util/src/fs.rs`

```rust
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, NamedError> { /* ... */ }
```

### Task 5.6: Implement write_json() (FR-446)

**File**: `opencode-rust/crates/util/src/fs.rs`

```rust
pub async fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), NamedError> { /* ... */ }
```

---

## 6. Phase 6: Helpers Module (P2)

### Task 6.1: Implement Lazy<T> (FR-448)

**File**: `opencode-rust/crates/util/src/helpers.rs` (new file)

```rust
use std::sync::OnceCell;

pub struct Lazy<T> {
    cell: OnceCell<T>,
    init: fn() -> T,
}

impl<T: Send + Sync> Lazy<T> {
    pub const fn new(init: fn() -> T) -> Self { /* ... */ }
    pub fn get(&self) -> &T { /* ... */ }
}
```

### Task 6.2: Implement iife! Macro (FR-449)

**File**: `opencode-rust/crates/util/src/helpers.rs`

```rust
#[macro_export]
macro_rules! iife {
    (|| $expr:expr) => { $expr };
    (||$($tokens:tt)*) => { ($( $tokens )*) };
}
```

### Task 6.3: Implement with_timeout() (FR-450)

**File**: `opencode-rust/crates/util/src/helpers.rs`

```rust
pub struct TimeoutError;

pub async fn with_timeout<T>(
    duration: Duration,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> { /* ... */ }
```

### Task 6.4: Implement wait_for() (FR-451)

**File**: `opencode-rust/crates/util/src/helpers.rs`

```rust
pub async fn wait_for<F, T>(
    condition: F,
    timeout: Duration,
) -> Result<T, TimeoutError>
where
    F: Fn() -> Option<T>,
{ /* ... */ }
```

### Task 6.5: Implement retry_until() (FR-452)

**File**: `opencode-rust/crates/util/src/helpers.rs`

```rust
pub async fn retry_until<F, T, E>(
    config: RetryConfig,
    condition: F,
) -> Result<T, (E, u32)>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Option<T>, E>>,
    E: std::fmt::Debug,
{ /* ... */ }
```

---

## 7. Phase 7: Tests (P1)

### Task 7.1: Error Tests (FR-461.1-461.8)

| Test ID | Test Name | Module |
|---------|-----------|--------|
| FR-461.1 | `test_named_error_display` | error |
| FR-461.2 | `test_named_error_with_code` | error |
| FR-461.3 | `test_named_error_with_data` | error |
| FR-461.4 | `test_named_error_serde` | error |
| FR-461.5 | `test_named_error_from_io_error` | error |
| FR-461.6 | `test_named_error_from_reqwest_error` | error |
| FR-461.7 | `test_with_context_display` | error |
| FR-461.8 | `test_context_trait_extension` | error |

### Task 7.2: Retry Tests (FR-461.9-461.13)

| Test ID | Test Name | Module |
|---------|-----------|--------|
| FR-461.9 | `test_retry_config_default` | retry |
| FR-461.10 | `test_retry_succeeds_on_first_attempt` | retry |
| FR-461.11 | `test_retry_retries_on_failure` | retry |
| FR-461.12 | `test_retry_returns_err_after_max_attempts` | retry |
| FR-461.13 | `test_retry_with_jitter` | retry |

### Task 7.3: FS Tests (FR-461.14-461.17)

| Test ID | Test Name | Module |
|---------|-----------|--------|
| FR-461.14 | `test_atomic_write` | fs |
| FR-461.15 | `test_read_json` | fs |
| FR-461.16 | `test_write_json` | fs |
| FR-461.17 | `test_ensure_dir` | fs |

### Task 7.4: Helper Tests (FR-461.18-461.24)

| Test ID | Test Name | Module |
|---------|-----------|--------|
| FR-461.18 | `test_lazy_thread_safe` | helpers |
| FR-461.19 | `test_lazy_initializes_once` | helpers |
| FR-461.20 | `test_iife_macro` | helpers |
| FR-461.21 | `test_with_timeout_fires` | helpers |
| FR-461.22 | `test_with_timeout_success` | helpers |
| FR-461.23 | `test_wait_for_success` | helpers |
| FR-461.24 | `test_wait_for_timeout` | helpers |

### Task 7.5: Logging Tests (FR-461.25)

| Test ID | Test Name | Module |
|---------|-----------|--------|
| FR-461.25 | `test_log_level_serde` | logging |

---

## 8. Dependency Graph

```
Phase 1: Crate Structure
├── Task 1.1: Create directories
├── Task 1.2: Update workspace Cargo.toml
├── Task 1.3: Create Cargo.toml
└── Task 1.4: Create lib.rs re-exports

Phase 2: Core Types (depends on Phase 1)
├── Task 2.1: LogLevel enum
├── Task 2.2: Rotation struct
├── Task 2.3: Logger struct + methods
├── Task 2.4: log_file_path()
└── Task 2.5: NamedError struct

Phase 3: Error Utilities (depends on Phase 2)
├── Task 3.1: From<std::io::Error>
├── Task 3.2: From<reqwest::Error>
├── Task 3.3: WithContext<E>
└── Task 3.4: Context trait

Phase 4: Retry Module (depends on Phase 1)
├── Task 4.1: RetryConfig
└── Task 4.2: retry() function

Phase 5: FS Module (depends on Phase 1, 2)
├── Task 5.1: read_to_string()
├── Task 5.2: write()
├── Task 5.3: atomic_write()
├── Task 5.4: ensure_dir()
├── Task 5.5: read_json()
└── Task 5.6: write_json()

Phase 6: Helpers (depends on Phase 1)
├── Task 6.1: Lazy<T>
├── Task 6.2: iife! macro
├── Task 6.3: with_timeout()
├── Task 6.4: wait_for()
└── Task 6.5: retry_until()

Phase 7: Tests (depends on all phases)
├── Task 7.1: Error tests (8 tests)
├── Task 7.2: Retry tests (5 tests)
├── Task 7.3: FS tests (4 tests)
├── Task 7.4: Helper tests (7 tests)
└── Task 7.5: Logging tests (1 test)
```

---

## 9. Implementation Notes

### 9.1 Circular Dependency Avoidance

- `opencode-util` depends on `opencode-core` only for `Global::path_opencode()` in `log_file_path()`
- If this creates circular dependency issues, `log_file_path()` can be moved to `opencode-core` or the path can be configurable

### 9.2 Async Runtime

- All async functions use `tokio` runtime
- FS operations use `tokio::fs`
- Timeout uses `tokio::time::timeout`

### 9.3 Feature Flags

No feature flags planned for initial implementation. All functionality is enabled by default.

---

## 10. Verification

After implementation, verify:

1. [ ] `cargo build -p opencode-util` succeeds
2. [ ] `cargo test -p opencode-util` passes all 25 tests
3. [ ] `cargo clippy -p opencode-util -- -D warnings` passes
4. [ ] `cargo doc -p opencode-util` generates documentation

---

*Plan generated based on spec_v41.md (FR-422 to FR-461)*
