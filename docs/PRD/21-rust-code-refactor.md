# PRD: Code Refactor — Rust Conventions Compliance

## Scope

This document defines the refactoring requirements and standards for the OpenCode Rust codebase to achieve full compliance with the project-wide Rust conventions defined in `.opencode/rules/rust/`.

This document is authoritative for:

- Coding style enforcement (`coding-style.md`)
- Rust-specific patterns adoption (`patterns.md`)
- Testing standards compliance (`testing.md`)
- Hooks configuration (`hooks.md`)
- Security practices verification (`security.md`)

This document is **not** authoritative for:

- Architectural changes (see `01-core-architecture.md`)
- API design decisions (see `07-server-api.md`)
- Feature specifications (see individual PRDs)

---

## Guiding Principles

### Rule Hierarchy

1. **Rust-specific rules** (`.opencode/rules/rust/`) take precedence over common rules
2. **Language idioms** override generic recommendations — prefer idiomatic Rust over generic patterns
3. **Zero tolerance** for warnings — `cargo clippy -- -D warnings` must pass
4. **Immutability by default** — prefer `let` over `let mut`, borrow over mutate

### Refactor Philosophy

- **Incremental compliance** — don't break working code; refactor systematically crate-by-crate
- **Preserve behavior** — every refactor must maintain identical behavior; tests are the contract
- **Enforcement over documentation** — prefer automated hooks over manual discipline
- **Visibility boundaries** — respect `pub`, `pub(crate)`, and private visibility rules

---

## Coding Style Requirements

### Formatting Enforcement

| Requirement | Command | Gate |
|-------------|---------|------|
| rustfmt formatting | `cargo fmt --all` | Pre-commit hook |
| clippy linting | `cargo clippy -- -D warnings` | CI gate |
| 4-space indent | Enforced by rustfmt | Auto |
| Max 100 char line width | Enforced by rustfmt | Auto |

### Immutability Standards

**DO:**
```rust
// Return new values, don't mutate in place
fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', "_"))
    } else {
        Cow::Borrowed(input)
    }
}
```

**DON'T:**
```rust
// Avoid unless mutation is genuinely required
fn normalize_bad(input: &mut String) {
    *input = input.replace(' ', "_");
}
```

### Ownership and Borrowing

**DO:**
```rust
// Borrow when ownership isn't needed
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

// Take ownership in constructors via Into
fn new(name: impl Into<String>) -> Self {
    Self { name: name.into() }
}
```

**DON'T:**
```rust
// Take String when &str suffices
fn word_count_bad(text: String) -> usize {
    text.split_whitespace().count()
}
```

### Error Handling

| Context | Approach |
|---------|----------|
| Libraries | `thiserror` for typed errors |
| Applications | `anyhow` for flexible context |
| Propagation | `?` operator |
| Forbidden | `unwrap()` in production |

**Required pattern:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid config format: {0}")]
    Parse(String),
}
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Functions/variables | `snake_case` | `get_session`, `session_id` |
| Types/traits/enums | `PascalCase` | `Session`, `ToolRegistry` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_TOKEN_BUDGET` |
| Lifetime parameters | `'a`, `'de` (short) | `'input` (complex only) |

### Module Organization

**DO:** Organize by domain
```text
src/
├── auth/
│   ├── mod.rs
│   ├── token.rs
│   └── middleware.rs
├── orders/
│   ├── mod.rs
│   └── service.rs
```

**DON'T:** Organize by type
```text
src/
├── structs.rs    # Don't do this
├── enums.rs
├── traits.rs
├── functions.rs
```

### Visibility Rules

- Default to **private**
- Use `pub(crate)` for internal crate sharing
- Only mark `pub` what is part of the public API
- Re-export public API from `lib.rs`

---

## Pattern Requirements

### Repository Pattern with Traits

All data access must be encapsulated behind traits:

```rust
pub trait OrderRepository: Send + Sync {
    fn find_by_id(&self, id: u64) -> Result<Option<Order>, StorageError>;
    fn find_all(&self) -> Result<Vec<Order>, StorageError>;
    fn save(&self, order: &Order) -> Result<Order, StorageError>;
    fn delete(&self, id: u64) -> Result<(), StorageError>;
}
```

### Service Layer Pattern

Business logic in service structs with injected dependencies:

