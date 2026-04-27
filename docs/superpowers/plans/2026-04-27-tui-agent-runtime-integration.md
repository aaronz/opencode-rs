# TUI AgentRuntime Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable TUI to use AgentRuntime with event-based streaming, allowing proper tool call detection and execution during LLM streaming.

**Architecture:** Add `LlmEvent` enum and `complete_with_events()` method to Provider trait. Implement event-based streaming in OpenAI provider. Create streaming variant of AgentRuntime loop. Update TUI to use AgentRuntime with event streaming.

**Tech Stack:** Rust, tokio async, serde_json, async_trait

---

## File Structure

```
crates/llm/src/provider.rs          - Add LlmEvent, complete_with_events()
crates/llm/src/openai.rs             - Implement complete_with_events with tool call parsing
crates/llm/src/anthropic.rs          - Implement complete_with_events
crates/llm/src/ollama.rs             - Implement complete_with_events
crates/agent/src/agent.rs            - Add streaming variant to Agent trait
crates/agent/src/build_agent.rs      - Implement streaming run
crates/agent/src/general_agent.rs    - Implement streaming run
crates/agent/src/runtime.rs          - Add streaming run_loop variant
crates/tui/src/app.rs                - Integrate AgentRuntime with event streaming
crates/tui/src/action.rs             - Add tool call action types
```

---

## Task 1: Add LlmEvent and EventCallback to provider.rs

**Files:**
- Modify: `opencode-rust/crates/llm/src/provider.rs:1-50` (add imports)
- Modify: `opencode-rust/crates/llm/src/provider.rs:175-215` (add LlmEvent after StreamChunk)
- Modify: `opencode-rust/crates/llm/src/provider.rs:181-211` (extend Provider trait)

- [ ] **Step 1: Read current provider.rs structure**

Run: `head -220 opencode-rust/crates/llm/src/provider.rs`

- [ ] **Step 2: Add LlmEvent enum after StreamChunk (line ~179)**

```rust
#[derive(Debug, Clone)]
pub enum LlmEvent {
    TextChunk(String),
    ToolCall { name: String, arguments: serde_json::Value, id: String },
    ToolResult { id: String, output: String },
    Done,
    Error(String),
}

pub type EventCallback = Box<dyn FnMut(LlmEvent) + Send>;
```

- [ ] **Step 3: Add complete_with_events to Provider trait (after complete_streaming)**

```rust
async fn complete_with_events(
    &self,
    prompt: &str,
    context: Option<&str>,
    mut callback: EventCallback,
) -> Result<Option<String>, OpenCodeError> {
    let content = self.complete(prompt, context).await?;
    callback(LlmEvent::TextChunk(content));
    callback(LlmEvent::Done);
    Ok(None)
}
```

- [ ] **Step 4: Add complete_with_events to CancellableProvider wrapper (after complete_streaming)**

Add to `impl<P: Provider> CancellableProvider<'_, P>`:
```rust
pub async fn complete_with_events(
    &self,
    prompt: &str,
    context: Option<&str>,
    callback: EventCallback,
) -> Result<Option<String>, OpenCodeError> {
    if self.cancellation_token.is_cancelled() {
        return Err(crate::error::LlmError::Cancelled.into());
    }
    self.inner.complete_with_events(prompt, context, callback).await
}
```

- [ ] **Step 5: Run tests to verify no regression**

Run: `cargo test -p opencode-llm --lib -- provider 2>&1 | head -50`
Expected: Tests pass

- [ ] **Step 6: Commit**

```bash
git add opencode-rust/crates/llm/src/provider.rs
git commit -m "feat(llm): add LlmEvent enum and complete_with_events to Provider trait"
```

---

## Task 2: Implement complete_with_events in OpenAI Provider

**Files:**
- Modify: `opencode-rust/crates/llm/src/openai.rs:446-530` (extend complete_streaming)
- Create: `opencode-rust/crates/llm/src/openai.rs` (add tool call parsing structs)

