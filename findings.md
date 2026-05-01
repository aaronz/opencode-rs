# Findings & Decisions

## Requirements
From design document `docs/DESIGN/agent-runtime-design.md`:

### Critical Architectural Requirements
1. **UI ↔ Runtime strict separation** - RuntimeCommand/Event API
2. **Explicit lifecycle states** - TaskStatus, TurnStatus, RuntimeStatus state machines
3. **Event-driven architecture** - All major actions emit events
4. **ProviderGateway abstraction** - Normalize provider complexity
5. **ToolRouter with validation** - Schema + risk + permission before dispatch
6. **StateStore trait** - Enable testability with fakes
7. **Trace replay** - JSONL persistence + ReplayEngine

### Already Well-Implemented (Skip)
1. ContextBundle with provenance (`core/src/context/mod.rs`)
2. HookEngine (`core/src/hook/engine.rs`)
3. EventBus with DomainEvent (`core/src/bus/`)
4. Tool trait and registry (`tools/src/`)

## Research Findings

### Crate Structure (opencode-rust/crates/)
| Crate | Purpose |
|-------|---------|
| core | IDs, events, session, context, permission, storage |
| runtime | RuntimeFacade, RuntimeFacadeServices, command handling |
| agent | AgentRuntime, agent types |
| llm | Provider implementations (OpenAI, Anthropic, Ollama, etc.) |
| tools | Tool implementations (read, write, bash, git, etc.) |
| tui | Terminal UI (app, dialogs, components) |
| cli | CLI commands |
| server | HTTP server |
| storage | Database layer (rusqlite) |
| plugin | Plugin system (WASM) |
| mcp | MCP protocol integration |
| config | Configuration |

### Component Locations
| Component | Location |
|-----------|----------|
| Runtime | `runtime/src/runtime.rs` - RuntimeFacade |
| Session | `core/src/session/mod.rs` |
| Agent | `agent/src/runtime.rs` - AgentRuntime |
| Context | `core/src/context/mod.rs` - ContextBuilder |
| ToolRouter | `runtime/src/tool_router.rs` |
| PermissionManager | `core/src/permission/types.rs` |
| EventBus | `core/src/bus/mod.rs` + `core/src/bus/types.rs` |
| Storage | `core/src/storage/types.rs` - concrete, not trait |

### Key Gaps Identified
1. **TUI creates RuntimeFacadeServices directly** - No RuntimeCommand API (but RuntimeFacadeCommand exists, just not abstracted as trait)
2. **RuntimeFacadeStatus is minimal** - Only has Idle/Busy/Degraded, not the explicit 12-state lifecycle from design
3. **Storage is concrete** - No StateStore trait for testability
4. **No ProviderGateway** - Each provider handles own request format via Provider trait
5. **ToolRouter lacks validation** - Just wraps registry, no schema validation at dispatch
6. **No trace replay** - In-memory only
7. **Path logic scattered** - No PathResolver trait

### Already Implemented (Skip or Extend)
1. **TaskStatus state machine** ✅ Already in `runtime/src/types.rs` as `RuntimeFacadeTaskStatus` with all design states (Pending, Preparing, Running, WaitingForPermission, Cancelling, Completed, Failed, Cancelled)
2. **RuntimeFacadeEvent projection** ✅ Already in `runtime/src/events.rs` with `from_domain_event()` converter
3. **DomainEvent enum** ✅ Already comprehensive in `core/src/events/mod.rs`
4. **EventBus** ✅ Already in `core/src/bus/types.rs` using broadcast channel
5. **HookEngine** ✅ Already functional in `core/src/hook/engine.rs`

### Design Priority Order (from §3.18 Practical Priority)
```
1. ✅ Already done: ContextBundle with provenance
2. 🔲 Expand RuntimeFacadeStatus to explicit lifecycle states ← TOP PRIORITY
3. 🔲 Add RuntimeHandle trait abstraction for TUI/runtime boundary
4. 🔲 Add StateStore trait for testability
5. 🔲 Introduce ProviderGateway for normalization (lower priority - Provider trait exists)
6. 🔲 Add ToolRouter with schema validation (lower priority)
7. 🔲 Persist trace.jsonl (lower priority)
```

## Technical Decisions

| Decision | Rationale |
|----------|-----------|
| Broadcast channel for EventBus | Multiple subscribers (TUI, CLI, logging) need concurrent access |
| StateStore trait with Arc<dyn> | Allows runtime substitution between file/in-memory implementations |
| ProviderGateway trait | Normalizes all providers so runtime doesn't know provider details |
| JSONL for traces | Append-only, streaming-friendly, easy to replay |
| PathResolver trait | Centralizes all path logic to avoid scattered filesystem access |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| TUI direct runtime coupling | Need to introduce RuntimeCommand API and refactor TUI to use it |
| No state machine for tasks | Add explicit TaskStatus enum with state transitions |

## Resources
- Design document: `/Users/aaronzh/Documents/GitHub/opencode-rs/docs/DESIGN/agent-runtime-design.md`
- Runtime implementation: `opencode-rust/crates/runtime/src/runtime.rs`
- Event definitions: `opencode-rust/crates/core/src/events/mod.rs`
- Context engine: `opencode-rust/crates/core/src/context/mod.rs`
- Tool registry: `opencode-rust/crates/tools/src/registry.rs`
- Provider trait: `opencode-rust/crates/llm/src/provider.rs`

## Visual/Browser Findings
None - all findings from code exploration.