```rust
pub struct OrderService {
    repo: Box<dyn OrderRepository>,
    payment: Box<dyn PaymentGateway>,
}

impl OrderService {
    pub fn new(repo: Box<dyn OrderRepository>, payment: Box<dyn PaymentGateway>) -> Self {
        Self { repo, payment }
    }
}
```

### Newtype Pattern for Type Safety

Prevent argument mix-ups with distinct wrapper types:

```rust
struct UserId(u64);
struct OrderId(u64);

fn get_order(user: UserId, order: OrderId) -> anyhow::Result<Order>
```

### Enum State Machines

Model states as enums — make illegal states unrepresentable:

```rust
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Failed { reason: String, retries: u32 },
}
```

Always match exhaustively — no wildcard `_` for business-critical enums.

### Builder Pattern

Use for structs with many optional parameters:

```rust
impl ServerConfig {
    pub fn builder(host: impl Into<String>, port: u16) -> ServerConfigBuilder {
        ServerConfigBuilder { host: host.into(), port, max_connections: 100 }
    }
}
```

### Sealed Traits for Extensibility Control

Use a private module to seal a trait, preventing external implementations:

```rust
mod private {
    pub trait Sealed {}
}

pub trait Format: private::Sealed {
    fn encode(&self, data: &[u8]) -> Vec<u8>;
}
```

---

## Testing Requirements

### Test Organization

```
crate/
├── src/
│   ├── lib.rs           # Unit tests in #[cfg(test)] modules
│   ├── auth/
│   │   └── mod.rs       # #[cfg(test)] mod tests { ... }
├── tests/               # Integration tests
│   ├── api_test.rs
│   └── common/
│       └── mod.rs
└── benches/             # Criterion benchmarks
```

### Unit Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_user_with_valid_email() {
        let user = User::new("Alice", "alice@example.com").unwrap();
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn rejects_invalid_email() {
        let result = User::new("Bob", "not-an-email");
        assert!(result.is_err());
    }
}
```

### Test Naming

Use descriptive names that explain the scenario:
- `creates_user_with_valid_email()`
- `rejects_order_when_insufficient_stock()`
- `returns_none_when_not_found()`

### Async Tests

```rust
#[tokio::test]
async fn fetches_data_successfully() {
    let client = TestClient::new().await;
    let result = client.get("/data").await;
    assert!(result.is_ok());
}
```

### Mocking with mockall

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;

    mockall::mock! {
        pub Repo {}
        impl UserRepository for Repo {
            fn find_by_id(&self, id: u64) -> Option<User>;
        }
    }

    #[test]
    fn service_returns_user_when_found() {
        let mut mock = MockRepo::new();
        mock.expect_find_by_id()
            .with(eq(42))
            .times(1)
            .returning(|_| Some(User { id: 42, name: "Alice".into() }));
    }
}
```

### Coverage Requirements

- **Target:** 80%+ line coverage
- **Tool:** `cargo-llvm-cov`
- **Gate:** `cargo llvm-cov --fail-under-lines 80`

### Testing Commands

```bash
cargo test                          # Run all tests
cargo test -- --nocapture          # Show println output
cargo test test_name                # Run tests matching pattern
cargo test --lib                   # Unit tests only
cargo test --test api_test         # Specific integration test
cargo test --doc                   # Doc tests only
cargo llvm-cov --fail-under-lines 80  # Coverage gate
```

---

## Hooks Configuration

### Required PostToolUse Hooks

Configure in `~/.claude/settings.json`:

| Hook | Trigger | Purpose |
|------|---------|---------|
| `cargo fmt` | After editing `.rs` files | Auto-format |
| `cargo clippy` | After editing `.rs` files | Lint checks |
| `cargo check` | After editing `.rs` files | Fast compilation verify |

---

## Security Requirements

### Secrets Management

**FORBIDDEN:**
```rust
const API_KEY: &str = "sk-abc123...";
```

**REQUIRED:**
```rust
fn load_api_key() -> anyhow::Result<String> {
    std::env::var("PAYMENT_API_KEY")
        .context("PAYMENT_API_KEY must be set")
}
```

### SQL Injection Prevention

**FORBIDDEN:**
```rust
let query = format!("SELECT * FROM users WHERE name = '{name}'");
```

