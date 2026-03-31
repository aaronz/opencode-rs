## ADDED Requirements

### Requirement: LSP Diagnostics Retrieval

The LSP tool SHALL retrieve diagnostics for a given file or workspace.

#### Scenario: Get file diagnostics
- **WHEN** tool is called with action "diagnostics" and file path
- **THEN** tool returns array of diagnostics for that file

#### Scenario: Get workspace diagnostics
- **WHEN** tool is called with action "diagnostics" and no file path
- **THEN** tool returns diagnostics for all open files

#### Scenario: No diagnostics available
- **WHEN** LSP server has no diagnostics
- **THEN** tool returns empty array

### Requirement: Diagnostic Format

The tool SHALL return diagnostics in a structured format.

#### Scenario: Error diagnostic
- **WHEN** LSP reports an error
- **THEN** tool returns `{"severity": "error", "message": "...", "range": {...}, "source": "..."}`

#### Scenario: Warning diagnostic
- **WHEN** LSP reports a warning
- **THEN** tool returns `{"severity": "warning", "message": "...", "range": {...}, "source": "..."}`

#### Scenario: Multiple diagnostics
- **WHEN** file has multiple issues
- **THEN** tool returns array sorted by line number

### Requirement: LSP Tool Actions

The LSP tool SHALL support multiple LSP actions.

#### Scenario: Hover action
- **WHEN** tool is called with action "hover" and position
- **THEN** tool returns hover information (type, documentation)

#### Scenario: Definition action
- **WHEN** tool is called with action "definition" and position
- **THEN** tool returns definition location(s)

#### Scenario: References action
- **WHEN** tool is called with action "references" and position
- **THEN** tool returns all references to the symbol

#### Scenario: Symbols action
- **WHEN** tool is called with action "symbols" and file path
- **THEN** tool returns document symbols (classes, functions, etc.)

### Requirement: LSP Server Lifecycle

The tool SHALL manage LSP server lifecycle.

#### Scenario: Auto-start LSP server
- **WHEN** tool is called and no LSP server is running
- **THEN** tool starts appropriate LSP server for the file type

#### Scenario: LSP server timeout
- **WHEN** LSP server doesn't respond within 10 seconds
- **THEN** tool returns timeout error

#### Scenario: LSP server crash
- **WHEN** LSP server crashes during operation
- **THEN** tool returns error and attempts to restart server

### Requirement: LSP Tool Configuration

The tool SHALL respect LSP configuration settings.

#### Scenario: Custom LSP server
- **WHEN** user configures custom LSP server for file type
- **THEN** tool uses configured server instead of default

#### Scenario: Disabled LSP
- **WHEN** LSP is disabled in configuration
- **THEN** tool returns "LSP disabled" error
