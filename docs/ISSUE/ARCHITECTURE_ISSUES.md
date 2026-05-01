# opencode-rs Architectural Issues

Generated from review against `docs/DESIGN/agent-runtime-design.md`

**Last Updated**: 2026-04-30
**Review Method**: Critical analysis with source code verification

---

## RESOLVED ISSUES

The following issues have been fixed:

| Issue | Status | Summary |
|-------|--------|---------|
| ISSUE-001 | ✅ FIXED | TUI now sends `RuntimeFacadeCommand::RunAgent` instead of directly creating AgentRuntime |
| ISSUE-002 | ✅ FIXED | Shell commands now route through Runtime's ToolRouter via `ExecuteShell` command |
| ISSUE-005 | ✅ FIXED | Added `ToolSchema`, `chat_with_tools()` to Provider, `input_schema()` to Tool, implemented in agents |
| ISSUE-008 | ✅ FIXED | Fixed `trim_to_budget()` edge case - now properly drops lowest-ranked messages |
| ISSUE-009 | ✅ FIXED | CLI context commands implemented: inspect, explain, dump, why |
| ISSUE-011 | ✅ FIXED | PermissionResponse command fully implemented in Runtime |
| ISSUE-012 | ✅ FIXED | AgentRuntime emits to EventBus via `with_event_bus()` |
| ISSUE-013 | ✅ FIXED | RuntimeFacade's RunAgent handler calls `run_loop_streaming()` - loop executes in Runtime |
| ISSUE-014 | ✅ FIXED | RuntimeFacadeHandle::execute() delegates to execute_standalone without cloning |

---

## CORRECTIONS FROM RE-VERIFICATION

After re-checking the source code, some initial findings were inaccurate or overstated:

| Original Issue | Correction |
|----------------|------------|
| ISSUE-003 (Config writes) | TUI calling `config.save()` IS actually calling a public config service method. The design says TUI shouldn't write config files directly, but `Config` is a service type, not a Runtime internal. **Downgraded to informational** |
| ISSUE-004 (Provider calls) | TUI makes provider API calls for **user-initiated** connection flows (OAuth, API key validation). These are appropriate as they happen **before** Runtime is involved. **Downgraded to informational** |
| ISSUE-007 (Error strings) | `LlmError` is actually well-structured with 10+ specific variants. The string variants (`Provider(String)`, `Auth(String)`) are catch-all for provider-specific errors that don't fit elsewhere. **Accurate but severity lower** |

---

## CONFIRMED ISSUES (Most have been resolved - see table above)

## ISSUE-001: TUI Directly Instantiates AgentRuntime ~~(CONFIRMED - HIGH)~~ - ✅ FIXED

**Severity**: HIGH
**Design Principle Violated**: "TUI should only send commands and subscribe to events"
**Reference**: `agent-runtime-design.md` Section 3.12

### Finding

~~The TUI at lines 4755-4813 creates `AgentRuntime` directly in a spawned thread rather than going through the Runtime facade's command API.~~

**UPDATE**: This issue has been fixed. TUI now sends `RuntimeFacadeCommand::RunAgent` via `runtime.execute()`.

### Resolution

**File**: `crates/tui/src/app.rs:4773-4795` now uses:
```rust
let cmd = RuntimeFacadeCommand::RunAgent(Box::new(RunAgentCommand {
    session,
    agent_type: AgentType::Build,
}));
match runtime.execute(cmd).await { ... }
```

**File**: `crates/runtime/src/runtime.rs:242-279` executes the loop:
```rust
RuntimeFacadeCommand::RunAgent(cmd) => {
    let runtime = AgentRuntime::new(cmd.session, cmd.agent_type)
        .with_event_bus(services.event_bus.clone());
    // ...
    runtime.run_loop_streaming(&*agent, p, t, None).await
}
```

---

## ISSUE-002: Shell Execution Bypasses Runtime Tool Layer ~~(CONFIRMED - HIGH)~~ - ✅ FIXED

