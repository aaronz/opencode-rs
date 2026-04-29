# Runtime Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Incrementally evolve `opencode-rs` toward the runtime architecture in `docs/DESIGN/agent-runtime-design.md` by introducing a unified runtime boundary and progressively moving session, turn, task, event, execution, permission, and persistence ownership behind it.

**Architecture:** Start by adding a thin `runtime` façade with no behavior changes, then introduce first-class `Turn`, canonical runtime events, a unified submission path, a broader task model, and finally collapse duplicate tool, permission, and persistence boundaries behind runtime-owned interfaces. Avoid a big-bang rewrite; each PR must preserve existing behavior while creating the next migration seam.

**Tech Stack:** Rust 2021, Tokio, Serde, ThisError, existing `opencode-*` workspace crates, SQLite-backed storage.

---

## File Structure Map

### New files planned

- `opencode-rust/crates/runtime/Cargo.toml` — new runtime crate manifest
- `opencode-rust/crates/runtime/src/lib.rs` — runtime public exports
- `opencode-rust/crates/runtime/src/runtime.rs` — runtime façade and handle
- `opencode-rust/crates/runtime/src/types.rs` — runtime statuses/responses
- `opencode-rust/crates/runtime/src/commands.rs` — runtime command types
- `opencode-rust/crates/runtime/src/services.rs` — runtime dependency container
- `opencode-rust/crates/runtime/src/errors.rs` — runtime façade errors
- `opencode-rust/crates/core/src/turn/mod.rs` — turn module exports
- `opencode-rust/crates/core/src/turn/types.rs` — `Turn`, `TurnId`, `TurnStatus`

### Existing files central to this plan

- `opencode-rust/crates/agent/src/runtime.rs` — current orchestration loop; later runtime integration point
- `opencode-rust/crates/agent/src/delegation.rs` — existing task model limited to subagent delegation
- `opencode-rust/crates/core/src/session/mod.rs` — session model and persistence shape
- `opencode-rust/crates/core/src/session_state/types.rs` — existing explicit session lifecycle
- `opencode-rust/crates/core/src/context/mod.rs` — current context builder and token budgeting
- `opencode-rust/crates/core/src/bus/types.rs` — current `InternalEvent` bus
- `opencode-rust/crates/core/src/tool/mod.rs` — sync tool registry surface
- `opencode-rust/crates/core/src/executor/mod.rs` — sync tool executor wrapper
- `opencode-rust/crates/tools/src/registry.rs` — async tool registry
- `opencode-rust/crates/tools/src/tool.rs` — async tool trait/context
- `opencode-rust/crates/core/src/permission/types.rs` — core permission wrapper
- `opencode-rust/crates/permission/src/lib.rs` — richer permission subsystem
- `opencode-rust/crates/storage/src/service.rs` — storage service façade
- `opencode-rust/crates/storage/src/repository.rs` — storage traits
- `opencode-rust/crates/storage/src/sqlite_repository.rs` — SQLite persistence
- `opencode-rust/crates/server/src/lib.rs` — server application state wiring
- `opencode-rust/crates/server/src/routes/run.rs` — prompt execution path
- `opencode-rust/crates/server/src/routes/execute/mod.rs` — execute endpoint wiring
- `opencode-rust/crates/server/src/routes/execute/integration.rs` — execution integration path
- `opencode-rust/crates/server/src/streaming/mod.rs` — stream/replay adapter layer
- `opencode-rust/crates/tui/src/app.rs` — TUI runtime wiring
- `opencode-rust/crates/tui/src/session.rs` — TUI-local session manager

### Boundary decisions locked in by this plan

- Create a **new `runtime` crate** rather than extending `core`
- Keep `crates/tools/` as the long-term authoritative tool execution layer
- Keep `crates/permission/` as the long-term authoritative permission system
- Introduce `Turn` as embedded session metadata first; avoid a schema-heavy redesign in the first turn PR

---

