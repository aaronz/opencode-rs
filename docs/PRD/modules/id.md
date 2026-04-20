# PRD: id Module

## Module Overview

- **Module Name**: id
- **Source Path**: `packages/opencode/src/id/id.ts`
- **Type**: Utility
- **Purpose**: Typed, prefixed, sortable unique identifier generation. All entity IDs in OpenCode use this module for consistent formatting, prefix validation, and sort-direction control.

---

## Functionality

### Core Features

1. **Prefixed IDs** — All IDs have a short prefix derived from the entity type (e.g., `ses`, `msg`, `pty`)
2. **Ascending IDs** — Timestamps encoded in ascending sort order (oldest first) — used for most entities
3. **Descending IDs** — Timestamps encoded in descending sort order (newest first)
4. **Monotonic Generation** — Same-millisecond IDs increment a counter to prevent collisions
5. **Schema Validation** — Zod schema for each ID type (prefix-validated)
6. **Fixed Length** — All IDs are 26 characters total (prefix + timestamp + random)
7. **Base62 Encoding** — Random component uses `0-9A-Za-z` characters

### Prefix Map

```typescript
const prefixes = {
  event:      "evt",
  session:    "ses",
  message:    "msg",
  permission: "per",
  question:   "que",
  user:       "usr",
  part:       "prt",
  pty:        "pty",
  tool:       "tool",
  workspace:  "wrk",
  entry:      "ent",
}
```

---

## API Surface

```typescript
// Schema for validating an ID (startsWith check)
function schema(prefix: keyof typeof prefixes): ZodString

// Generate ascending-sorted ID
function ascending(prefix: keyof typeof prefixes, given?: string): string

// Generate descending-sorted ID
function descending(prefix: keyof typeof prefixes, given?: string): string

// Low-level: create ID with explicit prefix string and direction
function create(prefix: string, direction: "ascending" | "descending", timestamp?: number): string
```

### ID Format

```
<prefix><timestamp_component><counter><random>
Total length: 26 characters
```

- Timestamp is encoded so that ascending IDs sort lexicographically oldest-first
- Descending IDs invert the timestamp encoding for newest-first sort
- Same-millisecond collision resistance via monotonic counter

---

## Dependencies

- Node.js `crypto.randomBytes` — for base62 random component
- No external dependencies

---

## Acceptance Criteria

- [ ] Generated IDs start with the correct prefix for each entity type
- [ ] Ascending IDs sort lexicographically with oldest first
- [ ] Descending IDs sort lexicographically with newest first
- [ ] Two IDs generated in the same millisecond are distinct (monotonic counter)
- [ ] `schema(prefix)` returns a Zod string validator that rejects wrong prefixes
- [ ] `ascending(prefix, given)` returns `given` unchanged if it already has correct prefix
- [ ] Throws if `given` has wrong prefix

---

## Rust Implementation Guidance

### Module: `crates/core/src/id.rs` or `crates/util/src/id.rs`

### Key Crates

```toml
ulid = "1"         # ULID for timestamp-based sortable IDs
rand = "0.8"       # Random component
```

### Implementation Strategy

Use ULID (Universally Unique Lexicographically Sortable Identifier) as the base — it's 26 chars, base32, timestamp-prefixed, and monotonic. Then prepend the entity prefix.

```rust
pub const PREFIXES: &[(&str, &str)] = &[
    ("event",      "evt"),
    ("session",    "ses"),
    ("message",    "msg"),
    ("permission", "per"),
    ("question",   "que"),
    ("user",       "usr"),
    ("part",       "prt"),
    ("pty",        "pty"),
    ("tool",       "tool"),
    ("workspace",  "wrk"),
    ("entry",      "ent"),
];

pub struct IdGenerator {
    last_ms: AtomicU64,
    counter: AtomicU32,
}

impl IdGenerator {
    pub fn ascending(&self, prefix: &str) -> String {
        let ts = self.next_timestamp();
        let random = self.random_base62(20);
        format!("{}{:013}{}", prefix, ts, random)
    }

    pub fn descending(&self, prefix: &str) -> String {
        let ts = self.next_timestamp();
        let inverted = u64::MAX - ts;
        let random = self.random_base62(20);
        format!("{}{:013}{}", prefix, inverted, random)
    }

    fn random_base62(&self, len: usize) -> String {
        const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let bytes: Vec<u8> = (0..len).map(|_| CHARS[rand::random::<u8>() as usize % 62]).collect();
        String::from_utf8(bytes).unwrap()
    }
}

pub fn validate_prefix(id: &str, entity: &str) -> bool {
    let prefix = PREFIXES.iter().find(|(e, _)| *e == entity).map(|(_, p)| *p);
    prefix.map(|p| id.starts_with(p)).unwrap_or(false)
}
```

### Using ULID (simpler alternative)

```rust
use ulid::Ulid;

pub fn ascending(prefix: &str) -> String {
    format!("{}{}", prefix, Ulid::new().to_string().to_lowercase())
}
```

---

## Test Design

### Unit Tests

```rust
#[test]
fn test_ascending_starts_with_prefix() {
    let id = ascending("ses");
    assert!(id.starts_with("ses"), "Expected ses prefix, got: {}", id);
}

#[test]
fn test_ascending_sort_order() {
    let id1 = ascending("msg");
    std::thread::sleep(std::time::Duration::from_millis(1));
    let id2 = ascending("msg");
    assert!(id1 < id2, "id1={} should sort before id2={}", id1, id2);
}

#[test]
fn test_descending_sort_order() {
    let id1 = descending("msg");
    std::thread::sleep(std::time::Duration::from_millis(1));
    let id2 = descending("msg");
    assert!(id1 > id2, "Descending: id1={} should sort after id2={}", id1, id2);
}

#[test]
fn test_same_millisecond_ids_are_unique() {
    // Generate many IDs rapidly
    let ids: std::collections::HashSet<String> = (0..1000).map(|_| ascending("msg")).collect();
    assert_eq!(ids.len(), 1000, "All IDs should be unique");
}

#[test]
fn test_validate_prefix_correct() {
    let id = ascending("ses");
    assert!(validate_prefix(&id, "session"));
}

#[test]
fn test_validate_prefix_wrong_prefix_fails() {
    let id = "msg_notasession".to_string();
    assert!(!validate_prefix(&id, "session"));
}

#[test]
fn test_given_id_returned_if_prefix_matches() {
    let existing = "ses_existing_id_here".to_string();
    // If function validates prefix and returns as-is
    let result = ascending_or_given("ses", Some(&existing));
    assert_eq!(result, existing);
}
```
