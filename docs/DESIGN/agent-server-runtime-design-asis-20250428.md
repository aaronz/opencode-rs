# Agent Server & Runtime Design Review

## Architecture Overview

The system follows a **layered architecture** with clear separation:

```
┌─────────────────────────────────────────────────────────┐
│                    Client Layer                         │
│   (CLI, TUI, Web, Desktop - all connect via WebSocket)  │
└─────────────────────┬───────────────────────────────────┘
                      │ WebSocket / HTTP
┌─────────────────────▼───────────────────────────────────┐
│                   Server Layer                           │
│  ┌─────────┐  ┌──────────┐  ┌────────┐  ┌───────────┐  │
│  │  actix- │  │ Session  │  │  MCP   │  │  Streaming│  │
│  │   web   │  │   Hub    │  │ Server │  │  (SSE/WS) │  │
│  └────┬────┘  └────┬─────┘  └────┬───┘  └───────────┘  │
└───────┼────────────┼────────────┼───────────────────────┘
        │            │            │
┌───────▼────────────▼────────────▼───────────────────────┐
│                   Agent Layer                            │
│  ┌─────────────────────────────────────────────────┐    │
│  │              AgentRuntime (2200 LOC)            │    │
│  │  - PrimaryAgentTracker (enforces single active) │    │
│  │  - run_loop / run_loop_streaming                 │    │
│  │  - Subagent delegation with forked sessions      │    │
│  └─────────────────────────────────────────────────┘    │
│  ┌──────────┐ ┌──────────┐ ┌───────────┐ ┌─────────┐  │
│  │  Build   │ │  Plan    │ │  General  │ │ Explore │  │
│  │  Agent   │ │  Agent   │ │   Agent   │ │  Agent  │  │
│  └──────────┘ └──────────┘ └───────────┘ └─────────┘  │
└───────────────────────────┬─────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│               Tool / Integration Layer                  │
│  ┌────────────┐  ┌────────────┐  ┌────────────────┐   │
│  │ ToolRegistry│  │    MCP     │  │    Plugin      │   │
│  │  (26 tools) │  │   Client   │  │   (WASM)       │   │
│  └────────────┘  └────────────┘  └────────────────┘   │
└───────────────────────────────────────────────────────┘
```

## Key Components

### 1. Server Layer (`crates/server/`)

**actix-web HTTP Server** with:
- WebSocket endpoints (`/ws`, `/ws/{session_id}`)
- SSE streaming endpoints
- REST API routes (session, config, model, MCP, etc.)
- mDNS service discovery
- CORS middleware
- API key authentication

**ServerState** holds all shared state:
```rust
pub struct ServerState {
    pub storage: Arc<StorageService>,
    pub models: Arc<ModelRegistry>,
    pub config: Arc<RwLock<Config>>,
    pub event_bus: SharedEventBus,
    pub reconnection_store: ReconnectionStore,
    pub connection_monitor: Arc<ConnectionMonitor>,
    pub share_server: Arc<RwLock<ShareServer>>,
    pub tool_registry: Arc<ToolRegistry>,
    pub session_hub: Arc<SessionHub>,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    pub approval_queue: Arc<RwLock<ApprovalQueue>>,
    // ...
}
```

### 2. Agent Layer (`crates/agent/`)

**Agent Trait** - Core abstraction:
```rust
#[async_trait]
pub trait Agent: Send + Sync + sealed::Sealed {
    fn agent_type(&self) -> AgentType;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn can_execute_tools(&self) -> bool;
    fn can_write_files(&self) -> bool;
    fn can_run_commands(&self) -> bool;
    fn is_visible(&self) -> bool { true }

    async fn run(&self, session: &mut Session, provider: &dyn Provider, tools: &ToolRegistry) -> Result<AgentResponse, OpenCodeError>;

    async fn run_streaming(&self, session: &mut Session, provider: &dyn Provider, tools: &ToolRegistry, mut events: EventCallback) -> Result<AgentResponse, OpenCodeError> {
        let response = self.run(session, provider, tools).await?;
        events(LlmEvent::TextChunk(response.content.clone()));
        events(LlmEvent::Done);
        Ok(response)
    }

    fn preferred_model(&self) -> Option<String> { None }
    fn preferred_variant(&self) -> Option<String> { None }
    fn preferred_reasoning_budget(&self) -> Option<ReasoningBudget> { None }
}
```

**Agent Types** (10 total):
- `Build` - Full access agent
- `Plan` - Read-only planning
- `General` - General purpose
- `Explore` - Code exploration
- `Compaction` - Context compaction
- `Title` - Session titling
- `Summary` - Summarization
- `Review` - Code review
- `Refactor` - Refactoring assistance
- `Debug` - Debugging assistance

**AgentRuntime** (2200+ LOC) - Orchestration engine:
- Primary agent state machine
- Tool execution loop
- Subagent delegation with session forking
- Permission scope management

### 3. MCP Layer (`crates/mcp/`)

**McpServer** - Protocol implementation:
```rust
pub struct McpServer {
    name: String,
    version: String,
    tools: Arc<RwLock<HashMap<String, Box<dyn ToolHandler>>>>,
    resources: Arc<RwLock<HashMap<String, Box<dyn ResourceHandler>>>>,
    initialized: Arc<RwLock<bool>>,
}
```

**Supported Methods**:
- `initialize` - Protocol version negotiation
- `initialized` - Client ready notification
- `tools/list` - List available tools
- `tools/call` - Execute tool
- `resources/list` - List resources
- `resources/read` - Read resource

---

## Design Strengths

