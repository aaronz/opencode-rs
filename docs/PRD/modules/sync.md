# PRD: sync Module

## Module Overview

- **Module Name**: sync
- **Source Path**: `packages/opencode/src/sync/`
- **Type**: Event Sourcing / Synchronization
- **Purpose**: Durable event sourcing system that records typed domain events to SQLite, replays them, and bridges them to the in-process bus and GlobalBus for multi-workspace synchronization.

---

## Functionality

### Core Features

1. **Event Definition** — Typed, versioned event schemas (Zod + aggregate key)
2. **Event Registry** — Global map of all defined event types
3. **Event Running** — Persist an event to SQLite using an `IMMEDIATE` transaction, then project it and publish to the bus
4. **Idempotent Replay** — Replay events from the control plane, skipping already-applied sequences
5. **Sequence Tracking** — Per-aggregate sequence numbers enforced in SQLite (`EventSequenceTable`)
6. **Projectors** — Side-effect functions that apply event data to the database
7. **Version Migration** — Multiple versions of the same event type can coexist; only latest versions are run from code
8. **GlobalBus Emission** — Events are also emitted to the inter-process GlobalBus for workspace syncing

### Workflow

```
run(def, data)
  → IMMEDIATE SQLite transaction
    → EventSequenceTable: upsert seq
    → EventTable: insert event row
    → projector(tx, data)   # domain side-effects
  → Bus.publish(def, data)  # in-process
  → GlobalBus.emit("event", ...)  # cross-process
```

---

## API Surface

### Event Definition

```typescript
type Definition = {
  type: string          // e.g. "session.created"
  version: number       // for migration
  aggregate: string     // field name in data that is the aggregate ID
  schema: ZodObject     // full event payload
  properties: ZodObject // bus-facing schema (may differ from storage)
}
```

### Core Functions

```typescript
function define(input: {
  type: string
  version: number
  aggregate: string
  schema: ZodObject
  busSchema?: ZodObject
}): Definition

function init(input: {
  projectors: Array<[Definition, ProjectorFunc]>
  convertEvent?: (type: string, data: unknown) => Record<string, unknown> | Promise<...>
}): void

function run<Def>(def: Def, data: Event<Def>["data"], opts?: { publish?: boolean }): void

function replay(event: SerializedEvent, opts?: { publish: boolean }): void
function replayAll(events: SerializedEvent[], opts?: { publish: boolean }): string | undefined

function project<Def>(def: Def, func: (db, data) => void): [Definition, ProjectorFunc]

function remove(aggregateID: string): void
function payloads(): ZodObject[]
function reset(): void
```

### Types

```typescript
type Event<Def> = {
  id: string
  seq: number
  aggregateID: string
  data: z.infer<Def["schema"]>
}

type SerializedEvent = Event & { type: string }
type ProjectorFunc = (db: Database.TxOrDb, data: unknown) => void
```

---

## Data Structures

### Database Tables

```sql
-- EventSequenceTable: tracks latest sequence per aggregate
CREATE TABLE event_sequence (
  aggregate_id TEXT PRIMARY KEY,
  seq          INTEGER NOT NULL
);

-- EventTable: full event log
CREATE TABLE event (
  id           TEXT PRIMARY KEY,
  seq          INTEGER NOT NULL,
  aggregate_id TEXT NOT NULL,
  type         TEXT NOT NULL,  -- "session.created.1"
  data         JSON NOT NULL
);
```

### EventID

```typescript
// Ascending ULID with "event" prefix: "evt..."
EventID.ascending() => string
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `storage` module | SQLite transaction (`Database.transaction`) |
| `bus` module | In-process event publication |
| `bus/global` module | Cross-process emission |
| `flag` module | `OPENCODE_EXPERIMENTAL_WORKSPACES` feature flag |
| `control-plane` module | Workspace context for cross-process routing |
| `zod` | Schema validation |

---

## Acceptance Criteria

- [ ] Events are defined with type, version, aggregate key, and schema
- [ ] `run()` persists to SQLite atomically with `IMMEDIATE` transaction
- [ ] Projectors receive transactional DB handle and modify read-models
- [ ] Sequence is enforced: `replay()` rejects out-of-order events
- [ ] `replayAll()` validates sequence contiguity before applying
- [ ] Events are published to in-process bus and GlobalBus
- [ ] `remove()` deletes all event history for an aggregate
- [ ] Old event versions can be replayed without breaking newer code
- [ ] `reset()` allows clean test teardown

---

## Rust Implementation Guidance

### Crate: `crates/sync/`

### Key Crates

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
serde_json = "1"
tokio = { features = ["full"] }
```

