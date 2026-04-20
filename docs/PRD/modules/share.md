# PRD: share Module

## Module Overview

- **Module Name**: `share`
- **Source Path**: `packages/opencode/src/share/`
- **Type**: Integration Service
- **Rust Crate**: `crates/share/`
- **Purpose**: Session sharing — exports sessions to a public URL via the OpenCode share service, stores share records in SQLite, and supports workspace-aware sharing via the control plane.

---

## Functionality

### Core Features

1. **Session Share** — Uploads session data (messages, tool results) to remote share service
2. **Share Links** — Returns a public URL for the shared session
3. **Share Records** — Persists share ID → session ID mapping in SQLite
4. **Unshare** — Removes share record and invalidates URL
5. **Get Share Info** — Retrieves share info by session ID or share ID
6. **Share-Next** — Advanced sharing via control plane API with workspace-aware context

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareInfo {
    pub id: ShareId,
    pub session_id: SessionId,
    pub url: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShareRequest {
    session_id: String,
    messages: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShareResponse {
    id: String,
    url: String,
}
```

### `ShareService`

```rust
pub struct ShareService {
    db: Arc<Mutex<Connection>>,
    http: reqwest::Client,
    session_store: Arc<SessionStore>,
    control_plane: Option<Arc<ControlPlaneClient>>,
}

impl ShareService {
    /// Share a session: fetch messages, upload, store record
    pub async fn share(&self, session_id: &SessionId) -> Result<ShareInfo, ShareError> {
        // 1. Fetch session messages
        let messages = self.session_store.get_messages(session_id).await?;

        // 2. Upload to share service
        let share_url = std::env::var("OPENCODE_SHARE_URL")
            .unwrap_or_else(|_| "https://opencode.ai/api/share".into());

        let resp: ShareResponse = self.http
            .post(&share_url)
            .json(&ShareRequest {
                session_id: session_id.to_string(),
                messages,
            })
            .send().await?
            .json().await
            .map_err(ShareError::Upload)?;

        // 3. Store share record
        let info = ShareInfo {
            id: ShareId(resp.id),
            session_id: session_id.clone(),
            url: resp.url,
            created_at: chrono::Utc::now().timestamp(),
        };
        self.store_share_record(&info).await?;

        Ok(info)
    }

    /// Unshare: remove share record
    pub async fn unshare(&self, session_id: &SessionId) -> Result<(), ShareError> {
        let conn = self.db.lock().unwrap();
        conn.execute(
            "DELETE FROM share WHERE session_id = ?1",
            params![session_id.to_string()],
        )?;
        Ok(())
    }

    /// Get share info by session ID
    pub async fn get(&self, session_id: &SessionId) -> Result<Option<ShareInfo>, ShareError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, url, created_at FROM share WHERE session_id = ?1"
        )?;
        let result = stmt.query_row(params![session_id.to_string()], |row| {
            Ok(ShareInfo {
                id: ShareId(row.get(0)?),
                session_id: SessionId(row.get(1)?),
                url: row.get(2)?,
                created_at: row.get(3)?,
            })
        });
        match result {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ShareError::Database(e)),
        }
    }

    /// Share with workspace context via control plane
    pub async fn share_next(
        &self,
        session_id: &SessionId,
        workspace_id: &str,
    ) -> Result<ShareInfo, ShareError> {
        let cp = self.control_plane.as_ref()
            .ok_or(ShareError::ControlPlaneNotConfigured)?;

        let messages = self.session_store.get_messages(session_id).await?;

        #[derive(Serialize)]
        struct ShareNextRequest<'a> {
            session_id: &'a str,
            messages: &'a Vec<serde_json::Value>,
            workspace_id: &'a str,
        }

        #[derive(Deserialize)]
        struct ShareNextResponse {
            id: String,
            url: String,
        }

        let resp: ShareNextResponse = cp.client()
            .post("/api/share/next")
            .json(&ShareNextRequest { session_id: &session_id.to_string(), messages: &messages, workspace_id })
            .send().await?
            .json().await
            .map_err(ShareError::ControlPlane)?;

        let info = ShareInfo {
            id: ShareId(resp.id),
            session_id: session_id.clone(),
            url: resp.url,
            created_at: chrono::Utc::now().timestamp(),
        };
        self.store_share_record(&info).await?;
        Ok(info)
    }
}
```

### `ShareError`

```rust
#[derive(Debug, Error)]
pub enum ShareError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Upload failed: {0}")]
    Upload(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Database error: {0}")]
    Database(#[source] rusqlite::Error),

    #[error("Control plane not configured")]
    ControlPlaneNotConfigured,

    #[error("Control plane error: {0}")]
    ControlPlane(String),
}
```

---

## Database Schema

```sql
CREATE TABLE IF NOT EXISTS share (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL UNIQUE,
    url         TEXT NOT NULL,
    created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_share_session ON share(session_id);
```

---

## Crate Layout

```
crates/share/
├── Cargo.toml
├── src/
│   ├── lib.rs       # ShareService, ShareError, types
│   ├── service.rs   # Share/unshare/get implementation
│   └── db.rs        # ShareRepository, SQLite schema
└── tests/
    └── share_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-share"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["sync", "rt"] }
rusqlite = { version = "0.32", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
wiremock = "0.6"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `storage` module | SQLite for share record persistence, `SessionStore` |
| `session` module | Fetching session data to share |
| `control-plane` module | ShareNext workspace-aware sharing |
| `reqwest` | Uploading to share service |

---

## Acceptance Criteria

- [x] `share(sessionID)` uploads session data and returns a public URL
- [x] Share records are persisted in SQLite
- [x] `unshare(sessionID)` removes the share record
- [x] `get(sessionID)` returns existing share info if present
- [x] ShareNext works with workspace context from control plane

---

## Test Design

```rust
#[tokio::test]
async fn test_share_returns_url() {
    let mock = MockServer::start().await;
    Mock::given(post, "/api/share")
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "id": "share123",
                "url": "https://opencode.ai/s/share123"
            })))
        .mount(&mock)
        .await;

    let svc = ShareService::new_test(&mock.uri()).await;
    let info = svc.share(&SessionId::new()).await.unwrap();
    assert_eq!(info.url, "https://opencode.ai/s/share123");
    assert_eq!(info.id.to_string(), "share123");
}

#[tokio::test]
async fn test_share_stored_in_db() {
    let mock = MockServer::start().await;
    Mock::given(post, "/api/share")
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({ "id": "s1", "url": "http://x.com/s/s1" })))
        .mount(&mock)
        .await;

    let svc = ShareService::new_test(&mock.uri()).await;
    let session_id = SessionId::new();
    svc.share(&session_id).await.unwrap();

    let info = svc.get(&session_id).await.unwrap();
    assert!(info.is_some());
    assert_eq!(info.unwrap().id.to_string(), "s1");
}

#[tokio::test]
async fn test_unshare_removes_record() {
    let mock = MockServer::start().await;
    Mock::given(post, "/api/share")
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({ "id": "s1", "url": "http://x.com/s/s1" })))
        .mount(&mock)
        .await;

    let svc = ShareService::new_test(&mock.uri()).await;
    let session_id = SessionId::new();
    svc.share(&session_id).await.unwrap();
    svc.unshare(&session_id).await.unwrap();

    let info = svc.get(&session_id).await.unwrap();
    assert!(info.is_none());
}
```

---

## Source Reference

*Source: `packages/opencode/src/share/index.ts`*
*No existing Rust equivalent — implement in `crates/share/`*
