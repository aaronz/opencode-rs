# Constitution Update Recommendations - Iteration 18

**Generated**: 2026-04-07  
**Source Documents**: 
- `outputs/.specify/memory/constitution.md` (Master Constitution v1.0)
- `outputs/iteration-18/gap-analysis.md` (Gap Analysis Report)
- `TUI_PRD.md` (TUI PRD v1.4)
- `rust-opencode-port/crates/tui/src/file_ref_handler.rs`
- `rust-opencode-port/crates/tui/src/shell_handler.rs`

---

## Executive Summary

The existing Constitution (v1.0) does **not** cover any of the three P0 issues identified in the Gap Analysis:

| P0 Issue | Gap Analysis Finding | Constitution Coverage |
|----------|---------------------|----------------------|
| OpenTUI vs Ratatui Architecture | Implementation uses Ratatui, not OpenTUI as specified in PRD | **NONE** |
| @ File Reference Integration | FileRefHandler exists but not integrated into LLM context | **NONE** |
| ! Shell Command Integration | ShellHandler exists but output not rendered to UI | **PARTIAL** (permission only) |

---

## Recommended Constitution Amendments

### Amendment 1: Add Article 9 - TUI Architecture

**Proposed Section 9.1: TUI Framework Selection**

The Constitution should explicitly document the TUI framework decision.

```markdown
## Article 9: TUI Architecture

### Section 9.1: Framework Selection

| Aspect | Decision | Rationale |
|--------|----------|------------|
| **Framework** | Ratatui (Rust) | Cross-platform, mature, dependency-free core |
| **Language** | Pure Rust | Single language codebase, no Zig/TypeScript/Bun requirement |
| **Integration** | External LLM Engine | LLM calls via opencode-llm crate |

### Section 9.2: Architectural Boundaries (TUI)

| Boundary | Principle |
|----------|-----------|
| TUI ↔ LLM | TUI sends messages, receives stream; LLM processing is external |
| TUI ↔ Shell | ShellHandler executes commands; output formatted for display/context |
| TUI ↔ Files | FileRefHandler resolves references; content injected into context |
| UI ↔ Logic | UI components (ratatui) separate from business logic |
```

**Why This Matters**: The Gap Analysis identified "OpenTUI 架构缺失" as a P0 blocker because the PRD specified OpenTUI (Zig + TypeScript) but implementation uses Ratatui. This is an **architectural decision** that should be documented in the Constitution to prevent future confusion.

---

### Amendment 2: Add Article 10 - TUI-Specific Patterns

**Proposed Section 10.1: File Reference Integration Pattern**

```markdown
### Section 10.1: File Reference Integration (@ Syntax)

#### Pattern Overview

When a user references files using `@` syntax in a message, the following flow must be implemented:

1. **Parse**: InputParser extracts `@filepath` tokens from user input
2. **Resolve**: FileRefHandler.fuzzy_search_files() finds matching files
3. **Select**: User selects target file from FileSelectionDialog
4. **Load**: FileRefHandler.resolve() reads file content (with truncation if > MAX_CONTENT_SIZE)
5. **Format**: FileRefHandler.format_for_context() generates context string
6. **Inject**: Formatted content appended to LLM message context

#### Required Methods

| Handler | Method | Purpose |
|---------|--------|---------|
| FileRefHandler | `fuzzy_search_files(query, max_results)` | Find files matching query |
| FileRefHandler | `resolve(path)` | Load file content with validation |
| FileRefHandler | `format_for_context(result)` | Generate context string for LLM |

#### Constraints

| Parameter | Value | Location |
|-----------|-------|----------|
| MAX_FILE_SIZE | 1MB | file_ref_handler.rs |
| MAX_CONTENT_SIZE | 5000 bytes | file_ref_handler.rs |
| Excluded dirs | .git, hidden files | file_ref_handler.rs |

#### Integration Requirement

**CRITICAL**: FileRefHandler.resolve() output MUST be integrated into LLM context before sending to LLM. This is not optional.

```rust
// CORRECT: File content injected into context
let file_result = file_ref_handler.resolve(&selected_path);
let context_string = file_ref_handler.format_for_context(&file_result);
llm_message.add_context(context_string);

// WRONG: File content not used
let file_result = file_ref_handler.resolve(&selected_path);
// file_result dropped, never added to context
```
```

---

**Proposed Section 10.2: Shell Command Integration Pattern (! Syntax)**

