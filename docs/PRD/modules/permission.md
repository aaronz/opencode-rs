# PRD: permission Module

## Module Overview

- **Module Name**: permission
- **Source Path**: `packages/opencode/src/permission/`
- **Type**: Infrastructure Service
- **Purpose**: Permission and access control system. Evaluates access rules for tools and skills, manages user approval flows, persists per-project approvals, and handles "ask" prompts via the bus.

---

## Functionality

### Core Features

1. **Rule Evaluation** — Matches permission requests against ordered rules using wildcard patterns
2. **Rule Precedence** — Rules evaluated in order; first match wins (allow/deny/ask)
3. **"Ask" Flow** — When action is `ask`, defers to the user via `permission.asked` bus event and waits for reply
4. **Always Approvals** — `reply="always"` stores approval in SQLite for the session lifetime
5. **Per-Project Approvals** — Persistent approvals survive session restarts
6. **Arity Checking** — Validates argument count/types for tool calls
7. **Config Rules** — Default rules loaded from `config.permissions`

### Rule Evaluation Algorithm

```
evaluate(permission, pattern, agent.rules) →
  for each rule in rules:
    if rule.permission matches && rule.pattern glob-matches pattern:
      return rule.action
  return default action (usually "ask" for unknown tools)
```

### Permission Request Flow

```
tool.execute()
  → Permission.check("tool", toolName, patterns, agentRules)
    → evaluate() → "allow" → proceed
    → evaluate() → "deny"  → throw PermissionDeniedError
    → evaluate() → "ask"   → publish permission.asked event
      → wait for permission.replied event
        → "once"   → proceed this time only
        → "always" → store in DB + proceed
        → "reject" → throw PermissionDeniedError
```

---

## API Surface

### Types

```typescript
type Action = "allow" | "deny" | "ask"
type Reply  = "once" | "always" | "reject"

class Rule {
  permission: string   // e.g. "tool", "skill"
  pattern: string      // glob pattern e.g. "bash", "file/*"
  action: Action
}

type Ruleset = Rule[]

class Request {
  id: PermissionID
  sessionID: SessionID
  permission: string
  patterns: string[]
  metadata: Record<string, unknown>
  always: string[]     // patterns already permanently approved
  tool?: { messageID: MessageID; callID: string }
}

class ReplyBody {
  reply: Reply
  message?: string
}

class Approval {
  projectID: ProjectID
  patterns: string[]
}
```

### Service Interface

```typescript
interface Interface {
  // Check permission and return action (may prompt user)
  check: (
    permission: string,
    patterns: string[],
    opts: { sessionID: SessionID; metadata?: Record<string, unknown>; tool?: ... }
  ) => Effect<Action>

  // Get stored always-approvals
  always: (projectID: ProjectID) => Effect<Approval>

  // Reply to a pending permission request
  reply: (id: PermissionID, body: ReplyBody) => Effect<void>
}
```

### Events

```typescript
Event.Asked   // Request
Event.Replied // { id: PermissionID; reply: Reply; message?: string }
```

---

## Data Structures

### Database Table (`permission.sql.ts`)

```sql
CREATE TABLE permission (
  id         TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  permission TEXT NOT NULL,
  pattern    TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `bus` module | Publishing `permission.asked` / subscribing `permission.replied` |
| `storage` module | Persistent always-approvals in SQLite |
| `config/permission` | Config-defined default rules |
| `util/wildcard` | Glob pattern matching |
| `permission/evaluate` | Rule evaluation logic |
| `permission/arity` | Argument arity checking |

---

## Acceptance Criteria

- [ ] "allow" rules pass immediately without user prompt
- [ ] "deny" rules block execution immediately
- [ ] "ask" rules pause execution and publish `permission.asked`
- [ ] `reply="once"` allows this execution only
- [ ] `reply="always"` stores approval in SQLite and allows future calls
- [ ] `reply="reject"` blocks execution
- [ ] Config-defined rules are loaded at startup
- [ ] Wildcard patterns match correctly (e.g., `bash/*` matches `bash/ls`)
- [ ] Arity checking validates tool argument counts

---

## Rust Implementation Guidance

### Crate: `crates/permission/`

### Key Crates

```toml
glob = "0.3"
rusqlite = { features = ["bundled"] }
tokio = { features = ["full"] }
```

### Rule Evaluation

```rust
pub fn evaluate(permission: &str, pattern: &str, rules: &[Rule]) -> Action {
    for rule in rules {
        if rule.permission == permission && wildcard_match(&rule.pattern, pattern) {
            return rule.action.clone();
        }
    }
    Action::Ask  // default
}

fn wildcard_match(rule_pattern: &str, subject: &str) -> bool {
    glob::Pattern::new(rule_pattern)
        .map(|p| p.matches(subject))
        .unwrap_or(false)
}
```

### Ask Flow

```rust
impl PermissionService {
    pub async fn check(&self, permission: &str, patterns: &[String], opts: CheckOpts) -> Result<Action> {
        let action = self.evaluate(permission, patterns, &opts.rules);
        match action {
            Action::Allow => Ok(Action::Allow),
            Action::Deny  => Err(PermissionError::Denied),
            Action::Ask   => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let id = PermissionId::new();
                self.pending.lock().await.insert(id.clone(), tx);
                self.bus.publish("permission.asked", Request { id, .. }).await;
                let reply = rx.await?;
                match reply.reply {
                    Reply::Once | Reply::Always => {
                        if reply.reply == Reply::Always {
                            self.store_approval(permission, patterns, &opts.project_id).await?;
                        }
                        Ok(Action::Allow)
                    }
                    Reply::Reject => Err(PermissionError::Denied),
                }
            }
        }
    }
}
```

---

## Test Design

```rust
#[test]
fn test_evaluate_allow_rule_matches() {
    let rules = vec![Rule { permission: "tool".into(), pattern: "bash".into(), action: Action::Allow }];
    assert_eq!(evaluate("tool", "bash", &rules), Action::Allow);
}

#[test]
fn test_evaluate_wildcard_matches() {
    let rules = vec![Rule { permission: "tool".into(), pattern: "file/*".into(), action: Action::Allow }];
    assert_eq!(evaluate("tool", "file/read", &rules), Action::Allow);
    assert_eq!(evaluate("tool", "bash", &rules), Action::Ask); // no match → default
}

#[test]
fn test_evaluate_deny_blocks() {
    let rules = vec![Rule { permission: "tool".into(), pattern: "*".into(), action: Action::Deny }];
    assert_eq!(evaluate("tool", "anything", &rules), Action::Deny);
}

#[tokio::test]
async fn test_ask_flow_once_allows() {
    let svc = PermissionService::new_test();
    let task = tokio::spawn(async move {
        svc.check("tool", &["bash".into()], CheckOpts::ask()).await
    });
    // Simulate user reply
    svc.reply(&pending_id, ReplyBody { reply: Reply::Once, .. }).await.unwrap();
    assert!(matches!(task.await.unwrap(), Ok(Action::Allow)));
}
```

### Integration Tests (from TS patterns)

- `permission/arity.test.ts`: Argument count validation
- `permission/next.test.ts`: Full permission ask/reply flow with bus events
