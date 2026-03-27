## ADDED Requirements

### Requirement: agent-docs
Technical documentation writing agent.

#### Scenario: Write documentation
- **WHEN** user invokes docs agent
- **THEN** system uses relaxed and friendly tone
- **AND** writes concise documentation (chunks ≤ 2 sentences)
- **AND** uses imperative mood for section titles
- **AND** follows docs style: 3-dash dividers, short titles, no "The" in description
- **AND** removes trailing semicolons from JS/TS code snippets

#### Scenario: Commit documentation
- **WHEN** making a commit for docs
- **THEN** system prefixes commit message with "docs:"
