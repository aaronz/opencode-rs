## Context

The TUI (Terminal User Interface) needs to support multiple input syntax patterns to match user workflow expectations from other AI coding assistants. Currently, the input field only supports plain text. The gap analysis identified this as a High priority item for v1 release.

**Current State:**
- Input field accepts plain text only
- No special syntax handling for file references, shell commands, or commands
- Command registry exists but lacks dynamic invocation

**Constraints:**
- Must maintain backward compatibility with existing plain text input
- Parser must be efficient for real-time typing
- Must integrate with existing permission system for shell commands
- Must work in both interactive TUI and potential future server modes

## Goals / Non-Goals

**Goals:**
1. Implement `@filename` syntax to attach file contents to conversation context
2. Implement `!command` syntax to execute shell commands inline
3. Implement `/command` syntax for built-in TUI commands
4. Create unified parser that detects and routes input types
5. Support command history with different input types

**Non-Goals:**
- Full REPL functionality (out of scope)
- Interactive shell with persistent state (future consideration)
- MCP command syntax (covered by separate MCP gap)
- UI/UX redesign of input component (existing design is sufficient)

## Decisions

### 1. Parser Architecture: Unified vs Separate

**Decision:** Unified parser with type enum routing

**Rationale:** 
- Single pass through input is more efficient
- Consistent error handling across all input types
- Easier to extend with new prefixes in the future
- Clear separation between detection and execution phases

**Alternatives Considered:**
- Separate handlers per prefix: Would require input duplication, harder to maintain
- Regex-based detection: Less performant, harder to handle edge cases

### 2. File Reference Resolution

**Decision:** Relative to current working directory, with optional absolute path support

**Rationale:**
- Matches user expectations from other tools
- CWD is well-defined in the TUI context
- Can be extended to support workspace-relative paths later

**Resolution Order:**
1. Absolute path (if starts with `/` or `C:\`)
2. Relative to current working directory
3. Relative to workspace root (if configured)

### 3. Shell Command Execution

**Decision:** Reuse existing permission system and bash/pty tools

**Rationale:**
- Avoids duplicating permission logic
- Consistent with how shell commands already work
- Permission prompts will naturally integrate with existing flow

### 4. Command Registry Integration

**Decision:** Extend existing command registry with prefix-based routing

**Rationale:**
- Leverages existing infrastructure
- Commands can be registered once and invoked via both `/` and potential voice/input methods
- Consistent with skill registry patterns already in codebase

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Input parsing latency | Medium | Parser runs in separate async task, no UI blocking |
| File permission bypass | High | All file reads go through existing permission system |
| Shell command injection | High | Force permission prompt for every `!` command |
| Command prefix conflicts | Low | `/` is reserved for commands only, fail gracefully for unknown commands |
| Unicode in filenames | Low | Use Rust's `std::path` which handles Unicode natively |

## Migration Plan

**Phase 1: Parser Foundation**
1. Create `InputParser` module in `crates/tui`
2. Define `InputType` enum (Plain, FileRef, Shell, Command)
3. Implement prefix detection logic

**Phase 2: Feature Implementation**
1. Implement file reference handler with permission check
2. Integrate shell execution with existing tools
3. Register built-in commands (/help, /clear, /retry, /model, /context)

**Phase 3: Testing & Polish**
1. Add integration tests for each input type
2. Update documentation and help text
3. User acceptance testing

**Rollback:** Feature flag controlled. Can disable via config without breaking existing plain text input.

## Open Questions

1. **Should `@` support glob patterns?** (e.g., `@*.ts`) - Deferred to v1.5
2. **Should `!` support chaining?** (e.g., `!cmd1 | cmd2`) - Use shell's own chaining, not our syntax
3. **Should `/` commands be case-insensitive?** - Yes, for usability
4. **How to handle quoted arguments?** (e.g., `/command "arg with space"`) - Future enhancement
