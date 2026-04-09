# PRD: Core Architecture

## Scope

This document defines the canonical domain entities, ownership relationships, persistence boundaries, and lifecycle invariants for the OpenCode Rust port.

This document is authoritative for:

- Core entities and their relationships
- Identity and ownership invariants
- Persistence and storage boundaries
- Session lifecycle, snapshot, revert, and compaction semantics

This document is **not** authoritative for:

- HTTP route definitions (see [07-server-api.md](./07-server-api.md))
- Configuration schema or precedence (see [06-configuration-system.md](./06-configuration-system.md))
- Plugin APIs (see [08-plugin-system.md](./08-plugin-system.md), [15-tui-plugin-api.md](./15-tui-plugin-api.md))

---

## Canonical Entities

### Project

A `Project` is the durable container for a workspace being operated on by OpenCode.

Canonical responsibilities:

- Defines the root filesystem boundary for a workspace
- Tracks VCS/worktree context when available
- Owns zero or more sessions
- Serves as the top-level logical container for persisted conversational state

Required invariants:

- `Project.id` is stable once created
- A project has exactly one canonical root path
- A project may have a distinct VCS worktree root
- Deleting a project invalidates all descendant session state

### Session

A `Session` is a durable conversational execution context within a project.

Canonical responsibilities:

- Owns an ordered sequence of messages
- Carries execution context such as model/agent selection and lifecycle status
- May reference a parent session when forked
- May have derived artifacts such as summaries, snapshots, and share state

Required invariants:

- A session belongs to exactly one project
- `Session.id` is stable once created
- Forking creates a new session identity; it does not mutate the parent session identity
- Session status transitions are monotonic within a run (`idle` → `running` → terminal state)

### Message

A `Message` is a durable conversational record within a session.

Canonical responsibilities:

- Preserves ordered history for replay, resume, sharing, and summarization
- Captures the user/assistant turn structure
- References structured content parts

Required invariants:

- A message belongs to exactly one session
- Message order is stable within a session
- Persisted messages are append-oriented; destructive mutation is modeled via higher-level session operations such as compaction or revert

### Part

A `Part` is a structured content element attached to a message.

Representative categories include:

- Text
- File references/content
- Images or binary references
- Tool calls and tool results
- Other structured assistant/runtime parts required by the session model

Required invariants:

- A part belongs to exactly one message
- Part order is stable within a message
- The Rust port must treat parts as a versioned extensibility surface, not a closed fixed enum copied from an older PRD draft

---

## Ownership and Boundary Model

The canonical ownership tree is:

```text
Project
└── Session
    └── Message
        └── Part
```

Additional non-owning references may exist:

- Session → parent session (`fork` relationship)
- Session → share/publication state
- Session → snapshot/checkpoint history

The ownership tree must remain acyclic.

---

## Workspace, Directory, Root, and Worktree

The Rust port must distinguish the following filesystem concepts:

- **Project root**: canonical root directory for the project/workspace
- **Working directory**: current execution directory for a given turn or tool call
- **Worktree root**: VCS worktree boundary when different from project root

Invariants:

- A working directory must resolve within the project/worktree boundary unless explicitly permitted
- Session execution context records the relevant path state used for that run
- Filesystem identity and session identity are related but not interchangeable

---

## Session Lifecycle Invariants

### Creation

- A session is created inside exactly one project
- A newly created session starts in an idle/non-running state

### Execution

- A session may enter a running state while processing a prompt, command, or tool-driven workflow
- Execution produces new messages and/or session state changes
- Abort transitions terminate the active run without changing session identity

### Forking

- Forking creates a new child session
- The child references the parent session lineage
- The parent session history remains intact

### Sharing

- Sharing creates share metadata over existing session state
- Sharing must not alter the semantic content of the original conversation history

### Compaction

- Compaction is a session-level history transformation used to reduce context size
- Compaction may summarize, prune, or restructure older history
- Compaction must preserve replay/recovery semantics through snapshot/checkpoint support

### Revert

- Revert is a session-level state restoration operation
- Revert targets a prior checkpoint/snapshot boundary, not an implicit Git commit per edit
- Revert restores session-visible history/state according to the snapshot model

---

## Persistence Model

The Rust port must support durable storage for at least:

- Project metadata
- Session metadata
- Ordered message history
- Snapshot/checkpoint metadata
- Shared key-value or session-adjacent runtime state where required by features

This PRD intentionally does **not** lock the implementation to a specific file layout copied from an older draft. Storage layout is an implementation detail as long as these invariants hold:

- Persistence boundaries align with the ownership model
- Session recovery is possible after process restart
- Message ordering is stable
- Snapshot/revert can be implemented without requiring user-visible Git commits

---

## Snapshot, Revert, and Compaction Model

### Snapshot / Checkpoint

A snapshot (checkpoint) represents a restorable session state boundary.

It exists to support:

- Revert/undo behavior
- Safe compaction
- Recovery and inspection workflows

### Revert

Revert restores a session to a prior checkpoint boundary.

Required invariants:

- Revert does not change project identity
- Revert does not require a per-edit Git commit model
- Revert is expressed in session/state terms, even if the underlying implementation uses VCS objects or snapshot artifacts internally

### Compaction

Compaction reduces active context cost while preserving durable semantics.

Required invariants:

- Compaction must preserve enough information for continuation
- Compaction should occur against a safe checkpoint boundary
- Post-compaction state must still be shareable, resumable, and reviewable

---

## Non-Goals

This document does not define:

- Concrete HTTP route paths
- Exact on-disk storage file names or directories
- Exact wire format for every message part variant
- Plugin, TUI, or provider-specific runtime behavior

---

## Cross-References

- [06-configuration-system.md](./06-configuration-system.md) — configuration ownership and precedence
- [07-server-api.md](./07-server-api.md) — HTTP resource groups and route authority
- [02-agent-system.md](./02-agent-system.md) — agent/session execution model
- [03-tools-system.md](./03-tools-system.md) — tool execution lifecycle
