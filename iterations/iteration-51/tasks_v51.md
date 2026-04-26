# Task List v51

**Document Version:** 51
**Generated:** 2026-04-26
**Total Tasks:** 45

---

## Task Priority Legend

- **[P0]** - Critical Blocker
- **[P1]** - Important Issue
- **[P2]** - Nice to Have

---

## Phase 1: ACP Module (New Crate)

### P0 Tasks - ACP Crate Creation

| # | Task | FR | Status | Dependencies |
|---|------|-----|--------|--------------|
| 1.1 | Create `crates/acp/` directory structure | FR-100 | ✅ Done | None |
| 1.2 | Create `crates/acp/Cargo.toml` with dependencies | FR-100 | ✅ Done | 1.1 |
| 1.3 | Create `crates/acp/src/lib.rs` with module exports | FR-100 | ✅ Done | 1.2 |
| 1.4 | Create `crates/acp/src/client.rs` (empty stub) | FR-100 | ✅ Done | 1.1 |

### P0 Tasks - ACP Core Types

| # | Task | FR | Status | Dependencies |
|---|------|-----|--------|--------------|
| 1.5 | Implement `AcpConnectionState` enum | FR-101 | TODO | 1.1 |
| 1.6 | Implement `AcpState` struct | FR-102 | TODO | 1.5 |
| 1.7 | Implement `AcpClient` struct with `http`, `state`, `bus` fields | FR-103 | ✅ Done | 1.6 |
| 1.8 | Implement `AcpError` enum with all variants | FR-104 | TODO | 1.1 |
| 1.9 | Implement `AcpStatus` struct | FR-105 | ✅ Done | 1.1 |
| 1.10 | Implement `HandshakeRequest` type | FR-105 | ✅ Done | 1.1 |
| 1.11 | Implement `HandshakeResponse` type | FR-105 | ✅ Done | 1.1 |
| 1.12 | Implement `ConnectRequest` type | FR-105 | ✅ Done | 1.1 |
| 1.13 | Implement `AckRequest` type | FR-105 | ✅ Done | 1.1 |
| 1.14 | Implement `AcpMessage` type | FR-105 | ✅ Done | 1.1 |

### P0 Tasks - ACP State Machine & Events

| # | Task | FR | Status | Dependencies |
|---|------|-----|--------|--------------|
| 1.15 | Implement `AcpClient::status()` method | FR-103 | ✅ Done | 1.7 |
| 1.16 | Implement `AcpClient::connect()` with state transitions | FR-103 | ✅ Done | 1.7, 1.8 |
| 1.17 | Implement `AcpClient::handshake()` method | FR-103 | ✅ Done | 1.7, 1.8 |
| 1.18 | Implement `AcpClient::ack()` method | FR-103 | ✅ Done | 1.7, 1.8 |
| 1.19 | Implement `AcpClient::send_message()` method | FR-103 | ✅ Done | 1.7, 1.8 |
| 1.20 | Implement `AcpClient::disconnect()` method | FR-103 | ✅ Done | 1.7, 1.8 |
| 1.21 | Implement `AcpClient::connection_state()` method | FR-103 | ✅ Done | 1.7 |
| 1.22 | Implement bus event publishing for `acp.connected` | FR-107 | ✅ Done | 1.16 |
| 1.23 | Implement bus event publishing for `acp.disconnected` | FR-107 | ✅ Done | 1.20 |

### P0 Tasks - ACP Tests

| # | Task | FR | Status | Dependencies |
|---|------|-----|--------|--------------|
| 1.24 | Write unit test: `status_returns_disconnected_initially` | FR-109 | TODO | 1.15 |
| 1.25 | Write unit test: `connect_transitions_state` | FR-109 | TODO | 1.16 |
| 1.26 | Write unit test: `send_message_returns_error_when_not_connected` | FR-109 | TODO | 1.19 |
| 1.27 | Write unit test: `disconnect_transitions_to_disconnected` | FR-109 | TODO | 1.20 |
| 1.28 | Create `crates/acp/tests/acp_tests.rs` | FR-109 | TODO | 1.1 |

### P1 Tasks - ACP CLI & Integration

| # | Task | FR | Status | Dependencies |
|---|------|-----|--------|--------------|
| 1.29 | Implement `acp status` CLI command | FR-108 | TODO | 1.15 |
| 1.30 | Implement `acp connect` CLI command | FR-108 | TODO | 1.16 |
| 1.31 | Implement `acp ack` CLI command | FR-108 | TODO | 1.18 |
| 1.32 | Create `crates/acp/src/cli.rs` | FR-108 | TODO | 1.29-1.31 |
| 1.33 | Write integration test: full connect-message-disconnect cycle | FR-110 | TODO | 1.24-1.27 |

