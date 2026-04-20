# PRD: share Module

## Module Overview

- **Module Name**: share
- **Source Path**: `packages/opencode/src/share/`
- **Type**: Integration Service
- **Purpose**: Session sharing functionality — exports sessions to a public URL via the OpenCode share service, and provides next-generation sharing via the control plane.

---

## Functionality

### Core Features

1. **Session Share** — Uploads session data (messages, tool results) to a remote share service
2. **Share Links** — Returns a public URL for the shared session
3. **Share-Next** — Advanced sharing via control plane API with workspace-aware context
4. **SQLite Persistence** — `share.sql.ts` tracks share records (share ID, URL, session ID)

### Share Flow

```
share(sessionID)
  → fetch session messages from storage
  → POST /share with session data
  → receive share URL
  → store in SQLite (shareID → sessionID mapping)
  → return URL
```

---

## API Surface

```typescript
// Re-exports from sub-modules
export * as ShareNext from "./share-next"
export * as SessionShare from "./session"
```

### SessionShare (session.ts)

```typescript
function share(sessionID: string): Promise<{ url: string }>
function unshare(sessionID: string): Promise<void>
function get(sessionID: string): Promise<ShareInfo | undefined>
```

### ShareNext (share-next.ts)

```typescript
// Control-plane integrated sharing
function shareNext(sessionID: string, workspaceID: string): Promise<{ url: string }>
```

---

## Data Structures

### Database (`share.sql.ts`)

```sql
CREATE TABLE share (
  id         TEXT PRIMARY KEY,  -- share ID from remote
  session_id TEXT NOT NULL,
  url        TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `storage` module | SQLite for share record persistence |
| `session` module | Fetching session data to share |
| `control-plane` module | ShareNext workspace-aware sharing |
| `effect/http` | Uploading to share service |

---

## Acceptance Criteria

- [ ] `share(sessionID)` uploads session data and returns a public URL
- [ ] Share records are persisted in SQLite
- [ ] `unshare(sessionID)` removes the share record
- [ ] `get(sessionID)` returns existing share info if present
- [ ] ShareNext works with workspace context from control plane

---

## Rust Implementation Guidance

### Crate: `crates/share/`

```rust
pub struct ShareService {
    db: Arc<Mutex<Connection>>,
    http: reqwest::Client,
    session_store: Arc<SessionStore>,
}

impl ShareService {
    pub async fn share(&self, session_id: &str) -> Result<ShareInfo> {
        let messages = self.session_store.get_messages(session_id).await?;
        let resp: ShareResponse = self.http
            .post("https://opencode.ai/api/share")
            .json(&ShareRequest { session_id, messages })
            .send().await?
            .json().await?;

        self.store_share(session_id, &resp.id, &resp.url).await?;
        Ok(ShareInfo { id: resp.id, url: resp.url })
    }
}
```

---

## Test Design

```rust
#[tokio::test]
async fn test_share_returns_url() {
    let mock = MockServer::start().await;
    Mock::given(method("POST")).and(path("/api/share"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"id": "share123", "url": "https://opencode.ai/s/share123"})))
        .mount(&mock).await;
    let svc = ShareService::new_test(&mock.uri());
    let info = svc.share("ses1").await.unwrap();
    assert_eq!(info.url, "https://opencode.ai/s/share123");
}

#[tokio::test]
async fn test_share_stored_in_db() {
    // verify share record persisted after share()
}
```

### Integration Tests (from TS patterns)

- `share/share-next.test.ts`: ShareNext with mock control plane
