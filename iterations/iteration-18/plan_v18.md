# Implementation Plan - Iteration 18

**Version:** 1.8.0  
**Generated:** 2026-04-14  
**Status:** Implementation ~90-95% Complete  
**Phase:** Phase 1-5 Complete, Phase 6 (Release Qualification) Pending

---

## Executive Summary

Implementation is approximately **90-95% complete**. All P0 blocking issues from prior iterations have been resolved. Phase 6 (Release Qualification) is the primary focus for this iteration.

---

## Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~100% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~100% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~100% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~95% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~95% |
| **Phase 6** | **Release Qualification** | ❌ **NOT STARTED** | **~0%** |

---

## P0/P1/P2 Issue Classification

### P0 - Blocking Issues (ALL RESOLVED)
- ✅ Custom tool discovery format mismatch
- ✅ Custom tools not registered with ToolRegistry
- ✅ Plugin tool registration missing
- ✅ Non-deterministic hook execution order
- ✅ Plugin config ownership not enforced
- ✅ Config crate empty re-export (violates PRD 19)

### P1 - High Priority (This Iteration)
| Issue | Module | Status |
|-------|--------|--------|
| **Begin Phase 6 Release Qualification** | all | ❌ NOT STARTED |
| GitLab CI integration tests proper handling | git | ⚠️ NEEDS FIX |

### P2 - Medium Priority
| Issue | Module | Status |
|-------|--------|--------|
| Desktop/web smoke test port conflict | cli | ❌ NOT FIXED |
| Deprecated `mode` field cleanup | config | ⚠️ Deferred |
| Deprecated `tools` field cleanup | config | ⚠️ Deferred |

---

## Phase 6: Release Qualification Plan

### 6.1 Prerequisites
- [ ] Fix GitLab CI integration tests (mark as `#[ignore]` or use mock server)
- [ ] Fix desktop/web smoke test port conflict

### 6.2 End-to-End Integration Tests
- [ ] Session creation and lifecycle
- [ ] Message processing flow
- [ ] Tool execution pipeline
- [ ] MCP server connection and tool execution
- [ ] LSP diagnostics flow
- [ ] Plugin hook execution

### 6.3 Performance Benchmarking
- [ ] Session startup time
- [ ] Tool execution latency
- [ ] Memory usage under load
- [ ] MCP context cost measurement

### 6.4 Security Audit
- [ ] Permission boundary verification
- [ ] Auth token storage review
- [ ] Input validation completeness
- [ ] Config precedence enforcement

### 6.5 Observability Validation
- [ ] Logging at all major decision points
- [ ] Error reporting completeness
- [ ] Tracing integration verification

---

## Test Results

```
cargo test --all-features --all:
- ~1020 passed
- 8 failed

Failed tests breakdown:
- GitLab CI tests: 7 failures (integration tests require real GitLab server)
- CLI tests: 1 failure (desktop_web_different_ports - port conflict)
```

---

## Immediate Actions (P1)

1. **Begin Phase 6 Release Qualification**
   - End-to-end integration tests
   - Performance benchmarking
   - Security audit
   - Observability validation

2. **Fix GitLab CI Integration Tests**
   - Mark environment-dependent tests with `#[ignore]` and document as requiring external GitLab
   - Or provide mock GitLab server for CI

3. **Fix Desktop/Web Smoke Test**
   - Use dynamic port allocation instead of hardcoded port 3000

---

## Medium-term Actions (P2)

4. **Legacy Cleanup** (for v4.0)
   - Remove deprecated `mode` field
   - Remove deprecated `tools` field

---

## Crate-Level Implementation Status

| Crate | Status | Notes |
|-------|--------|-------|
| `crates/core/` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | ✅ Done | Persistence, recovery, snapshots |
| `crates/agent/` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ✅ Done | Registry and custom tool discovery |
| `crates/plugin/` | ✅ Done | Hooks and tool registration |
| `crates/tui/` | ✅ Done | Full implementation, all tests passing |
| `crates/server/` | ✅ Done | API routes, auth, streaming |
| `crates/mcp/` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | ✅ Done | Full config implementation |
| `crates/cli/` | ✅ Done | Desktop/web implemented |
| `crates/control-plane/` | ✅ Done | ACP stream, events, enterprise features |
| `ratatui-testing/` | ✅ Done | TUI testing framework crate |

---

**Document Version:** 1.8  
**Iteration:** 18  
**Last Updated:** 2026-04-14  
**Priority:** Begin Phase 6 release qualification, fix test infrastructure