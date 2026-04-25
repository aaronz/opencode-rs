# PRD: Error Code Catalog

## Overview

This document provides a unified catalog of all error codes used in OpenCode RS. Error codes follow a structured numbering scheme for easy identification and routing.

## Error Code Ranges

| Range | Category | Description |
|-------|----------|-------------|
| 1xxx | Authentication | Token expired, invalid, missing credentials |
| 2xxx | Authorization | Insufficient permissions, access denied |
| 3xxx | Provider | Provider not found, auth failed, unavailable |
| 4xxx | Tool | Tool not found, timeout, invalid arguments |
| 5xxx | Session | Session not found, expired, corrupted |
| 6xxx | Config | Config missing, invalid, load failed |
| 7xxx | Validation | Invalid parameter, missing field, format mismatch |
| 9xxx | Internal | Internal error, service unavailable, database error |

---

## 1xxx: Authentication Errors

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 1001 | `TokenExpired` | Authentication token has expired | Refresh token or re-authenticate |
| 1002 | `InvalidToken` | Authentication token is invalid or malformed | Check token format, obtain new token |
| 1003 | `MissingCredentials` | No credentials provided for authentication | Provide valid credentials |

**Reference**: `crates/core/src/error.rs`

---

## 2xxx: Authorization Errors

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 2001 | `InsufficientPermissions` | User lacks required permissions | Request elevated permissions |
| 2002 | `AccessDenied` | Access to resource explicitly denied | Contact administrator |

**Reference**: `crates/core/src/error.rs`

---

## 3xxx: Provider Errors

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 3001 | `ProviderNotFound` | Requested LLM provider not configured | Configure provider or use default |
| 3002 | `ProviderAuthFailed` | Provider authentication failed | Check API key, update credentials |
| 3003 | `ProviderUnavailable` | Provider service is currently unavailable | Wait and retry, or switch provider |

**Reference**: `crates/core/src/error.rs`

---

## 4xxx: Tool Errors

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 4001 | `ToolNotFound` | Requested tool does not exist | Check tool name, list available tools |
| 4002 | `ToolTimeout` | Tool execution exceeded timeout | Increase timeout or simplify operation |
| 4003 | `ToolInvalidArgs` | Tool arguments are invalid | Check tool schema, provide valid args |

**Reference**: `crates/core/src/error.rs`

---

## 5xxx: Session Errors

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 5001 | `SessionNotFound` | Session ID does not exist | Verify session ID, list sessions |
| 5002 | `SessionExpired` | Session has expired | Resume from checkpoint or start new |
| 5003 | `SessionCorrupted` | Session data is corrupted | Restore from backup if available |
| 5004 | `VersionMismatchError` | Session version incompatible | Update OpenCode or migrate session |

**Reference**: `crates/core/src/error.rs`

---

## 6xxx: Configuration Errors

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 6001 | `ConfigMissing` | Required configuration key missing | Add configuration to config file |
| 6002 | `ConfigInvalid` | Configuration value is invalid | Fix configuration value |
| 6003 | `ConfigLoadFailed` | Failed to load configuration | Check file permissions, path, syntax |

**Reference**: `crates/core/src/error.rs`

---

## 7xxx: Validation Errors

### General Validation (7000-7099)

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 7001 | `ValidationError` | Generic validation failure | Check input format and constraints |
| 7002 | `MissingRequiredField` | Required field not provided | Provide all required fields |
| 7003 | `FormatMismatch` | Value format does not match expected | Use correct format |

### Workspace Validation (7100-7199)

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 7101 | `WorkspacePathNotFound` | Workspace path does not exist | Create directory or use valid path |
| 7102 | `WorkspacePathNotAccessible` | Workspace path is not accessible | Check permissions |
| 7103 | `WorkspacePathNotDirectory` | Path exists but is not a directory | Use directory path |
| 7104 | `WorkspacePathNotReadable` | Workspace is not readable | Check read permissions |

**Reference**: `crates/core/src/error.rs`

---

## 9xxx: Internal Errors

| Code | Error Type | Description | Resolution |
|------|------------|-------------|------------|
| 9001 | `InternalError` | Unexpected internal error | Report bug, check logs |
| 9002 | `ServiceUnavailable` | Service is temporarily unavailable | Wait and retry |
| 9003 | `StorageError` | Database or storage operation failed | Check storage availability |
| 9004 | `DatabaseError` | Database operation failed | Check database connection |

**Reference**: `crates/core/src/error.rs`, `crates/storage/src/error.rs`

---

## Server-Specific Error Codes

| Code | Error Type | Description | Reference |
|------|------------|-------------|-----------|
| S001 | `ParseError` | Failed to parse request | `crates/tui/src/server_protocol.rs` |
| S002 | `InvalidRequest` | Request format is invalid | `crates/tui/src/server_protocol.rs` |
| S003 | `SessionNotFound` | Session not found on server | `crates/tui/src/server_protocol.rs` |
| S004 | `SessionLoadError` | Failed to load session | `crates/tui/src/server_protocol.rs` |
| S005 | `InvalidReconnectToken` | Reconnect token is invalid | `crates/tui/src/server_protocol.rs` |
| S006 | `UnsupportedBinary` | Binary type not supported | `crates/tui/src/server_protocol.rs` |
| S007 | `SerializationError` | Serialization/deserialization failed | `crates/tui/src/server_protocol.rs` |
| S008 | `ConnectionTimeout` | Connection timed out | `crates/tui/src/server_protocol.rs` |
| S009 | `HeartbeatTimeout` | Heartbeat timeout | `crates/tui/src/server_protocol.rs` |
| S010 | `MaxReconnectAttempts` | Max reconnection attempts reached | `crates/tui/src/server_protocol.rs` |

---

## Legacy Error Mappings

For backward compatibility, legacy error types are mapped to new codes:

| Legacy Type | Mapped To | Notes |
|-------------|-----------|-------|
| `Network` | 3xxx | Provider-related network errors |
| `Parse` | 7xxx | Validation/parse errors |
| `Config` | 6xxx | Configuration errors |
| `Session` | 5xxx | Session errors |
| `Tool` | 4xxx | Tool errors |
| `Llm` | 3xxx | Provider/LLM errors |
| `Tui` | 9xxx | Internal/TUI errors |
| `Storage` | 9xxx | Storage errors |

---

## Error Handling Guidelines

### For Developers

1. **Use structured errors**: Prefer structured error variants over string-based errors
2. **Include context**: Provide detail fields for debugging
3. **Follow code ranges**: Add new errors to appropriate range
4. **Document resolutions**: Always suggest resolution steps

### For AI Coding

1. **Check error codes first**: Extract code to identify category
2. **Use resolution column**: Follow suggested resolutions
3. **Log full error**: Include detail fields in logs
4. **Report consistently**: Use same error codes for same issues

---

## Cross-References

| Topic | Reference |
|-------|-----------|
| Core errors | `crates/core/src/error.rs` |
| Storage errors | `crates/storage/src/error.rs` |
| Server errors | `crates/tui/src/server_protocol.rs` |
| LSP errors | `crates/lsp/src/error.rs` |
| Logging | `crates/logging/src/event.rs` |

---

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2026-04-26 | Initial creation | AI |
