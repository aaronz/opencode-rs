## ADDED Requirements

### Requirement: Offset-based History Retrieval
The storage and session services SHALL support retrieving a subset of message history using `limit` and `offset` parameters for efficient UI rendering and agent context management.

#### Scenario: Paging history
- **WHEN** the UI requests the latest 10 messages with an offset of 20
- **THEN** the system MUST return exactly messages 21-30 from the session history.

### Requirement: Smart Compaction Integration
The compaction logic SHALL be integrated into the session lifecycle, automatically triggering when token counts exceed the provider's threshold while preserving essential context (system prompt, latest turns).

#### Scenario: Auto-compacting on overflow
- **WHEN** an agent is about to send a prompt that exceeds the model's limit
- **THEN** the compactor MUST run first to prune the history while maintaining required context.