- [ ] **Step 1: Read OpenAI streaming implementation**

Run: `sed -n '446,540p' opencode-rust/crates/llm/src/openai.rs`

- [ ] **Step 2: Add tool call parsing structs at top of openai.rs**

```rust
#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIChoice>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    delta: OpenAIDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAIToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCallDelta {
    id: Option<String>,
    #[serde(rename = "type")]
    tool_type: Option<String>,
    function: Option<OpenAIFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}
```

- [ ] **Step 3: Add complete_with_events implementation after complete_streaming**

```rust
async fn complete_with_events(
    &self,
    prompt: &str,
    _context: Option<&str>,
    mut callback: EventCallback,
) -> Result<Option<String>, OpenCodeError> {
    if self.uses_browser_auth() {
        let content = self.complete_browser_auth(prompt).await?;
        callback(LlmEvent::TextChunk(content));
        callback(LlmEvent::Done);
        return Ok(None);
    }

    let messages = vec![Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    }];

    let reasoning = self
        .reasoning_effort
        .as_ref()
        .map(|e| ReasoningRequest { effort: e.clone() });

    let request = ChatRequest {
        model: self.model.clone(),
        messages,
        stream: true,
        reasoning,
    };

    let mut req = self
        .client
        .post(format!("{}/chat/completions", self.base_url))
        .header("Authorization", format!("Bearer {}", self.api_key))
        .header("Content-Type", "application/json")
        .json(&request);

    for (key, value) in &self.headers {
        req = req.header(key, value);
    }

    let response = req
        .send()
        .await
        .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(OpenCodeError::Llm(format!(
            "OpenAI API error {}: {}",
            status, error_text
        )));
    }

    let mut tool_call_buffer: Option<(String, String, String)> = None; // (id, name, arguments)
    let mut lines = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(item) = lines.next().await {
        match item {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let data = line.strip_prefix("data: ").unwrap_or("");
                        if data == "[DONE]" {
                            if let Some((id, name, args)) = tool_call_buffer.take() {
                                let arguments: serde_json::Value = serde_json::from_str(&args)
                                    .unwrap_or(serde_json::Value::String(args));
                                callback(LlmEvent::ToolCall { name, arguments, id });
                            }
                            callback(LlmEvent::Done);
                            return Ok(None);
                        }
                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                            // Handle text content
                            if let Some(content) = chunk.pointer("/choices/0/delta/content")
                                .and_then(|v| v.as_str())
                            {
                                callback(LlmEvent::TextChunk(content.to_string()));
                            }
                            // Handle tool calls
                            if let Some(tool_calls) = chunk.pointer("/choices/0/delta/tool_calls")
                                .and_then(|v| v.as_array())
                            {
                                for tc in tool_calls {
                                    let id = tc.pointer("/id")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    let name = tc.pointer("/function/name")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    let args = tc.pointer("/function/arguments")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());

                                    if let (Some(id), Some(name), Some(args)) = (id, name, args) {
                                        // Flush any buffered tool call first
                                        if let Some((buf_id, buf_name, buf_args)) = tool_call_buffer.take() {
                                            let arguments: serde_json::Value = serde_json::from_str(&buf_args)
                                                .unwrap_or(serde_json::Value::String(buf_args));
                                            callback(LlmEvent::ToolCall { name: buf_name, arguments, id: buf_id });
                                        }
                                        tool_call_buffer = Some((id, name, args));
                                    } else if let Some((buf_id, buf_name, mut buf_args)) = tool_call_buffer.take() {
                                        // Append to existing tool call
                                        if let Some(new_args) = args {
                                            buf_args.push_str(&new_args);
                                        }
                                        tool_call_buffer = Some((buf_id, buf_name, buf_args));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                callback(LlmEvent::Error(format!("Stream error: {}", e)));
                return Err(OpenCodeError::Llm(format!("Stream error: {}", e)));
            }
        }
    }

    // Flush any remaining tool call
    if let Some((id, name, args)) = tool_call_buffer.take() {
        let arguments: serde_json::Value = serde_json::from_str(&args)
            .unwrap_or(serde_json::Value::String(args));
        callback(LlmEvent::ToolCall { name, arguments, id });
    }
    callback(LlmEvent::Done);
    Ok(None)
}
```