## PR Sequence Overview

### PR1: Runtime façade scaffolding
- Add `crates/runtime/`
- Define `Runtime`, `RuntimeHandle`, `RuntimeCommand`, `RuntimeResponse`, `RuntimeServices`
- Make server/TUI able to hold a runtime instance
- No behavior change

### PR2: First-class `Turn`
- Add `Turn`, `TurnId`, `TurnStatus`
- Persist turns inside `Session`
- Return `turn_id` from submit-shaped runtime responses

### PR3: Canonical internal event model
- Introduce one runtime event schema
- Adapt server/control-plane streaming from it

### PR4: Unified runtime submission path
- Make server/TUI submit via runtime instead of separate orchestration

### PR5: Generalize task model
- Expand tasking beyond subagent delegation

### PR6: Tool router unification
- Make runtime use one authoritative tool-routing boundary

### PR7: Permission API unification
- Route checks/approval/audit through one runtime permission interface

### PR8: Persistence abstraction
- Introduce runtime-owned persistence traits for sessions/turns/tasks/events/checkpoints

### PR9: Richer context bundle
- Add provenance, ranking, truncation inspectability

### PR10: Cleanup and deprecation
- Remove or thin duplicate orchestration layers

---

## Task 1: PR1 Runtime Façade Scaffolding

**Files:**
- Create: `opencode-rust/crates/runtime/Cargo.toml`
- Create: `opencode-rust/crates/runtime/src/lib.rs`
- Create: `opencode-rust/crates/runtime/src/runtime.rs`
- Create: `opencode-rust/crates/runtime/src/types.rs`
- Create: `opencode-rust/crates/runtime/src/commands.rs`
- Create: `opencode-rust/crates/runtime/src/services.rs`
- Create: `opencode-rust/crates/runtime/src/errors.rs`
- Modify: `opencode-rust/Cargo.toml`
- Modify: `opencode-rust/crates/server/Cargo.toml`
- Modify: `opencode-rust/crates/tui/Cargo.toml`
- Modify: `opencode-rust/crates/server/src/lib.rs`
- Modify: `opencode-rust/crates/tui/src/app.rs`
- Test: `opencode-rust/crates/runtime/tests/runtime_smoke.rs`

- [ ] **Step 1: Add the new runtime crate to the workspace**

Update `opencode-rust/Cargo.toml` workspace members to include:

```toml
"crates/runtime",
```

Run: `cargo metadata --no-deps`
Expected: command succeeds and includes `opencode-runtime`

- [ ] **Step 2: Create the runtime crate manifest**

Create `opencode-rust/crates/runtime/Cargo.toml`:

```toml
[package]
name = "opencode-runtime"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true, features = ["sync", "rt", "macros"] }
opencode-agent = { path = "../agent" }
opencode-core = { path = "../core" }
opencode-storage = { path = "../storage" }

[dev-dependencies]
tempfile = { workspace = true }
```

Run: `cargo check -p opencode-runtime`
Expected: crate is discovered; compile may still fail until source files are added

- [ ] **Step 3: Add runtime public exports**

Create `opencode-rust/crates/runtime/src/lib.rs`:

```rust
pub mod commands;
pub mod errors;
pub mod runtime;
pub mod services;
pub mod types;

pub use commands::{PermissionResponse, RuntimeCommand, SubmitUserInput, TaskControlCommand};
pub use errors::RuntimeFacadeError;
pub use runtime::{Runtime, RuntimeHandle};
pub use services::RuntimeServices;
pub use types::{RuntimeResponse, RuntimeStatus};
```

Run: `cargo check -p opencode-runtime`
Expected: compile still fails only because the referenced modules are not created yet

- [ ] **Step 4: Define small runtime response/status types**

Create `opencode-rust/crates/runtime/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeStatus {
    Idle,
    Busy,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResponse {
    pub session_id: Option<String>,
    pub accepted: bool,
    pub message: String,
}
```

