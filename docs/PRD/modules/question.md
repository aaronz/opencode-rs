# PRD: question Module

## Module Overview

- **Module Name**: question
- **Source Path**: `packages/opencode/src/question/`
- **Type**: Infrastructure Service
- **Purpose**: Interactive multi-choice question/answer system. Allows the agent to present structured questions to the user during a session and collect typed responses via the bus.

---

## Functionality

### Core Features

1. **Question Publishing** — Publishes a `QuestionPrompt` to the bus with options and metadata
2. **Await Answer** — Blocks until the user replies via `question.replied` event
3. **Multi-select Support** — Optional `multiple: true` for multi-option selection
4. **Custom Answers** — Optional `custom: true` allows typing a free-form answer
5. **Tool Integration** — Questions can be associated with a tool call (messageID + callID)
6. **Deferred Resolution** — Uses `Deferred` (one-shot async channel) to bridge bus events to Effect

### Question Flow

```
agent.question(prompt)
  → QuestionService.ask(prompt, opts)
    → publish "question.asked" with Question info
    → await question.replied event (by question ID)
    → return selected option(s)
```

---

## API Surface

### Types

```typescript
class Option {
  label: string        // Display text (1-5 words, concise)
  description: string  // Explanation of choice
}

class Info {
  question: string      // Complete question text
  header: string        // Very short label (max 30 chars)
  options: Option[]     // Available choices
  multiple?: boolean    // Allow selecting multiple
  custom?: boolean      // Allow typing custom answer (default: true)
}

class Prompt {
  question: string
  header: string
  options: Option[]
  multiple?: boolean
}

class Tool {
  messageID: MessageID
  callID: string
}
```

### Events

```typescript
Event.Asked   // { id: QuestionID; sessionID: SessionID; prompt: Info; tool?: Tool }
Event.Replied // { id: QuestionID; sessionID: SessionID; answer: string[] }
```

### Service Interface

```typescript
interface Interface {
  ask: (
    sessionID: SessionID,
    prompt: Info,
    tool?: Tool
  ) => Effect<string[]>   // returns selected answers

  reply: (
    id: QuestionID,
    sessionID: SessionID,
    answers: string[]
  ) => Effect<void>

  list: () => Effect<ActiveQuestion[]>
}
```

---

## Data Structures

### Internal State

```typescript
type ActiveQuestion = {
  id: QuestionID
  sessionID: SessionID
  prompt: Info
  deferred: Deferred<string[]>  // resolved by reply()
  tool?: Tool
}
```

### QuestionID

```typescript
// "que..." prefix ascending ULID
QuestionID.ascending() => string
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `bus` module | Publishing `question.asked` / `question.replied` |
| `effect` | `Deferred` for one-shot async resolution |
| `session/schema` | `SessionID`, `MessageID` |
| `id` module | `QuestionID` generation |

---

## Acceptance Criteria

- [ ] `ask()` publishes `question.asked` and blocks until reply
- [ ] `reply()` resolves the corresponding deferred and publishes `question.replied`
- [ ] Unknown question IDs in `reply()` are silently ignored
- [ ] `multiple: true` allows multi-value answers
- [ ] `tool` metadata is attached when question comes from a tool call
- [ ] `list()` returns all pending unanswered questions

---

## Rust Implementation Guidance

### Crate: `crates/question/`

### Key Crates

```toml
tokio = { features = ["full"] }
serde = { features = ["derive"] }
serde_json = "1"
```

### Architecture

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInfo {
    pub question: String,
    pub header: String,
    pub options: Vec<QuestionOption>,
    pub multiple: Option<bool>,
    pub custom: Option<bool>,
}

pub struct QuestionService {
    pending: Arc<Mutex<HashMap<QuestionId, tokio::sync::oneshot::Sender<Vec<String>>>>>,
    bus: Arc<BusService>,
}

impl QuestionService {
    pub async fn ask(
        &self,
        session_id: &str,
        prompt: QuestionInfo,
        tool: Option<QuestionTool>,
    ) -> Result<Vec<String>> {
        let id = QuestionId::new();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending.lock().await.insert(id.clone(), tx);

        self.bus.publish("question.asked", json!({
            "id": id,
            "sessionID": session_id,
            "prompt": prompt,
            "tool": tool
        })).await;

        rx.await.map_err(|_| QuestionError::Cancelled)
    }

    pub async fn reply(
        &self,
        id: &str,
        session_id: &str,
        answers: Vec<String>,
    ) -> Result<()> {
        if let Some(tx) = self.pending.lock().await.remove(id) {
            let _ = tx.send(answers.clone());
            self.bus.publish("question.replied", json!({
                "id": id,
                "sessionID": session_id,
                "answer": answers
            })).await;
        }
        Ok(())
    }
}
```

---

## Test Design

```rust
#[tokio::test]
async fn test_ask_resolves_on_reply() {
    let svc = QuestionService::new_test();
    let svc2 = svc.clone();

    let task = tokio::spawn(async move {
        svc.ask("ses1", QuestionInfo {
            question: "Choose one".into(),
            header: "Choose".into(),
            options: vec![QuestionOption { label: "A".into(), description: "Option A".into() }],
            multiple: None,
            custom: None,
        }, None).await
    });

    // Give ask time to register
    tokio::time::sleep(Duration::from_millis(10)).await;
    // Find the pending question ID from the bus event
    let id = get_pending_id(&svc2).await;
    svc2.reply(&id, "ses1", vec!["A".into()]).await.unwrap();

    let answers = task.await.unwrap().unwrap();
    assert_eq!(answers, vec!["A"]);
}

#[tokio::test]
async fn test_reply_to_unknown_id_is_ignored() {
    let svc = QuestionService::new_test();
    // Should not panic or error
    svc.reply("nonexistent", "ses1", vec!["A".into()]).await.unwrap();
}

#[tokio::test]
async fn test_list_returns_pending_questions() {
    let svc = QuestionService::new_test();
    tokio::spawn({
        let svc = svc.clone();
        async move { svc.ask("ses1", test_prompt(), None).await }
    });
    tokio::time::sleep(Duration::from_millis(10)).await;
    let pending = svc.list().await.unwrap();
    assert_eq!(pending.len(), 1);
}
```

### Integration Tests (from TS patterns)

- `question/question.test.ts`: Ask → reply cycle with bus event verification, multi-select answers