- [ ] **Step 4: Add necessary imports**

Find where `StreamChunk` is used and ensure `LlmEvent`, `EventCallback` are imported from parent module.

- [ ] **Step 5: Run tests**

Run: `cargo test -p opencode-llm -- openai --nocapture 2>&1 | tail -30`
Expected: Tests pass

- [ ] **Step 6: Commit**

```bash
git add opencode-rust/crates/llm/src/openai.rs
git commit -m "feat(llm): implement complete_with_events in OpenAI provider with tool call parsing"
```

---

## Task 3: Add streaming Agent trait method

**Files:**
- Modify: `opencode-rust/crates/agent/src/agent.rs:56-88` (add run_streaming to trait)

- [ ] **Step 1: Read current Agent trait**

Run: `sed -n '56,88p' opencode-rust/crates/agent/src/agent.rs`

- [ ] **Step 2: Add run_streaming method to Agent trait**

Add after `async fn run(...)` (around line 75):
```rust
async fn run_streaming(
    &self,
    session: &mut Session,
    provider: &dyn Provider,
    tools: &ToolRegistry,
    events: EventCallback,
) -> Result<AgentResponse, OpenCodeError> {
    // Default implementation: call run() and emit text chunks
    let response = self.run(session, provider, tools).await?;
    events(LlmEvent::TextChunk(response.content.clone()));
    events(LlmEvent::Done);
    Ok(response)
}
```

Also add import:
```rust
use opencode_llm::{EventCallback, LlmEvent};
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p opencode-agent --lib -- agent 2>&1 | head -30`
Expected: Tests pass

- [ ] **Step 4: Commit**

```bash
git add opencode-rust/crates/agent/src/agent.rs
git commit -m "feat(agent): add run_streaming method to Agent trait"
```

---

## Task 4: Implement run_streaming in BuildAgent

**Files:**
- Modify: `opencode-rust/crates/agent/src/build_agent.rs:130-167` (implement run_streaming)

- [ ] **Step 1: Read current BuildAgent run implementation**

Run: `sed -n '130,167p' opencode-rust/crates/agent/src/build_agent.rs`

- [ ] **Step 2: Add run_streaming implementation**

Add after the existing `run` method (around line 154):
```rust
async fn run_streaming(
    &self,
    session: &mut Session,
    provider: &dyn Provider,
    _tools: &ToolRegistry,
    mut events: EventCallback,
) -> Result<AgentResponse, OpenCodeError> {
    let mut all_messages: Vec<ChatMessage> = vec![ChatMessage {
        role: "system".to_string(),
        content: self.composed_system_prompt(),
    }];

    let prompt_messages =
        session.prepare_messages_for_prompt(TokenBudget::default().main_context_tokens());
    all_messages.extend(messages_to_llm_format(&prompt_messages));

    // Use complete_with_events to get streaming responses
    let prompt = all_messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let final_content = provider.complete_with_events(&prompt, None, |event| {
        match event {
            LlmEvent::TextChunk(text) => events(LlmEvent::TextChunk(text)),
            LlmEvent::ToolCall { name, arguments, id } => {
                events(LlmEvent::ToolCall { name, arguments, id })
            }
            LlmEvent::Done => events(LlmEvent::Done),
            LlmEvent::Error(e) => events(LlmEvent::Error(e)),
            LlmEvent::ToolResult { .. } => {} // Ignore tool results during build agent
        }
    }).await?;

    let content = final_content.unwrap_or_default();
    session.add_message(Message::assistant(content.clone()));

    Ok(AgentResponse {
        content,
        tool_calls: Vec::new(),
    })
}
```

- [ ] **Step 3: Add required imports**