Run: `cargo check -p opencode-runtime`
Expected: compile advances to missing command/service/runtime definitions

- [ ] **Step 5: Define runtime command envelope**

Create `opencode-rust/crates/runtime/src/commands.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitUserInput {
    pub session_id: Option<String>,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskControlCommand {
    Cancel { task_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub request_id: String,
    pub granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeCommand {
    SubmitUserInput(SubmitUserInput),
    TaskControl(TaskControlCommand),
    PermissionResponse(PermissionResponse),
}
```

Run: `cargo check -p opencode-runtime`
Expected: compile advances to missing service/runtime/error definitions

- [ ] **Step 6: Define narrow runtime error type**

Create `opencode-rust/crates/runtime/src/errors.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeFacadeError {
    #[error("runtime command not yet implemented: {0}")]
    NotImplemented(&'static str),

    #[error("runtime dependency error: {0}")]
    Dependency(String),
}
```

Run: `cargo check -p opencode-runtime`
Expected: compile advances to missing service/runtime definitions

- [ ] **Step 7: Define runtime service container**

Create `opencode-rust/crates/runtime/src/services.rs`:

```rust
use std::sync::Arc;

use tokio::sync::RwLock;

use opencode_agent::AgentRuntime;
use opencode_core::bus::SharedEventBus;
use opencode_core::permission::PermissionManager;
use opencode_storage::StorageService;

pub struct RuntimeServices {
    pub event_bus: SharedEventBus,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    pub storage: Arc<StorageService>,
    pub agent_runtime: Arc<RwLock<AgentRuntime>>,
}

impl RuntimeServices {
    pub fn new(
        event_bus: SharedEventBus,
        permission_manager: Arc<RwLock<PermissionManager>>,
        storage: Arc<StorageService>,
        agent_runtime: Arc<RwLock<AgentRuntime>>,
    ) -> Self {
        Self {
            event_bus,
            permission_manager,
            storage,
            agent_runtime,
        }
    }
}
```

Run: `cargo check -p opencode-runtime`
Expected: compile advances to missing runtime façade only

- [ ] **Step 8: Implement minimal runtime façade and handle**

Create `opencode-rust/crates/runtime/src/runtime.rs`:

```rust
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::commands::RuntimeCommand;
use crate::errors::RuntimeFacadeError;
use crate::services::RuntimeServices;
use crate::types::{RuntimeResponse, RuntimeStatus};

pub struct Runtime {
    services: RuntimeServices,
    status: Arc<RwLock<RuntimeStatus>>,
}

impl Runtime {
    pub fn new(services: RuntimeServices) -> Self {
        Self {
            services,
            status: Arc::new(RwLock::new(RuntimeStatus::Idle)),
        }
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle {
            services: RuntimeServices {
                event_bus: self.services.event_bus.clone(),
                permission_manager: self.services.permission_manager.clone(),
                storage: self.services.storage.clone(),
                agent_runtime: self.services.agent_runtime.clone(),
            },
            status: Arc::clone(&self.status),
        }
    }

    pub async fn status(&self) -> RuntimeStatus {
        self.status.read().await.clone()
    }

    pub async fn execute(
        &self,
        command: RuntimeCommand,
    ) -> Result<RuntimeResponse, RuntimeFacadeError> {
        match command {
            RuntimeCommand::SubmitUserInput(cmd) => Ok(RuntimeResponse {
                session_id: cmd.session_id,
                accepted: false,
                message: "submit_user_input not yet wired".to_string(),
            }),
            RuntimeCommand::TaskControl(_) => {
                Err(RuntimeFacadeError::NotImplemented("task control"))
            }
            RuntimeCommand::PermissionResponse(_) => {
                Err(RuntimeFacadeError::NotImplemented("permission response"))
            }
        }
    }
}

#[derive(Clone)]
pub struct RuntimeHandle {
    services: RuntimeServices,
    status: Arc<RwLock<RuntimeStatus>>,
}

impl RuntimeHandle {
    pub async fn status(&self) -> RuntimeStatus {
        self.status.read().await.clone()
    }

    pub async fn execute(
        &self,
        command: RuntimeCommand,
    ) -> Result<RuntimeResponse, RuntimeFacadeError> {
        let runtime = Runtime {
            services: RuntimeServices {
                event_bus: self.services.event_bus.clone(),
                permission_manager: self.services.permission_manager.clone(),
                storage: self.services.storage.clone(),
                agent_runtime: self.services.agent_runtime.clone(),
            },
            status: Arc::clone(&self.status),
        };
        runtime.execute(command).await
    }
}
```

