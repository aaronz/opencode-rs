## 1. Parser Foundation

- [x] 1.1 Create InputParser module in crates/tui/src/
- [x] 1.2 Define InputType enum (Plain, FileRef, Shell, Command)
- [x] 1.3 Implement prefix detection logic
- [x] 1.4 Implement ParseResult struct with type/content/args fields
- [x] 1.5 Add unit tests for parser edge cases
- [x] 1.6 Integrate parser with TUI input component

## 2. File Reference Implementation

- [x] 2.1 Create FileRefHandler struct
- [x] 2.2 Implement file path resolution (relative, absolute, workspace)
- [x] 2.3 Integrate with existing permission system
- [x] 2.4 Add binary file detection
- [x] 2.5 Add file size limit check (1MB default)
- [x] 2.6 Implement content formatting with file header
- [x] 2.7 Add unit and integration tests
- [x] 2.8 Handle multiple file references in single input

## 3. Inline Shell Implementation

- [x] 3.1 Create ShellHandler struct
- [x] 3.2 Integrate with existing bash/pty tools
- [x] 3.3 Integrate with existing permission system
- [x] 3.4 Implement stdout/stderr capture
- [x] 3.5 Add exit code handling
- [x] 3.6 Implement output truncation (100KB limit)
- [x] 3.7 Add timeout handling (30 seconds default)
- [x] 3.8 Add unit and integration tests

## 4. Slash Commands Implementation

- [x] 4.1 Create CommandRegistry in crates/tui/src/
- [x] 4.2 Define Command trait with execute method
- [x] 4.3 Implement /help command
- [x] 4.4 Implement /clear command
- [x] 4.5 Implement /retry command
- [x] 4.6 Implement /model command
- [x] 4.7 Implement /context command
- [x] 4.8 Implement /quit and /exit commands
- [x] 4.9 Add case-insensitive command matching
- [x] 4.10 Add argument parsing (quoted args, multiple args)
- [x] 4.11 Add dynamic command registration API
- [x] 4.12 Add unknown command error handling

## 5. Integration & Testing

- [x] 5.1 Integrate all handlers with main TUI loop
- [x] 5.2 Add integration tests for full input flow
- [x] 5.3 Test permission system integration
- [x] 5.4 Test command history with different input types
- [x] 5.5 Add error handling for all edge cases
- [x] 5.6 Update TUI documentation
- [x] 5.7 Run existing test suite to ensure no regressions

## 6. Polish & Documentation

- [x] 6.1-6.5 Core feature complete
