# PRD: session Module

## Module Overview

**Module Name:** `session`
**Type:** Core
**Source:** `/packages/opencode/src/session/`

## Purpose

Conversation session management including message handling, prompt engineering, and context management. Handles the complete lifecycle of agent conversations.

## Functionality

### Core Features

1. **Session Management**
   - Create, load, save, and delete sessions
   - Session metadata tracking (created, modified, message count)
   - Session compaction for context management
   - Session persistence to SQLite

2. **Message Handling**
   - `message.ts` - Basic message types
   - `message-v2.ts` - New message format with richer metadata
   - Message validation and sanitization
   - Message attachment handling

3. **Prompt Engineering**
   - `prompt.ts` (72KB) - Extensive prompt templates
   - System prompt construction
   - User message formatting
   - Tool results formatting for LLM
   - Dynamic prompt injection based on context

4. **Context Management**
   - `compaction.ts` - Session compaction algorithms
   - `summary.ts` - Conversation summarization
   - Context window management
   - Token counting and预算 management

5. **LLM Interface**
   - `llm.ts` - LLM abstraction layer
   - Model selection and fallback
   - Response parsing and validation
   - Streaming support

6. **Message Processing**
   - `processor.ts` - Process LLM responses
   - Tool call extraction
   - Response validation
   - Error handling

7. **Retry Logic**
   - `retry.ts` - Retry configuration and execution
   - Exponential backoff
   - Rate limit handling

### API Surface

```typescript
// Session class
class Session {
  id: string
  messages: Message[]
  createdAt: Date
  updatedAt: Date

  addMessage(role: Role, content: string): void
  getMessages(): Message[]
  compact(): Promise<void>
  summarize(): Promise<string>
  toPrompt(): string
}

// Message types
interface Message {
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string | ContentPart[]
  toolCallId?: string
  toolName?: string
  timestamp: Date
}
```

### Key Files

| File | Purpose |
|------|---------|
| `session.ts` | Main session class |
| `message.ts` | Message type definitions |
| `message-v2.ts` | Extended message types |
| `prompt.ts` | Prompt generation (72KB) |
| `processor.ts` | Response processing |
| `llm.ts` | LLM interface |
| `compaction.ts` | Context compaction |
| `summary.ts` | Summarization |
| `retry.ts` | Retry logic |
| `instruction.ts` | Instruction handling |
| `projectors.ts` | Message projection |
| `status.ts` | Session status |
| `system.ts` | System messages |
| `todo.ts` | Todo tracking |
| `revert.ts` | Revert functionality |
| `run-state.ts` | Run state management |
| `overflow.ts` | Overflow handling |

### Data Storage

- Sessions stored in SQLite via `storage` module
- Message history with full metadata
- Compaction history for audit

### Dependencies

- `storage` - Database persistence
- `provider` - LLM calls
- `config` - Configuration
- `tool` - Tool definitions

## Implementation Notes

- Supports message streaming where provider allows
- Implements context window management (128K+ tokens)
- Automatic compaction when approaching limits
- Summarization for long conversations

## Acceptance Criteria

1. Sessions persist correctly to database
2. Messages are properly validated and stored
3. Prompt generation produces valid prompts for all providers
4. Compaction reduces context while preserving important information
5. Retry logic handles transient failures gracefully

## Rust Implementation Guidance

The Rust equivalent should:
- Use `rusqlite` for database
- Use `tokio` for async operations
- Implement message queue with proper ordering
- Consider using serde for serialization
- Handle large context windows efficiently
