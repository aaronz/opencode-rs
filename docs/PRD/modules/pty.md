# PRD: pty Module

## Module Overview

- **Module Name**: pty
- **Source Path**: `packages/opencode/src/pty/`
- **Type**: Infrastructure Service
- **Purpose**: Pseudo-terminal (PTY) management — spawning, streaming, resizing, and WebSocket-multiplexing interactive terminal sessions.

---

## Functionality

### Core Features

1. **PTY Session Lifecycle** — Create, list, get, update, and remove PTY sessions
2. **Process Spawning** — Launch shell commands via platform-specific PTY backends (`node-pty` on Node, `bun-pty` on Bun)
3. **Output Buffering** — Rolling 2 MB ring buffer with byte-cursor tracking per session
4. **WebSocket Multiplexing** — Multiple clients can connect to the same PTY session and receive streamed output
5. **Terminal Resize** — Propagate `cols × rows` resize events to the underlying process
6. **Input Forwarding** — Write keystrokes/data to the running process
7. **Event Publishing** — Emit `pty.created`, `pty.updated`, `pty.exited`, `pty.deleted` bus events
8. **Shell Detection** — Uses the `shell` module to pick the preferred shell and inject login flags
9. **Environment Injection** — Merges process env, user env, plugin `shell.env` hooks, and `TERM=xterm-256color`

### Shell Environment Hook

On session creation, the plugin system is queried for `shell.env` to inject additional environment variables (e.g., conda/nvm path augmentation).

---

## API Surface

### Types

```typescript
// Session identity
type PtyID = branded string ("pty" prefix)

// Session info (returned to callers)
interface Info {
  id: PtyID
  title: string
  command: string
  args: string[]
  cwd: string
  status: "running" | "exited"
  pid: number
}

// Create a new PTY session
interface CreateInput {
  command?: string   // defaults to Shell.preferred()
  args?: string[]
  cwd?: string
  title?: string
  env?: Record<string, string>
}

// Update title or resize terminal
interface UpdateInput {
  title?: string
  size?: { rows: number; cols: number }
}
```

### Service Interface

```typescript
interface Interface {
  list: () => Effect<Info[]>
  get: (id: PtyID) => Effect<Info | undefined>
  create: (input: CreateInput) => Effect<Info>
  update: (id: PtyID, input: UpdateInput) => Effect<Info | undefined>
  remove: (id: PtyID) => Effect<void>
  resize: (id: PtyID, cols: number, rows: number) => Effect<void>
  write: (id: PtyID, data: string) => Effect<void>
  connect: (id: PtyID, ws: Socket, cursor?: number) => Effect<{
    onMessage: (message: string | ArrayBuffer) => void
    onClose: () => void
  } | undefined>
}
```

### Events

```typescript
Event.Created   // { info: Info }
Event.Updated   // { info: Info }
Event.Exited    // { id: PtyID, exitCode: number }
Event.Deleted   // { id: PtyID }
```

### WebSocket Protocol

- Client connects with optional `cursor` position (byte offset in the output stream)
- Server replays buffered output from `cursor` position
- Server sends a control frame `0x00 + JSON({ cursor })` after replay to indicate current position
- Subsequent data frames are raw UTF-8 terminal output chunks

---

## Data Structures

### Active Session (internal)

```typescript
type Active = {
  info: Info
  process: Proc           // PTY process handle
  buffer: string          // Rolling output ring buffer (max 2 MB)
  bufferCursor: number    // Byte offset of buffer[0] in full stream
  cursor: number          // Total bytes written so far
  subscribers: Map<unknown, Socket>  // WebSocket clients
}
```

### Proc (PTY process abstraction)