---

## Phase 2: CLI Module P0 Gaps

| # | Task | FR | Status | Location |
|---|------|-----|--------|----------|
| 2.1 | Implement `agent run` with AgentRegistry + LLM provider | FR-004 | TODO | `cmd/agent.rs:48` |
| 2.2 | Implement `config set` with key-value persistence | FR-005 | TODO | `cmd/config.rs:199-202` |
| 2.3 | Implement `run --format ndjson/json` with actual LLM streaming | FR-006 | ✅ Done | `cmd/run.rs:228-255` |

---

## Phase 3: CLI Module P1 Gaps

| # | Task | FR | Status | Location |
|---|------|-----|--------|----------|
| 3.1 | Implement `account login/logout/status` | FR-007 | TODO | `cmd/account.rs` |
| 3.2 | Implement `attach` command | FR-008 | ✅ Done | `cmd/attach.rs:83-90` |
| 3.3 | Add `mcp add` subcommand | FR-009 | ✅ Done | `cmd/mcp.rs` |
| 3.4 | Implement `agent list` | FR-011 | ✅ Done | `cmd/agent.rs:48` |
| 3.5 | Implement `session fork` with TUI integration | FR-014 | TODO | `cmd/session.rs:673-698` |
| 3.6 | Implement `github install` persistence | FR-015 | ✅ Done | `cmd/github.rs:189-211` |
| 3.7 | Extend `providers login` for multi-provider (Anthropic, etc.) | FR-016 | TODO | `cmd/providers.rs:142-145` |
| 3.8 | Implement `acp handshake` session persistence | FR-017 | TODO | `cmd/acp.rs:250-292` |
| 3.9 | Implement `config migrate` | FR-018 | TODO | `cmd/config.rs:204-208` |
| 3.10 | Add `acp ack` command | FR-027 | TODO | `cmd/acp.rs` |

---

## Phase 4: CLI Module P2 Gaps

| # | Task | FR | Status | Location |
|---|------|-----|--------|----------|
| 4.1 | Parse environment variables before config load | FR-019 | TODO | `main.rs` |
| 4.2 | Model visibility config integration | FR-020 | TODO | `cmd/models.rs:238-267` |
| 4.3 | Default model from config (remove hardcoded "gpt-4o") | FR-021 | TODO | `cmd/run.rs:226` |

---

## Technical Debt Tasks

| # | Task | Status | Notes |
|---|------|--------|-------|
| T1 | Move protocol types from `crates/core/src/acp.rs` to `crates/acp/src/protocol.rs` | TODO | Consolidate types |
| T2 | Consolidate `AcpHandshakeResponse` to single definition | TODO | Remove duplication |
| T3 | Rewrite `AcpProtocol` as async `AcpClient` | TODO | Fix sync/async mismatch |
| T4 | Move hardcoded URLs to configurable base URL | TODO | Improve flexibility |
| T5 | Add connection timeout configuration | TODO | Improve robustness |
| T6 | Add retry logic with backoff | TODO | Improve reliability |
| T7 | Add version field to `AcpStatus` | TODO | Improve monitoring |

---

## Task Summary by Priority

| Priority | Count | Completed | TODO |
|----------|-------|----------|------|
| P0 | 28 | 0 | 28 |
| P1 | 13 | 0 | 13 |
| P2 | 4 | 0 | 4 |
| **Total** | **45** | **0** | **45** |

---

## Dependencies Graph

```
Phase 1 (ACP Module):
1.1 → 1.2 → 1.3 → 1.4
                ↓
1.5 → 1.6 → 1.7 → 1.8
                ↓
1.15-1.21 (methods)
   ↓         ↓
1.22      1.23 (bus events)
   ↓         ↓
1.24-1.27 (tests)
   ↓
1.28 → 1.33 (integration tests)

Phase 2 (CLI P0):
2.1, 2.2, 2.3 (independent, parallel)

Phase 3 (CLI P1):
3.1-3.10 (independent, parallel after Phase 2)

Phase 4 (CLI P2):
4.1-4.3 (independent, parallel after Phase 3)
```

---

## Next Steps

1. **Immediate:** Begin Task 1.1 - Create `crates/acp/` directory structure
2. **Then:** Implement Tasks 1.2-1.14 (ACP core types)
3. **Next:** Implement Tasks 1.15-1.23 (state machine and events)
4. **Then:** Implement Tasks 2.1-2.3 (CLI P0 gaps)
5. **Parallel:** Tasks 1.24-1.33 and 3.1-3.10
6. **Finally:** Tasks 4.1-4.3 and Technical Debt

---

*Task list generated: 2026-04-26*
*Total estimated tasks: 45*
*P0 tasks must be completed before P1 tasks*