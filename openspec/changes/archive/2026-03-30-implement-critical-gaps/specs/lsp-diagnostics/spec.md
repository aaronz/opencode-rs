## ADDED Requirements

### Requirement: LSP diagnostics provides code quality analysis
The system SHALL provide LSP-based diagnostics for code quality checks.

#### Scenario: Request diagnostics for open file
- **WHEN** user requests diagnostics for a file
- **THEN** system SHALL:
  - Connect to appropriate LSP server for file type
  - Request diagnostics via textDocument/publishDiagnostics
  - Return all available diagnostics

#### Scenario: Receive diagnostics from LSP server
- **WHEN** LSP server publishes diagnostics
- **THEN** system SHALL:
  - Parse diagnostic information
  - Include severity (error, warning, info, hint)
  - Include line and column information
  - Include diagnostic message

#### Scenario: Multiple LSP servers
- **WHEN** file requires multiple LSP servers (e.g., template + TypeScript)
- **THEN** system SHALL aggregate diagnostics from all servers

### Requirement: Diagnostics updates automatically
The system SHALL provide real-time diagnostic updates.

#### Scenario: Diagnostics on file save
- **WHEN** user saves a file
- **THEN** system SHALL trigger new diagnostic request

#### Scenario: Diagnostics on file change
- **WHEN** user modifies open file
- **THEN** system SHALL request updated diagnostics (debounced)

### Requirement: Diagnostics integrates with tools
The diagnostics system SHALL integrate with existing tools.

#### Scenario: Access diagnostics via lsp tool
- **WHEN** user calls lsp tool with diagnostic options
- **THEN** tool SHALL return current diagnostics for the file

#### Scenario: Diagnostics available to agents
- **WHEN** ReviewAgent needs code quality information
- **THEN** agent SHALL be able to query diagnostics system

### Requirement: Handle LSP server lifecycle
The system SHALL manage LSP server connections properly.

#### Scenario: LSP server not available
- **WHEN** LSP server fails to start or crashes
- **THEN** system SHALL:
  - Attempt restart
  - Notify user of diagnostic unavailability
  - Continue without blocking operation

#### Scenario: Language not supported
- **WHEN** user requests diagnostics for unsupported language
- **THEN** system SHALL return message indicating language not supported
