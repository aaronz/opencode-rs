## ADDED Requirements

### Requirement: TUI application state
The system SHALL manage TUI application state.

#### Scenario: Application startup
- **WHEN** application launches in TUI mode
- **THEN** main application state is initialized with default values

#### Scenario: Input mode
- **WHEN** user is typing a message
- **THEN** input mode is active with cursor visible

#### Scenario: Command mode
- **WHEN** user presses Escape
- **THEN** input is cleared and command mode is activated

### Requirement: Message display
The system SHALL display conversation messages in the TUI.

#### Scenario: User message display
- **WHEN** user sends a message
- **THEN** message is displayed in user style (right-aligned or differentiated)

#### Scenario: Assistant message display
- **WHEN** assistant responds
- **THEN** response is displayed in assistant style with markdown rendering

#### Scenario: Streaming response
- **WHEN** assistant is generating a response
- **THEN** response is displayed incrementally as tokens arrive

### Requirement: Input area
The system SHALL provide an input area for user messages.

#### Scenario: Text input
- **WHEN** user types text
- **THEN** text appears in input area

#### Scenario: Submit message
- **WHEN** user presses Enter
- **THEN** message is submitted and input is cleared

#### Scenario: Multi-line input
- **WHEN** user presses Shift+Enter
- **THEN** new line is inserted in input

### Requirement: Command palette
The system SHALL provide a command palette for quick actions.

#### Scenario: Open command palette
- **WHEN** user presses Ctrl+P
- **THEN** command palette opens with available commands

#### Scenario: Execute command
- **WHEN** user selects and executes a command
- **THEN** command is executed and palette closes

### Requirement: Tool output panel
The system SHALL display tool execution results.

#### Scenario: Tool execution
- **WHEN** tool is executing
- **THEN** loading indicator is shown

#### Scenario: Tool result
- **WHEN** tool completes
- **THEN** result is displayed in output panel

#### Scenario: Tool error
- **WHEN** tool fails
- **THEN** error message is displayed in output panel

### Requirement: Status bar
The system SHALL display status information.

#### Scenario: Show current agent
- **WHEN** agent is active
- **THEN** status bar shows current agent name

#### Scenario: Show provider
- **WHEN** LLM provider is connected
- **THEN** status bar shows provider and model name

### Requirement: Keyboard shortcuts
The system SHALL support keyboard navigation.

#### Scenario: Agent switching
- **WHEN** user presses Tab
- **THEN** agent cycles between build and plan

#### Scenario: Scroll history
- **WHEN** user presses Up/Down in empty input
- **THEN** input cycles through command history

### Requirement: Window resize handling
The system SHALL handle terminal window resizing.

#### Scenario: Resize event
- **WHEN** terminal window is resized
- **THEN** TUI layout adapts to new dimensions