Run: `cargo check -p opencode-runtime`
Expected: compile succeeds or reports only trait/bounds issues that can be fixed without changing behavior

- [ ] **Step 9: Add a smoke test for runtime construction**

Create `opencode-rust/crates/runtime/tests/runtime_smoke.rs` with three tests:

```rust
use std::sync::Arc;

use tokio::sync::RwLock;

use opencode_agent::{AgentRuntime, AgentType};
use opencode_core::{bus::EventBus, permission::PermissionManager, Session};
use opencode_runtime::{Runtime, RuntimeCommand, RuntimeServices, RuntimeStatus, SubmitUserInput};
use opencode_storage::{InMemoryProjectRepository, InMemorySessionRepository, StoragePool, StorageService};

fn build_runtime() -> Runtime {
    let event_bus = Arc::new(EventBus::new());
    let permission_manager = Arc::new(RwLock::new(PermissionManager::default()));
    let session_repo = Arc::new(InMemorySessionRepository::default());
    let project_repo = Arc::new(InMemoryProjectRepository::default());
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("runtime-smoke.db");
    let pool = StoragePool::new(&db_path).expect("storage pool");
    let storage = Arc::new(StorageService::new(session_repo, project_repo, pool));
    let agent_runtime = Arc::new(RwLock::new(AgentRuntime::new(Session::default(), AgentType::Build)));

    Runtime::new(RuntimeServices::new(
        event_bus,
        permission_manager,
        storage,
        agent_runtime,
    ))
}

#[tokio::test]
async fn runtime_constructs_and_reports_idle_status() {
    let runtime = build_runtime();
    assert_eq!(runtime.status().await, RuntimeStatus::Idle);
}

#[tokio::test]
async fn runtime_accepts_submit_command_shape() {
    let runtime = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::SubmitUserInput(SubmitUserInput {
            session_id: None,
            input: "hello".to_string(),
        }))
        .await
        .expect("submit command should return placeholder response");

    assert!(!result.accepted);
    assert_eq!(result.message, "submit_user_input not yet wired");
}

#[tokio::test]
async fn runtime_unimplemented_commands_return_explicit_errors() {
    let runtime = build_runtime();
    let result = runtime
        .execute(opencode_runtime::RuntimeCommand::TaskControl(
            opencode_runtime::TaskControlCommand::Cancel {
                task_id: "task-1".to_string(),
            },
        ))
        .await;

    assert!(matches!(
        result,
        Err(opencode_runtime::RuntimeFacadeError::NotImplemented("task control"))
    ));
}
```

Run: `cargo test -p opencode-runtime`
Expected: all runtime smoke tests pass

- [ ] **Step 10: Make server able to hold runtime**

Modify `opencode-rust/crates/server/Cargo.toml` to depend on `opencode-runtime`:

```toml
opencode-runtime = { path = "../runtime" }
```

Modify `opencode-rust/crates/server/src/lib.rs` server state to include:

```rust
pub runtime: Arc<opencode_runtime::Runtime>,
```

Construct it from existing dependencies already created in server startup. Do **not** reroute endpoints yet.

Run: `cargo check -p opencode-server`
Expected: server compiles with runtime available in app state; no route behavior changes

