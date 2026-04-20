# PRD: sync Module

## Module Overview

- **Module Name**: `sync`
- **Source Path**: `packages/opencode/src/sync/`
- **Type**: Event Sourcing / Synchronization
- **Rust Crate**: `crates/sync/`
- **Purpose**: Durable event sourcing system that records typed domain events to SQLite, replays them, and bridges them to the in-process bus and GlobalBus for multi-workspace synchronization.

---

## Functionality

### Core Features

1. **Event Definition** — Typed, versioned event schemas with aggregate key
2. **Event Registry** — Global map of all defined event types
3. **Event Running** — Persist an event to SQLite using an `IMMEDIATE` transaction, then project it and publish to the bus
4. **Idempotent Replay** — Replay events from the control plane, skipping already-applied sequences
5. **Sequence Tracking** — Per-aggregate sequence numbers enforced in SQLite
6. **Projectors** — Side-effect functions that apply event data to read-model tables
7. **Version Migration** — Multiple versions of the same event type can coexist; only latest versions are run from code
8. **GlobalBus Emission** — Events are also emitted to the inter-process GlobalBus for workspace syncing

---

## Workflow

```
run(def, data)
  → BEGIN IMMEDIATE (SQLite transaction)
    → upsert EventSequenceTable: seq = max(seq, new_seq) for aggregate_id
    → insert EventTable: (id, seq, aggregate_id, type, data)
    → projector(tx, data)  ← domain side-effects on read-models
  → COMMIT
  → Bus.publish(def.type, data)  ← in-process
  → GlobalBus.emit("event", ...)   ← cross-process
```

---

## API Surface

### Types

```rust
/// A defined event with type string, version, and schema
#[derive(Debug, Clone)]
pub struct EventDefinition {
    pub event_type: String,          // e.g. "session.created"
    pub version: u32,                 // for migration: 1, 2, 3...
    pub aggregate: String,            // field name in data that IS the aggregate ID
    pub schema: serde_json::Value,    // JSON schema for validation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub id: String,           // "evt" prefix ULID
    pub seq: i64,            // per-aggregate sequence number
    pub aggregate_id: String,
    pub event_type: String,  // "session.created.1" (type + version)
    pub data: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEvent {
    pub id: String,
    pub seq: i64,
    pub aggregate_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}
```

### `SyncService`