Check imports in build_agent.rs and add:
```rust
use opencode_llm::{EventCallback, LlmEvent};
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p opencode-agent -- build_agent --nocapture 2>&1 | tail -40`
Expected: Tests pass

- [ ] **Step 5: Commit**

```bash
git add opencode-rust/crates/agent/src/build_agent.rs
git commit -m "feat(agent): implement run_streaming in BuildAgent"
```

---

## Task 5: Implement run_streaming in GeneralAgent

**Files:**
- Modify: `opencode-rust/crates/agent/src/general_agent.rs` (implement run_streaming)

- [ ] **Step 1: Read GeneralAgent implementation**

Run: `cat opencode-rust/crates/agent/src/general_agent.rs | head -100`

- [ ] **Step 2: Add run_streaming to GeneralAgent similar to BuildAgent**

Pattern follows build_agent.rs - add similar implementation.

- [ ] **Step 3: Run tests**

Run: `cargo test -p opencode-agent -- general_agent 2>&1 | tail -30`
Expected: Tests pass

- [ ] **Step 4: Commit**

```bash
git add opencode-rust/crates/agent/src/general_agent.rs
git commit -m "feat(agent): implement run_streaming in GeneralAgent"
```

---

## Task 6: Add streaming run_loop to AgentRuntime

**Files:**
- Modify: `opencode-rust/crates/agent/src/runtime.rs:285-391` (add streaming variant)

- [ ] **Step 1: Read current run_loop implementation**

Run: `sed -n '285,410p' opencode-rust/crates/agent/src/runtime.rs`

- [ ] **Step 2: Add run_loop_streaming method**