### 1. Clean Agent Trait Abstraction

The sealed trait pattern prevents external implementations while allowing internal extension. The default `run_streaming()` implementation provides backward compatibility.

### 2. Primary Agent State Machine

Enforces **exactly one primary agent active** invariant:

```rust
pub enum PrimaryAgentState {
    Inactive,      // No primary agent
    Running,       // Primary agent executing
    Transitioning, // Switching agents
}

pub struct PrimaryAgentTracker {
    state: PrimaryAgentState,
    agent_type: Option<AgentType>,
}
```

Error cases:
- `RuntimeError::MultiplePrimaryAgents` - Attempted to activate second primary
- `RuntimeError::AgentTransitionInProgress` - Attempted action during transition
- `RuntimeError::NoActivePrimaryAgent` - Attempted action with no active agent

### 3. Subagent Delegation with Session Forking

Parent session forks a child session for subagent execution:

```rust
pub struct SubagentResult {
    pub response: AgentResponse,
    pub child_session_id: Uuid,
    pub agent_type: AgentType,
    pub effective_permission_scope: AgentPermissionScope,
}
```

Key invariants:
- Permission scope = parent ∩ subagent
- Parent context modification is detected and prevented
- Child session tracks lineage for audit

### 4. WebSocket Bidirectional Streaming

- Full duplex communication
- Heartbeat/keepalive (30s interval, 120s timeout)
- Token-based reconnection with message replay (last 100 messages)
- Multi-client broadcast per session
- Client-to-server messages: `run`, `resume`, `ping`, `close`

### 5. Comprehensive Error Types

Error catalog with ranges:
- 1xxx: Authentication errors
- 2xxx: Authorization errors
- 3xxx: Provider errors
- 4xxx: Tool errors
- 5xxx: Session errors
- 6xxx: Config errors
- 7xxx: Validation errors
- 9xxx: Internal errors

---

## Design Observations

### 1. Session Hub Bottleneck Risk

The `SessionHub` uses a `tokio::sync::RwLock` for all session management. With many concurrent WebSocket connections, this could become a contention point.

**Recommendation**: Consider per-session locks or session sharding.

### 2. Primary Agent Invariant Complexity

The `PrimaryAgentTracker` enforces exactly-one-primary-agent, but `SubagentResult` allows parallel subagent execution. This is correct but the interaction could be confusing.

**Recommendation**: Clearer naming: "primary" vs "delegated" agents. Document state transitions.

### 3. RuntimeConfig Clone Pattern

```rust
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub permission_scope: AgentPermissionScope,
}
```

If permission scope is meant to be shared, this could lead to unexpected clones.

**Recommendation**: Verify intent; consider `Arc<AgentPermissionScope>` if shared.

### 4. MCP Server Missing Notifications

The MCP server handles requests but doesn't support server-initiated notifications (sampling, rootsChanged, etc.).

**Recommendation**: Add notification support for MCP 1.0 compliance.

### 5. LLM Retry with Exponential Backoff

The `RetryConfig` in `crates/llm/src/error.rs` provides retry protection for transient errors:
- 3 retries by default
- Exponential backoff: 1s → 2s → 4s (capped at 30s)
- Handles: `RateLimitExceeded`, `NetworkError`, `ServerError`, `RequestTimeout`

While a full circuit breaker pattern (open/close/half-open states) isn't implemented, the existing retry mechanism provides reasonable resilience for transient failures.

### 6. Session Fork Memory Pressure

Forks clone the entire message history. Long-running sessions could cause memory pressure.

**Recommendation**: Consider copy-on-write or compaction before fork.

### 7. WebSocket Reconnection Store (Already Bounded)

The `ReconnectionStore` in `crates/server/src/streaming/mod.rs` already implements bounded buffering:
- `DEFAULT_REPLAY_LIMIT = 100` messages per session
- Old messages are popped from the front when limit is exceeded
- No memory spike risk from simultaneous disconnects

---

## Recommendations (Priority Order)

### Completed ✓

1. **Add observability** - AgentRuntime now emits `AgentStarted`, `AgentStopped`, `ToolCallStarted`, `ToolCallEnded` events to the event bus

2. **Document PrimaryAgentTracker state machine** - Added state diagram and transition documentation

3. **Verify build passes** - Build verified after changes

4. **WebSocket Reconnection Store** - Already has bounded buffer (100 messages per session)

5. **LLM Retry** - Already has exponential backoff (1s→30s max, 3 retries)

### Medium Priority

6. **Complete MCP notification support** - For MCP 1.0 compliance

### Low Priority

7. **Consider actor model per session** - For simpler state management

8. **Graceful degradation for offline mode** - Queue requests with TTL

---

## File Reference

| Component | Path | Key Structures |
|-----------|------|----------------|
| Agent Trait | `crates/agent/src/agent.rs` | `Agent`, `AgentType`, `AgentResponse` |
| Runtime | `crates/agent/src/runtime.rs` | `AgentRuntime`, `PrimaryAgentTracker`, `RuntimeConfig` |
| Build Agent | `crates/agent/src/build_agent.rs` | `BuildAgent` |
| MCP Server | `crates/mcp/src/server.rs` | `McpServer`, `ToolHandler`, `ResourceHandler` |
| Server | `crates/server/src/lib.rs` | `ServerState`, `run_server` |
| WebSocket | `crates/server/src/routes/ws/mod.rs` | `SessionHub`, `StreamMessage` |
| Session | `crates/core/src/session/mod.rs` | `Session`, `SessionState` |

---

*Document generated: 2026-04-28*
