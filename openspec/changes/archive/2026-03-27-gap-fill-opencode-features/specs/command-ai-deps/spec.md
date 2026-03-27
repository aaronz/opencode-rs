## ADDED Requirements

### Requirement: command-ai-deps
Analyze AI SDK dependencies for minor/patch updates.

#### Scenario: Report updatable dependencies
- **WHEN** user invokes the ai-deps command
- **THEN** system reads package.json and packages/opencode/package.json
- **AND** identifies AI SDK dependencies that can be upgraded (minor/patch only, no major)
- **AND** provides version upgrade info, changelog summary, and references for each dependency
- **AND** writes findings to ai-sdk-updates.md
- **AND** does NOT upgrade dependencies yet, only reports
