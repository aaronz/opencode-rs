# TUI AgentRuntime Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace direct `provider.complete_with_events()` call in TUI with `AgentRuntime::run_loop_streaming()` to enable proper tool execution during LLM streaming.

**Architecture:** TUI will create a `Session` and `AgentRuntime` instance, then use `runtime.run_loop_streaming()` instead of calling the provider directly. The runtime handles the full agent loop: streaming → detect tool calls → execute → continue → repeat.

**Tech Stack:** Rust, tokio async runtime, opencode-agent, opencode-llm

---

## File Structure

- Modify: `opencode-rust/crates/tui/src/app.rs` - Add Session field, replace provider call with AgentRuntime

## Task 1: Add Session and runtime field to App

**Files:**
- Modify: `opencode-rust/crates/tui/src/app.rs:444-552`

- [ ] **Step 1: Find the App struct definition and add session and runtime fields**

Find the `pub struct App {` section (around line 444) and add:
```rust
session: Option<Session>,
agent_runtime: Option<AgentRuntime>,
```

- [ ] **Step 2: Initialize session and runtime fields in App::new()**

Find the `Self {` in `impl App` (around line 1169), add:
```rust
session: None,
agent_runtime: None,
```

- [ ] **Step 3: Commit**

```bash
cd opencode-rust && git add crates/tui/src/app.rs && git commit -m "feat(tui): add session and runtime fields to App struct"
```

---

## Task 2: Create agent_runtime when submitting message

**Files:**
- Modify: `opencode-rust/crates/tui/src/app.rs:4346-4411`

- [ ] **Step 1: Replace direct provider call with AgentRuntime**

In the thread spawn block (lines 4376-4411), replace:
```rust
std::thread::spawn(move || {
    // ...
    rt.block_on(async {
        // ... callback setup ...
        match provider_clone
            .complete_with_events(&llm_input, None, callback)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                let _ = tx.send(LlmEvent::Error(e.to_string()));
            }
        }
    });
});
```

With:
```rust
std::thread::spawn(move || {
    // ...
    rt.block_on(async {
        // Get session from app (passed as Arc<RwLock<Session>>)
        let session_guard = session.read().await;
        let session = session_guard.clone();
        drop(session_guard);

        // Create BuildAgent
        let agent = BuildAgent::new();

        // Create AgentRuntime
        let runtime = AgentRuntime::new(session.clone(), AgentType::Build);

        // Use run_loop_streaming instead of direct provider call
        // Note: We need to pass events that forward to tx
        let tx_clone = tx.clone();
        let events_callback: opencode_llm::EventCallback = Box::new(move |event| {
            let _ = tx_clone.send(match event {
                opencode_llm::LlmEvent::TextChunk(text) => LlmEvent::Chunk(text),
                opencode_llm::LlmEvent::ToolCall { name, arguments, id } => LlmEvent::ToolCall { name, arguments, id },
                opencode_llm::LlmEvent::Done => LlmEvent::Done,
                opencode_llm::LlmEvent::Error(e) => LlmEvent::Error(e),
                opencode_llm::LlmEvent::ToolResult { id, output } => LlmEvent::ToolResult { id, output },
            });
        });

        match runtime.run_loop_streaming(&agent, &provider_clone, &tools_clone, events_callback).await {
            Ok(response) => {
                // Done event is sent by run_loop_streaming
            }
            Err(e) => {
                let _ = tx.send(LlmEvent::Error(e.to_string()));
            }
        }
    });
});
```

- [ ] **Step 2: Add imports for BuildAgent and RuntimeConfig if not already present**

Check line 29 imports:
```rust
use opencode_agent::{AgentRuntime, AgentType, BuildAgent, RuntimeConfig};
```

- [ ] **Step 3: Pass tools_clone and session to the thread**

Currently the thread captures `provider_clone`. Need to also capture `tools_clone` and the `session` Arc. The session should be an `Arc<RwLock<Session>>` stored in App.

- [ ] **Step 4: Commit**

```bash
cd opencode-rust && git add crates/tui/src/app.rs && git commit -m "feat(tui): integrate AgentRuntime::run_loop_streaming"
```

---

## Task 3: Handle session creation and update

**Files:**
- Modify: `opencode-rust/crates/tui/src/app.rs`

- [ ] **Step 1: Create/update Session when user submits message**

When the user submits input (around line 4346), before spawning the thread:
```rust
// Ensure we have a session
if self.session.is_none() {
    self.session = Some(Session::new());
}

// Get Arc<RwLock<Session>> for the runtime
let session_arc = Arc::new(RwLock::new(self.session.take().unwrap()));
```

- [ ] **Step 2: Store results back to session after completion**

After `run_loop_streaming` completes, store the updated session back:
```rust
if let Ok(response) = result {
    // Session was modified by runtime, extract it
    let session = Arc::try_unwrap(session_arc)
        .map(|lock| lock.into_inner())
        .unwrap_or_else(|_| Session::new());
    self.session = Some(session);
}
```

- [ ] **Step 3: Commit**

```bash
cd opencode-rust && git add crates/tui/src/app.rs && git commit -m "feat(tui): handle session creation and persistence"
```

---

## Task 4: Verify build and run tests

**Files:**
- Test: `opencode-rust/crates/tui`

- [ ] **Step 1: Build the tui package**

```bash
cd opencode-rust && cargo build -p opencode-tui 2>&1 | head -100
```

Expected: Should compile without errors (may have type mismatches to fix)

- [ ] **Step 2: Fix any type errors**

Common issues:
- Session vs tool_registry ownership
- Missing Arc/RwLock wrapping
- Callback signature mismatch

- [ ] **Step 3: Run tui tests**

```bash
cd opencode-rust && cargo test -p opencode-tui --lib 2>&1 | tail -50
```

- [ ] **Step 4: Run full test suite**

```bash
cd opencode-rust && cargo test --all 2>&1 | tail -100
```

---

## Task 5: Verify tool execution works end-to-end

**Files:**
- Test: Manual testing or add integration test

- [ ] **Step 1: Test that tool calls are executed during streaming**

Start opencode-rs TUI and send a message that triggers a tool call (e.g., "Read the file README.md")

Expected: Tool call should be detected, executed, and result sent back to LLM

- [ ] **Step 2: Verify final response includes tool context**

The final response should reflect the tool execution results

- [ ] **Step 3: Commit all changes**

```bash
cd opencode-rust && git add . && git commit -m "feat(tui): full AgentRuntime integration for tool execution"
```

---

## Verification Checklist

- [ ] TUI builds without errors
- [ ] `cargo test -p opencode-tui --lib` passes
- [ ] `cargo test -p opencode-agent --lib` still passes (no regression)
- [ ] `cargo test -p opencode-llm --lib` still passes (no regression)
- [ ] Manual test: Tool calls are detected and executed
- [ ] Manual test: LLM continues after tool execution with results