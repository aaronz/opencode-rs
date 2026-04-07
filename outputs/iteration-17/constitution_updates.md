# Constitution Update Recommendations

**Date**: 2026-04-07  
**Source**: Analysis of PRD.md, gap-analysis documents, and codebase structure  
**Target**: `outputs/.specify/memory/constitution.md`  

---

## Executive Summary

The current Constitution (v1.0, 2026-04-07) requires updates to reflect:
1. Iteration-17 status (currently claims iteration-16 is "Current")
2. New crates that have emerged (TUI, MCP, Plugin, Control-Plane, Auth)
3. P1 issues from gap analysis not adequately covered
4. PRD features not represented in Constitution

**Verdict**: Constitution NEEDS UPDATE. Critical gaps identified.

---

## 1. Issues Requiring Constitution Updates

### 1.1 Historical Documents (Article 8) - CRITICAL

**Problem**: Constitution Article 8 lists iteration-16 as "Current" but we are now in iteration-17.

**Current Text**:
```markdown
| Document | Description | Status |
|----------|-------------|--------|
...
| `outputs/iteration-16/` | v16 implementation spec | **Current** |
```

**Recommendation**: Update to:
```markdown
| `outputs/iteration-17/` | v17 implementation spec | **Current** |
```

---

### 1.2 Architectural Boundaries (Article 2.2) - INCOMPLETE

**Problem**: The architectural boundaries section does not mention several new crates that have been created.

**Current Crate List in Constitution**:
- Core
- Tools
- Server
- Agent
- Permission (opencode-permission)
- Storage

**Missing Crates** (now exist in `rust-opencode-port/crates/`):
| Crate | Purpose | Constitutional Status |
|-------|---------|----------------------|
| `opencode-tui` | Terminal UI with ratatui | Not mentioned |
| `opencode-mcp` | MCP protocol bridge | Not mentioned |
| `opencode-plugin` | WASM/sidecar plugin host | Not mentioned |
| `opencode-control-plane` | Enterprise control plane | Not mentioned |
| `opencode-auth` | Authentication (separate from permission) | Not mentioned |
| `opencode-git` | Git operations | Not mentioned |

**Recommendation**: Add new section or update Article 2.2:

```markdown
### Section 2.2: Architectural Boundaries

| Boundary | Principle |
|----------|-----------|
| Core ↔ Tools | Core is dependency-free; Tools depend on Core |
| Server ↔ Agent | Server handles HTTP; Agent handles execution |
| Permission | Separate crate (`opencode-permission`) with clear API |
| Auth | Separate crate (`opencode-auth`) for credential management |
| Storage | Abstracted behind `StorageService` trait |
| TUI | Separate crate (`opencode-tui`) using ratatui |
| MCP | Separate crate (`opencode-mcp`) for protocol bridge |
| Plugin | Separate crate (`opencode-plugin`) for WASM/sidecar plugins |
| Control-Plane | Separate crate (`opencode-control-plane`) for enterprise |
```

---

### 1.3 Technical Debt T4 (Article 6.1) - OUTDATED

**Problem**: T4 "TUI permission confirmation" is marked as "Low" risk "Accepted" but TUI is now a first-class crate with significant implementation.

**Current**:
```markdown
| T4 | TUI permission confirmation | Low | Accepted |
```

**Reality**: 
- `opencode-tui` crate exists with full implementation
- Uses ratatui + crossterm
- Permission confirmation is a core UX feature, not just technical debt
- This should be elevated from "Accepted" to a tracked feature

