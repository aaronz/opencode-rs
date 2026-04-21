# Gap Analysis Report: `util` Module

**Module**: `util` (`crates/util/` - proposed structure)
**Status**: ❌ **NOT IMPLEMENTED** - Module does not exist
**Analysis Date**: 2026-04-21

---

## 0. Executive Summary

The `util` module as specified in the PRD **does not exist** as an independent crate. The functionality described in the PRD is **distributed across multiple locations**:

| PRD Requirement | Current Location | Status |
|---|---|---|
| Logging (tracing-based) | `crates/core/src/observability.rs` | ⚠️ Partial |
| Error Types (NamedError) | `crates/core/src/error.rs` + `crates/llm/src/error.rs` | ⚠️ Partial |
| Retry Utilities | `crates/llm/src/error.rs` | ⚠️ Partial |
| Filesystem Helpers | `crates/core/src/filesystem.rs` | ⚠️ Partial |
| Helpers (Lazy, IIFE) | `crates/core/src/util.rs` | ⚠️ Partial |
| `crates/util/` crate | **Missing** | ❌ |

---

## 1. Gap List

| Gap Item | Severity | Module | 修复建议 |
|---|---|---|---|
| `crates/util/` crate does not exist | **P0** | Workspace | Create `crates/util/` with submodules: `logging.rs`, `error.rs`, `retry.rs`, `fs.rs`, `helpers.rs` |
| `NamedError` struct as specified in PRD | **P0** | error.rs | Create unified `NamedError` struct with `name`, `code`, `message`, `data` fields per PRD spec |
| `From<std::io::Error>` for NamedError | **P0** | error.rs | Implement `From<std::io::Error>` returning `NamedError::new("IOError", ...)` with `IO_*` code |
| `From<reqwest::Error>` for NamedError | **P1** | error.rs | Implement `From<reqwest::Error>` returning `NamedError::new("HttpError", ...)` with `HTTP` code |
| `WithContext<E>` error wrapper | **P1** | error.rs | Create `WithContext<E>` struct with `context()` method for error chaining |
| `RetryConfig` with jitter support | **P1** | retry.rs | Implement `RetryConfig` with `base_delay`, `max_delay`, `jitter` fields per PRD |
| `retry()` function with exponential backoff + jitter | **P1** | retry.rs | Implement `retry<R, E, F, Fut>()` function per PRD signature |
| `Rotation` struct for log file rotation | **P1** | logging.rs | Implement `Rotation::new(max_size_mb, max_files)` for log rotation |
| `log_file_path()` function | **P2** | logging.rs | Implement `Global::path_opencode().join("logs").join("opencode.log")` |
| `atomic_write()` async function | **P1** | fs.rs | Implement async `atomic_write()` - write to temp file then rename |
| Async `read_json()` / `write_json()` | **P1** | fs.rs | Implement async versions using `tokio::fs` |
| `with_timeout()` async function | **P2** | helpers.rs | Implement `with_timeout<T>()` using `tokio::time::timeout` |
| `Lazy<T>` thread-safe once cell | **P2** | helpers.rs | Implement `Lazy<T>` using `OnceCell` with `get_or_init()` |
| `iife!` macro | **P2** | helpers.rs | Implement `#[macro_export] iife!` macro per PRD |
| `wait_for` / `retry_until` helpers | **P2** | helpers.rs | Implement if needed per PRD specification |
| Logging `Logger` struct with `init()`, `with_level()`, `with_file()`, `with_no_console()` | **P0** | logging.rs | Create `Logger` struct wrapping `tracing-subscriber` |
| `LogLevel` enum (Debug, Info, Warn, Error) | **P0** | logging.rs | Create `LogLevel` enum with `Serialize`, `Deserialize` derives |
| Dependency on `tracing-appender` for file rotation | **P1** | logging.rs | Add `tracing-appender = "0.2"` dependency |
| Dependency on `tracing-subscriber` with `json` feature | **P1** | logging.rs | Update workspace deps to include `features = ["json", "env-filter"]` |

---

## 2. P0/P1/P2 Classification

### P0 - Blocking Issues

