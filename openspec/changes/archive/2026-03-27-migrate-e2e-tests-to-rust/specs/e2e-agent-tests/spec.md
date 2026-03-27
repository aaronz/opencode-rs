## ADDED Requirements

### Requirement: e2e-agent-tests
Tests for agent system functionality.

#### Scenario: Build agent executes tools
- **WHEN** build agent processes a request
- **THEN** tools are executed correctly

#### Scenario: Plan agent is read-only
- **WHEN** plan agent tries to edit
- **THEN** edit is denied

#### Scenario: Agent handles tool results
- **WHEN** agent receives tool results
- **THEN** results are processed correctly
