# TUI AgentRuntime Integration Design

## Context

opencode-rs (Rust) has two conflicting architectures:
- **TUI**: Direct streaming via `provider.complete_streaming(&prompt, callback)` - text chunks only, no tool execution
- **AgentRuntime**: Proper agent loop with tool execution via `agent.run()` → `tool_calls` → execute → continue

opencode (TypeScript) uses event-based streaming via the `ai` SDK where `tool-call` events are emitted during streaming, allowing interleaved display and tool execution.

This design adds event-based streaming to opencode-rs's Provider trait to enable proper tool execution during streaming.

## Design

### 1. New Event Types

Add `LlmEvent` enum in `provider.rs` to represent streaming events:

```rust
pub enum LlmEvent {
    TextChunk(String),      // Partial text content
    ToolCall { name: String, arguments: Value, id: String },  // Tool invocation detected
    ToolResult { id: String, output: String },               // Tool result (for continuation)
    Done,                    // Stream completed
    Error(String),           // Error occurred
}
```

### 2. Provider Trait Changes

Add new method to `Provider` trait:

```rust
async fn complete_with_events(
    &self,
    prompt: &str,
    context: Option<&str>,
    mut callback: EventCallback,
) -> Result<Option<String>, OpenCodeError>;
```

Where `EventCallback = Box<dyn FnMut(LlmEvent) + Send>`.

Default implementation: call `complete()`, emit as single `TextChunk`, emit `Done`.

### 3. Streaming Providers Override

Providers that support streaming override `complete_with_events`:
- `OpenAiProvider` - Parse SSE stream for `tool_calls` in delta
- `AnthropicProvider` - Handle `tool_use` events
- `OllamaProvider` - Handle streaming tool calls
- etc.

### 4. TUI Integration

Replace direct `complete_streaming` call with `complete_with_events`:

```rust
// TUI app.rs
match provider.complete_with_events(&llm_input, None, Box::new(|event| {
    match event {
        LlmEvent::TextChunk(text) => { /* display text */ }
        LlmEvent::ToolCall { name, arguments, id } => { /* queue tool call */ }
        LlmEvent::Done => { /* finish */ }
        LlmEvent::Error(e) => { /* show error */ }
        _ => {}
    }
})).await {
    // Tool execution happens via AgentRuntime after stream
}
```

### 5. Tool Execution Flow

1. During streaming, when `ToolCall` event fires:
   - Pause text display (or use placeholder)
   - Execute tool via `ToolRegistry`
   - Send result back to LLM via continuation message

2. After `Done`:
   - If tools were executed, LLM continues with results
   - Loop until no more tool calls

### 6. AgentRuntime Usage

TUI creates/fuses with `AgentRuntime` for tool execution:

```rust
let runtime = AgentRuntime::new(config, session);
let response = runtime.run_loop(agent, provider_with_events, tools).await;
```

### 7. Architecture Decision

The `AgentRuntime::run_loop()` expects `Agent::run()` which returns `AgentResponse{content, tool_calls}`. For streaming, we need a streaming variant:

```rust
trait Agent {
    async fn run_streaming(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &ToolRegistry,
        events: EventCallback,
    ) -> Result<AgentResponse, AgentError>;
}
```

## Components to Modify

| File | Change |
|------|--------|
| `crates/llm/src/provider.rs` | Add `LlmEvent`, extend `Provider` trait |
| `crates/llm/src/openai.rs` | Implement `complete_with_events` for streaming tool calls |
| `crates/llm/src/anthropic.rs` | Handle `tool_use` in streaming |
| `crates/llm/src/ollama.rs` | Handle streaming |
| `crates/agent/src/runtime.rs` | Add streaming variant of `run_loop` |
| `crates/agent/src/build_agent.rs` | Implement streaming run |
| `crates/tui/src/app.rs` | Use AgentRuntime + event streaming |
| `crates/tui/src/action.rs` | Handle tool call UI state |

## Open Questions

1. **Tool call parsing**: Should we rely on provider-native tool calls (OpenAI function calling) or parse from text? Recommendation: provider-native since it's more reliable.

2. **UI display**: How to handle partial tool calls while streaming? Option: show "Thinking..." or similar placeholder until tool call completes.

3. **Error handling**: If tool execution fails, how to communicate back to LLM? Recommendation: send error message as tool result.

## Verification

- [ ] TUI streams text from LLM
- [ ] Tool calls are detected during streaming
- [ ] Tools execute and results are sent back
- [ ] LLM continues after tool execution
- [ ] Final response is displayed correctly
- [ ] No regression in non-streaming providers

## Risk Assessment

- **Medium-High**: Requires changes to Provider trait across all providers
- **Medium**: Tool call parsing from SSE requires careful handling per provider
- **Low**: Session and AgentRuntime are well-tested in existing code