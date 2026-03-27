## ADDED Requirements

### Requirement: Token-aware Compaction
The system SHALL implement a session message compactor that is aware of LLM token limits and prioritizes preserving critical context.

#### Scenario: Compacting large session
- **WHEN** a session exceeds the configured token limit
- **THEN** the compactor SHALL remove older messages while keeping system prompts and recent context to stay within the limit.

### Requirement: Message Pagination
The system SHALL support paginated retrieval of session messages for efficient memory and bandwidth usage.

#### Scenario: Listing messages with limit
- **WHEN** the user requests messages for a session with a `limit` and `offset`
- **THEN** the system MUST return only the requested slice of message history.
