## ADDED Requirements

### Requirement: e2e-config-tests
Tests for .opencode configuration loading.

#### Scenario: Loads commands from .opencode/command/
- **WHEN** opencode loads
- **THEN** command files are discovered and loaded

#### Scenario: Loads agents from .opencode/agent/
- **WHEN** opencode loads
- **THEN** agent definitions are discovered

#### Scenario: Loads skills from .opencode/skills/
- **WHEN** skill tool is invoked
- **THEN** skills are discovered from configured locations
