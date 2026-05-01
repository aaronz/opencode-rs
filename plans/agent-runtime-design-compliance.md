# Implementation Plan: Agent Runtime Design Compliance

## Objective

Bring `opencode-rs` implementation into alignment with `docs/DESIGN/agent-runtime-design.md` (§3 Runtime Design Principles and §3.3 Core Abstractions).

## Background

Analysis of the codebase reveals ~60% alignment with the design. Key gaps identified:

| Gap | Severity | File(s) |
|-----|----------|---------|
| Session lacks `workspace_id` | High | `crates/core/src/session/mod.rs` |
| PathResolver incomplete | High | `crates/runtime/src/path_resolver.rs` |
| ToolDefinition missing `risk_level` | High | `crates/core/src/tool/types.rs` |
| Runtime state machine partial | Medium | `crates/runtime/src/types.rs` (already done) |
| TraceStore stub | Medium | `crates/runtime/src/trace_store.rs` |
| CommandDefinition missing | Medium | `crates/core/src/command/` |
| ContextBundle provenance | Medium | `crates/core/src/context/` |

---

## Step 1: Add `workspace_id` to Session struct

**Context Brief:** The design (§3.3 Session) specifies that a Session should be "workspace-bound" with a `workspace_id` field. Currently `Session` in `core/src/session/mod.rs` has no workspace association - it's purely conversation-bound.

**Files to modify:**
- `crates/core/src/session/mod.rs` — add `workspace_id: WorkspaceId` field and `WorkspaceId` type
- `crates/core/src/id/types.rs` — add `WorkspaceId` type if not present
- `crates/core/src/session/session_info.rs` — update `SessionInfo` if needed

**Verification:**
```bash
cd opencode-rust && cargo build -p opencode-core
```

**Exit criteria:** `Session` struct contains `workspace_id: WorkspaceId` field; `WorkspaceId` is a proper newtype wrapper around `Uuid`.

**Rollback:** `git checkout` on modified files.

---

## Step 2: Complete `PathResolver` trait

**Context Brief:** The design (§3.6 Storage Layout, §3.6 Path Strategy) specifies a `PathResolver` trait with methods for user-level and project-level paths. The stub exists at `runtime/src/path_resolver.rs` but is missing project-level methods.

**Files to modify:**
- `crates/runtime/src/path_resolver.rs` — add `project_config_dir`, `project_state_dir` methods; add `Workspace`-bound methods
- Add tests for project path resolution

**Verification:**
```bash
cd opencode-rust && cargo build -p opencode-runtime
cargo test -p opencode-runtime path_resolver
```

**Exit criteria:** `PathResolver` trait has all 5 methods per design; `DefaultPathResolver` implements all; tests pass.

**Rollback:** `git checkout` on modified files.

---

## Step 3: Add `risk_level` to ToolDefinition

**Context Brief:** The design (§3.8 Tool Execution and Safety, Risk Levels table) specifies 5 risk levels (ReadOnly, Low, Medium, High, Critical). Currently `ToolDefinition` only has `requires_approval: bool` - not sufficient for risk classification.

**Files to modify:**
- `crates/core/src/tool/types.rs` — add `RiskLevel` enum and `risk_level` field to `ToolDefinition`
- `crates/tools/src/` — update tool implementations to set appropriate risk levels
- `crates/runtime/src/tool_router.rs` — use risk level for permission checks

**Verification:**
```bash
cd opencode-rust && cargo build --all
cargo test -p opencode-core tool
```

**Exit criteria:** `RiskLevel` enum exists with 5 levels; `ToolDefinition` has `risk_level: RiskLevel`; all built-in tools have risk levels assigned.

**Rollback:** `git checkout` on modified files.

---

## Step 4: Verify Runtime state machine

**Context Brief:** Already implemented in `runtime/src/types.rs` with `RuntimeStatus` enum covering Idle → Completed/Failed/Cancelled/Interrupted states. This step verifies completeness and adds any missing transitions.

**Verification:**
```bash
cd opencode-rust && cargo build -p opencode-runtime
cargo test -p opencode-runtime types
```