**Severity**: HIGH
**Design Principle Violated**: "Tool execution must go through ToolRouter"
**Reference**: `agent-runtime-design.md` Section 3.8

### Finding

~~`ShellHandler` executes shell commands via `std::process::Command` directly, bypassing Runtime's `ToolRouter` and `PermissionManager`.~~

**UPDATE**: Shell commands now route through Runtime's ToolRouter.

### Resolution

**File**: `crates/runtime/src/commands.rs` - Added `ExecuteShellCommand`:
```rust
pub struct ExecuteShellCommand {
    pub command: String,
    pub timeout_secs: Option<u64>,
    pub workdir: Option<String>,
}
```

**File**: `crates/runtime/src/runtime.rs:280-297` - Handler routes to ToolRouter:
```rust
RuntimeFacadeCommand::ExecuteShell(cmd) => {
    let args = serde_json::json!({
        "command": cmd.command,
        "timeout": cmd.timeout_secs,
        "workdir": cmd.workdir,
    });
    services.tool_router.execute("bash", args, None).await
}
```

**File**: `crates/tui/src/app.rs:4605-4652` - TUI now uses Runtime:
```rust
let shell_cmd = RuntimeFacadeCommand::ExecuteShell(ExecuteShellCommand {
    command: cmd.clone(),
    timeout_secs: None,
    workdir: None,
});
let result = runtime.execute(shell_cmd).await;
```

---

---

## ISSUE-005: Tool Schemas In Plain Text, Not Provider API (CONFIRMED - HIGH)

**Severity**: HIGH
**Design Principle Violated**: "Provider Complexity should not leak into Runtime Core"
**Reference**: `agent-runtime-design.md` Section 3.10 "LLM Provider Gateway"

### Finding

Tool schemas are embedded as plain text in system prompt, not passed through Provider's formal `tools` parameter.

### Evidence

**File**: `crates/agent/src/build_agent.rs:22-36`
```rust
system_prompt: r#"You are OpenCode...
You have access to tools to help you complete coding tasks:
- file_read: Read file contents
- file_write: Write content to files
...
When you need to use a tool, respond with a JSON object containing tool_calls."#.to_string(),
```

**File**: `crates/llm/src/provider.rs:200-241` - `Provider` trait has no `tools` parameter:
```rust
pub trait Provider: Send + Sync + sealed::Sealed {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError>;
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError>;
    // No tools parameter!
}
```

### Why This Is A Real Problem

1. **Not using provider-native tool calling** - OpenAI, Anthropic, Google have formal `tools`/`tool_use` parameters with JSON schemas
2. **Model can't see full tool signatures** - plain text descriptions don't give the model parameter types
3. **No structured tool results** - model returns JSON that must be parsed, rather than provider handling tool call/result natively
4. **Limits model capability** - models with native tool calling perform better when given formal schemas

---

## ISSUE-008: Context Ranking Defined But Not Used (CONFIRMED - MEDIUM)

**Severity**: MEDIUM
**Design Principle Violated**: "Context must be explainable and inspectable"
**Reference**: `agent-runtime-design.md` Section 3.7

### Finding

`ContextRanking` with weights is defined in `compaction/types.rs` but never used in context assembly.

### Evidence

**File**: `crates/core/src/compaction/types.rs:228-259`
```rust
pub struct ContextRanking {
    pub message_index: usize,
    pub recency: f64,
    pub relevance: f64,
    pub importance: f64,
    pub overall: f64,
}

// Weights defined but never applied:
pub const CONTEXT_RANKING_RECENTCY_WEIGHT: f64 = 0.4;
pub const CONTEXT_RANKING_RELEVANCE_WEIGHT: f64 = 0.3;
pub const CONTEXT_RANKING_IMPORTANCE_WEIGHT: f64 = 0.3;
```

**File**: `crates/core/src/context/mod.rs` - `ContextBuilder` doesn't use ranking:
```rust
pub struct ContextBuilder {
    // Has methods: collect_file_context, collect_tool_context, collect_session_context
    // But NO method applies ContextRanking to select items
}
```