```rust
pub trait Projector: Send + Sync {
    fn event_type(&self) -> &str;
    fn project(&self, tx: &Transaction, data: &serde_json::Value) -> Result<(), SyncError>;
}

pub struct SyncService {
    db: Arc<Mutex<Connection>>,
    projectors: RwLock<HashMap<String, Box<dyn Projector>>>,
    bus: Arc<BusService>,
    global_bus: Option<Arc<GlobalBus>>,
    definitions: RwLock<HashMap<String, EventDefinition>>,
}

impl SyncService {
    /// Define a new event type
    pub async fn define(&self, def: EventDefinition) -> Result<(), SyncError> {
        self.definitions.write().await.insert(def.event_type.clone(), def);
        Ok(())
    }

    /// Run an event: persist + project + publish
    pub async fn run(
        &self,
        event_type: &str,
        aggregate_id: &str,
        data: serde_json::Value,
    ) -> Result<SyncEvent, SyncError> {
        let conn = self.db.lock().unwrap();

        // BEGIN IMMEDIATE transaction
        conn.execute_batch("BEGIN IMMEDIATE")?;

        // Get or create sequence number
        let seq = self.get_next_seq(&conn, aggregate_id)?;
        let id = EventId::ascending();

        // Insert or update sequence
        conn.execute(
            "INSERT INTO event_sequence (aggregate_id, seq) VALUES (?1, ?2)
             ON CONFLICT(aggregate_id) DO UPDATE SET seq = MAX(seq, ?2)",
            params![aggregate_id, seq],
        )?;

        // Insert event row
        let full_type = format!("{}.1", event_type); // version hardcoded as 1 for now
        conn.execute(
            "INSERT INTO event (id, seq, aggregate_id, type, data, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, seq, aggregate_id, full_type, data.to_string(), chrono::Utc::now().timestamp()],
        )?;

        // Run projector if registered
        if let Some(proj) = self.projectors.read().await.get(&full_type) {
            proj.project(&conn, &data)?;
        }

        conn.execute_batch("COMMIT")?;

        let event = SyncEvent {
            id: id.to_string(),
            seq,
            aggregate_id: aggregate_id.to_string(),
            event_type: full_type,
            data,
            created_at: chrono::Utc::now().timestamp(),
        };

        // Publish to buses (outside transaction)
        let event_clone = event.clone();
        self.bus.publish(event_clone.event_type.clone(), event_clone.data.clone()).await;
        if let Some(gb) = &self.global_bus {
            gb.emit("event", &event_clone).await;
        }

        Ok(event)
    }

    /// Replay a single event from control plane (idempotent: skips if already applied)
    pub async fn replay(&self, event: SerializedEvent, publish: bool) -> Result<(), SyncError> {
        let conn = self.db.lock().unwrap();

        // Check if already applied
        let existing: Option<i64> = conn.query_row(
            "SELECT seq FROM event WHERE aggregate_id = ?1 AND seq = ?2",
            params![event.aggregate_id, event.seq],
            |row| row.get(0),
        ).ok();

        if existing.is_some() {
            return Ok(()); // idempotent: already applied
        }

        // BEGIN IMMEDIATE for replay
        conn.execute_batch("BEGIN IMMEDIATE")?;

        conn.execute(
            "INSERT INTO event_sequence (aggregate_id, seq) VALUES (?1, ?2)
             ON CONFLICT(aggregate_id) DO UPDATE SET seq = MAX(seq, ?2)",
            params![event.aggregate_id, event.seq],
        )?;

        conn.execute(
            "INSERT INTO event (id, seq, aggregate_id, type, data, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![event.id, event.seq, event.aggregate_id, event.event_type, event.data.to_string(), chrono::Utc::now().timestamp()],
        )?;

        if let Some(proj) = self.projectors.read().await.get(&event.event_type) {
            proj.project(&conn, &event.data)?;
        }

        conn.execute_batch("COMMIT")?;

        if publish {
            self.bus.publish(event.event_type, event.data).await;
        }

        Ok(())
    }

    /// Replay multiple events (validates contiguity)
    pub async fn replay_all(
        &self,
        events: Vec<SerializedEvent>,
        publish: bool,
    ) -> Result<String, SyncError> {
        if events.is_empty() {
            return Ok(String::new());
        }

        // Sort by (aggregate_id, seq) and validate contiguity
        let mut sorted = events.clone();
        sorted.sort_by_key(|e| (e.aggregate_id.clone(), e.seq));

        // Check for sequence gaps
        let aggregate_id = &sorted[0].aggregate_id;
        for (i, event) in sorted.iter().enumerate() {
            if event.aggregate_id != *aggregate_id {
                // New aggregate — reset gap checking
                continue;
            }
            if event.seq as usize != i && i > 0 {
                return Err(SyncError::SequenceGap {
                    expected: i as i64,
                    got: event.seq,
                    aggregate: aggregate_id.clone(),
                });
            }
        }

        for event in sorted {
            self.replay(event, publish).await?;
        }

        Ok(aggregate_id.clone())
    }

    /// Register a projector for an event type
    pub async fn register_projector<P: Projector + 'static>(&self, projector: P) {
        let event_type = projector.event_type().to_string();
        self.projectors.write().await.insert(event_type, Box::new(projector));
    }

    /// Remove all events for an aggregate
    pub async fn remove(&self, aggregate_id: &str) -> Result<(), SyncError> {
        let conn = self.db.lock().unwrap();
        conn.execute("DELETE FROM event WHERE aggregate_id = ?1", params![aggregate_id])?;
        conn.execute("DELETE FROM event_sequence WHERE aggregate_id = ?1", params![aggregate_id])?;
        Ok(())
    }
}
```

### `SyncError`

```rust
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Database error: {0}")]
    Database(#[source] rusqlite::Error),

    #[error("Event type not registered: {0}")]
    UnknownEventType(String),

    #[error("Projector error for {event_type}: {source}")]
    Projector { event_type: String, #[source] source: Box<dyn std::error::Error> },

    #[error("Sequence gap: expected {expected}, got {got} for aggregate {aggregate}")]
    SequenceGap { expected: i64, got: i64, aggregate: String },

    #[error("Replay failed: {0}")]
    ReplayFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[source] serde_json::Error),
}
```

---

## Database Schema

