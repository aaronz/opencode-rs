## ADDED Requirements

### Requirement: command-rmslop
Remove AI code slop from diff against dev branch.

#### Scenario: Remove AI slop
- **WHEN** user invokes the rmslop command
- **THEN** system checks diff against dev branch
- **AND** identifies and removes AI generated slop including:
  - Extra comments a human wouldn't add
  - Extra defensive checks or try/catch blocks
  - Casts to any for type workarounds
  - Inconsistent style with rest of file
  - Unnecessary emoji usage
- **AND** provides a 1-3 sentence summary of changes
