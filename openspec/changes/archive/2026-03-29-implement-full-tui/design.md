## Context

The current TUI implementation uses a simple mode-based architecture with direct `Ctrl+<Key>` bindings. To match the design document's UX vision, we need a state-machine-based event handling system with leader keys, floating overlays, and tokenized input elements. The existing `ratatui` + `crossterm` stack is capable of supporting all required features without additional dependencies.

### Current State
- **Input**: Basic string with cursor, no chip/token support
- **Commands**: Centered palette with static list
- **Tool Output**: Appended as plain text, no collapsing
- **Scrolling**: PageUp/PageDown with fixed offset
- **Shortcuts**: Direct Ctrl+<Key> combinations
- **Theme**: Single theme, hex color support

### Constraints
- Must work with existing `ratatui` v0.28 and `crossterm` v0.28
- Must preserve existing dialog system (Settings, Models, etc.)
- Must not break existing keyboard shortcuts during migration

## Goals / Non-Goals

**Goals:**
1. Implement leader key state machine for Tmux-style navigation
2. Add floating autocomplete for slash commands (`/`)
3. Create tokenized file reference "chips" with atomic deletion
4. Implement diff review loop with Y/N/E confirmation
5. Add accordion-style tool execution output
6. Support external editor integration
7. Implement smooth scrolling with acceleration
8. Add SIGINT interruption for LLM generation

**Non-Goals:**
- Multi-platform drag-and-drop (terminal limitation)
- Full mouse support (keyboard-first design)
- Custom syntax highlighting engine (use existing `syntect` or similar)
- Real-time collaboration features

## Decisions

### Decision 1: Leader Key State Machine

**Approach**: Implement a `LeaderKeyState` enum in the App struct.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum LeaderKeyState {
    Idle,
    WaitingForAction,  // ctrl+x pressed, waiting for next key
}
```

**Rationale**: Tmux-style leader keys avoid conflicts with terminal shortcuts. The 2-second timeout prevents accidental activation.

**Alternatives Considered**:
- Direct Ctrl+<Key> (current) - Causes conflicts with terminal shortcuts
- Modal editing (Vim-style) - Too complex for this use case
- Chord sequences (Ctrl+X then Ctrl+C) - Less discoverable

### Decision 2: Input Tokenization (Chips)

**Approach**: Extend `InputWidget` to support `InputElement` enum:

```rust
#[derive(Debug, Clone)]
pub enum InputElement {
    Text(String),
    Chip { display: String, value: String, color: Color },
}
```

**Rationale**: Atomic deletion is critical UX. Chips must be deleted as units, not character-by-character.

**Alternatives Considered**:
- Inline markers (like `@file.rs`) - Harder to style, no atomic deletion
- Separate context list - Breaks visual flow
- Rich text spans - Overkill for this use case

### Decision 3: Floating Overlay Architecture

**Approach**: Use a `OverlayStack` to manage modal layers:

```rust
pub struct OverlayStack {
    overlays: Vec<Overlay>,
}

pub enum Overlay {
    SlashCommands { input: String, filtered: Vec<Command> },
    FilePicker { input: String, filtered: Vec<PathBuf> },
    DiffReview { diff: DiffHunk, state: ReviewState },
    ToolAccordion { tool_calls: Vec<ToolCall> },
}
```

**Rationale**: Z-indexed overlays allow stacking (e.g., file picker over chat). Backdrop dimming provides visual hierarchy.

**Alternatives Considered**:
- Single overlay (no stacking) - Too restrictive
- Full modal system - Overkill, complex state management
- Widget-based (no overlay) - Can't achieve visual hierarchy

### Decision 4: Diff Review Loop

**Approach**: Intercept diff output before file write, render as overlay:

```rust
pub enum DiffState {
    Pending(DiffHunk),      // Waiting for user confirmation
    Accepted(DiffHunk),     // User pressed Y
    Rejected(DiffHunk),     // User pressed N
    Editing(DiffHunk),      // User pressed E, external editor open
}
```

**Rationale**: Users must approve code changes. The Y/N/E workflow is standard in code review tools.

**Alternatives Considered**:
- Auto-apply (no review) - Dangerous, could break code
- Side-by-side only - Too much space in terminal
- Git-style hunks - Too complex for this use case

### Decision 5: Tool Accordion UI

**Approach**: Transform `tool_output` from `Vec<String>` to `Vec<ToolCall>`:

```rust
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,           // "grep", "read_file", etc.
    pub status: ToolStatus,     // Running, Success, Failed
    pub output: String,         // Full output (collapsed by default)
    pub expanded: bool,         // User toggled with /details
    pub start_time: Instant,    // For duration display
}

