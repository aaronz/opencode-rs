## ADDED Requirements

### Requirement: command-learn
Extract non-obvious learnings from session to AGENTS.md files.

#### Scenario: Create AGENTS.md learnings
- **WHEN** user invokes the learn command
- **THEN** system analyzes the session for discoveries, errors, unexpected connections
- **AND** determines the scope for each learning (project-wide, package/module-specific, feature-specific)
- **AND** reads existing AGENTS.md files at relevant levels
- **AND** creates or updates AGENTS.md at appropriate level with 1-3 line insights
- **AND** summarizes which AGENTS.md files were created/updated

#### Scenario: What counts as learning
- **WHEN** recording a learning
- **THEN** include: hidden relationships, execution paths, non-obvious config/env vars, debugging breakthroughs, API quirks, build/test commands, architectural decisions
- **AND** exclude: obvious facts, standard behavior, things already in AGENTS.md, verbose explanations, session-specific details