**Recommendation**: Either:
1. Remove T4 from technical debt (it's now implemented)
2. Or reclassify as "Implemented - verify UX compliance"

---

### 1.4 Technical Debt T5 (Article 6.1) - STATUS UNCLEAR

**Problem**: T5 "auth_layered not integrated" is marked "Low" risk "Pending" but `auth_layered` module exists in `crates/llm/src/`.

**Current**:
```markdown
| T5 | auth_layered not integrated | Low | Pending |
```

**Reality**:
- `auth_layered` exists and is referenced in `provider_registry.rs`
- The technical debt may already be addressed or in progress

**Recommendation**: Verify current status and update accordingly.

---

### 1.5 PRD Features Not Covered by Constitution

The following PRD v1 features are NOT represented in Constitution:

| PRD Feature | Current Coverage | Gap |
|-------------|-------------------|-----|
| Commands system (`/command` syntax) | Implemented | Not in Constitution |
| Skills system (`.opencode/skills/`) | Implemented | Not in Constitution |
| Share capability (local + service) | Partial | Not in Constitution |
| MCP protocol | Implemented | Only in C-058 (LSP), not MCP-specific |
| Plugin system (WASM/sidecar) | Implemented | Not in Constitution |
| Context Engine (token budget, compaction) | Implemented | Not in Constitution |
| Multiple Agent types (Build/Plan/Review/Refactor/Debug) | Only Build/Plan mentioned | Incomplete |

---

### 1.6 Config Format Status (Article 3.3) - INCONSISTENT

**Problem**: Constitution Article 3.3 references C-056 for TOML→JSONC migration and claims JSONC is "Preferred", but:

1. Gap analysis (iteration-16) shows TOML is still primary in `config.rs`
2. The `config.rs` code shows both JSONC and TOML support with TOML triggering deprecation warnings
3. Constitution claims JSONC is preferred but doesn't reflect actual implementation state

**Current**:
```markdown
| Priority | Format | Status |
|----------|--------|--------|
| 1 | `.opencode/config.jsonc` | Preferred |
| 2 | `.opencode/config.json` | Supported |
| 3 | `.opencode/config.toml` | **Deprecated** |
```

**Gap Analysis Finding**:
> "配置格式使用 TOML 而非 JSONC - PRD 要求 JSONC/JSON，实现使用 TOML，需迁移"

**Recommendation**: Add a technical debt entry or update status:
```markdown
| ID | Description | Risk | Status |
|----|-------------|------|--------|
| T6 | Config format: JSONC migration incomplete | High | In Progress |
```

---

### 1.7 Missing API Endpoints (PRD Section 7.16)

**Problem**: Gap analysis identified missing P1 endpoints:
- `GET /health` - Health check endpoint
- `POST /sessions/{id}/abort` - Session abort

Constitution Section 3.2 only references C-057 which covers health, abort, permission reply endpoints - but these may not all be implemented.

**Recommendation**: Verify C-057 reflects actual implementation.

---

## 2. Proposed Constitution Amendments

### Amendment 1: Update Historical Documents

Update Article 8 to reflect iteration-17 status.

### Amendment 2: Expand Architectural Boundaries

Add new crates to Article 2.2 as shown in section 1.2 above.

### Amendment 3: Add New Constitution Documents

Add to Article 1.2 (Incorporation by Reference):

| ID | Title | Description |
|----|-------|-------------|
| C-059 | TUI Architecture | `opencode-tui` crate, permission UX, layout |
| C-060 | MCP Protocol | MCP bridge, tool discovery, permission integration |
| C-061 | Plugin System | WASM/sidecar plugins, event bus, capability model |
| C-062 | Commands & Skills | Command registry, skill loader, template expansion |

### Amendment 4: Update Technical Debt Table

1. Remove or reclassify T4 (TUI permission confirmation)
2. Verify and update T5 (auth_layered)
3. Add T6 (Config JSONC migration incomplete)

### Amendment 5: Add Agent Types

Update Article 3 to document all v1 agent types:
- Build Agent (default, full access)
- Plan Agent (read-only analysis)
- Review Agent (code review, read-only)
- Refactor Agent (cross-file refactoring)
- Debug Agent (error diagnosis)

---

## 3. Gap Analysis: P0/P1 Issues from PRD

### P0 Issues (Blocking - from PRD Gap Analysis)

Based on iteration-6 gap analysis, these P0 issues were identified in PRD but NOT adequately covered by Constitution:

| Issue | PRD Reference | Gap | Recommendation |
|-------|---------------|-----|----------------|
| Provider authentication protocol layering | PRD 7.13, Section 8 | PRD requires 4-layer auth architecture (Credential Source → Auth Mechanism → Provider Transport → Runtime Access Control). Constitution does not mandate this layered model. | Add explicit auth layering requirement to Article 2 or new C-063 |
| OAuth Browser/Device Code flows missing | PRD 7.13, Section 8.4 | PRD requires OAuth authorization code + PKCE for browser login and device code flow for CLI. Gap analysis flags this as P0. | Add OAuth flow requirements to Constitution if v1 scope |

**Evidence**: `outputs/iteration-6/gap-analysis.md` explicitly lists "Provider认证分层架构未完成" and "OAuth/Device Code 浏览器登录未实现" as P0 items.

### P1 Issues (High Priority - Affect v1 Core)

| Issue | PRD Reference | Constitutional Coverage | Recommendation |
|-------|---------------|------------------------|----------------|
| Config format mismatch | PRD 7.14, C-056 | Partially covered | Add T6 technical debt entry |
| session_load/session_save tools | PRD 7.2, 7.3 | Not covered | Add to Article 3 (Session Tools) |
| Missing /health endpoint | PRD 7.16, C-057 | Referenced in C-057 | Verify implementation |
| Missing session abort endpoint | PRD 7.16, C-057 | Referenced in C-057 | Verify implementation |

---

## 4. Summary of Required Changes

| Priority | Change | Article/Section | Type |
|----------|--------|----------------|------|
| 🔴 Critical | Update iteration-16 → iteration-17 | Article 8 | Fix |
| 🔴 Critical | Add missing crates to Article 2.2 | Article 2.2 | Amendment |
| 🟡 High | Add C-059 through C-062 | Article 1.2 | Amendment |
| 🟡 High | Update/reclassify T4 (TUI) | Article 6.1 | Amendment |
| 🟡 High | Verify T5 (auth_layered) status | Article 6.1 | Amendment |
| 🟡 High | Add T6 (Config migration) | Article 6.1 | Amendment |
| 🟢 Medium | Document all Agent types | Article 3 | Amendment |
| 🟢 Medium | Add Commands/Skills reference | Article 3 | Amendment |

---

## 5. Recommendations

### Immediate Actions Required:

1. **Update Article 8** (Historical Documents) - iteration-17 status
2. **Expand Article 2.2** (Architectural Boundaries) - add new crates
3. **Add new Constitution documents** (C-059 through C-062)
4. **Reclassify T4** - TUI is no longer "Accepted" debt, it's implemented
5. **Add T6** - Config JSONC migration incomplete (High risk)

### Follow-up Actions:

1. Verify C-057 implementation matches actual server endpoints
2. Add session_load/session_save to Constitution if they are P0 requirements
3. Document multiple agent types in Constitution

---

## 6. Compliance Check

| Constitution Requirement | Current Status | Compliance |
|-------------------------|----------------|------------|
| Core is dependency-free | Likely compliant | ✅ |
| Server handles HTTP | Compliant | ✅ |
| Permission separate crate | `opencode-permission` exists | ✅ |
| Storage behind trait | Likely compliant | ✅ |
| JSONC preferred config | TOML still primary | ⚠️ Partial |
| Test coverage targets | Defined in C-055 | ✅ |
| Permission model (Read/Safe/Write) | Implemented | ✅ |

---

**Prepared by**: Constitution Analysis Agent  
**Analysis Date**: 2026-04-07  
**Next Review**: After iteration-17 implementation complete
