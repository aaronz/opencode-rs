## Why

The PRD requires TUI input syntax support (@file references, !shell commands, /slash commands) to provide a seamless command-line experience similar to Claude Code. Currently, these input patterns are not implemented, limiting user workflow efficiency. This feature is required for v1 release as specified in the PRD v1.1 gap analysis.

## What Changes

- **New @file Reference Syntax**: Support `@filename` to attach file contents to the conversation context
- **New !shell Command Syntax**: Support `!command` to execute shell commands inline
- **New /slash Command Syntax**: Support `/command` to invoke built-in TUI commands
- **Parser Implementation**: Create a unified input parser that detects and routes these different input types
- **Command Registry Enhancement**: Extend existing command registry to support dynamic command registration

## Capabilities

### New Capabilities

- `tui-input-parser`: Core parser for detecting @, !, / prefixes and routing to appropriate handlers
- `file-reference`: Ability to read and attach file contents using @filename syntax
- `inline-shell`: Ability to execute shell commands using !command syntax
- `slash-commands`: Support for /command syntax to invoke built-in TUI commands

### Modified Capabilities

- `skill-registry`: May need extension to support command-style skill invocation
- `session-management`: Session state may need to include command history with different types

## Impact

- **Rust Implementation**: New parser module in `rust-opencode-port/`
- **Frontend**: TUI component updates for input handling
- **Commands**: 3-5 new built-in commands to implement (/help, /clear, /retry, etc.)
- **Documentation**: User guide for TUI input syntax