### Core Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub id: String,
    pub seq: i64,
    pub aggregate_id: String,
    pub event_type: String,   // "session.created.1"
    pub data: serde_json::Value,
}

pub trait Projector: Send + Sync {
    fn event_type(&self) -> &str;
    fn project(&self, tx: &Transaction, data: &serde_json::Value) -> Result<()>;
}

pub struct SyncEventStore {
    db: Arc<Mutex<Connection>>,
    projectors: HashMap<String, Box<dyn Projector>>,
    bus: Arc<BusService>,
}
```

### Transaction Pattern

```rust
impl SyncEventStore {
    pub fn run(&self, event_type: &str, aggregate_id: &str, data: serde_json::Value) -> Result<()> {
        let conn = self.db.lock().unwrap();
        conn.execute_batch("BEGIN IMMEDIATE")?;

        let seq = self.next_seq(&conn, aggregate_id)?;
        let id = EventId::ascending();

        // Insert sequence tracker
        conn.execute(
            "INSERT INTO event_sequence (aggregate_id, seq) VALUES (?1, ?2)
             ON CONFLICT(aggregate_id) DO UPDATE SET seq = ?2",
            params![aggregate_id, seq],
        )?;

        // Insert event row
        conn.execute(
            "INSERT INTO event (id, seq, aggregate_id, type, data) VALUES (?1,?2,?3,?4,?5)",
            params![id, seq, aggregate_id, event_type, data.to_string()],
        )?;

        // Run projector
        if let Some(proj) = self.projectors.get(event_type) {
            proj.project(&conn, &data)?;
        }

        conn.execute_batch("COMMIT")?;

        // Publish to bus (outside transaction)
        self.bus.publish(event_type, data)?;
        Ok(())
    }
}
```

---

## Test Design

### Unit Tests

```rust
#[test]
fn test_run_persists_and_sequences() {
    let store = SyncEventStore::new_test();
    store.run("session.created", "ses123", json!({"sessionID": "ses123"})).unwrap();
    let seq = store.get_seq("ses123").unwrap();
    assert_eq!(seq, 0);
}

#[test]
fn test_replay_skips_already_applied() {
    let store = SyncEventStore::new_test();
    // Apply seq=0 first
    store.run("session.created", "ses1", json!({})).unwrap();
    // Replay seq=0 again — should be a no-op
    let event = SyncEvent { seq: 0, aggregate_id: "ses1".into(), .. };
    store.replay(event, false).unwrap(); // idempotent
}

#[test]
fn test_replay_out_of_order_errors() {
    let store = SyncEventStore::new_test();
    let event = SyncEvent { seq: 5, aggregate_id: "ses1".into(), .. }; // missing 0-4
    assert!(store.replay(event, false).is_err());
}

#[test]
fn test_replay_all_validates_contiguity() {
    let store = SyncEventStore::new_test();
    let events = vec![
        SyncEvent { seq: 0, .. },
        SyncEvent { seq: 2, .. },  // gap!
    ];
    assert!(store.replay_all(events, false).is_err());
}

#[test]
fn test_remove_deletes_aggregate_history() {
    let store = SyncEventStore::new_test();
    store.run("session.created", "ses1", json!({})).unwrap();
    store.remove("ses1").unwrap();
    assert!(store.get_seq("ses1").is_none());
}
```

### Integration Tests (from TS test patterns)

- `sync/index.test.ts`: Full lifecycle — define, init projectors, run events, verify projector side-effects in DB, replay from control plane