```typescript
type Proc = {
  pid: number
  onData(listener: (data: string) => void): Disposable
  onExit(listener: (event: { exitCode: number; signal?: number|string }) => void): Disposable
  write(data: string): void
  resize(cols: number, rows: number): void
  kill(signal?: string): void
}
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `node-pty` / `bun-pty` | Platform PTY backend |
| `bus` module | Event publishing |
| `plugin` module | `shell.env` hook |
| `shell` module | Shell detection & login flag |
| `effect` | Async runtime, layers |

---

## Acceptance Criteria

- [ ] PTY sessions can be created with custom commands, args, cwd, env
- [ ] Sessions auto-clean up on process exit
- [ ] Multiple WebSocket clients can connect to the same session
- [ ] Buffered output replay works correctly from any cursor position
- [ ] Terminal resize propagates to the underlying process
- [ ] `pty.created`, `pty.exited`, `pty.deleted` events are published on the bus
- [ ] Shell env plugins can inject environment variables
- [ ] Sessions terminate cleanly when the service shuts down

---

## Rust Implementation Guidance

### Crate: `crates/pty/`

### Key Crates

```toml
portable-pty = "0.8"       # Cross-platform PTY
tokio = { features = ["full"] }
tokio-tungstenite = "0.21" # WebSocket multiplexing
bytes = "1"                # Byte buffer management
```

### Architecture

```rust
pub struct PtySession {
    pub id: PtyId,
    pub info: PtyInfo,
    process: Box<dyn Child + Send>,   // portable-pty child
    master: Box<dyn MasterPty + Send>,
    buffer: RingBuffer,               // 2MB rolling buffer
    subscribers: HashMap<ClientId, WebSocketSink>,
}

pub struct PtyService {
    sessions: Arc<Mutex<HashMap<PtyId, PtySession>>>,
    bus: Arc<BusService>,
}
```

### Ring Buffer Strategy

```rust
const BUFFER_LIMIT: usize = 2 * 1024 * 1024;
const BUFFER_CHUNK: usize = 64 * 1024;

struct RingBuffer {
    data: String,
    global_cursor: usize,  // total bytes ever written
    buffer_start: usize,   // global offset of buffer[0]
}
impl RingBuffer {
    fn append(&mut self, chunk: &str) {
        self.data.push_str(chunk);
        self.global_cursor += chunk.len();
        let excess = self.data.len().saturating_sub(BUFFER_LIMIT);
        if excess > 0 {
            self.data = self.data[excess..].to_string();
            self.buffer_start += excess;
        }
    }
    fn slice_from(&self, cursor: usize) -> &str {
        let offset = cursor.saturating_sub(self.buffer_start);
        &self.data[offset.min(self.data.len())..]
    }
}
```

### ID Format

```rust
// PtyID: "pty" + ascending ULID
pub struct PtyId(pub String);
impl PtyId {
    pub fn new() -> Self { Self(format!("pty{}", ulid())) }
}
```

---

## Test Design

### Unit Tests

```rust
#[tokio::test]
async fn test_pty_create_returns_running_status() {
    let svc = PtyService::new_test();
    let info = svc.create(CreateInput { command: Some("echo".into()), ..Default::default() }).await.unwrap();
    assert_eq!(info.status, PtyStatus::Running);
    assert!(!info.id.is_empty());
}

#[tokio::test]
async fn test_pty_write_and_output_capture() {
    let svc = PtyService::new_test();
    let info = svc.create(CreateInput::default()).await.unwrap();
    svc.write(&info.id, "echo hello\n").await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    // verify via subscriber that "hello" appears in buffer
}

#[tokio::test]
async fn test_ring_buffer_wraps_at_limit() {
    let mut buf = RingBuffer::new();
    let chunk = "x".repeat(1024 * 1024);
    buf.append(&chunk);
    buf.append(&chunk);
    buf.append(&chunk); // total 3MB > 2MB limit
    assert!(buf.data.len() <= BUFFER_LIMIT);
    assert!(buf.buffer_start > 0);
}

#[tokio::test]
async fn test_resize_propagates() {
    let svc = PtyService::new_test();
    let info = svc.create(CreateInput::default()).await.unwrap();
    svc.resize(&info.id, 120, 40).await.unwrap(); // should not error
}

#[tokio::test]
async fn test_remove_cleans_up_session() {
    let svc = PtyService::new_test();
    let info = svc.create(CreateInput::default()).await.unwrap();
    svc.remove(&info.id).await.unwrap();
    assert!(svc.get(&info.id).await.unwrap().is_none());
}
```

### Integration Tests (from TS test patterns)

- `pty-session.test`: Create session → write commands → verify stdout captured
- `pty-shell.test`: Shell selection, login flags injected
- `pty-output-isolation.test`: Multiple sessions don't cross-contaminate output buffers

### Test Patterns from TypeScript Tests

```typescript
// pty-session.test.ts
test("creates session with default shell", async () => {
  const info = await Pty.create({})
  expect(info.status).toBe("running")
  expect(info.command).toBeTruthy()
})

// pty-output-isolation.test.ts
test("sessions are isolated", async () => {
  const a = await Pty.create({ command: "echo", args: ["session-a"] })
  const b = await Pty.create({ command: "echo", args: ["session-b"] })
  // verify buffers are separate
})
```
