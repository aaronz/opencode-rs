# Constitution Update Recommendations - Iteration 19

**Project**: OpenCode-RS  
**Analysis Date**: 2026-04-08  
**Source Documents**: 
- Constitution v1.0 (2026-04-07)
- Gap Analysis v19 (2026-04-08)

---

## Executive Summary

The Gap Analysis v19 identifies **no P0 blocking issues**, indicating the core architecture is stable. However, the analysis reveals several gaps between the Constitution and current implementation realities that require constitutional updates.

| Priority | Gaps Found | Constitutional Action |
|----------|------------|----------------------|
| **High** | Context Engine requirements missing | Add new Article 9 |
| **High** | Thinking Mode not documented | Add to Article 3 or new section |
| **Medium** | Share/Unshare functionality incomplete | Update Article 3 |
| **Medium** | TUI requirements not specified | Add TUI standards section |
| **Medium** | LSP v1.1 capabilities not clarified | Update Section 3.4 |
| **Low** | Historical documents outdated | Update Section 8.1 |
| **Low** | Missing data model documentation | Add to Article 2 |

---

## 1. Immediate Updates Required

### 1.1 Update Article 8: Historical Documents (LOW EFFORT)

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
| `outputs/iteration-19/` | v19 implementation spec | **Current** |
```

**Rationale**: The Constitution must accurately reflect the current iteration.

---

### 1.2 Update Section 3.4: LSP Integration (MEDIUM EFFORT)

**Current**: References C-058 without specifying capability levels.

**Recommended**:
```markdown
### Section 3.4: LSP Integration

**Reference**: C-058

Two LSP modes:
1. **Server Mode**: OpenCode-RS serves LSP to editors (tower_lsp)
2. **Client Mode**: OpenCode-RS spawns external servers (rust-analyzer, tsserver)

All JSON-RPC messages use Content-Length headers.

#### LSP Capability Levels

| Level | Capabilities | Target Version |
|-------|--------------|----------------|
| **v1.0 (Must Have)** | diagnostics, workspace symbols, document symbols | v1.0 |
| **v1.1 (Should Have)** | definition, references, hover, code actions | v1.1 |

**Implementation Note**: All LSP 1.1 capabilities should be implemented via C-058 extension process.
```

**Rationale**: Gap Analysis shows LSP 1.1 features (definition, references, hover, code actions) are not implemented. Constitution should mandate these for v1.1.

---

### 1.3 Add Context Engine Requirements (HIGH PRIORITY)

The Constitution does not mention the Context Engine at all. Based on Gap Analysis findings, add new section:

**Recommended New Section 3.5: Context Engine**

```markdown
### Section 3.5: Context Engine

#### Context Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| L0 | System prompt only | Minimal context |
| L1 | L0 + current file | Single file editing |
| L2 | L1 + project files | Project-aware |
| L3 | L2 + relevant history | Session-aware |
| L4 | L3 + external context | Full context |

#### Token Budget Management

Context compaction MUST respect the following thresholds:

| Threshold | Action | Trigger |
|-----------|--------|---------|
| 85% | Warning | Log warning, suggest compact |
| 92% | Auto-compact | Trigger SummaryAgent |
| 95% | Force-compact | Aggressive trimming |

#### Relevance Ranking

When context exceeds budget:
1. Score messages by relevance (recency, topic match, tool usage)
2. Preserve system prompt and recent conversation
3. Prioritize tool call/results for context-heavy sessions
4. Implement LFU (Least Frequently Used) eviction for file context

**Reference**: Gap Analysis Section 3.1.6
```

**Rationale**: Gap Analysis identifies context compaction framework exists but thresholds are not implemented. Constitution should mandate these.

---

## 2. Medium Priority Updates

### 2.1 Add Thinking Mode Documentation (MEDIUM PRIORITY)

**Recommended Addition to Article 3**:

```markdown
### Section 3.6: Thinking Mode

The `/thinking` command enables extended reasoning mode for complex tasks.

#### Behavior

| Setting | LLM Provider Flag | Description |
|---------|-------------------|-------------|
| Default | `thinking_mode: false` | Standard response |
| Extended | `thinking_mode: true` | Enable extended thinking |

#### Implementation Requirements

1. **Flag Propagation**: The `thinking_mode` flag MUST be passed to LLM provider via message construction
2. **Provider Support**: Not all providers support extended thinking - gracefully degrade
3. **Token Budget**: Extended thinking consumes more tokens - respect budget thresholds per Section 3.5

**Reference**: Gap Analysis Section 2.2, P1 Issue "/thinking 模式切换"
```

**Rationale**: Gap Analysis P1 issue - `thinking_mode` flag exists but is not passed to LLM provider.

---

### 2.2 Update Share/Unshare Documentation (MEDIUM PRIORITY)

**Current**: Constitution mentions `/share` implicitly in Commands section.

**Recommended** - Add to Article 3:

```markdown
### Section 3.7: Session Share

#### Share Modes

| Mode | Description | Implementation |
|------|-------------|----------------|
| Local | Export to temporary file | Default (v1.0) |
| Remote | Export to sharing service | Future (v1.5) |

#### Share State

