# PRD: bus Module

## Module Overview

- **Module Name**: bus
- **Source Path**: `packages/opencode/src/bus/`
- **Type**: Infrastructure Service
- **Purpose**: In-process typed pub/sub event bus for loosely-coupled communication between modules, backed by Effect's `PubSub`. Also emits to the cross-process GlobalBus.

---

## Functionality

### Core Features

1. **Typed Event Definitions** — Each event has a `type` string and a Zod schema (`BusEvent.define`)
2. **Per-type PubSubs** — Dedicated `PubSub` per event type for filtered subscriptions
3. **Wildcard PubSub** — Single catch-all `PubSub` for subscribing to all events
4. **Stream-based Subscriptions** — `subscribe()` returns an Effect `Stream`
5. **Callback Subscriptions** — `subscribeCallback()` / `subscribeAllCallback()` for non-Effect consumers
6. **Lifecycle Cleanup** — `InstanceDisposed` event is published before the PubSubs are shut down
7. **GlobalBus Bridge** — Every publish also emits to the inter-process `GlobalBus`
8. **Instance Scoping** — Each project instance gets its own independent bus state

### Event Registry (`BusEvent`)

```typescript
BusEvent.define("session.created", z.object({ sessionID: z.string() }))
BusEvent.define("pty.exited", z.object({ id: PtyID.zod, exitCode: z.number() }))
// etc.
```

---

## API Surface

### BusEvent (event definitions)

```typescript
type Definition = { type: string; properties: ZodType }

function define<Type, Properties>(type: Type, properties: Properties): Definition
function payloads(): ZodObject[]  // all registered event schemas
```

### Bus Service Interface

```typescript
interface Interface {
  publish<D>(def: D, properties: z.output<D["properties"]>): Effect<void>
  subscribe<D>(def: D): Stream<Payload<D>>
  subscribeAll(): Stream<Payload>
  subscribeCallback<D>(def: D, callback: (event: Payload<D>) => unknown): Effect<() => void>
  subscribeAllCallback(callback: (event: any) => unknown): Effect<() => void>
}
```

### Standalone Functions (for non-Effect callers)

```typescript
async function publish<D>(def: D, properties: z.output<D["properties"]>): Promise<void>
function subscribe<D>(def: D, callback: (event: Payload<D>) => unknown): () => void  // returns unsubscribe
function subscribeAll(callback: (event: any) => unknown): () => void
```

### Event Payload Shape

```typescript
type Payload<D = Definition> = {
  type: D["type"]
  properties: z.infer<D["properties"]>
}
```

---

## Data Structures

### Internal State

```typescript
type State = {
  wildcard: PubSub<Payload>
  typed: Map<string, PubSub<Payload>>
}
```

- One `wildcard` unbounded PubSub receives every published event
- `typed` is a lazy map: PubSub is created on first subscription for a given type

### Instance Disposal

```typescript
// Published before shutdown:
InstanceDisposed = BusEvent.define("server.instance.disposed", z.object({ directory: z.string() }))
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `effect` | `PubSub`, `Stream`, `Layer`, `Scope` |
| `bus/global` | Cross-process GlobalBus bridge |
| `effect/instance-state` | Per-project-instance state scoping |

---

## Acceptance Criteria

- [ ] `publish()` delivers to per-type subscribers and wildcard subscribers
- [ ] `subscribe()` returns a Stream that emits only matching event types
- [ ] `subscribeAll()` returns a Stream of all events
- [ ] Callbacks fire asynchronously; errors are caught and logged, not propagated
- [ ] `InstanceDisposed` is published before PubSubs shut down
- [ ] Multiple instances are fully isolated (different directories = different buses)
- [ ] Standalone `publish`/`subscribe`/`subscribeAll` work outside Effect context
- [ ] GlobalBus receives every published event

---

## Rust Implementation Guidance

### Crate: `crates/bus/`

### Key Crates

```toml
tokio = { features = ["full"] }
tokio-stream = "0.1"
broadcast = "tokio::sync::broadcast"  # or flume for multi-consumer
serde_json = "1"
```

### Architecture

```rust
use tokio::sync::broadcast;

pub struct BusService {
    // Wildcard channel: all events
    wildcard_tx: broadcast::Sender<BusPayload>,
    // Per-type channels: lazy-created
    typed: Arc<Mutex<HashMap<String, broadcast::Sender<BusPayload>>>>,
    global_bus: Arc<GlobalBus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BusPayload {
    pub event_type: String,
    pub properties: serde_json::Value,
}

impl BusService {
    pub async fn publish(&self, event_type: &str, properties: serde_json::Value) {
        let payload = BusPayload { event_type: event_type.to_string(), properties };

        // Per-type delivery
        let typed = self.typed.lock().unwrap();
        if let Some(tx) = typed.get(event_type) {
            let _ = tx.send(payload.clone());
        }
        // Wildcard delivery
        let _ = self.wildcard_tx.send(payload.clone());
        // Cross-process
        self.global_bus.emit("event", payload);
    }

    pub fn subscribe(&self, event_type: &str) -> broadcast::Receiver<BusPayload> {
        let mut typed = self.typed.lock().unwrap();
        let tx = typed.entry(event_type.to_string())
            .or_insert_with(|| broadcast::channel(1024).0);
        tx.subscribe()
    }

    pub fn subscribe_all(&self) -> broadcast::Receiver<BusPayload> {
        self.wildcard_tx.subscribe()
    }
}
```

---

## Test Design

### Unit Tests

```rust
#[tokio::test]
async fn test_publish_delivers_to_typed_subscriber() {
    let bus = BusService::new_test();
    let mut rx = bus.subscribe("session.created");
    bus.publish("session.created", json!({"sessionID": "ses1"})).await;
    let event = rx.recv().await.unwrap();
    assert_eq!(event.event_type, "session.created");
}

#[tokio::test]
async fn test_wildcard_receives_all_events() {
    let bus = BusService::new_test();
    let mut rx = bus.subscribe_all();
    bus.publish("session.created", json!({})).await;
    bus.publish("pty.exited", json!({})).await;
    let e1 = rx.recv().await.unwrap();
    let e2 = rx.recv().await.unwrap();
    assert_eq!(e1.event_type, "session.created");
    assert_eq!(e2.event_type, "pty.exited");
}

#[tokio::test]
async fn test_typed_subscriber_does_not_receive_other_events() {
    let bus = BusService::new_test();
    let mut rx = bus.subscribe("session.created");
    bus.publish("pty.exited", json!({})).await; // different type
    bus.publish("session.created", json!({})).await;
    // First recv should be session.created, not pty.exited
    let event = rx.recv().await.unwrap();
    assert_eq!(event.event_type, "session.created");
}

#[tokio::test]
async fn test_instance_disposed_published_on_shutdown() {
    let (bus, shutdown_tx) = BusService::new_with_shutdown();
    let mut rx = bus.subscribe("server.instance.disposed");
    shutdown_tx.send(()).unwrap();
    let event = rx.recv().await.unwrap();
    assert_eq!(event.event_type, "server.instance.disposed");
}
```

### Integration Tests (from TS patterns)

- `bus.test.ts`: Publish/subscribe across Effect layers
- `bus-integration.test.ts`: Bus with real session/pty events flowing end-to-end
- `bus-effect.test.ts`: Stream-based subscription inside Effect runtime