Add after existing `run_loop` (around line 391):
```rust
pub async fn run_loop_streaming<A: Agent>(
    &self,
    agent: &A,
    provider: &dyn Provider,
    tools: &ToolRegistry,
) -> Result<AgentResponse, RuntimeError> {
    if !self.primary_tracker.is_active() {
        tracing::error!("No active primary agent");
        return Err(RuntimeError::NoActivePrimaryAgent);
    }
    if self.primary_tracker.active_type() != Some(agent.agent_type()) {
        tracing::error!(current = ?self.primary_tracker.active_type(), requested = ?agent.agent_type(), "Agent type mismatch");
        return Err(RuntimeError::NoSuchAgent {
            agent_type: agent.agent_type(),
        });
    }

    let session_id = self.session.read().await.id.to_string();
    tracing::info!(session_id = %session_id, agent = ?agent.agent_type(), max_iterations = self.config.max_iterations, "Starting streaming agent run loop");

    let mut iteration = 0;
    let mut final_response = AgentResponse {
        content: String::new(),
        tool_calls: Vec::new(),
    };

    loop {
        iteration += 1;
        if iteration > self.config.max_iterations {
            tracing::warn!(session_id = %session_id, iteration = iteration, limit = self.config.max_iterations, "Max iterations exceeded");
            return Err(RuntimeError::MaxIterationsExceeded {
                limit: self.config.max_iterations,
            });
        }

        tracing::debug!(session_id = %session_id, iteration = iteration, "Agent iteration starting");

        // Collect tool calls during streaming
        let tool_calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let content_buffer = Arc::new(std::sync::Mutex::new(String::new()));
        let tc_clone = tool_calls.clone();
        let cb_clone = content_buffer.clone();

        let events_callback: EventCallback = Box::new(move |event| {
            match event {
                LlmEvent::TextChunk(text) => {
                    if let Ok(mut guard) = cb_clone.lock() {
                        guard.push_str(&text);
                    }
                }
                LlmEvent::ToolCall { name, arguments, id } => {
                    if let Ok(mut guard) = tc_clone.lock() {
                        guard.push(crate::ToolCall { name, arguments, id });
                    }
                }
                _ => {}
            }
        });

        let response = agent
            .run_streaming(&mut *self.session.write().await, provider, tools, events_callback)
            .await
            .map_err(|e| {
                tracing::error!(session_id = %session_id, error = %e, "Agent run streaming failed");
                RuntimeError::ToolExecutionFailed {
                    tool: "agent".to_string(),
                    reason: e.to_string(),
                }
            })?;

        if let Ok(mut guard) = content_buffer.lock() {
            final_response.content = guard.clone();
        }
        final_response.tool_calls = tool_calls.lock().unwrap().clone();

        if final_response.tool_calls.is_empty() {
            tracing::info!(session_id = %session_id, iteration = iteration, response_len = final_response.content.len(), "Agent completed successfully");
            break;
        }

        tracing::debug!(session_id = %session_id, iteration = iteration, tool_count = final_response.tool_calls.len(), "Processing tool calls");

        for call in final_response
            .tool_calls
            .iter()
            .take(self.config.max_tool_results_per_iteration)
        {
            let tool_call = ToolsToolCall {
                name: call.name.clone(),
                args: call.arguments.clone(),
                ctx: None,
            };

            let ctx = ToolContext {
                session_id: self.session.read().await.id.to_string(),
                message_id: Uuid::new_v4().to_string(),
                agent: agent.name().to_string(),
                worktree: None,
                directory: None,
                permission_scope: Some(self.config.permission_scope),
            };

            tracing::debug!(session_id = %session_id, tool = %call.name, "Executing tool");
            let result = tools
                .execute(&call.name, tool_call.args, Some(ctx))
                .await
                .map_err(|e| {
                    tracing::error!(session_id = %session_id, tool = %call.name, error = %e, "Tool execution failed");
                    RuntimeError::ToolExecutionFailed {
                        tool: call.name.clone(),
                        reason: e.to_string(),
                    }
                })?;

            let result_text = if result.success {
                tracing::debug!(session_id = %session_id, tool = %call.name, "Tool execution succeeded");
                result.content.clone()
            } else {
                tracing::warn!(session_id = %session_id, tool = %call.name, error = ?result.error, "Tool execution returned error");
                format!("Error: {}", result.error.clone().unwrap_or_default())
            };

            let result_message =
                Message::user(format!("Tool '{}' result:\n{}", call.name, result_text));
            self.session.write().await.add_message(result_message);
        }
    }

    let assistant_msg = Message::assistant(&final_response.content);
    self.session.write().await.add_message(assistant_msg);

    Ok(final_response)
}
```

Add imports:
```rust
use opencode_llm::{EventCallback, LlmEvent};
use std::sync::Arc;
```

- [ ] **Step 3: Run build to verify compilation**

Run: `cargo build -p opencode-agent 2>&1 | tail -50`
Expected: Compiles successfully

- [ ] **Step 4: Run tests**

Run: `cargo test -p opencode-agent --lib 2>&1 | tail -30`
Expected: Tests pass

- [ ] **Step 5: Commit**

```bash
git add opencode-rust/crates/agent/src/runtime.rs
git commit -m "feat(agent): add streaming run_loop to AgentRuntime"
```

---

## Task 7: Update TUI to use AgentRuntime with event streaming

**Files:**
- Modify: `opencode-rust/crates/tui/src/app.rs:4330-4390` (replace complete_streaming with AgentRuntime)
- Modify: `opencode-rust/crates/tui/src/action.rs` (add tool call states)

- [ ] **Step 1: Read current TUI streaming implementation**

Run: `sed -n '4330,4450p' opencode-rust/crates/tui/src/app.rs`

- [ ] **Step 2: Create AgentRuntime instance and call run_loop_streaming**

