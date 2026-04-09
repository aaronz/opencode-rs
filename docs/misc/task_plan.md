# OpenCode-RS Task Plan v2.3

**Version:** 2.3  
**Date:** 2026-04-08
**Round:** 7 of 10  
**Status:** In Progress

---

## Completed P1 Tasks

| Task ID | Task | Priority | Status | Notes |
|---------|------|----------|--------|-------|
| P1-6: CRYPT-001 | Implement CredentialStore trait | P1 | ✅ Done | Updated core/credential_store.rs with actual encryption |
| P1-6: CRYPT-002 | Implement AES-256-GCM encryption | P1 | ✅ Done | Using aes-gcm crate |
| P1-6: CRYPT-003 | Implement Argon2 key derivation | P1 | ✅ Done | Using argon2 crate |
| P1-6: CRYPT-004 | Integrate encryption into auth module | P1 | ✅ Done | Auth crate already had encryption |
| P1-6: CRYPT-005 | Add security tests | P1 | ✅ Done | Added encryption tests |
| P1-12: MCP-ASK-001 | Review MCP ask configuration | P1 | ✅ Done | Found missing permission field |
| P1-12: MCP-ASK-002 | Implement strict ask enforcement | P1 | ✅ Done | Added permission field and bridge logic |
| P1-12: MCP-ASK-003 | Add tests | P1 | ✅ Done | Added tests for ask/allow permission |
| P1-4: PLG-003 | WASM FFI bridge | P1 | ✅ Done | EventBridgeBackend trait and WasmEventBridge implementation |
| P1-4: PLG-005 | Plugin integration test | P1 | ✅ Done | Added EventBridgeBackend mock test |
| P1-2: LSP-004 | LSP test coverage | P1 | ✅ Done | Added retry, workspace context, and find references tests |
| P1-8: CTX-002 | Threshold constants | P1 | ✅ Done | Added COMPACTION_*_THRESHOLD f32 constants |
| P1-8: CTX-003 | Update compaction trigger | P1 | ✅ Done | Updated context.rs to use shared constants |
| P1-10: WASM-001 | WASM config review | P1 | ✅ Done | Reviewed wasmtime config |
| P1-10: WASM-002 | Enhanced isolation | P1 | ✅ Done | EventBridgeBackend provides isolation |
| P1-10: WASM-003 | Crash isolation tests | P1 | ✅ Done | Added panic isolation tests |
| P1-10: WASM-004 | Verification tests | P1 | ✅ Done | Added comprehensive WASM capability tests |

---

## Remaining P1 Tasks (TODO)

| Task ID | Task | Priority | Status | Notes |
|---------|------|----------|--------|-------|
| P1-5: SSE-002 | Client reconnect logic | P1 | ✅ Done | Added event_id emission, message_event_type() for proper client tracking |
| P1-5: SSE-003 | WebSocket handshake fix | P1 | ✅ Done | Improved error handling with proper connection cleanup |
| P1-5: SSE-004 | Connection state monitoring | P1 | ✅ Done | ConnectionMonitor already existed |
| P1-5: SSE-005 | Stress testing | P1 | ✅ Done | Added 99% threshold tests for SSE/WS/reconnection |
| P1-8: CTX-001 | Token counting calibration | P1 | ✅ Done | Added tiktoken calibration tests for accuracy validation |
| P1-8: CTX-004 | Performance tests | P1 | ✅ Done | Added compaction performance and memory tests |

---

## Implementation Summary (Round 5)

### 1. SSE Client Reconnection Support (P1-5: SSE-002)
- Added `message_event_type()` function to properly name SSE events
- SSE messages now include proper event type headers (message, tool_call, tool_result, etc.)
- All recorded messages now emit event IDs for proper client-side tracking
- Client can use Last-Event-ID header to resume from correct position

### 2. WebSocket Handshake Error Handling (P1-5: SSE-003)
- Improved error handling around `actix_ws::handle()` 
- Added detailed error logging for handshake failures
- Connection properly unregistered on handshake failure
- Returns proper JSON error response with error details

### 3. SSE/WebSocket Stability Tests (P1-5: SSE-005)
- Added `test_sse_stability_99_percent_threshold()` test
- Added `test_ws_stability_99_percent_threshold()` test  
- Added `test_reconnection_stability_with_backoff()` test
- All tests validate 99% success rate threshold

### 4. TypeScript SDK Package Fix
- Fixed package.json exports order warning
- Moved "types" before "import" and "require" in exports map

