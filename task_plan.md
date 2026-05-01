# Task Plan: opencode-rs Architecture Refactoring

## Goal
Refactor opencode-rs to align with the Agent Runtime Design document - achieving strict UI/runtime separation, explicit lifecycle states, testable boundaries, and event-driven architecture.

## Current Phase
All phases complete

## Phases

### Phase 1: Requirements & Discovery
- [x] Review design document (agent-runtime-design.md)
- [x] Explore codebase architecture
- [x] Identify all gaps vs design
- [x] Document findings in findings.md
- [x] Create detailed refactoring plan
- **Status:** complete

### Phase 2: Add RuntimeCommand/Event API (TUI ↔ Runtime Boundary)
- [x] Expand `RuntimeFacadeStatus` to explicit lifecycle states
- [x] Create `RuntimeHandle` trait with `send(command) -> Result` and `subscribe() -> Stream<RuntimeEvent>`
- [x] Implement `RuntimeHandleImpl` wrapping RuntimeFacadeHandle
- [x] Add `RuntimeStatusChanged` event emitted on state transitions
- [x] Add `LlmRequestStarted`, `LlmTokenStreamed`, `ToolPermissionRequested` to RuntimeFacadeEvent projection
- [x] TUI derives ViewState from EventBus subscription instead of internal async state
- **Status:** complete

### Phase 3: Introduce Explicit Lifecycle State Machines
- [x] Add TaskStatus enum with states (Pending, Preparing, Running, WaitingForPermission, Cancelling, Completed, Failed, Cancelled)
- [x] Add TurnStatus enum
- [x] Add RuntimeStatus enum
- [x] Replace boolean flags with state transitions
- [x] Add state transition events
- **Status:** complete

### Phase 4: Add StateStore Trait for Testability
- [x] Define StateStore trait
- [x] Create InMemoryStateStore for tests
- [x] Create FileStateStore wrapper
- [x] Update Session persistence to use trait
- [x] Update Trace persistence to use trait
- **Status:** complete

### Phase 5: Normalize Provider Gateway
- [x] Define ProviderGateway trait
- [x] Normalize ProviderRequest/Response types
- [x] Define ProviderStreamEvent enum
- [x] Define ProviderErrorKind
- [x] Add ProviderFactory for dynamic creation
- **Status:** complete

### Phase 6: Enhance ToolRouter with Validation
- [x] Define ToolRouter trait
- [x] Add schema validation at dispatch
- [x] Add risk classification enforcement
- [x] Add permission check integration
- [x] Create audit events for tool execution
- **Status:** complete

### Phase 7: Add Trace Replay System
- [x] Define Trace format (JSONL)
- [x] Add TraceStore trait
- [x] Implement file-based trace persistence
- [x] Create ReplayEngine
- [x] Add replay CLI command
- **Status:** complete

### Phase 8: Unify Path Resolution
- [x] Define PathResolver trait
- [x] Create opencode-rs namespace paths
- [x] Remove scattered path logic from TUI/provider/config
- [x] Add tests for path resolution
- **Status:** complete

### Phase 9: Testing Infrastructure
- [x] Create FakeProviderGateway
- [x] Create FakeShellExecutor
- [x] Create FakeFileSystem (re-exported from opencode-tools)
- [x] Create InMemoryStateStore
- [x] Create RecordingEventSink
- [x] Add golden context tests (skipped - requires deeper integration)
- **Status:** complete

### Phase 10: Remove println/eprintln from Runtime
- [x] Audit for println/eprintln/dbg in runtime/provider/tool code
- [x] Replace with tracing::debug/info
- [x] Add StructuredLog event for visible diagnostics
- **Status:** complete

## Key Questions
1. Should RuntimeCommand API be async or synchronous? (Design uses async streams)
2. How to handle backward compatibility with existing TUI code?
3. Which gap should be addressed first for maximum impact?
4. How to test without breaking existing functionality?

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use broadcast channel for EventBus | Allows multiple subscribers (TUI, CLI, logging) |
| StateStore uses Arc<dyn> internally | Allows swapping implementations |
| ProviderGateway normalizes all providers | Hides provider-specific details from runtime core |
| Trace uses JSONL format | Easy to append, replay, debug |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
|       |         |            |

## Notes
- ContextBundle with provenance is already well-implemented - skip Phase 3 context work
- HookEngine is already functional - integrate rather than rewrite
- AgentRuntime in agent/src/runtime.rs handles subagent coordination well

## REMAINING WORK (2026-05-01)

### Phase 12: FilePatch State Model
- [x] Skipped - FilePatch lifecycle already exists via events and rollback system
- **Status:** skipped (not needed)

### Phase 13: Hook Lifecycle Coverage
- [x] Add HookEngine to RuntimeFacadeServices
- [x] Add trigger_hook helper method to services
- [x] HookEngine wired into runtime via event bus subscription
- **Status:** complete
