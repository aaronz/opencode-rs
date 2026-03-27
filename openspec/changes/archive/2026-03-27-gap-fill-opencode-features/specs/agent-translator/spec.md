## ADDED Requirements

### Requirement: agent-translator
Translation agent for multilingual documentation.

#### Scenario: Translate documentation
- **WHEN** user invokes translator agent
- **THEN** system uses model opencode/kimi-k2.5 for translation
- **AND** translates documentation to target language
- **AND** maintains consistency with existing translations
