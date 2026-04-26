# Implementation Plan v51

**Document Version:** 51
**Generated:** 2026-04-26
**Status:** Draft

---

## 1. Overview

This plan covers the implementation of the CLI Module gaps and the new ACP Module for OpenCode RS.

---

## 2. Priority Classification

### P0 - Critical Blockers (Must Fix)
- CLI: `agent run`, `config set`, `run --format ndjson/json`
- ACP: Complete new crate creation

### P1 - Important Issues (Should Fix)
- CLI: `account`, `attach`, `mcp add`, `agent list`, `session fork`, `github install`, `providers login`, `acp handshake` persistence, `config migrate`
- ACP: `ack` CLI command, protocol type alignment

### P2 - Nice to Have (Consider for Future)
- Environment variables, model visibility config, default model from config
- Session sharing, integration tests

---

## 3. Implementation Phases

### Phase 1: ACP Module (New Crate)

**Goal:** Create `crates/acp/` from scratch per PRD specification

| Step | Task | FR | Priority |
|------|------|-----|----------|
| 1.1 | Create `crates/acp/` directory structure with Cargo.toml | FR-100 | P0 |
| 1.2 | Implement `AcpConnectionState` enum | FR-101 | P0 |
| 1.3 | Implement `AcpState` struct | FR-102 | P0 |
| 1.4 | Implement `AcpClient` struct with async methods | FR-103 | P0 |
| 1.5 | Implement `AcpError` enum | FR-104 | P0 |
| 1.6 | Implement ACP Protocol types | FR-105 | P0 |
| 1.7 | Implement ACP State machine | FR-106 | P0 |
| 1.8 | Implement Bus Event Publishing | FR-107 | P0 |
| 1.9 | Implement ACP CLI commands | FR-108 | P1 |
| 1.10 | Write unit tests | FR-109 | P0 |
| 1.11 | Write integration tests | FR-110 | P2 |

### Phase 2: CLI Module Gaps - P0

**Goal:** Complete the P0 critical gaps in CLI commands

| Step | Task | FR | Priority | Location |
|------|------|-----|----------|----------|
| 2.1 | Implement `agent run` with AgentRegistry + LLM provider | FR-004 | P0 | `cmd/agent.rs:48` |
| 2.2 | Implement `config set` with key-value persistence | FR-005 | P0 | `cmd/config.rs:199-202` |
| 2.3 | Implement `run --format ndjson/json` with actual LLM streaming | FR-006 | P0 | `cmd/run.rs:228-255` |

### Phase 3: CLI Module Gaps - P1

**Goal:** Complete the P1 important gaps

| Step | Task | FR | Priority | Location |
|------|------|-----|----------|----------|
| 3.1 | Implement `account login/logout/status` | FR-007 | P1 | `cmd/account.rs` |
| 3.2 | Implement `attach` command | FR-008 | P1 | `cmd/attach.rs:83-90` |
| 3.3 | Add `mcp add` subcommand | FR-009 | P1 | `cmd/mcp.rs` |
| 3.4 | Implement `agent list` | FR-011 | P1 | `cmd/agent.rs:48` |
| 3.5 | Implement `session fork` with TUI integration | FR-014 | P1 | `cmd/session.rs:673-698` |
| 3.6 | Implement `github install` persistence | FR-015 | P1 | `cmd/github.rs:189-211` |
| 3.7 | Extend `providers login` for multi-provider | FR-016 | P1 | `cmd/providers.rs:142-145` |
| 3.8 | Implement `acp handshake` session persistence | FR-017 | P1 | `cmd/acp.rs:250-292` |
| 3.9 | Implement `config migrate` | FR-018 | P1 | `cmd/config.rs:204-208` |
| 3.10 | Add `acp ack` command | FR-027 | P1 | `cmd/acp.rs` |

### Phase 4: CLI Module Gaps - P2

**Goal:** Complete the P2 nice-to-have gaps

| Step | Task | FR | Priority | Location |
|------|------|-----|----------|----------|
| 4.1 | Parse environment variables before config load | FR-019 | P2 | `main.rs` |
| 4.2 | Model visibility config integration | FR-020 | P2 | `cmd/models.rs:238-267` |
| 4.3 | Default model from config | FR-021 | P2 | `cmd/run.rs:226` |

---

## 4. ACP Module Implementation Details

### 4.1 Crate Structure

```
crates/acp/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Re-exports AcpClient, AcpError, types
│   ├── client.rs       # AcpClient implementation
│   ├── protocol.rs      # Protocol types
│   └── cli.rs           # CLI command handlers
└── tests/
    └── acp_tests.rs
```

### 4.2 State Machine

```
Disconnected
    │
    │ connect()
    ▼
Handshaking ──(success)──► Connected
    │                          │
    │                          │ disconnect()
    └──(failure)──► Failed
```

### 4.3 Dependencies to Add

| Dependency | Version | Purpose |
|------------|---------|---------|
| reqwest | 0.12 | HTTP client |
| tokio | 1.45 | Async runtime |
| serde | 1.0 | Serialization |
| thiserror | 2.0 | Error handling |
| tracing | 0.1 | Logging |
| chrono | 0.4 | Timestamps |
| uuid | 1.0 | Message IDs |
| wiremock | 0.6 | Testing |
| tokio-test | 0.4 | Async testing |

---

## 5. Technical Debt Remediations

| Item | Remediation |
|------|-------------|
| Protocol types in core | Move `crates/core/src/acp.rs` types to `crates/acp/src/protocol.rs` |
| Duplicate AcpHandshakeResponse | Consolidate to single definition in ACP crate |
| Mixed sync/async | Rewrite `AcpProtocol` as async `AcpClient` |
| Hardcoded URLs | Move to configurable base URL |
| Missing version in AcpStatus | Add version field |

---

## 6. File Locations Summary

| File | Action |
|------|--------|
| `crates/acp/` | Create new |
| `crates/core/src/acp.rs` | Consolidate types to ACP crate |
| `crates/cli/src/cmd/acp.rs` | Add `ack` command, import from ACP crate |
| `crates/cli/src/cmd/agent.rs` | Implement `run` and `list` |
| `crates/cli/src/cmd/config.rs` | Implement `set` and `migrate` |
| `crates/cli/src/cmd/run.rs` | Implement actual LLM streaming |

---

## 7. Implementation Order

1. **Create ACP crate structure** (FR-100)
2. **Implement ACP core types** (FR-101 to FR-105)
3. **Implement AcpClient state machine** (FR-106 to FR-107)
4. **Add ACP CLI commands** (FR-108)
5. **Write ACP tests** (FR-109)
6. **Fix CLI P0 gaps** (FR-004, FR-005, FR-006)
7. **Fix CLI P1 gaps** (FR-007 to FR-018, FR-027)
8. **Fix CLI P2 gaps** (FR-019, FR-020, FR-021)
9. **Consolidate protocol types** (Technical Debt)
10. **Write ACP integration tests** (FR-110)

---

*Plan generated: 2026-04-26*
*Next Action: Begin Phase 1 - Create `crates/acp/` crate structure*