# PRD: control-plane Module

## Module Overview

- **Module Name**: control-plane
- **Source Path**: `packages/opencode/src/control-plane/`
- **Type**: Integration Service
- **Purpose**: Client for the OpenCode control plane service — manages remote workspace configurations, workspace adaptors, SSE event streaming, and cross-workspace state routing.

---

## Functionality

### Core Features

1. **Workspace Management** — CRUD for remote workspace records
2. **Workspace Adaptors** — Pluggable adaptors for different workspace types (local, remote, cloud)
3. **SSE Streaming** — Receives server-sent events from the control plane for real-time sync
4. **Workspace Context** — Per-request workspace context for routing events to the right instance
5. **SQL Tables** — Persists workspace records locally in SQLite

### Workspace Adaptor Interface

```typescript
type WorkspaceAdaptor = {
  name: string
  description: string
  configure(info: WorkspaceInfo): WorkspaceInfo | Promise<WorkspaceInfo>
  create(info: WorkspaceInfo, env: Record<string, string | undefined>, from?: WorkspaceInfo): Promise<void>
  remove(info: WorkspaceInfo): Promise<void>
  target(info: WorkspaceInfo): Target | Promise<Target>
}

type Target =
  | { type: "local"; directory: string }
  | { type: "remote"; url: string | URL; headers?: HeadersInit }
```

---

## API Surface

### Types

```typescript
interface WorkspaceInfo {
  id: WorkspaceID
  type: string
  name: string
  branch: string | null
  directory: string | null
  extra: unknown | null
  projectID: ProjectID
}
```

### WorkspaceContext

```typescript
// Per-request context injection
WorkspaceContext.workspaceID: string | undefined
```

---

## Data Structures

### Database Tables (`workspace.sql.ts`)

```sql
CREATE TABLE workspace (
  id         TEXT PRIMARY KEY,
  type       TEXT NOT NULL,
  name       TEXT NOT NULL,
  branch     TEXT,
  directory  TEXT,
  extra      JSON,
  project_id TEXT NOT NULL
);
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `storage` module | SQLite for workspace persistence |
| `flag` module | `OPENCODE_EXPERIMENTAL_WORKSPACES` |
| `bus` module | Event routing |
| `project` module | Project context |
| `effect/http` | SSE and REST API calls |

---

## Acceptance Criteria

- [ ] Workspace records are stored and retrieved from SQLite
- [ ] Workspace adaptors can be registered and invoked
- [ ] SSE stream connects and delivers events
- [ ] `WorkspaceContext.workspaceID` is available per-request
- [ ] `target()` returns local or remote target for the workspace
- [ ] `create()` / `remove()` call through to the adaptor

---

## Rust Implementation Guidance

### Crate: `crates/control-plane/`

```rust
pub trait WorkspaceAdaptor: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn configure(&self, info: WorkspaceInfo) -> Result<WorkspaceInfo>;
    async fn create(&self, info: &WorkspaceInfo, env: &HashMap<String, String>) -> Result<()>;
    async fn remove(&self, info: &WorkspaceInfo) -> Result<()>;
    async fn target(&self, info: &WorkspaceInfo) -> Result<WorkspaceTarget>;
}

pub enum WorkspaceTarget {
    Local { directory: PathBuf },
    Remote { url: Url, headers: HashMap<String, String> },
}

pub struct ControlPlaneService {
    db: Arc<Mutex<Connection>>,
    http: reqwest::Client,
    adaptors: HashMap<String, Box<dyn WorkspaceAdaptor>>,
}
```

### SSE Client

```rust
use eventsource_client as es;

impl ControlPlaneService {
    pub async fn connect_sse(&self, url: &str) -> impl Stream<Item = SseEvent> {
        es::Client::for_url(url).unwrap().build().stream()
    }
}
```

---

## Test Design

```rust
#[tokio::test]
async fn test_workspace_crud() {
    let svc = ControlPlaneService::new_test();
    let info = WorkspaceInfo { id: "wrk1".into(), type_: "local".into(), .. };
    svc.create_workspace(&info).await.unwrap();
    let fetched = svc.get_workspace("wrk1").await.unwrap();
    assert_eq!(fetched.id, "wrk1");
}

#[tokio::test]
async fn test_sse_events_received() {
    let mock = MockSseServer::start().await;
    mock.push_event("event: sync\ndata: {}\n\n");
    let svc = ControlPlaneService::new_test_sse(&mock.url());
    let event = svc.next_event().await.unwrap();
    assert_eq!(event.event_type, "sync");
}
```

### Integration Tests (from TS patterns)

- `control-plane/adaptors.test.ts`: Register adaptor → create → target
- `control-plane/sse.test.ts`: SSE connection and event delivery