**Exit criteria:** `RuntimeStatus` enum matches design §3.3 Task state machine (Pending, Preparing, Running, WaitingForPermission, Cancelling, Completed, Failed, Cancelled); state transition methods exist and are tested.

**Note:** This step may be trivial - confirm and document.

---

## Step 5: Complete TraceStore with replay capability

**Context Brief:** The design (§3.6 Replay and Debugging, §3.3 Trace) specifies a `TraceStore` that persists replayable execution traces. The stub exists but is incomplete.

**Files to modify:**
- `crates/runtime/src/trace_store.rs` — implement full trace persistence
- `crates/runtime/src/events.rs` — ensure events are traceable
- Add integration tests with fake provider gateway

**Verification:**
```bash
cd opencode-rust && cargo build -p opencode-runtime
cargo test -p opencode-runtime trace
```

**Exit criteria:** Traces can be persisted and replayed; `TraceStore` trait is fully implemented by `InMemoryTraceStore`; replay tests pass.

**Rollback:** `git checkout` on modified files.

---

## Step 6: Add CommandDefinition workflow system

**Context Brief:** The design (§3.9 Commands) specifies a `CommandDefinition` struct for named workflows (e.g., `/fix-tests`, `/review`). Currently commands are ad-hoc in `crates/core/src/command/`.

**Files to modify:**
- `crates/core/src/command/types.rs` — add `CommandDefinition` struct with workflow support
- `crates/core/src/command/registry.rs` — update registry to handle workflow commands
- `crates/core/src/command/builtins.rs` — refactor existing commands

**Verification:**
```bash
cd opencode-rust && cargo build -p opencode-core
cargo test -p opencode-core command
```

**Exit criteria:** `CommandDefinition` exists with `workflow: CommandWorkflow` field; at least one built-in command uses the workflow system.

**Rollback:** `git checkout` on modified files.

---

## Step 7: Add ContextBundle provenance and ranking reports

**Context Brief:** The design (§3.7 Context Engineering) specifies `ContextBuilt` events should include `truncation_report` and `ranking_report`. Currently `Context` in `core/src/context/` provides `ContextLayer` but not the full provenance tracking.

**Files to modify:**
- `crates/core/src/context/mod.rs` — add provenance tracking to `ContextBuilder`
- `crates/core/src/events/mod.rs` — update `ContextBuilt` event with provenance fields
- `crates/runtime/src/context_view.rs` — expose provenance in `RuntimeFacadeContextSummary`

**Verification:**
```bash
cd opencode-rust && cargo build --all
cargo test -p opencode-core context
```

**Exit criteria:** Context items carry provenance info; `ContextBuilt` event includes truncation and ranking reports; context inspect command works.

**Rollback:** `git checkout` on modified files.

---

## Step 8: Final verification

**Verification:**
```bash
cd opencode-rust && cargo fmt --all
cargo clippy --all -- -D warnings
cargo test --all
```

**Exit criteria:** All formatting passes; clippy passes; all tests pass.

---

## Dependency Graph

```
Step 1 (Session.workspace_id)
    └── Step 2 (PathResolver completion)
    └── Step 3 (risk_level)
    └── Step 4 (Runtime state)      [independent]
    └── Step 5 (TraceStore)
    └── Step 6 (CommandDefinition)
    └── Step 7 (Context provenance)
              └── Step 8 (final verification)
```

**Parallel execution:** Steps 2, 3, 4, 5, 6, 7 can proceed in parallel after Step 1.

**Model tier:** Steps 1-3 are medium complexity (default model OK); Steps 5-7 require careful trait design (use stronger model or thorough review).

---

## Anti-Patterns to Avoid

- Do not add `workspace_id` without updating all Session consumers
- Do not change `RiskLevel` enum after tools are registered (breakage)
- Do not implement TraceStore replay without deterministic event ordering
- Do not add CommandDefinition workflow without parsing/validation

## Success Criteria

After all steps: opencode-rs implementation is ≥85% aligned with agent-runtime-design.md §3.