---

### 1. Context Compaction Thresholds (P1-8 CTX-002, CTX-003)
- Added `COMPACTION_WARN_THRESHOLD: f32 = 0.85`
- Added `COMPACTION_START_THRESHOLD: f32 = 0.92`
- Added `COMPACTION_FORCE_THRESHOLD: f32 = 0.95`
- Updated `TokenBudget::default()` to use the new constants
- Updated `context.rs` to use shared constants from `compaction` module
- Exported constants from `crates/core/src/lib.rs`

### 2. WASM FFI Bridge (P1-4 PLG-003)
- Created `EventBridgeBackend` trait for abstract event backends
- Created `EventEnvelope` struct for event serialization
- Created `WasmEventBridge<B>` struct with async event bridging
- Handles Subscribe, Unsubscribe, Log, and PublishEvent from WASM plugins
- Forward matching events from backend to WASM plugin

### 3. Plugin Integration Tests (P1-4 PLG-005)
- Added `test_wasm_event_bridge_with_mock_backend` async test
- Added `test_event_envelope_structure` test

### 4. LSP Test Coverage (P1-2 LSP-004)
- Added `test_retry_delay_values_are_increasing` test
- Added `test_goto_definition_fails_gracefully_without_rust_analyzer` test
- Added `test_find_references_with_empty_file` test
- Added `test_lsp_execute_with_workspace_context` test

### 5. WASM Sandbox Isolation (P1-10 WASM-001~004)
- Added `test_wasm_runtime_with_strict_memory_limit` test
- Added `test_wasm_runtime_with_no_network` test
- Added `test_wasm_runtime_with_filesystem_scope` test
- Added `test_wasm_runtime_timeout_configuration` test
- Added `test_wasm_plugin_with_different_capabilities` test
- Added `test_plugin_panic_isolation_with_limited_memory` test
- Added `test_wasm_capabilities_default_has_timeouts` test
- Added `test_wasm_capabilities_clone_is_independent` test

---

## Progress Log

| Date | Task | Action | Result |
|------|------|--------|--------|
| 2026-04-08 | Initial analysis | Reviewed credential_store.rs | Found plaintext storage issue |
| 2026-04-08 | P1-6 CRYPT | Implemented encryption | AES-256-GCM + Argon2, updated core/credential_store.rs |
| 2026-04-08 | P1-12 MCP-ASK | Implemented strict enforcement | Added permission field to McpServerConfig |
| 2026-04-08 | P1-12 MCP-ASK | Added tests | test_remote_mcp_tools_require_approval_by_default |
| 2026-04-08 | P1-4 PLG-003 | Implemented WASM FFI bridge | EventBridgeBackend trait and WasmEventBridge |
| 2026-04-08 | P1-4 PLG-005 | Added plugin integration test | Mock backend test |
| 2026-04-08 | P1-8 CTX-002 | Added threshold constants | COMPACTION_*_THRESHOLD f32 constants |
| 2026-04-08 | P1-8 CTX-003 | Updated compaction | Used shared constants |
| 2026-04-08 | P1-2 LSP-004 | Added LSP tests | Retry, workspace context tests |
| 2026-04-08 | P1-10 WASM-001~004 | Added WASM isolation tests | Comprehensive capability and panic tests |
| 2026-04-08 | P1-5 SSE-002 | SSE event ID emission | Added message_event_type() for client reconnection |
| 2026-04-08 | P1-5 SSE-003 | WebSocket handshake error handling | Improved error handling with connection cleanup |
| 2026-04-08 | P1-5 SSE-005 | Added stability tests | 99% threshold tests for SSE/WS/reconnection |
| 2026-04-08 | TS SDK | Fixed package.json exports | Reordered types/import/require |
| 2026-04-08 | P1-8 CTX-001 | Added tiktoken calibration tests | Tests for single tokens, English text, code, Chinese, special chars |
| 2026-04-08 | P1-8 CTX-004 | Added compaction performance tests | Small/large message perf, iteration perf, memory efficiency |
| 2026-04-08 | Round 7 | Verified implementations | Confirmed crash_recovery.rs, pool.rs, WASM FFI bridge exist and are complete |
| 2026-04-08 | Round 7 | Updated task list | tasks_v23.md updated to reflect all P0/P1 as Done |
| 2026-04-08 | Round 7 | TypeScript SDK build | npm run build and typecheck pass |