```markdown
### Section 10.2: Shell Command Integration (! Syntax)

#### Pattern Overview

When a user prefix a message with `!`, it is treated as a shell command:

1. **Parse**: InputParser detects `!` prefix, extracts command
2. **Validate**: ShellHandler.validate_command() checks for dangerous commands
3. **Execute**: ShellHandler.execute() or execute_interruptible() runs command
4. **Format**: Command output formatted for display
5. **Inject**: Output added to conversation as tool result
6. **Display**: ExecuteResult rendered in UI via MessageBubble

#### Required Methods

| Handler | Method | Purpose |
|---------|--------|---------|
| ShellHandler | `execute(command)` | Execute with timeout |
| ShellHandler | `execute_interruptible(command)` | Execute allowing Ctrl+C |
| ShellHandler | `validate_command(command)` | Check dangerous commands |
| ShellHandler | `is_dangerous(command)` | Detect risky operations |

#### Constraints

| Parameter | Value | Location |
|-----------|-------|----------|
| DEFAULT_TIMEOUT_SECS | 30 | shell_handler.rs |
| MAX_OUTPUT_SIZE | 100KB | shell_handler.rs |
| DANGEROUS_COMMANDS | 20+ patterns | shell_handler.rs |

#### Integration Requirement

**CRITICAL**: ShellHandler.execute() output MUST be rendered in the UI conversation and added as a tool result to LLM context.

```rust
// CORRECT: Output displayed and added to context
let result = shell_handler.execute(&command);
ui.add_message(Message::tool_result(&result));
llm_context.add_tool_result(format!("stdout: {}\nstderr: {}", result.stdout, result.stderr));

// WRONG: Output not displayed
let result = shell_handler.execute(&command);
// result.stdout/stderr never shown to user
```
```

---

**Proposed Section 10.3: Typewriter Effect (P1)**

```markdown
### Section 10.3: Stream Rendering

#### Typewriter Effect

When LLM provides streaming response, implement typewriter effect:

| Config Key | Type | Default | Purpose |
|------------|------|---------|---------|
| typewriter_speed | number | 0 (instant) | Characters per render cycle |
| typewriter_enabled | boolean | true | Enable/disable effect |

#### Implementation

```rust
// Stream handler must support incremental rendering
pub trait StreamRenderer {
    fn start_typing(&mut self) -> Result<(), RenderError>;
    fn append_chunk(&mut self, chunk: &str) -> Result<(), RenderError>;
    fn finish_typing(&mut self) -> Result<(), RenderError>;
}
```
```

---

### Amendment 3: Update Article 2 - Foundational Principles

**Proposed Section 2.4: Context Integration Principles**

Add to existing Section 2.1:

```markdown
### Section 2.4: Context Integration

All tool outputs that modify conversation state MUST be integrated into:
1. **UI Display**: Visible to user via appropriate component
2. **LLM Context**: Added to message context for continued conversation

| Tool | UI Display | LLM Context |
|------|------------|-------------|
| @ file reference | FileSelectionDialog | Injected into message |
| ! shell command | MessageBubble (tool result) | As tool_result message |
| /tools | ToolResultPanel | As tool_result message |
```

---

### Amendment 4: Update Article 3 - Implementation Standards

**Proposed Section 3.5: TUI Command Patterns**

Add after Section 3.4:

```markdown
### Section 3.5: TUI Command Registry

All slash commands must be fully implemented, not use Custom(String) placeholder.

#### Required Commands (P0)

| Command | Action | Status |
|---------|--------|--------|
| /share | Share session | Must implement |
| /unshare | Unshare session | Must implement |
| /thinking | Toggle thinking display | Must implement |

#### Command Implementation Template

```rust
#[derive(Debug, Clone)]
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: CommandAction,
}

pub enum CommandAction {
    // NOT Custom(String) - must be concrete type
    ShareSession,
    UnshareSession,
    ToggleThinking,
    // ... other concrete actions
}
```

**Anti-Pattern**: `Custom(String)` command placeholders indicate incomplete implementation and MUST be resolved before PR approval.
```

---

### Amendment 5: Update Technical Debt Table

**Proposed Section 6.1: Known Technical Debt (Updated)**

Add new items:

| ID | Description | Risk | Status | Related Amendment |
|----|-------------|------|--------|-------------------|
| T6 | OpenTUI vs Ratatui mismatch | High | Mitigated by C-059 (Article 9) | Amendment 1 |
| T7 | FileRefHandler not in LLM context | High | Pending | Amendment 2 |
| T8 | ShellHandler output not in UI | High | Pending | Amendment 2 |
| T9 | Custom(String) command placeholders | Medium | Pending | Amendment 4 |

---

## Proposed New Constitution Document IDs

| Document | Title | Amendments |
|----------|-------|------------|
| C-059 | TUI Architecture | Amendment 1 |
| C-060 | TUI Context Integration Patterns | Amendment 2 |
| C-061 | Stream Rendering Requirements | Amendment 3 |

---

## Implementation Checklist

Before marking Constitution update as complete:

- [ ] Create new C-059 document with Article 9 (TUI Architecture)
- [ ] Create new C-060 document with Article 10 (TUI Patterns)
- [ ] Update Section 2.4 with Context Integration Principles
- [ ] Update Section 6.1 with new Technical Debt items
- [ ] Verify all P0 issues are addressed in new amendments
- [ ] Add PR checklist item for Custom(String) command review

---

## Backward Compatibility

These amendments maintain backward compatibility:
- No existing behavior is changed
- New patterns are additive
- Technical debt items are informational only

---

## References

- Gap Analysis: `outputs/iteration-18/gap-analysis.md`
- FileRefHandler: `rust-opencode-port/crates/tui/src/file_ref_handler.rs`
- ShellHandler: `rust-opencode-port/crates/tui/src/shell_handler.rs`
- TUI PRD: `TUI_PRD.md`
- Current Constitution: `outputs/.specify/memory/constitution.md`