**REQUIRED:**
```rust
sqlx::query("SELECT * FROM users WHERE name = $1")
    .bind(&name)
    .fetch_one(&pool)
    .await?;
```

### Input Validation

- Validate all input at system boundaries
- Use the type system to enforce invariants (newtype pattern)
- Parse, don't validate — convert unstructured data to typed structs

### Unsafe Code

- Minimize `unsafe` blocks — prefer safe abstractions
- Every `unsafe` block requires `// SAFETY:` comment
- Never use `unsafe` to bypass the borrow checker
- Audit all `unsafe` during code review

```rust
// SAFETY: `ptr` is non-null, aligned, points to an initialized Widget,
// and no mutable references exist for its lifetime.
unsafe { &*ptr }
```

### Dependency Security

```bash
cargo audit                           # Security audit
cargo deny check                      # License/advisory compliance
cargo tree -d                         # Show duplicate dependencies
```

---

## Refactor Areas

### Priority 1: Error Handling Standardization

| Crate | Current State | Target State |
|-------|--------------|--------------|
| `crates/core/` | Mixed `anyhow`/`thiserror` | `thiserror` for library errors |
| `crates/tools/` | `unwrap()` in production | Proper `Result` handling |
| `crates/server/` | Untyped errors | `thiserror` with `#[from]` |

### Priority 2: Visibility Audits

- [ ] Audit all `pub` items — reduce to `pub(crate)` where possible
- [ ] Ensure `lib.rs` re-exports only intended public API
- [ ] Verify no implementation leaks through `pub` interfaces

### Priority 3: Ownership Compliance

- [ ] Replace `&mut` with immutable borrow where mutation isn't required
- [ ] Convert `String` parameters to `&str` where applicable
- [ ] Use `Into<String>` for constructors that need ownership

### Priority 4: Test Coverage Gaps

| Crate | Current Coverage | Target |
|-------|------------------|--------|
| `crates/core/` | ~60% | 80%+ |
| `crates/tools/` | ~50% | 80%+ |
| `crates/agent/` | ~45% | 80%+ |

### Priority 5: Pattern Adoption

- [ ] Identify repositories lacking trait abstraction
- [ ] Add service layer where business logic exists in handlers
- [ ] Replace primitive obsession with newtype wrappers

---

## Enforcement Gates

### Pre-Commit Hooks

```bash
cargo fmt --all
cargo clippy --all -- -D warnings
cargo check
```

### CI Pipeline

| Stage | Command | Fail Condition |
|-------|---------|----------------|
| Format check | `cargo fmt --all -- --check` | Exit != 0 |
| Clippy | `cargo clippy --all -- -D warnings` | Warnings |
| Unit tests | `cargo test --lib` | Failures |
| Integration | `cargo test --test '*'` | Failures |
| Coverage | `cargo llvm-cov --fail-under-lines 80` | Below 80% |
| Security | `cargo audit` | CVEs found |
| Deny check | `cargo deny check` | Advisories |

### Convention Tests

Maintain `tests/conventions/` with automated checks for:
- No `unwrap()` in production code
- Proper error type usage per crate context
- Visibility compliance
- Naming convention enforcement

---

## Cross-References

- [01-core-architecture.md](./01-core-architecture.md) — entity definitions
- [02-agent-system.md](./02-agent-system.md) — agent patterns
- [03-tools-system.md](./03-tools-system.md) — tool registry
- [16-test-plan.md](./16-test-plan.md) — validation strategy
- [17-rust-test-implementation-roadmap.md](./17-rust-test-implementation-roadmap.md) — test phasing
- [18-crate-by-crate-test-backlog.md](./18-crate-by-crate-test-backlog.md) — backlog by crate
- [19-implementation-plan.md](./19-implementation-plan.md) — implementation phases

---

## Appendix: Quick Reference

### Rust Rules File Locations

| Rule | Location |
|------|----------|
| Coding Style | `.opencode/rules/rust/coding-style.md` |
| Patterns | `.opencode/rules/rust/patterns.md` |
| Testing | `.opencode/rules/rust/testing.md` |
| Hooks | `.opencode/rules/rust/hooks.md` |
| Security | `.opencode/rules/rust/security.md` |

### Command Reference

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings

# Check
cargo check

# Test
cargo test

# Coverage
cargo llvm-cov --fail-under-lines 80

# Security
cargo audit
cargo deny check
```
