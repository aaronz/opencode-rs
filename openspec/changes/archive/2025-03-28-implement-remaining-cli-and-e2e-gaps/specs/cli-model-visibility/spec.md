## ADDED Requirements

### Requirement: Model visibility command
The CLI SHALL provide a `models visibility` command to control which models are visible.

#### Scenario: Hide a model
- **WHEN** user runs `opencode models visibility --hide <model-id>`
- **THEN** the model is marked as hidden
- **AND** it does not appear in model lists by default

#### Scenario: Show a model
- **WHEN** user runs `opencode models visibility --show <model-id>`
- **THEN** the model is marked as visible
- **AND** it appears in model lists

#### Scenario: List hidden models
- **WHEN** user runs `opencode models visibility --list-hidden`
- **THEN** all hidden models are displayed
- **AND** their visibility status is shown

### Requirement: Model filtering
The models command SHALL support filtering by visibility and provider.

#### Scenario: Filter by visibility
- **WHEN** user runs `opencode models --visibility visible`
- **THEN** only visible models are displayed

#### Scenario: Filter by provider and visibility
- **WHEN** user runs `opencode models --provider openai --visibility hidden`
- **THEN** only hidden OpenAI models are displayed