| Issue | Description | Impact |
|---|---|---|
| **No `crates/util/` crate** | The entire module structure as specified does not exist. No centralized utility crate for logging, errors, retry, fs, and helpers. | All PRD features are inaccessible via `opencode-util` crate |
| **Missing `NamedError` struct** | PRD specifies a unified `NamedError` struct with `name`, `code`, `message`, `data` fields. Current implementations use `thiserror` enums scattered across crates. | Error handling is inconsistent; cannot serialize/deserialize errors with full context |
| **Missing `Logger` struct** | No centralized logging setup with `init()`, `with_level()`, `with_file()`, `with_no_console()` as specified | Logging initialization is ad-hoc in `observability.rs` |
| **Missing `LogLevel` enum** | `LogLevel` enum (Debug, Info, Warn, Error) with serde derives not implemented | Cannot configure logging levels consistently |

### P1 - High Priority Issues

| Issue | Description | Impact |
|---|---|---|
| **Incomplete retry implementation** | `crates/llm/src/error.rs` has `with_retry()` but lacks `RetryConfig` with `jitter` support per PRD | Retry logic exists but doesn't match PRD API (uses `backoff_multiplier` not exponential backoff with jitter) |
| **Missing `atomic_write()`** | PRD specifies async atomic write (write to temp then rename). Current `filesystem.rs` has sync `write_with_dirs()` only | Cannot safely write files atomically |
| **Missing async `read_json()`/`write_json()`** | Current `AppFileSystem::read_json()`/`write_json()` are sync. PRD specifies async versions | Async file operations unavailable |
| **Missing `From<reqwest::Error>` for NamedError** | `From<std::io::Error>` implemented for `OpenCodeError` but not for `NamedError` | HTTP errors cannot be conveniently converted to NamedError |
| **Missing `WithContext<E>` error wrapper** | PRD specifies `WithContext<E>` with `.context()` method for error chaining | Cannot add context to errors elegantly |

### P2 - Low Priority / Technical Debt

| Issue | Description | Impact |
|---|---|---|
| **Missing `Lazy<T>` implementation** | PRD specifies thread-safe `Lazy<T>` using `OnceCell`. Current `util.rs` has simple `Util` struct only | Cannot use lazy static patterns |
| **Missing `iife!` macro** | PRD specifies `#[macro_export] iife!` macro | IIFE pattern unavailable |
| **Missing `with_timeout()`** | PRD specifies `with_timeout<T>()` using `tokio::time::timeout` | Timeout wrapper not available |
| **Missing `Rotation` struct** | Log file rotation struct not implemented | Log files will grow unbounded |
| **Missing `log_file_path()`** | `Global::path_opencode().join("logs").join("opencode.log")` function not exposed | Cannot determine log file location |
| **Missing `wait_for`/`retry_until`** | Async helpers for polling not implemented | Custom polling logic required |

---

## 3. Technical Debt清单

| Debt Item | Location | 描述 | 建议 |
|---|---|---|---|
| **分散的错误处理** | `crates/core/src/error.rs`, `crates/llm/src/error.rs` | `OpenCodeError` and `LlmError` are separate `thiserror` enums instead of unified `NamedError` struct | Create `NamedError` as specified, deprecate legacy enums |
| **重复的日志初始化** | `crates/core/src/observability.rs`, `crates/cli/src/main.rs` | `setup_tracing()` exists but `Logger` struct not implemented | Consolidate into `Logger` struct with fluent API |
| **同步文件系统操作** | `crates/core/src/filesystem.rs` | `AppFileSystem` methods are sync, PRD requires async | Add async wrappers using `tokio::fs` |
| **缺少 `tracing-appender` 依赖** | `Cargo.toml` | Log rotation requires `tracing-appender` | Add dependency to workspace |
| **Retry 实现不一致** | `crates/llm/src/error.rs` | Current retry uses `backoff_multiplier`, PRD specifies jitter-based exponential backoff | Align with PRD API |

---

## 4. 实现进度总结

### 功能完整性: ❌ 0%

