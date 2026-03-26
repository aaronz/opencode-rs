## ADDED Requirements

### Requirement: Agent trait definition
The system SHALL provide an Agent trait that defines the interface for all agents.

#### Scenario: Agent trait implementation
- **WHEN** a new agent type implements the Agent trait
- **THEN** the agent can be registered and invoked by the agent system

### Requirement: Build agent (default)
The system SHALL provide a build agent with full file system and command execution capabilities.

#### Scenario: Default agent selection
- **WHEN** user starts a new session without specifying agent
- **THEN** the build agent is selected by default

#### Scenario: Full file access
- **WHEN** build agent processes a request to modify files
- **THEN** agent can read, write, create, and delete files in allowed directories

#### Scenario: Command execution
- **WHEN** build agent needs to run shell commands
- **THEN** agent can execute arbitrary shell commands with user confirmation

### Requirement: Plan agent (read-only)
The system SHALL provide a plan agent with read-only capabilities for exploration.

#### Scenario: Plan agent activation
- **WHEN** user invokes `/plan` or switches to plan agent
- **THEN** agent operates in read-only mode

#### Scenario: File read access
- **WHEN** plan agent requests to read files
- **THEN** agent can read files but cannot modify them

#### Scenario: Edit request denied
- **WHEN** plan agent attempts to modify a file
- **THEN** agent receives a denial message explaining read-only mode

#### Scenario: Bash command confirmation
- **WHEN** plan agent requests to run a bash command
- **THEN** user is prompted for confirmation before execution

### Requirement: General subagent
The system SHALL provide a general subagent for complex multi-step searches.

#### Scenario: Subagent invocation
- **WHEN** user mentions `@general` in message
- **THEN** the general subagent is invoked for complex search tasks

### Requirement: Agent switching
The user SHALL be able to switch between agents during a session.

#### Scenario: Switch to plan agent
- **WHEN** user presses Tab or types `/plan`
- **THEN** active agent switches to plan mode

#### Scenario: Switch back to build
- **WHEN** user presses Tab again or types `/build`
- **THEN** active agent switches back to build mode