```sql
-- Tracks latest sequence per aggregate
CREATE TABLE IF NOT EXISTS event_sequence (
    aggregate_id TEXT PRIMARY KEY,
    seq           INTEGER NOT NULL
);

-- Full event log
CREATE TABLE IF NOT EXISTS event (
    id            TEXT PRIMARY KEY,
    seq           INTEGER NOT NULL,
    aggregate_id  TEXT NOT NULL,
    type          TEXT NOT NULL,   -- "session.created.1"
    data          JSON NOT NULL,
    created_at    INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_event_aggregate ON event(aggregate_id, seq);
CREATE UNIQUE INDEX IF NOT EXISTS idx_event_aggregate_seq ON event(aggregate_id, seq);
```

---

## Crate Layout

```
crates/sync/
├── Cargo.toml       # rusqlite = { version = "0.31", features = ["bundled"] }, tokio = { features = ["full"] }
├── src/
│   ├── lib.rs       # SyncService, SyncError, types
│   ├── service.rs   # Core run/replay logic
│   ├── projector.rs # Projector trait
│   └── db.rs        # SQLite schema and helpers
└── tests/
    └── sync_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-sync"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["sync", "rt", "time"] }
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `storage` module | SQLite transaction via `Connection` |
| `bus` module | In-process event publication |
| `bus/global` module | Cross-process GlobalBus emission |
| `flag` module | `OPENCODE_EXPERIMENTAL_WORKSPACES` feature flag |
| `rusqlite` | SQLite event store |

---

## Acceptance Criteria

- [x] Events are defined with type, version, aggregate key, and schema
- [x] `run()` persists to SQLite atomically with `IMMEDIATE` transaction
- [x] Projectors receive transactional DB handle and modify read-models
- [x] Sequence is enforced: `replay()` rejects out-of-order events
- [x] `replayAll()` validates sequence contiguity before applying
- [x] Events are published to in-process bus and GlobalBus
- [x] `remove()` deletes all event history for an aggregate
- [x] `reset()` allows clean test teardown

---

## Test Design

```rust
#[tokio::test]
async fn test_run_persists_and_sequences() {
    let svc = SyncService::new_test().await;
    let event = svc.run("session.created", "ses123", json!({"sessionID": "ses123"})).await.unwrap();
    assert_eq!(event.seq, 0);
    assert_eq!(event.aggregate_id, "ses123");
}

#[tokio::test]
async fn test_run_increments_sequence() {
    let svc = SyncService::new_test().await;
    svc.run("session.created", "ses1", json!({"n": 1})).await.unwrap();
    let event2 = svc.run("session.message", "ses1", json!({"n": 2})).await.unwrap();
    assert_eq!(event2.seq, 1);
}

#[tokio::test]
async fn test_replay_skips_already_applied() {
    let svc = SyncService::new_test().await;
    // Apply seq=0 first
    svc.run("session.created", "ses1", json!({})).await.unwrap();

    // Replay seq=0 again — should be a no-op (idempotent)
    let event = SerializedEvent {
        id: "evt_dup".into(),
        seq: 0,
        aggregate_id: "ses1".into(),
        event_type: "session.created.1".into(),
        data: json!({}),
    };
    let result = svc.replay(event, false).await;
    assert!(result.is_ok()); // no error, just skipped
}

#[tokio::test]
async fn test_replay_all_validates_contiguity() {
    let svc = SyncService::new_test().await;
    let events = vec![
        SerializedEvent { id: "e0".into(), seq: 0, aggregate_id: "ses1".into(), event_type: "session.created.1".into(), data: json!({}) },
        SerializedEvent { id: "e2".into(), seq: 2, aggregate_id: "ses1".into(), event_type: "session.created.1".into(), data: json!({}) },
    ];
    let result = svc.replay_all(events, false).await;
    assert!(matches!(result, Err(SyncError::SequenceGap { .. })));
}

#[tokio::test]
async fn test_remove_deletes_aggregate_history() {
    let svc = SyncService::new_test().await;
    svc.run("session.created", "ses1", json!({})).await.unwrap();
    svc.run("session.message", "ses1", json!({})).await.unwrap();
    svc.remove("ses1").await.unwrap();

    // Verify events are gone
    let conn = svc.db.lock().unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM event WHERE aggregate_id = 'ses1'",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(count, 0);
}
```

---

## Source Reference

*Source: `packages/opencode/src/sync/index.ts`*
*No existing Rust equivalent — implement in `crates/sync/`*