| PRD Feature | Status | Current Location |
|---|---|---|
| Structured Logging | ⚠️ Partial | `observability.rs::setup_tracing()` |
| `Logger` struct | ❌ Missing | N/A |
| `LogLevel` enum | ❌ Missing | N/A |
| Log file rotation | ❌ Missing | N/A |
| `log_file_path()` | ❌ Missing | N/A |
| `NamedError` struct | ❌ Missing | `OpenCodeError` (different design) |
| `From<std::io::Error>` | ⚠️ Partial | `OpenCodeError::Io` via `#[from]` |
| `From<reqwest::Error>` | ❌ Missing | Not implemented for `NamedError` |
| `WithContext<E>` | ❌ Missing | N/A |
| `RetryConfig` | ⚠️ Partial | `LlmError::RetryConfig` (different API) |
| `retry()` function | ⚠️ Partial | `llm::error::with_retry()` (different signature) |
| `read_json()` / `write_json()` | ⚠️ Partial | `AppFileSystem::read_json/write_json` (sync only) |
| `atomic_write()` | ❌ Missing | N/A |
| `ensure_dir()` | ⚠️ Partial | `AppFileSystem::ensure_dir()` (sync only) |
| `Lazy<T>` | ❌ Missing | N/A |
| `iife!` macro | ❌ Missing | N/A |
| `with_timeout()` | ❌ Missing | N/A |
| `wait_for` / `retry_until` | ❌ Missing | N/A |

### 接口完整性: ⚠️ ~15%

- `OpenCodeError` enum exists in `crates/core/src/error.rs` with many variants
- `LlmError` enum exists in `crates/llm/src/error.rs`
- `AppFileSystem` provides some fs operations (sync only)
- No unified `NamedError` struct per PRD specification

### 数据模型: ⚠️ ~10%

| PRD Data Model | Status | Notes |
|---|---|---|
| `NamedError { name, code, message, data }` | ❌ | Not implemented as struct |
| `LogLevel { Debug, Info, Warn, Error }` | ❌ | Not implemented as enum |
| `Logger { level, file_path, console }` | ❌ | Not implemented |
| `Rotation { max_size_bytes, max_files }` | ❌ | Not implemented |
| `RetryConfig { max_attempts, base_delay, max_delay, jitter }` | ⚠️ | `LlmError::RetryConfig` exists but different fields |
| `WithContext<E> { context, inner }` | ❌ | Not implemented |
| `Lazy<T> { cell, init }` | ❌ | Not implemented |

### 配置管理: N/A

No configuration options specified in PRD for this module (configuration is internal to Logger).

### 测试覆盖: ⚠️ ~20%

| Test Category | Existing | Missing |
|---|---|---|
| Error tests | `OpenCodeError` has tests | `NamedError` tests missing |
| Retry tests | `with_retry` tests exist | `retry()` function tests missing |
| Filesystem tests | `AppFileSystem` has tests | Async `read_json`/`write_json`/`atomic_write` tests missing |
| Logging tests | `ObservabilityTracker` has tests | `Logger` struct tests missing |

---

## 5. Recommended Implementation Order

### Phase 1: Create `crates/util/` crate structure

```
crates/util/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Re-exports
│   ├── logging.rs       # Logger, LogLevel, Rotation
│   ├── error.rs         # NamedError, WithContext
│   ├── retry.rs         # RetryConfig, retry()
│   ├── fs.rs            # atomic_write, async read_json/write_json
│   └── helpers.rs       # Lazy, iife!, with_timeout
└── tests/
    └── util_tests.rs
```

### Phase 2: Implement core types

1. `NamedError` struct with all required fields and `From` implementations
2. `LogLevel` enum with serde derives
3. `Logger` struct with fluent API
4. `WithContext<E>` wrapper

### Phase 3: Implement retry and fs utilities

1. `RetryConfig` with jitter
2. `retry()` function
3. `atomic_write()`, `read_json()`, `write_json()`

### Phase 4: Implement helpers

1. `Lazy<T>`
2. `iife!` macro
3. `with_timeout()`
4. `wait_for`, `retry_until` if needed

---

## 6. Dependencies Required

Add to workspace `Cargo.toml`:

```toml
tracing-appender = "0.2"
```

Update `tracing-subscriber` feature:

```toml
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```
