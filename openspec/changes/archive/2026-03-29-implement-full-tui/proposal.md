## Why

The current TUI implementation in `rust-opencode-port/crates/tui/` provides basic functionality but lacks the rich, keyboard-driven interaction model described in `docs/design-tui.md`. Users expect a terminal UI that matches modern IDE productivity standards—including leader key navigation, slash commands, file reference chips, and diff review workflows. Implementing these features will transform the TUI from a basic chat interface into a professional-grade AI coding assistant terminal experience.

## What Changes

### Core Interaction Patterns
- **Leader Key Mechanism**: Tmux-style `ctrl+x` prefix for quick actions (compact, quit, editor, sessions, models, details)
- **Slash Commands with Floating Autocomplete**: Typing `/` shows a floating autocomplete menu with available commands
- **File Reference "Chips"**: Typing `@` opens fuzzy file finder; selected files render as colored chips with atomic deletion
- **Shell Integration**: Typing `!` executes commands directly with captured output sent to LLM

### Dual-Mode Workflow
- **Plan Mode**: Read-only mode for architectural suggestions
- **Build Mode**: Execution mode allowing file modifications
- **Tab Toggle**: Switch modes with visual indicator in status bar

### Visual & Rendering Enhancements
- **Truecolor Theme Support**: 24-bit color with themes (catppuccin, tokyonight, nord, gruvbox, system)
- **Diff Rendering**: Side-by-side or stacked diff views with syntax highlighting
- **Tool Execution "Accordion"**: Collapsible tool output with loading animations
- **Smooth Scrolling**: macOS-style scroll acceleration

### Advanced Interaction Features
- **Diff Review Loop**: Y/N/E confirmation before applying code changes
- **External Editor Integration**: `ctrl+x e` opens `$EDITOR` for complex prompts
- **SIGINT Interruption**: `Ctrl+C` gracefully stops LLM generation
- **History Roaming**: Up/Down arrows navigate prompt history when input is empty
- **Drag and Drop**: Image/file drop support in terminal

### Session Management
- **Multi-Session Support**: List, switch, and restore previous sessions
- **Context Compaction**: `/compact` summarizes long conversations to save tokens

## Capabilities

### New Capabilities

- `leader-key`: Tmux-style leader key state machine for keyboard shortcuts
- `slash-commands`: Floating autocomplete menu for `/` commands
- `file-chips`: Tokenized file references with atomic deletion behavior
- `dual-mode`: Plan/Build mode switching with Tab key
- `diff-review`: Code change review with Accept/Reject/Edit workflow
- `tool-accordion`: Collapsible tool execution output with status indicators
- `external-editor`: Integration with system `$EDITOR` for complex prompts
- `theme-system`: Truecolor theme support with multiple presets
- `smooth-scroll`: Accelerated scrolling for long content
- `sigint-handler`: Graceful interruption of LLM generation
- `history-roaming`: Context-aware prompt history navigation
- `drag-drop`: File and image drop support
- `session-manager`: Multi-session list, switch, and restore

### Modified Capabilities

- `input-widget`: Current input widget needs chip rendering and leader key awareness
- `status-bar`: Needs mode indicator, model display, and leader key hints
- `command-palette`: Replace current CommandPalette with slash command autocomplete

## Impact

### Affected Code
- `rust-opencode-port/crates/tui/src/app.rs` - Core event loop and state machine
- `rust-opencode-port/crates/tui/src/components/input_widget.rs` - Input handling and chip rendering
- `rust-opencode-port/crates/tui/src/components/status_bar.rs` - Mode indicator
- `rust-opencode-port/crates/tui/src/components/diff_view.rs` - Diff rendering enhancement
- `rust-opencode-port/crates/tui/src/dialogs/` - New dialogs for slash commands, sessions
- `rust-opencode-port/crates/tui/src/theme.rs` - Theme system expansion

### Dependencies
- May need `crossterm` event handling enhancements for leader key
- May need `unicode-width` for chip rendering
- May need `syntect` or similar for syntax highlighting in diffs
- May need `fuzzy-matcher` for file/command fuzzy search

### Breaking Changes
- **BREAKING**: Keyboard shortcuts change from direct `Ctrl+<Key>` to leader key pattern
- **BREAKING**: CommandPalette UI replaced with slash command autocomplete