Each session MUST track:
- `ShareStatus`: `NotShared | LocalOnly | RemoteShared(URL)`
- Share timestamp
- Share expiration (if remote)

#### Unshare Behavior

- Local shares: Delete temporary file
- Remote shares: Revoke shared link

**Reference**: Gap Analysis Section 2.2, P1 Issue "/share 仅本地导出"
```

**Rationale**: Gap Analysis shows `/unshare` is not implemented. Constitution should define expected behavior.

---

### 2.3 Add TUI Requirements Section (MEDIUM PRIORITY)

**Recommended New Section 3.8: TUI Implementation Standards**

```markdown
### Section 3.8: TUI Requirements

#### Streaming Output

All LLM streaming responses MUST use **typewriter effect**:
- Incremental character/word rendering
- Configurable speed (default: 20ms per update)
- Interruptible on user input

**Implementation**: `input_widget.start_typewriter()` in `check_llm_events()`

#### Token Usage Display

Status bar MUST display real-time token usage:
- Input tokens
- Output tokens  
- Total budget consumption

**Implementation**: `status_bar.update_usage()` MUST be called on each streaming chunk

#### Session State Indication

TUI MUST clearly indicate current session state:
```
idle | thinking | awaiting_permission | executing_tool
streaming | applying_changes | verifying | summarizing
aborted | error | completed
```

**Reference**: Gap Analysis Section 2.2, P1 Issues "打字机效果" and "Token 实时显示"
```

**Rationale**: Gap Analysis identifies P1 issues for typewriter effect and token display not fully implemented.

---

## 3. Low Priority Updates

### 3.1 Add Missing Data Models to Article 2 (LOW PRIORITY)

**Recommended Addition - Section 2.4: Core Data Models**

```markdown
### Section 2.4: Required Data Models

The following data models MUST be defined in `opencode-core`:

| Model | Purpose | Status |
|-------|---------|--------|
| `Session` | Session state and history | ✅ Implemented |
| `Message` | Conversation messages | ✅ Implemented |
| `ToolCall` | Tool invocation record | ✅ Implemented |
| `Permission` | Permission grant/reject | ✅ Implemented |
| `Snapshot` | Session snapshots | ✅ Implemented |
| `ShareStatus` | Share state tracking | ⚠️ Missing |
| `ThinkingMode` | Thinking mode config | ⚠️ Missing |
| `BudgetLimit` | Token budget config | ⚠️ Missing |
| `UsageStats` | Usage aggregation | ⚠️ Missing |

**Reference**: Gap Analysis Section 5.2
```

**Rationale**: Gap Analysis identifies missing data models that Constitution should acknowledge.

---

### 3.2 Update Technical Debt Table (LOW PRIORITY)

**Current**: Section 6.1 lists T1-T5.

**Recommended Addition**:
```markdown
### Section 6.1: Known Technical Debt (Continued)

| ID | Description | Risk | Status |
|----|-------------|------|--------|
| T6 | Magic numbers (100, 5000, 2000) scattered | Medium | Pending |
| T7 | Unused constants (`MAX_HISTORY_SIZE`, `TOKEN_ESTIMATE_DIVISOR`) | Low | Pending |
| T8 | Duplicate command definitions | Medium | Pending |
```

**Rationale**: Gap Analysis Section 6 identifies new technical debt items.

---

## 4. Proposed Constitutional Amendment

### 4.1 Amendment Proposal: Add Article 9 (Context Engine)

```markdown
## Article 9: Context Engine

### Section 9.1: Context Hierarchy

Context MUST be layered from L0-L4 as specified in Section 3.5.

### Section 9.2: Token Budget Enforcement

Automatic compaction MUST trigger at 92% threshold.

### Section 9.3: Relevance-Based Eviction

When trimming context, use relevance scoring per Section 3.5.
```

### 4.2 Amendment Proposal: Update Version

```markdown
**Version**: 1.1  
**Date**: 2026-04-08  
**Status**: Draft - Pending Approval
```

---

## 5. Summary of Recommended Changes

| Section | Change Type | Priority | Effort |
|---------|-------------|----------|--------|
| 8.1 (Historical) | Update iteration reference | Low | 5 min |
| 3.4 (LSP) | Add capability levels | Medium | 15 min |
| **NEW 3.5** | Add Context Engine requirements | **High** | 30 min |
| 3.6 (NEW) | Add Thinking Mode | Medium | 15 min |
| 3.7 (NEW) | Add Share/Unshare spec | Medium | 15 min |
| **NEW 3.8** | Add TUI requirements | **High** | 30 min |
| 2.4 (NEW) | Add data model table | Low | 10 min |
| 6.1 | Add T6-T8 technical debt | Low | 10 min |

---

## 6. Files to Update

1. `/outputs/.specify/memory/constitution.md` - Master constitution

---

## 7. RFC Required?

Per Article 7.1, constitution amendments require:
- [x] RFC document (this file)
- [ ] 2+ senior maintainer approval
- [ ] Announcement in project communication channel

**Recommendation**: Given these are clarifying updates to match existing implementation (not architectural changes), senior maintainers may waive full RFC process for items marked "Low Priority" at their discretion.

---

**Prepared By**: Sisyphus Constitution Analysis  
**Date**: 2026-04-08  
**Version**: 1.0
