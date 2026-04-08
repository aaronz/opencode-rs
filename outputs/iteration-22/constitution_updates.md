# Constitution Update Recommendations - Iteration 22

**Project**: OpenCode-RS  
**Analysis Date**: 2026-04-08  
**Source Documents**: 
- Constitution v1.0 (2026-04-07)
- Gap Analysis v22 (2026-04-08)

---

## 1. Executive Summary

The Gap Analysis v22 shows the implementation at **88-92% completeness** with 15/15 crates building successfully. The Constitution needs updates to reflect completed iterations and address new technical debt identified.

| Priority | Updates Required | Constitutional Action |
|----------|------------------|----------------------|
| **High** | Historical documents outdated (iterations 17-22 missing) | Update Article 8 |
| **High** | Technical debt T6-T8 not in constitution | Update Section 6.1 |
| **Medium** | Missing session_load/session_save tools | Add to Article 3 |
| **Medium** | Binary size/binary bloat not specified | Add to Non-Functional |
| **Low** | Dead code warnings not addressed | Add to Section 6.1 |

---

## 2. Immediate Updates Required

### 2.1 Update Article 8: Historical Documents (HIGH PRIORITY)

**Current** (Section 8.1):
```
| Document | Description | Status |
|----------|-------------|--------|
| `outputs/iteration-16/` | v16 implementation spec | **Current** |
```

**Recommended**:
```
| Document | Description | Status |
|----------|-------------|--------|
| `outputs/iteration-17/` | v17 implementation spec | Superseded |
| `outputs/iteration-18/` | v18 implementation spec | Superseded |
| `outputs/iteration-19/` | v19 implementation spec | Superseded |
| `outputs/iteration-20/` | v20 implementation spec (LSP backend) | Superseded |
| `outputs/iteration-22/` | v22 implementation spec (MCP/TUI) | **Current** |
```

**Rationale**: Constitution must accurately reflect current iteration (v22).

---

### 2.2 Update Section 6.1: Technical Debt (HIGH PRIORITY)

**Current** (Section 6.1): Lists T1-T5

**Recommended Addition**:
```
| ID | Description | Risk | Status |
|----|-------------|------|--------|
| T6 | session_load/session_save tools missing | Medium | Pending |
| T7 | Dead code warnings in TUI (5 items) and CLI (3 items) | Low | Pending |
| T8 | Binary size ~15-20MB exceeds 10MB target | Medium | Pending |
| T9 | Memory usage ~40-50MB idle exceeds 30MB target | Medium | Pending |
| T10 | Context Panel not implemented | Low | v1.1 |
| T11 | Built-in skills only 5/10 implemented | Medium | v1.1 |
```

**Rationale**: Gap Analysis v22 identifies these new technical debt items.

---

## 3. Medium Priority Updates

### 3.1 Add Missing Tools to Article 3 (MEDIUM PRIORITY)

**Recommended Addition - Section 3.5: Session Persistence Tools**

```markdown
### Section 3.5: Session Persistence Tools

#### session_load Tool

| Property | Value |
|----------|-------|
| Category | Write (Full scope) |
| Description | Load session state from SQLite storage |
| Arguments | `session_id: string` |
| Returns | `Session` object |

**Implementation Note**: As of v22, this tool is NOT YET IMPLEMENTED. Priority T6.

#### session_save Tool

| Property | Value |
|----------|-------|
| Category | Write (Full scope) |
| Description | Save current session state to SQLite storage |
| Arguments | `session_id: string, state: SessionState` |
| Returns | `Result<()>` |

**Implementation Note**: As of v22, this tool is NOT YET IMPLEMENTED. Priority T6.
```

**Rationale**: Gap Analysis v22 shows these tools are missing and blocking feature parity.

---

### 3.2 Add Non-Functional Requirements (MEDIUM PRIORITY)

**Recommended Addition - Section 6.3: Performance Targets**

```markdown
### Section 6.3: Non-Functional Requirements

| Requirement | Target | Current Status |
|-------------|--------|----------------|
| Binary size | < 10MB | ~15-20MB (T8) |
| Memory (idle) | < 30MB | ~40-50MB (T9) |
| Memory (active) | < 100MB | Variable |
| TUI startup | < 300ms | Not measured |
| Build warnings | 0 | 5 warnings (T7) |

**Technical Debt**: Items T7-T9 track non-compliance with these targets.
```

**Rationale**: Constitution should specify performance targets to guide optimization efforts.

---

### 3.3 Update LSP Capability Levels (MEDIUM PRIORITY)

**Current** (Section 3.4): References C-058 without specifying capability levels.

