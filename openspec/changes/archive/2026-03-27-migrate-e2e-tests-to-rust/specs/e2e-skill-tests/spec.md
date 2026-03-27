## ADDED Requirements

### Requirement: e2e-skill-tests
Tests for skill loading and execution.

#### Scenario: Skill discovery finds skills
- **WHEN** discovering skills in .opencode/skills/
- **THEN** all skill directories are found

#### Scenario: Skill execution returns content
- **WHEN** executing a skill
- **THEN** skill content is returned

#### Scenario: Skill with files includes attachments
- **WHEN** skill has additional files
- **THEN** file paths are included in output