pub enum ToolStatus {
    Running,
    Success,
    Failed(i32),  // Exit code
}
```

**Rationale**: Collapse by default reduces noise. Expand on demand for debugging. Status indicators (✔/✖/⠋) provide at-a-glance feedback.

**Alternatives Considered**:
- Always expanded - Too noisy
- Hide completely - No debugging visibility
- Separate pane - Breaks conversation flow

### Decision 6: Smooth Scrolling

**Approach**: Implement scroll acceleration with momentum:

```rust
pub struct ScrollState {
    offset: usize,
    velocity: f64,      // Current scroll speed
    acceleration: f64,  // How much velocity increases per tick
    max_velocity: f64,  // Cap to prevent overshooting
}
```

**Rationale**: macOS-style scrolling feels natural. Acceleration prevents "jerky" scrolling on long content.

**Alternatives Considered**:
- Fixed step scrolling (current) - Feels mechanical
- Page-based - Too coarse
- Pixel-level - Not possible in terminal

### Decision 7: Theme System Expansion

**Approach**: Add preset themes and runtime switching:

```rust
pub struct ThemeManager {
    current: Theme,
    presets: HashMap<String, Theme>,  // catppuccin, tokyonight, etc.
}
```

**Rationale**: Developers expect theme variety. Presets provide instant visual customization.

**Alternatives Considered**:
- Single theme - Too limiting
- CSS-like theming - Overkill for terminal
- Plugin-based themes - Too complex

## Risks / Trade-offs

### Risk 1: Leader Key Conflicts
**Risk**: Some terminals intercept `ctrl+x` for other purposes.
**Mitigation**: Make leader key configurable in settings. Document known conflicts.

### Risk 2: Performance with Many Chips
**Risk**: Rendering many colored chips could impact performance.
**Mitigation**: Limit chips to ~20 per input. Cache rendered spans.

### Risk 3: External Editor Integration
**Risk**: Different editors have different behaviors (vim vs nano vs code).
**Mitigation**: Use `$EDITOR` environment variable. Handle missing editor gracefully.

### Risk 4: Scroll Acceleration Complexity
**Risk**: Acceleration math could cause overshooting or weird behavior.
**Mitigation**: Cap max velocity. Test with various terminal sizes.

### Trade-off: Breaking Keyboard Shortcuts
**Trade-off**: Moving from Ctrl+<Key> to leader key will break muscle memory.
**Acceptance**: Provide migration guide. Keep old shortcuts as aliases during transition.

## Migration Plan

### Phase 1: Leader Key Foundation (Low Risk)
1. Add `LeaderKeyState` to `App`
2. Implement leader key timeout logic
3. Migrate existing Ctrl+<Key> to leader key
4. Add fallback aliases for backward compatibility

### Phase 2: Input Enhancement (Medium Risk)
1. Extend `InputWidget` with `InputElement` enum
2. Implement chip rendering
3. Add atomic deletion for chips
4. Test with various terminal widths

### Phase 3: Overlay System (Medium Risk)
1. Create `OverlayStack` structure
2. Migrate existing dialogs to overlay system
3. Implement slash command autocomplete
4. Add file picker overlay

### Phase 4: Tool & Diff Enhancements (Higher Risk)
1. Transform `tool_output` to `Vec<ToolCall>`
2. Implement accordion UI
3. Create diff review state machine
4. Add external editor integration

### Phase 5: Polish (Low Risk)
1. Add preset themes
2. Implement smooth scrolling
3. Add SIGINT handling
4. Performance optimization

### Rollback Strategy
- Each phase is independently deployable
- Feature flags for new behaviors
- Keep old keyboard shortcuts as aliases

## Open Questions

1. **Leader Key Timeout**: Should timeout be configurable? (Proposed: 2000ms default)
2. **Chip Limit**: Maximum chips per input? (Proposed: 20)
3. **Scroll Acceleration**: Exact acceleration curve? (Proposed: linear with cap)
4. **External Editor**: Should we support multiple editors? (Proposed: $EDITOR only)
5. **Theme Presets**: Which themes to include? (Proposed: catppuccin, tokyonight, nord, gruvbox)
