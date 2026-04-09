# PRD: TUI (Terminal User Interface)

## Overview

OpenCode's TUI provides an interactive terminal interface for conversations with AI agents.

---

## Layout Structure

```
┌─────────────────────────────────────────────────────┐
│  [Title Bar / Status]                               │
├─────────────────────────────────────────────────────┤
│                                                     │
│  [Session Messages - scrollable]                    │
│                                                     │
│                                                     │
├─────────────────────────────────────────────────────┤
│  [Prompt Input Area]                                │
└─────────────────────────────────────────────────────┘
     [Sidebar - toggleable]
```

---

## Slash Commands

All commands start with `/`:

| Command | Alias | Description |
|---------|-------|-------------|
| `/compact` | `/summarize` | Compact session context |
| `/connect` | | Add provider credentials |
| `/details` | | Toggle tool details |
| `/editor` | | Open external editor |
| `/exit` | `/quit`, `/q` | Exit OpenCode |
| `/export` | | Export conversation |
| `/help` | | Show help |
| `/init` | | Create AGENTS.md |
| `/models` | | List models |
| `/new` | `/clear` | New session |
| `/redo` | | Redo undone |
| `/sessions` | `/resume`, `/continue` | List sessions |
| `/share` | | Share session |
| `/themes` | | List themes |
| `/thinking` | | Toggle thinking visibility |
| `/undo` | | Undo last message |
| `/unshare` | | Unshare session |

---

## Input Model

### File Reference

Use `@` to reference files with fuzzy search:

```
@packages/functions/src/api/index.ts
```

### Shell Commands

Prefix with `!` to execute as shell:

```
!ls -la
```

### External Editor

Use `/editor` to open `$EDITOR`. The keybinding to invoke the editor is configurable through TUI configuration.

---

## Keybindings

### Leader Key

Keybindings are organized by a **leader key** prefix. The default leader key is configurable through TUI configuration.

### Keybinding Categories

**Session:**
- `session_new` — New session
- `session_list` — List sessions
- `session_fork` — Fork session
- `session_rename` — Rename session
- `session_share` — Share session
- `session_interrupt` — Abort running
- `session_compact` — Compact context
- `session_child_*` — Navigate child sessions

**Messages:**
- `messages_page_up/down`
- `messages_line_up/down`
- `messages_copy`
- `messages_undo`
- `messages_redo`

**Model:**
- `model_list`
- `model_cycle_recent`
- `variant_cycle`

**Commands:**
- `command_list`
- `agent_list`
- `agent_cycle`

**Input:**
- `input_clear`, `input_submit`, `input_newline`
- `input_move_*`, `input_select_*`
- `input_delete_*`, `input_undo`, `input_redo`
- `input_word_forward/backward`

**Terminal:**
- `terminal_suspend`
- `terminal_title_toggle`

### Disabling Keybindings

Individual keybindings can be disabled via TUI configuration.

---

## Views

### Session View

Shows conversation with AI. Features:
- Markdown rendering
- Syntax highlighting for code blocks
- Diff display for file changes
- Tool execution details (collapsible)
- Thinking blocks (toggleable)

### Home View

Shows when no session is active:
- Recent sessions
- Quick actions
- Tips

---

## Mode Indicator

TUI shows current mode in bottom-right:
- Build mode indicator
- Plan mode indicator
- Agent type

---

## Prompt Features

### Multi-line Input

`shift+enter` for new line (when terminal supports it).

### History

Up/Down arrows cycle through input history.

### Autocomplete

- Tab completion for slash commands
- `@` for file references
- Tool names

---

## Sidebar

The sidebar is toggleable and shows context relevant to the current session:
- File tree
- MCP server status
- LSP diagnostics
- Todo items

Sidebar visibility is controlled by a keybinding.

---

## Configuration

TUI-specific configuration (`tui.json`) is owned by the TUI subsystem. For ownership details, theme behavior, and TUI plugin configuration, see:

- [Configuration System](./06-configuration-system.md) — for `tui.json` ownership boundaries
- [TUI Plugin API](./15-tui-plugin-api.md) — for TUI plugin configuration and plugin API reference

---

## Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | `tui.json` ownership boundaries and variable expansion |
| [TUI Plugin API](./15-tui-plugin-api.md) | TUI plugin config, plugin runtime API |
| [Agent System](./02-agent-system.md) | Agent types and session agent behavior |
| [Tools System](./03-tools-system.md) | Tool execution and details display |
