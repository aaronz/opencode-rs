## ADDED Requirements

### Requirement: Build Agent Mode
The build agent SHALL have full access to all tools including file modification and command execution.

#### Scenario: Build Agent Has Full Access
- **WHEN** user runs with --agent build
- **THEN** agent can use all tools including read, write, grep, bash, edit

#### Scenario: Build Agent Modifies Files
- **WHEN** build agent decides to modify a source file
- **THEN** file is modified with the agent's changes

#### Scenario: Build Agent Executes Commands
- **WHEN** build agent needs to run a build command
- **THEN** command is executed and output is returned to agent

### Requirement: Plan Agent Mode
The plan agent SHALL have read-only access to tools without modification capabilities.

#### Scenario: Plan Agent Read-Only
- **WHEN** user runs with --agent plan
- **THEN** agent can use read, grep, but not write, bash, edit

#### Scenario: Plan Agent Suggests Without Executing
- **WHEN** plan agent would modify a file
- **THEN** suggestion is returned instead of execution

### Requirement: General Agent Mode
The general agent SHALL have search and exploration capabilities.

#### Scenario: General Agent Search
- **WHEN** user runs with --agent general
- **THEN** agent can use grep, read, and web search tools

### Requirement: Agent Tool Execution Loop
The agent SHALL iteratively call tools based on LLM responses until task is complete.

#### Scenario: Agent Calls Tool
- **WHEN** LLM response contains a tool call
- **THEN** tool is executed and result is fed back to LLM

#### Scenario: Agent Receives Tool Result
- **WHEN** tool execution completes
- **THEN** result is added to conversation for next LLM call

#### Scenario: Agent Task Complete
- **WHEN** LLM response contains final answer without tool calls
- **THEN** response is returned to user and session is updated