### Impact

Context assembly may include items that are not most relevant to the current task. The ranking system exists but is a no-op.

---

## ISSUE-009: Context Inspectability Commands Missing ~~(CONFIRMED - MEDIUM)~~ - ✅ IMPLEMENTED

**Severity**: MEDIUM
**Design Principle Violated**: "Context must be inspectable"
**Reference**: `agent-runtime-design.md` Section 3.7 "Context Inspectability"

### Finding

~~Design specifies CLI commands that don't exist:~~
- ~~`opencode-rs context inspect`~~
- ~~`opencode-rs context explain`~~
- ~~`opencode-rs context dump --turn <id>`~~
- ~~`opencode-rs context why <file>`~~

**UPDATE**: These commands are now implemented in `crates/cli/src/cmd/context.rs`.

### Resolution

**File**: `crates/cli/src/cmd/context.rs` implements:
- `opencode context inspect` - Shows session info, messages, layer breakdown
- `opencode context explain` - Explains how context items are selected
- `opencode context dump --turn <id>` - Shows context for a specific turn
- `opencode context why --file <path>` - Explains why a file is or isn't in context

---

## ISSUE-010: ContextItem Priority on Layer, Not Item ~~(CONFIRMED - LOW)~~ - ⚠️ DEFERRED

**Severity**: LOW
**Design Principle Violated**: "ContextItem should have priority field"
**Reference**: `agent-runtime-design.md` Section 3.7

### Finding

Priority is on `ContextLayer` enum, not on individual `ContextItem`.

### Evidence

**Design spec**:
```rust
pub struct ContextItem {
    pub priority: ContextPriority,  // <-- priority on item
    // ...
}
```

**Implementation** (`context/mod.rs:48-54`):
```rust
pub struct ContextItem {
    pub layer: ContextLayer,  // <-- priority is on layer
    pub content: String,
    pub token_count: usize,
    pub source: String,
}

impl ContextLayer {
    pub fn priority(&self) -> u8 { ... }  // Priority derived from layer
}
```

### Impact

Low - two items in same layer cannot be prioritized differently. The layer-based priority may be sufficient.

---

## ISSUE-011: RuntimeCommand::PermissionResponse Not Implemented ~~(NEW - MEDIUM)~~ - ✅ FIXED

**Severity**: MEDIUM
**Reference**: `agent-runtime-design.md` Section 3.12

### Finding

~~`RuntimeCommand::PermissionResponse` exists in `commands.rs` but returns `NotImplemented` in `runtime.rs:133-135`.~~

**UPDATE**: PermissionResponse is now fully implemented in Runtime.

---

## ISSUE-012: AgentRuntime Uses Callbacks, Not Events ~~(NEW - MEDIUM)~~ - ✅ FIXED

**Severity**: MEDIUM
**Design Principle Violated**: "Runtime Core should be event-driven"
**Reference**: `agent-runtime-design.md` Section 3.1 Principle 2

### Finding

~~`AgentRuntime::run_loop_streaming` uses callbacks (`EventCallback`) for LLM events, not the Runtime's EventBus.~~

**UPDATE**: `AgentRuntime::new()` can be configured with `with_event_bus()` and the Runtime's RunAgent handler uses it:
```rust
let runtime = AgentRuntime::new(cmd.session, cmd.agent_type)
    .with_event_bus(services.event_bus.clone());
```

---

## ISSUE-013: Runtime Doesn't Execute Agent Loop ~~(NEW - HIGH)~~ - ✅ FIXED

**Severity**: HIGH
**Design Principle Violated**: "Runtime should own behavior, UI should own presentation"
**Reference**: `agent-runtime-design.md` Section 3.16 "Public API Boundary"

### Finding

~~`Runtime::execute(RuntimeCommand::SubmitUserInput)` creates a task and saves session, but does NOT actually run the agent loop. The agent runs in TUI's spawned thread.~~