- [ ] **Step 11: Make TUI able to hold runtime**

Modify `opencode-rust/crates/tui/Cargo.toml` to depend on `opencode-runtime`:

```toml
opencode-runtime = { path = "../runtime" }
```

Modify `opencode-rust/crates/tui/src/app.rs` to store either `Runtime` or `RuntimeHandle` alongside existing runtime-related fields. Instantiate it where TUI currently creates `AgentRuntime`. Do **not** reroute prompt execution in this PR.

Run: `cargo check -p opencode-tui`
Expected: TUI compiles and retains existing behavior

- [ ] **Step 12: Run full validation for PR1**

Run:

```bash
cargo test -p opencode-runtime
cargo test
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
```

Expected:
- all tests pass
- formatting clean
- clippy clean
- no endpoint behavior regressions

- [ ] **Step 13: Commit PR1**

```bash
git add opencode-rust/Cargo.toml \
  opencode-rust/crates/runtime \
  opencode-rust/crates/server/Cargo.toml \
  opencode-rust/crates/server/src/lib.rs \
  opencode-rust/crates/tui/Cargo.toml \
  opencode-rust/crates/tui/src/app.rs
git commit -m "add runtime facade scaffolding"
```

---

## Task 2: PR2 First-Class `Turn`

**Files:**
- Create: `opencode-rust/crates/core/src/turn/mod.rs`
- Create: `opencode-rust/crates/core/src/turn/types.rs`
- Modify: `opencode-rust/crates/core/src/lib.rs`
- Modify: `opencode-rust/crates/core/src/session/mod.rs`
- Modify: `opencode-rust/crates/storage/src/models.rs`
- Modify: `opencode-rust/crates/storage/src/repository.rs`
- Modify: `opencode-rust/crates/storage/src/sqlite_repository.rs`
- Modify: `opencode-rust/crates/storage/src/service.rs`
- Modify: `opencode-rust/crates/runtime/src/types.rs`
- Modify: `opencode-rust/crates/runtime/src/runtime.rs`
- Modify: `opencode-rust/crates/agent/src/runtime.rs` (only if a narrow session mutability accessor is needed)
- Test: `opencode-rust/crates/runtime/tests/runtime_smoke.rs`

- [ ] **Step 1: Add the turn core types**

Create `opencode-rust/crates/core/src/turn/types.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TurnId(pub Uuid);

impl TurnId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TurnId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub id: TurnId,
    pub session_id: Uuid,
    pub user_message_id: Option<String>,
    pub status: TurnStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

Create `opencode-rust/crates/core/src/turn/mod.rs`:

```rust
pub mod types;

pub use types::{Turn, TurnId, TurnStatus};
```

Run: `cargo check -p opencode-core`
Expected: compile advances to missing exports/session usage

- [ ] **Step 2: Export turns from core**

Modify `opencode-rust/crates/core/src/lib.rs`:

```rust
pub mod turn;
pub use turn::{Turn, TurnId, TurnStatus};
```

Run: `cargo check -p opencode-core`
Expected: core recognizes the new turn module publicly

- [ ] **Step 3: Add turn fields to `Session`**

Modify `opencode-rust/crates/core/src/session/mod.rs` `Session` struct:

```rust
#[serde(skip_serializing_if = "Vec::is_empty", default)]
pub turns: Vec<Turn>,
#[serde(skip_serializing_if = "Option::is_none", default)]
pub active_turn_id: Option<TurnId>,
```

Initialize in `Session::new()`:

```rust
turns: Vec::new(),
active_turn_id: None,
```

Run: `cargo check -p opencode-core`
Expected: session compiles with additive turn metadata

- [ ] **Step 4: Add minimal turn lifecycle helpers to `Session`**

Modify `opencode-rust/crates/core/src/session/mod.rs` with:

```rust
pub fn start_turn(&mut self, user_message_id: Option<String>) -> TurnId {
    let turn = Turn {
        id: TurnId::new(),
        session_id: self.id,
        user_message_id,
        status: TurnStatus::Running,
        started_at: Utc::now(),
        completed_at: None,
    };
    let turn_id = turn.id;
    self.active_turn_id = Some(turn_id);
    self.turns.push(turn);
    self.updated_at = Utc::now();
    turn_id
}

