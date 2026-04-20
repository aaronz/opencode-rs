# PRD: agent Module

## Module Overview

**Module Name:** `agent`
**Type:** Core
**Source:** `/packages/opencode/src/agent/`

## Purpose

Core AI agent implementation that orchestrates the interaction between tools, providers, and session management. This is the main brain of the OpenCode agent.

## Functionality

### Core Features

1. **Agent Loop**
   - Main event loop that processes user messages and generates responses
   - Handles tool execution orchestration
   - Manages conversation flow between user and LLM

2. **Tool Orchestration**
   - Coordinates tool execution based on LLM decisions
   - Handles tool result processing and feedback to LLM
   - Manages parallel tool execution when appropriate

3. **Context Management**
   - Maintains agent context across interactions
   - Handles context window management and truncation
   - Provides context to LLM for decision making

### API Surface

```typescript
// Main interfaces
interface Agent {
  run(input: AgentInput): Promise<AgentOutput>
  stop(): void
  getState(): AgentState
}

interface AgentInput {
  session: Session
  message: string
  tools: Tool[]
}

interface AgentOutput {
  response: string
  toolCalls?: ToolCall[]
  state: AgentState
}
```

### Data Structures

- `AgentState` - Current state of the agent
- `AgentConfig` - Agent configuration options
- `AgentContext` - Running context with session data

### Dependencies

- `session` - Session management
- `tool` - Tool registry and execution
- `provider` - LLM provider selection
- `config` - Configuration access

### Configuration

```json
{
  "agent": {
    "maxIterations": 100,
    "timeout": 300000,
    "tools": ["read", "write", "bash", "grep", "glob"]
  }
}
```

## Implementation Notes

- Uses Effect-based error handling for reliability
- Supports streaming responses where provider allows
- Implements proper cancellation support
- Handles concurrent tool execution safely

## Acceptance Criteria

1. Agent can process user messages and generate appropriate responses
2. Tool execution is properly orchestrated based on LLM decisions
3. Context is maintained correctly across interactions
4. Errors are handled gracefully with appropriate fallbacks
5. Agent state is properly tracked and persisted

## Rust Implementation Guidance

The Rust equivalent should:
- Use `tokio` for async runtime
- Implement proper state machines for agent loop
- Use channel-based communication for tool execution
- Consider using `Effect` pattern or similar for error handling