**UPDATE**: The `RunAgent` handler in Runtime now calls `runtime.run_loop_streaming()` directly.

---

## ISSUE-014: RuntimeHandle Clones Runtime On Every Execute (NEW - MEDIUM)

**Severity**: MEDIUM
**Reference**: `crates/runtime/src/runtime.rs:192-202`

### Finding

`RuntimeHandle::execute()` creates a new `Runtime` instance on every call.

### Evidence

```rust
pub async fn execute(
    &self,
    command: RuntimeCommand,
) -> Result<RuntimeResponse, RuntimeFacadeError> {
    let runtime = Runtime {  // <-- Clones services, creates new Runtime
        services: self.services.clone(),
        status: Arc::clone(&self.status),
    };
    runtime.execute(command).await  // <-- Just delegates
}
```

### Impact

Inefficient but more importantly: each `execute` call gets a potentially different view of services if they're not properly shared. The pattern suggests the handle isn't managing state correctly.

---

## Summary Table

| Issue | Severity | Status | Category | Resolution |
|-------|----------|--------|----------|------------|
| ISSUE-001 | HIGH | ✅ FIXED | TUI/Runtime | TUI sends RunAgent via Runtime facade |
| ISSUE-002 | HIGH | ✅ FIXED | TUI/Runtime | Shell commands routed via ExecuteShell |
| ISSUE-003 | MEDIUM | ℹ️ INFO | TUI/Runtime | Config writes through config service |
| ISSUE-004 | MEDIUM | ℹ️ INFO | TUI/Runtime | Provider calls for user auth flow |
| ISSUE-005 | HIGH | ✅ FIXED | Provider | ToolSchema + chat_with_tools() implemented |
| ISSUE-006 | MEDIUM | ⚠️ PARTIAL | Provider | May be sufficient for current needs |
| ISSUE-007 | MEDIUM | ℹ️ INFO | Provider | LlmError is well-structured |
| ISSUE-008 | MEDIUM | ✅ FIXED | Context | trim_to_budget edge case fixed |
| ISSUE-009 | MEDIUM | ✅ FIXED | Context | CLI commands implemented |
| ISSUE-010 | LOW | ⚠️ DEFERRED | Context | Layer-based priority sufficient |
| ISSUE-011 | MEDIUM | ✅ FIXED | Runtime | PermissionResponse implemented |
| ISSUE-012 | MEDIUM | ✅ FIXED | Events | AgentRuntime uses EventBus |
| ISSUE-013 | HIGH | ✅ FIXED | Runtime | Runtime executes agent loop |
| ISSUE-014 | MEDIUM | ✅ FIXED | Runtime | execute_standalone without cloning |

---

## Architecture Root Cause

The issues converge on a single architectural problem:

**The TUI is the runtime, and Runtime is just a task store.**

The TUI:
- Instantiates `AgentRuntime` directly
- Handles LLM events via callbacks
- Executes shell commands directly
- Has `#[allow(dead_code)] runtime` field that was never wired up

The `Runtime` struct:
- Creates tasks and sessions
- Saves state
- Returns "accepted" without executing

This is the inverse of the design which says:
```
Runtime owns behavior
UI owns presentation
```

---

## Recommended Priority Fix Sequence

1. **ISSUE-013 (Highest priority)** - Make Runtime actually execute the agent loop, not TUI
2. **ISSUE-001** - Route all TUI agent activity through `RuntimeCommand`
3. **ISSUE-012** - Change AgentRuntime to emit events to EventBus, not callbacks
4. **ISSUE-002** - Move ShellHandler execution through Runtime's ToolRouter
5. **ISSUE-011** - Implement `PermissionResponse` command
6. **ISSUE-005** - Add `tools` parameter to Provider trait
7. **ISSUE-008, ISSUE-009** - Context improvements

Steps 1-3 form a cohesive change: Runtime becomes the actual runtime, not just a facade.