pub fn complete_turn(&mut self, turn_id: TurnId) {
    if let Some(turn) = self.turns.iter_mut().find(|t| t.id == turn_id) {
        turn.status = TurnStatus::Completed;
        turn.completed_at = Some(Utc::now());
    }
    if self.active_turn_id == Some(turn_id) {
        self.active_turn_id = None;
    }
    self.updated_at = Utc::now();
}

pub fn fail_turn(&mut self, turn_id: TurnId) {
    if let Some(turn) = self.turns.iter_mut().find(|t| t.id == turn_id) {
        turn.status = TurnStatus::Failed;
        turn.completed_at = Some(Utc::now());
    }
    if self.active_turn_id == Some(turn_id) {
        self.active_turn_id = None;
    }
    self.updated_at = Utc::now();
}
```

Run: `cargo check -p opencode-core`
Expected: helper methods compile without altering existing flows

- [ ] **Step 5: Add core turn tests**

Add tests near existing `Session` tests:

```rust
#[test]
fn session_start_turn_sets_active_turn() {
    let mut session = Session::new();
    let turn_id = session.start_turn(None);

    assert_eq!(session.active_turn_id, Some(turn_id));
    assert_eq!(session.turns.len(), 1);
    assert_eq!(session.turns[0].status, TurnStatus::Running);
}

#[test]
fn session_complete_turn_clears_active_turn() {
    let mut session = Session::new();
    let turn_id = session.start_turn(None);
    session.complete_turn(turn_id);

    assert_eq!(session.active_turn_id, None);
    assert_eq!(session.turns[0].status, TurnStatus::Completed);
    assert!(session.turns[0].completed_at.is_some());
}