Replace the direct `complete_streaming` call with:
```rust
// Use AgentRuntime for proper tool execution
let runtime = AgentRuntime::new(
    RuntimeConfig::default(),
    self.session.clone(),
);

let agent = BuildAgent::new()
    .with_skill_prompt(skill_prompt.clone())
    .with_agents_md_content(self.get_agents_md_content());

// Set active agent
runtime.switch_primary_agent(AgentType::Build).await?;

// Run with streaming events
let events_callback: EventCallback = Box::new(move |event| {
    match event {
        LlmEvent::TextChunk(text) => {
            let _ = tx.send(LlmEvent::Chunk(text));
        }
        LlmEvent::ToolCall { name, arguments, id } => {
            // Queue tool call for display and execution
            let _ = tx.send(LlmEvent::ToolCallStart { name: name.clone(), id: id.clone() });
        }
        LlmEvent::Done => {
            let _ = tx.send(LlmEvent::Done);
        }
        LlmEvent::Error(e) => {
            let _ = tx.send(LlmEvent::Error(e));
        }
        _ => {}
    }
});

runtime.run_loop_streaming(&agent, provider, tools).await?;
```

- [ ] **Step 3: Add necessary imports to app.rs**

```rust
use opencode_agent::{AgentRuntime, RuntimeConfig, Agent, AgentType, BuildAgent};
use opencode_llm::{EventCallback, LlmEvent};
```

- [ ] **Step 4: Run build to verify compilation**

Run: `cargo build -p opencode-tui 2>&1 | tail -80`
Expected: Compiles with errors to fix

- [ ] **Step 5: Fix any compilation errors iteratively**

Fix errors as they appear.

- [ ] **Step 6: Run tests**

Run: `cargo test -p opencode-tui --lib 2>&1 | tail -30`
Expected: Tests pass

- [ ] **Step 7: Commit**

```bash
git add opencode-rust/crates/tui/src/app.rs opencode-rust/crates/tui/src/action.rs
git commit -m "feat(tui): integrate AgentRuntime with event streaming for tool execution"
```

---

## Task 8: Add tool call UI states to action.rs

**Files:**
- Modify: `opencode-rust/crates/tui/src/action.rs`

- [ ] **Step 1: Read current action.rs**

Run: `cat opencode-rust/crates/tui/src/action.rs | head -100`

- [ ] **Step 2: Add ToolCall related actions**

Add:
```rust
ToolCallStart { name: String, id: String },
ToolCallComplete { id: String, success: bool },
```

- [ ] **Step 3: Commit**

```bash
git add opencode-rust/crates/tui/src/action.rs
git commit -m "feat(tui): add tool call action types"
```

---

## Task 9: Integration testing

**Files:**
- Create: `opencode-rust/integration_tests/src/agent/tool_execution_tests.rs`

- [ ] **Step 1: Write integration test for tool execution via streaming**

```rust
#[tokio::test]
async fn test_streaming_tool_call_execution() {
    // Setup: Create mock provider that emits tool calls
    // Action: Call run_loop_streaming with agent
    // Verify: Tool was executed and result sent back
}
```

- [ ] **Step 2: Run integration test**

Run: `cargo test -p opencode-integration-tests -- tool_execution 2>&1 | tail -40`
Expected: Test passes

- [ ] **Step 3: Commit**

```bash
git add opencode-rust/integration_tests/src/agent/tool_execution_tests.rs
git commit -m "test: add integration test for streaming tool execution"
```

---

## Verification Checklist

- [ ] `cargo build -p opencode-llm` - OpenAI provider compiles
- [ ] `cargo build -p opencode-agent` - AgentRuntime compiles
- [ ] `cargo build -p opencode-tui` - TUI compiles
- [ ] `cargo test -p opencode-llm` - LLM tests pass
- [ ] `cargo test -p opencode-agent` - Agent tests pass
- [ ] `cargo test -p opencode-integration-tests` - Integration tests pass
- [ ] Manual verification: TUI streams text and executes tools

---

## Risk Mitigation

1. **Provider compatibility**: Start with OpenAI only (has native function calling). Other providers can use default implementation.
2. **TUI regression**: Keep existing complete_streaming path as fallback until new path is verified.
3. **Agent trait**: Default implementation ensures existing agents compile without changes.

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-27-tui-agent-runtime-integration.md`**

**Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**