**Recommended Update**:
```markdown
### Section 3.4: LSP Integration

**Reference**: C-058

Two LSP modes:
1. **Server Mode**: OpenCode-RS serves LSP to editors (tower_lsp)
2. **Client Mode**: OpenCode-RS spawns external servers (rust-analyzer, tsserver)

All JSON-RPC messages use Content-Length headers.

#### LSP Capability Levels

| Level | Capabilities | Target Version | Status |
|-------|--------------|----------------|--------|
| **v1.0 (Must Have)** | diagnostics, workspace symbols, document symbols | v1.0 | ✅ Done |
| **v1.1 (Should Have)** | definition, references, hover, code actions | v1.1 | ⚠️ Pending |
| **v1.2 (Nice to Have)** | inline hints, inlay type hints | v1.2 | Future |

**Implementation Note**: LSP backend requires tree-sitter or similar parser integration per spec_v22.
```

**Rationale**: Gap Analysis v22 shows LSP at 80% with v1.1 capabilities pending.

---

## 4. Low Priority Updates

### 4.1 Update Section 3.2: Architectural Boundaries (LOW PRIORITY)

**Current**: Mentions 15 crates but doesn't enumerate.

**Recommended Addition**:
```markdown
### Section 2.2: Architectural Boundaries (Updated)

| Boundary | Principle |
|----------|-----------|
| Core ↔ Tools | Core is dependency-free; Tools depend on Core |
| Server ↔ Agent | Server handles HTTP; Agent handles execution |
| Permission | Separate crate (`opencode-permission`) with clear API |
| Storage | Abstracted behind `StorageService` trait |

#### Crate Inventory (v22)

| Crate | Purpose | Modules |
|-------|---------|---------|
| `opencode-core` | Core types, session, config | 54 |
| `opencode-cli` | Entry point | - |
| `opencode-llm` | 16 LLM providers | - |
| `opencode-tools` | 33+ tools | 30+ |
| `opencode-tui` | Ratatui UI | 20 |
| `opencode-agent` | 10 agent types | - |
| `opencode-lsp` | LSP client/server | - |
| `opencode-storage` | SQLite persistence | - |
| `opencode-server` | REST/WS/SSE | - |
| `opencode-permission` | Allow/Ask/Deny | - |
| `opencode-auth` | Credentials | - |
| `opencode-control-plane` | ACP client | - |
| `opencode-plugin` | WASM runtime | ⚠️ 60% |
| `opencode-git` | GitHub integration | ⚠️ Partial |
| `opencode-mcp` | MCP protocol | ⚠️ 80% |
```

**Rationale**: Gap Analysis v22 provides complete crate inventory.

---

## 5. Summary of Recommended Changes

| Section | Change Type | Priority | Effort |
|---------|-------------|----------|--------|
| 8.1 (Historical) | Add iterations 17-22 | **High** | 5 min |
| 6.1 (Tech Debt) | Add T6-T11 | **High** | 10 min |
| **NEW 3.5** | Add session_load/session_save | Medium | 15 min |
| **NEW 6.3** | Add performance targets | Medium | 10 min |
| 3.4 (LSP) | Add capability levels table | Medium | 10 min |
| 2.2 (Crates) | Add crate inventory | Low | 10 min |

---

## 6. Files to Update

1. `/outputs/.specify/memory/constitution.md` - Master constitution

---

## 7. RFC Required?

Per Article 7.1, constitution amendments require:
- [x] RFC document (this file)
- [ ] 2+ senior maintainer approval
- [ ] Announcement in project communication channel

**Recommendation**: Given these are primarily documentation updates to reflect completed work (iterations 17-22) and tracked technical debt (T6-T11), senior maintainers may expedite approval at their discretion.

---

## 8. Comparison with Iteration-19 Recommendations

The iteration-19 constitution_updates.md identified several items. Status update:

| Item | Recommended Action | Status in v22 |
|------|-------------------|---------------|
| Context Engine (Article 9) | Add new article | Not added - Context Panel still missing |
| Thinking Mode | Add to Article 3 | Status unclear |
| Share/Unshare | Add to Article 3 | Status unclear |
| TUI Requirements | Add new section | Partially addressed in Gap Analysis |
| Historical (iter 17-19) | Update Section 8.1 | Still pending |
| LSP capability levels | Update Section 3.4 | Still pending |
| Data models (ShareStatus, etc.) | Add to Article 2 | Status unclear |

**New in v22**: Technical debt T7-T11, binary size targets, session_load/session_save missing.

---

**Prepared By**: Sisyphus Constitution Analysis  
**Date**: 2026-04-08  
**Version**: 1.0