#[test]
fn session_fail_turn_marks_failed() {
    let mut session = Session::new();
    let turn_id = session.start_turn(None);
    session.fail_turn(turn_id);

    assert_eq!(session.turns[0].status, TurnStatus::Failed);
}
```

Run: `cargo test -p opencode-core session_`
Expected: new turn-related session tests pass

- [ ] **Step 6: Keep persistence simple by embedding turns in session serialization**

Modify `opencode-rust/crates/storage/src/models.rs`, `repository.rs`, and `sqlite_repository.rs` only as needed to ensure sessions with `turns` and `active_turn_id` serialize/deserialize successfully. Do **not** introduce a new turns table in PR2.

If a storage-facing model is required, add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnModel {
    pub id: String,
    pub session_id: String,
    pub user_message_id: Option<String>,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

Run: `cargo check -p opencode-storage`
Expected: storage compiles without schema explosion

- [ ] **Step 7: Add storage round-trip coverage for turns**

In existing storage lifecycle tests, add:

```rust
#[tokio::test]
async fn session_with_turns_round_trips_through_storage() {
    let mut session = Session::new();
    let turn_id = session.start_turn(None);
    session.complete_turn(turn_id);

    service.save_session(&session).await.unwrap();
    let loaded = service
        .load_session(&session.id.to_string())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.turns.len(), 1);
    assert_eq!(loaded.active_turn_id, None);
}
```

Run: `cargo test -p opencode-storage round_trips`
Expected: turn metadata survives storage round-trip

- [ ] **Step 8: Extend runtime responses to carry `turn_id`**

Modify `opencode-rust/crates/runtime/src/types.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResponse {
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub accepted: bool,
    pub message: String,
}
```

Run: `cargo check -p opencode-runtime`
Expected: compile errors point only to response construction sites needing updates

- [ ] **Step 9: Add narrow mutable session access if runtime needs it**

If `Runtime` cannot create turns without reaching inside `AgentRuntime`, add the smallest accessor to `opencode-rust/crates/agent/src/runtime.rs`:

```rust
pub async fn with_session_mut<R>(
    &self,
    f: impl FnOnce(&mut Session) -> R,
) -> R {
    let mut guard = self.session.write().await;
    f(&mut guard)
}
```

Run: `cargo check -p opencode-agent`
Expected: agent crate compiles with no behavior changes

- [ ] **Step 10: Make runtime submit create a turn**

Modify `opencode-rust/crates/runtime/src/runtime.rs` submit branch so it creates a turn and returns it:

```rust
RuntimeCommand::SubmitUserInput(cmd) => {
    let (session_id, turn_id) = {
        let agent_runtime = self.services.agent_runtime.write().await;
        agent_runtime.with_session_mut(|session| {
            let turn_id = session.start_turn(None);
            (session.id.to_string(), turn_id.0.to_string())
        }).await
    };

    Ok(RuntimeResponse {
        session_id: Some(session_id),
        turn_id: Some(turn_id),
        accepted: true,
        message: "turn created".to_string(),
    })
}
```

If the exact async shape differs, preserve the semantics rather than the syntax.

Run: `cargo test -p opencode-runtime`
Expected: submit now returns `accepted == true` and `turn_id.is_some()`

- [ ] **Step 11: Update runtime smoke tests for turn creation**

Update `opencode-rust/crates/runtime/tests/runtime_smoke.rs` submit test to:

```rust
#[tokio::test]
async fn submit_user_input_creates_turn_and_returns_turn_id() {
    let runtime = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::SubmitUserInput(SubmitUserInput {
            session_id: None,
            input: "hello".to_string(),
        }))
        .await
        .expect("submit command should create a turn");

    assert!(result.accepted);
    assert!(result.turn_id.is_some());
    assert_eq!(result.message, "turn created");
}
```

Run: `cargo test -p opencode-runtime`
Expected: runtime smoke tests pass with explicit turn IDs

- [ ] **Step 12: Run full validation for PR2**

Run:

```bash
cargo test -p opencode-runtime
cargo test -p opencode-core
cargo test -p opencode-storage
cargo test
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
```

Expected:
- all tests pass
- existing sessions remain backward-compatible via serde defaults
- no route behavior regressions

- [ ] **Step 13: Commit PR2**

```bash
git add opencode-rust/crates/core/src/lib.rs \
  opencode-rust/crates/core/src/turn \
  opencode-rust/crates/core/src/session/mod.rs \
  opencode-rust/crates/storage/src/models.rs \
  opencode-rust/crates/storage/src/repository.rs \
  opencode-rust/crates/storage/src/sqlite_repository.rs \
  opencode-rust/crates/storage/src/service.rs \
  opencode-rust/crates/runtime/src/types.rs \
  opencode-rust/crates/runtime/src/runtime.rs \
  opencode-rust/crates/agent/src/runtime.rs \
  opencode-rust/crates/runtime/tests/runtime_smoke.rs
git commit -m "add first-class turn model"
```

---

## Self-Review

### Spec coverage
- Unified runtime seam is covered by Task 1 / PR1.
- First-class `Turn` is covered by Task 2 / PR2.
- The later PR sequence is enumerated so future plan extensions have a stable roadmap.

### Placeholder scan
- No `TODO`, `TBD`, or “implement later” placeholders remain in the actionable PR1/PR2 tasks.
- Every code-changing step includes concrete code or an explicit constrained edit target.

### Type consistency
- `Runtime`, `RuntimeHandle`, `RuntimeServices`, `RuntimeCommand`, `RuntimeResponse`, `RuntimeStatus` are used consistently.
- `Turn`, `TurnId`, `TurnStatus` are used consistently.

---

Plan complete and saved to `docs/superpowers/plans/2026-04-28-runtime-unification-pr-plan.md`.

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
