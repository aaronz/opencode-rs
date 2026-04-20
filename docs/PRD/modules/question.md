# PRD: question Module

## Module Overview

- **Module Name**: `question`
- **Source Path**: `packages/opencode/src/question/`
- **Type**: Infrastructure Service
- **Rust Crate**: `crates/question/`
- **Purpose**: Interactive multi-choice question/answer system. Presents structured questions to the user during a session via the bus and collects typed responses using a oneshot channel bridge.

---

## Functionality

### Core Features

1. **Question Publishing** — Publishes a `QuestionPrompt` to the bus with options and metadata
2. **Await Answer** — Blocks until the user replies via `question.replied` bus event
3. **Multi-select Support** — Optional `multiple: true` for multi-option selection
4. **Custom Answers** — Optional `custom: true` allows free-form typed answer
5. **Tool Association** — Questions can be tagged with `messageID` + `callID` from tool calls
6. **Deferred Resolution** — Uses `tokio::sync::oneshot` channel to bridge bus events to async results
7. **List Pending** — Returns all unanswered questions currently pending

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    /// Display text (1-5 words, concise)
    pub label: String,
    /// Explanation of the choice
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInfo {
    pub question: String,
    /// Very short label (max 30 chars)
    pub header: String,
    pub options: Vec<QuestionOption>,
    /// Allow selecting multiple options
    pub multiple: Option<bool>,
    /// Allow free-form typed answer (default: true)
    pub custom: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionTool {
    pub message_id: ulid::Ulid,
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuestion {
    pub id: QuestionId,
    pub session_id: SessionId,
    pub prompt: QuestionInfo,
    pub tool: Option<QuestionTool>,
}
```

### `QuestionError`

```rust
#[derive(Debug, Error)]
pub enum QuestionError {
    #[error("Question was cancelled (channel dropped)")]
    Cancelled,

    #[error("Question timed out after {0:?}")]
    Timeout(Duration),

    #[error("Invalid question ID: {0}")]
    InvalidId(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}
```

### Service Interface

```rust
pub struct QuestionService {
    /// Map from QuestionId → sender that resolves the answer
    pending: Arc<Mutex<HashMap<QuestionId, tokio::sync::oneshot::Sender<Vec<String>>>>>,
    bus: Arc<BusService>,
    timeout: Duration,
}

impl QuestionService {
    /// Ask a question and block until answered (or timeout)
    pub async fn ask(
        &self,
        session_id: &SessionId,
        prompt: QuestionInfo,
        tool: Option<QuestionTool>,
    ) -> Result<Vec<String>, QuestionError> {
        let id = QuestionId::new();
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.pending.lock().await.insert(id.clone(), tx);

        // Publish question.asked event on the bus
        self.bus.publish("question.asked", json!({
            "id": id.to_string(),
            "sessionID": session_id.to_string(),
            "prompt": prompt,
            "tool": tool,
        })).await;

        // Wait for answer with timeout
        tokio::time::timeout(self.timeout, rx)
            .await
            .map_err(|_| QuestionError::Timeout(self.timeout))?
            .map_err(|_| QuestionError::Cancelled)
    }

    /// Called by bus listener when user answers a question
    pub async fn reply(
        &self,
        id: &str,
        session_id: &str,
        answers: Vec<String>,
    ) -> Result<(), QuestionError> {
        let id: QuestionId = id.parse().map_err(|_| QuestionError::InvalidId(id.to_string()))?;

        let tx = self.pending.lock().await.remove(&id);

        if let Some(tx) = tx {
            // Send answer to the waiting asker
            let _ = tx.send(answers.clone());
        }
        // If no pending question, silently ignore (per acceptance criteria)

        // Publish question.replied event
        self.bus.publish("question.replied", json!({
            "id": id.to_string(),
            "sessionID": session_id,
            "answer": answers,
        })).await;

        Ok(())
    }

    /// List all pending (unanswered) questions
    pub async fn list(&self) -> Result<Vec<ActiveQuestion>, QuestionError> {
        let pending = self.pending.lock().await;
        let mut result = Vec::new();
        for (id, _) in pending.iter() {
            // NOTE: we only have the ID, not the full ActiveQuestion stored.
            // The full state is reconstructed from bus events or stored separately.
            // For a complete implementation, store ActiveQuestion in the map instead of just Sender.
        }
        Ok(result)
    }
}
```

### Better State Storage

```rust
struct PendingEntry {
    tx: tokio::sync::oneshot::Sender<Vec<String>>,
    active: ActiveQuestion,
}

pub struct QuestionService {
    pending: Arc<Mutex<HashMap<QuestionId, PendingEntry>>>,
    bus: Arc<BusService>,
    timeout: Duration,
}

impl QuestionService {
    pub async fn ask(
        &self,
        session_id: &SessionId,
        prompt: QuestionInfo,
        tool: Option<QuestionTool>,
    ) -> Result<Vec<String>, QuestionError> {
        let id = QuestionId::new();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let entry = PendingEntry {
            tx,
            active: ActiveQuestion { id: id.clone(), session_id: session_id.clone(), prompt: prompt.clone(), tool: tool.clone() },
        };

        self.pending.lock().await.insert(id.clone(), entry);

        self.bus.publish("question.asked", json!({
            "id": id.to_string(),
            "sessionID": session_id.to_string(),
            "prompt": prompt,
            "tool": tool,
        })).await;

        tokio::time::timeout(self.timeout, rx)
            .await
            .map_err(|_| QuestionError::Timeout(self.timeout))?
            .map_err(|_| QuestionError::Cancelled)
    }

    pub async fn list(&self) -> Result<Vec<ActiveQuestion>, QuestionError> {
        let pending = self.pending.lock().await;
        Ok(pending.values().map(|e| e.active.clone()).collect())
    }
}
```

---

## Event Bus Integration

```rust
/// Subscribe to question.replied events to resolve pending questions
pub async fn start_reply_listener(svc: Arc<QuestionService>) {
    let mut bus = BusListener::new("question.replied").await;
    while let Some(event) = bus.next().await {
        #[derive(Deserialize)]
        struct ReplyEvent {
            id: String,
            session_id: String,
            answer: Vec<String>,
        }
        if let Ok(reply) = serde_json::from_value::<ReplyEvent>(event.data) {
            let _ = svc.reply(&reply.id, &reply.session_id, reply.answer).await;
        }
    }
}
```

---

## Crate Layout

```
crates/question/
├── Cargo.toml       # ulid = "1", tokio = { features = ["full"] }, serde = { features = ["derive"] }
├── src/
│   ├── lib.rs       # QuestionService, QuestionError, types
│   ├── service.rs   # Service implementation
│   └── bus.rs       # Bus event subscription
└── tests/
    └── question_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-question"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["sync", "time", "rt"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
ulid = "1"
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
mockall = "0.13"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `bus` module | Publishing `question.asked` / `question.replied` events |
| `id` module | `QuestionId` generation (ULID with "que" prefix) |
| `session` module | `SessionId` type |
| `tokio::sync::oneshot` | One-shot async channel for deferred resolution |

---

## Acceptance Criteria

- [x] `ask()` publishes `question.asked` and blocks until reply
- [x] `reply()` resolves the corresponding deferred and publishes `question.replied`
- [x] Unknown question IDs in `reply()` are silently ignored
- [x] `multiple: true` allows multi-value answers
- [x] `tool` metadata is attached when question comes from a tool call
- [x] `list()` returns all pending unanswered questions
- [x] Questions timeout after configured duration

---

## Test Design

### Unit Tests

```rust
#[tokio::test]
async fn test_ask_resolves_on_reply() {
    let svc = QuestionService::new_test(Duration::from_secs(5));
    let svc2 = svc.clone();

    let handle = tokio::spawn(async move {
        svc.ask(
            &SessionId::new(),
            QuestionInfo {
                question: "Choose one".into(),
                header: "Choose".into(),
                options: vec![QuestionOption { label: "A".into(), description: "Option A".into() }],
                multiple: None,
                custom: None,
            },
            None,
        ).await
    });

    // Give ask time to register
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Simulate user reply
    let pending = svc2.list().await.unwrap();
    let question_id = pending.first().unwrap().id.to_string();
    let session_id = pending.first().unwrap().session_id.to_string();
    svc2.reply(&question_id, &session_id, vec!["A".into()]).await.unwrap();

    let answers = handle.await.unwrap().unwrap();
    assert_eq!(answers, vec!["A"]);
}

#[tokio::test]
async fn test_reply_to_unknown_id_is_ignored() {
    let svc = QuestionService::new_test(Duration::from_secs(1));
    // Should not panic or error
    svc.reply("que_unknown", "ses1", vec!["A".into()]).await.unwrap();
}

#[tokio::test]
async fn test_list_returns_pending_questions() {
    let svc = QuestionService::new_test(Duration::from_secs(5));
    let svc2 = svc.clone();

    tokio::spawn(async move {
        svc.ask(
            &SessionId::new(),
            test_prompt(),
            None,
        ).await
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let pending = svc2.list().await.unwrap();
    assert_eq!(pending.len(), 1);
}

#[tokio::test]
async fn test_ask_times_out() {
    let svc = QuestionService::new_test(Duration::from_millis(50));
    let result = svc.ask(&SessionId::new(), test_prompt(), None).await;
    assert!(matches!(result, Err(QuestionError::Timeout(_))));
}

fn test_prompt() -> QuestionInfo {
    QuestionInfo {
        question: "Test?".into(),
        header: "Test".into(),
        options: vec![QuestionOption { label: "A".into(), description: "a".into() }],
        multiple: None,
        custom: None,
    }
}
```

---

## Source Reference

*Source: `packages/opencode/src/question/index.ts`*
*No existing Rust equivalent — implement in `crates/question/`*
