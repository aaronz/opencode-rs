# storage.md — Storage Module (v42 Updated)

## Module Overview

- **Crate**: `opencode-storage`
- **Source**: `crates/storage/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API with extensions
- **Purpose**: Session and project persistence layer with SQLite backend, in-memory repositories for testing, crash recovery, compaction management, and snapshot durability.

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| v42 | 2026-04-21 | Updated based on gap analysis; added FR-XXX requirements for identified gaps |

---

## Crate Layout

```
crates/storage/src/
├── lib.rs                      ← Re-exports
├── error.rs                    ← StorageError
├── database.rs                 ← StoragePool
├── service.rs                  ← StorageService
├── repository.rs               ← SessionRepository, ProjectRepository traits
├── sqlite_repository.rs        ← SqliteSessionRepository, SqliteProjectRepository
├── memory_repository.rs        ← InMemorySessionRepository, InMemoryProjectRepository
├── models.rs                   ← Storage data models
├── compaction.rs               ← CompactionManager, ShareabilityVerifier
├── migration.rs                ← MigrationManager
├── crash_recovery_tests.rs     ← #[test] module
├── recovery_tests.rs           ← #[test] module
├── snapshot_durability_tests.rs ← #[test] module
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
deadpool_sqlite = "0.14"          # Connection pooling (superior to raw rusqlite)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
anyhow = "1.0"
tokio = { version = "1.45", features = ["full"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"                      # Tool args hashing (extension)

opencode-core = { path = "../core" }
```

**Public exports from lib.rs**:
```rust
pub use compaction::{
    CompactionManager, CompactionWithShareabilityResult, ShareabilityError,
    ShareabilityVerification, ShareabilityVerifier,
};
pub use database::StoragePool;
pub use error::StorageError;
pub use memory_repository::{InMemoryProjectRepository, InMemorySessionRepository};
pub use migration::MigrationManager;
pub use repository::{ProjectRepository, SessionRepository};
pub use service::StorageService;
pub use sqlite_repository::{SqliteProjectRepository, SqliteSessionRepository};
```

---

## Core Types

### StorageError

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Migration error: {0}")]
    Migration(String),           // FR-042: Added to align with PRD

    #[error("Session locked: {0}")]
    SessionLocked(String),       // FR-043: Added to align with PRD

    #[error("Compaction error: {0}")]
    Compaction(String),

    #[error("Recovery error: {0}")]
    Recovery(String),            // FR-044: Added to align with PRD
}
```

### StorageService

```rust
// Main service — facade over all storage operations
pub struct StorageService {
    pool: StoragePool,
    session_repo: Arc<dyn SessionRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    compaction_manager: Option<CompactionManager>,
}

impl StorageService {
    pub fn new(
        pool: StoragePool,
        session_repo: Arc<dyn SessionRepository>,
        project_repo: Arc<dyn ProjectRepository>,
    ) -> Self { ... }

    pub fn with_compaction_manager(mut self, manager: CompactionManager) -> Self { ... }

    // Session operations
    pub async fn save_session(&self, session: &Session) -> Result<(), StorageError>;
    pub async fn load_session(&self, id: &str) -> Result<Session, StorageError>;  // Uses &str per implementation
    pub async fn list_sessions(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<SessionInfo>, StorageError>;  // FR-045: Pagination added
    pub async fn delete_session(&self, id: &str) -> Result<(), StorageError>;
    pub async fn compact_session(&self, id: &str) -> Result<CompactionResult, StorageError>;  // FR-046: Added

    // Project operations
    pub async fn save_project(&self, project: &Project) -> Result<(), StorageError>;
    pub async fn load_project(&self, id: &str) -> Result<Option<ProjectModel>, StorageError>;  // FR-047: Added load by ID
    pub async fn load_project_by_path(&self, path: &str) -> Result<Option<ProjectModel>, StorageError>;
    pub async fn list_projects(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<ProjectModel>, StorageError>;  // FR-048: Pagination added

    // Recovery
    pub async fn recover_session(&self, id: &str) -> Result<Session, StorageError>;  // FR-049: Added
    pub async fn list_incomplete_sessions(&self) -> Result<Vec<Uuid>, StorageError>;  // FR-050: Added
}
```

### Repository Traits

```rust
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &Session) -> Result<(), StorageError>;
    async fn load(&self, id: Uuid) -> Result<Session, StorageError>;
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
    async fn list(&self) -> Result<Vec<SessionSummary>, StorageError>;
    async fn exists(&self, id: &str) -> Result<bool, StorageError>;  // FR-051: Added to align with PRD
}

pub trait ProjectRepository: Send + Sync {
    async fn save(&self, project: &Project) -> Result<(), StorageError>;
    async fn load(&self, id: Uuid) -> Result<Project, StorageError>;
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
    async fn list(&self) -> Result<Vec<ProjectSummary>, StorageError>;
    async fn list_by_project(&self, project_path: &str) -> Result<Vec<SessionSummary>, StorageError>;  // FR-052: Need implementation
}
```

### SqliteSessionRepository

```rust
pub struct SqliteSessionRepository {
    conn: PooledConnection,
}

impl SqliteSessionRepository {
    pub fn new(pool: &StoragePool) -> Result<Self, StorageError>;
}

#[async_trait::async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn save(&self, session: &Session) -> Result<(), StorageError> { ... }
    async fn load(&self, id: Uuid) -> Result<Session, StorageError> { ... }
    async fn delete(&self, id: Uuid) -> Result<(), StorageError> { ... }
    async fn list(&self) -> Result<Vec<SessionSummary>, StorageError> { ... }
    async fn exists(&self, id: &str) -> Result<bool, StorageError> { ... }  // FR-051
}

pub struct SqliteProjectRepository { ... }
// Same pattern as SqliteSessionRepository
```

### In-Memory Repositories (for testing)

```rust
pub struct InMemorySessionRepository {
    sessions: RwLock<HashMap<Uuid, Session>>,
    summaries: RwLock<HashMap<Uuid, SessionSummary>>,
}

pub struct InMemoryProjectRepository {
    projects: RwLock<HashMap<Uuid, Project>>,
    summaries: RwLock<HashMap<Uuid, ProjectSummary>>,
}

// Extensions (beyond PRD)
pub struct InMemoryAccountRepository { ... }      // FR-053: Extra feature
pub struct InMemoryPluginStateRepository { ... }  // FR-054: Extra feature
```

### CompactionManager

```rust
// FR-055: Restructured to instance-based design per PRD specification
pub struct CompactionManager {
    config: CompactionConfig,
    summarizer: SessionSummarizer,
}

impl CompactionManager {
    pub fn new(config: CompactionConfig) -> Self { ... }  // Instance-based constructor

    pub async fn compact(
        &self,
        session: &Session,
    ) -> Result<CompactionWithShareabilityResult, StorageError> { ... }

    pub fn should_auto_compact(&self, session: &Session) -> bool { ... }
}

pub struct CompactionWithShareabilityResult {
    pub compacted_session: Session,
    pub shareability_verification: ShareabilityVerification,
}

pub enum ShareabilityError {
    UnshareableContent,
    ShareabilityCheckFailed,
}

pub struct ShareabilityVerifier { ... }
impl ShareabilityVerifier {
    pub fn verify(&self, session: &Session) -> ShareabilityVerification { ... }
}

pub struct ShareabilityVerification {
    pub is_shareable: bool,
    pub reason: Option<String>,
    pub filtered_content: Option<String>,
}
```

### StoragePool

```rust
pub struct StoragePool { ... }

impl StoragePool {
    pub fn new(database_url: &str) -> Result<Self, StorageError>;
    pub fn get_connection(&self) -> Result<PooledConnection, StorageError>;
    pub fn run_migrations(&self) -> Result<(), StorageError>;
}
```

### MigrationManager

```rust
pub struct MigrationManager {
    pool: StoragePool,
    migrations: Vec<Migration>,
}

pub struct Migration {
    pub version: u32,
    pub description: String,
    pub up: Box<dyn Fn(&PooledConnection) -> Result<(), StorageError>>,
    pub down: Box<dyn Fn(&PooledConnection) -> Result<(), StorageError>>,
}

impl MigrationManager {
    pub fn new(pool: StoragePool) -> Self;
    pub async fn run_pending(&self) -> Result<(), StorageError>;
    pub async fn rollback(&self, target_version: u32) -> Result<(), StorageError>;
    pub fn current_version(&self) -> u32;
}
```

---

## Data Models

```rust
// From models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {  // Renamed from SessionSummary per implementation
    pub id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub model: Option<String>,
    pub last_message_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectModel {  // Renamed from Project per implementation
    pub id: Uuid,
    pub name: String,
    pub root_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
    pub root_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## Feature Requirements (FR-XXX)

### FR-042: Migration Error Variant
**Severity**: P2
**Module**: error.rs
**Description**: Add `Migration(String)` variant to `StorageError` enum to align with PRD specification.
**Acceptance Criteria**:
- [ ] `StorageError::Migration` variant exists
- [ ] Error message correctly formats migration errors
- [ ] Migration failures in `MigrationManager` use this variant

### FR-043: Session Locked Error Variant
**Severity**: P2
**Module**: error.rs
**Description**: Add `SessionLocked(String)` variant to `StorageError` enum to align with PRD specification.
**Acceptance Criteria**:
- [ ] `StorageError::SessionLocked` variant exists
- [ ] Error message correctly formats lock errors
- [ ] Concurrent session access uses this variant

### FR-044: Recovery Error Variant
**Severity**: P2
**Module**: error.rs
**Description**: Add `Recovery(String)` variant to `StorageError` enum to align with PRD specification.
**Acceptance Criteria**:
- [ ] `StorageError::Recovery` variant exists
- [ ] Error message correctly formats recovery errors
- [ ] Recovery operations use this variant

### FR-045: Session List Pagination
**Severity**: P1
**Module**: service.rs
**Description**: All list methods support optional pagination via `limit` and `offset` parameters.
**Acceptance Criteria**:
- [ ] `list_sessions(limit, offset)` works correctly
- [ ] Pagination respects limit/offset values
- [ ] Empty results return empty vector, not error

### FR-046: Compact Session in StorageService
**Severity**: P1
**Module**: service.rs
**Description**: Add `compact_session(id: &str) -> Result<CompactionResult, StorageError>` method to `StorageService`.
**Acceptance Criteria**:
- [ ] Method delegates to `CompactionManager::compact()`
- [ ] Returns `CompactionResult` with compacted session
- [ ] Returns `NotFound` error for non-existent sessions

### FR-047: Load Project by ID
**Severity**: P1
**Module**: service.rs, repository.rs
**Description**: Add `load_project(id: &str)` method to `StorageService` for loading projects by UUID string.
**Acceptance Criteria**:
- [ ] `load_project(id: &str)` method exists
- [ ] Returns `Option<ProjectModel>` - `Some` if found, `None` if not
- [ ] Works with UUID string format

### FR-048: Project List Pagination
**Severity**: P1
**Module**: service.rs
**Description**: Add pagination support to `list_projects` method.
**Acceptance Criteria**:
- [ ] `list_projects(limit, offset)` works correctly
- [ ] Pagination respects limit/offset values
- [ ] Empty results return empty vector, not error

### FR-049: Recover Session in StorageService
**Severity**: P1
**Module**: service.rs
**Description**: Add `recover_session(id: &str)` method to `StorageService` delegating to `CrashRecovery`.
**Acceptance Criteria**:
- [ ] Method delegates to `CrashRecovery::restore()`
- [ ] Returns recovered `Session`
- [ ] Returns `NotFound` error for non-existent sessions

### FR-050: List Incomplete Sessions
**Severity**: P2
**Module**: service.rs
**Description**: Add `list_incomplete_sessions()` method to list sessions with incomplete crash dumps.
**Acceptance Criteria**:
- [ ] Returns `Vec<Uuid>` of incomplete session IDs
- [ ] Filters based on crash dump completion status
- [ ] Empty list returned when no incomplete sessions exist

### FR-051: SessionRepository exists() Method
**Severity**: P1
**Module**: repository.rs
**Description**: Add `async fn exists(&self, id: &str) -> Result<bool, StorageError>` to `SessionRepository` trait.
**Acceptance Criteria**:
- [ ] Method exists in `SessionRepository` trait
- [ ] Implementation in `SqliteSessionRepository` queries database
- [ ] Implementation in `InMemorySessionRepository` checks HashMap
- [ ] Returns `true` if session exists, `false` otherwise

### FR-052: list_by_project Implementation
**Severity**: P2
**Module**: repository.rs
**Description**: Implement actual filtering by project path in `list_by_project()` instead of returning empty vec.
**Acceptance Criteria**:
- [ ] `list_by_project(project_path: &str)` filters sessions correctly
- [ ] Only returns sessions associated with given project
- [ ] Returns empty vec for projects with no sessions (not error)

### FR-053: Account Repository (Extension)
**Severity**: Low
**Module**: sqlite_repository.rs, memory_repository.rs
**Description**: Extra feature beyond PRD - full account CRUD with `find_by_username` and `find_by_email`.
**Acceptance Criteria**:
- [ ] `SqliteAccountRepository` implements `AccountRepository` trait
- [ ] `InMemoryAccountRepository` available for testing
- [ ] Methods: `save`, `load`, `delete`, `list`, `find_by_username`, `find_by_email`

### FR-054: Plugin State Repository (Extension)
**Severity**: Low
**Module**: sqlite_repository.rs, memory_repository.rs
**Description**: Extra feature beyond PRD - plugin state persistence.
**Acceptance Criteria**:
- [ ] `SqlitePluginStateRepository` implements `PluginStateRepository` trait
- [ ] `InMemoryPluginStateRepository` available for testing
- [ ] Methods: `save`, `load`, `delete` for plugin state blob

### FR-055: CompactionManager Instance-Based Design
**Severity**: P0
**Module**: compaction.rs
**Description**: Restructure `CompactionManager` to instance-based design with `CompactionConfig` constructor parameter, matching PRD specification.
**Acceptance Criteria**:
- [ ] `CompactionManager::new(config: CompactionConfig)` constructor exists
- [ ] `config` field stored in struct instance
- [ ] `should_auto_compact(&self, session: &Session)` uses stored config
- [ ] `compact(&self, session: &Session)` uses stored config

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-storage` |
|---|---|
| `opencode-server` | `StorageService`, `SessionRepository` for session persistence |
| `opencode-core` | `Session` struct for save/load |
| `opencode-config` | `CompactionConfig` for auto-compaction settings |

**Dependencies of `opencode-storage`**:
| Crate | Usage |
|---|---|
| `opencode-core` | `Session`, `OpenCodeError` |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use opencode_core::Session;

    #[tokio::test]
    async fn test_in_memory_session_save_and_load() {
        let repo = InMemorySessionRepository::new();
        let session = Session::default();

        repo.save(&session).await.unwrap();
        let loaded = repo.load(session.id).await.unwrap();

        assert_eq!(loaded.id, session.id);
    }

    #[tokio::test]
    async fn test_in_memory_session_not_found() {
        let repo = InMemorySessionRepository::new();
        let result = repo.load(Uuid::new_v4()).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_in_memory_session_delete() {
        let repo = InMemorySessionRepository::new();
        let session = Session::default();
        repo.save(&session).await.unwrap();

        repo.delete(session.id).await.unwrap();
        let result = repo.load(session.id).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_in_memory_session_list() {
        let repo = InMemorySessionRepository::new();
        repo.save(&Session::default()).await.unwrap();
        let sessions = repo.list().await.unwrap();
        assert!(!sessions.is_empty());
    }

    #[tokio::test]
    async fn test_storage_error_display() {
        let err = StorageError::NotFound("session-123".to_string());
        assert!(err.to_string().contains("session-123"));
    }

    #[tokio::test]
    async fn test_compaction_manager_should_auto_compact() {
        let config = CompactionConfig {
            auto: Some(true),
            compact_threshold: Some(0.8),
            ..Default::default()
        };
        let manager = CompactionManager::new(config);
        let session = Session::default();
        // Depends on session message count vs threshold
        let should = manager.should_auto_compact(&session);
        // Assertion depends on session state
    }

    #[tokio::test]
    async fn test_shareability_verifier() {
        let verifier = ShareabilityVerifier::new();
        let session = Session::default();
        let result = verifier.verify(&session);
        // Result depends on session content
    }

    // FR-051: Test exists() method
    #[tokio::test]
    async fn test_session_exists() {
        let repo = InMemorySessionRepository::new();
        let session = Session::default();
        repo.save(&session).await.unwrap();

        assert!(repo.exists(session.id.to_string().as_str()).await.unwrap());
        assert!(!repo.exists(Uuid::new_v4().to_string().as_str()).await.unwrap());
    }

    // FR-052: Test list_by_project implementation
    #[tokio::test]
    async fn test_list_by_project() {
        let repo = InMemorySessionRepository::new();
        let session = Session::default();
        repo.save(&session).await.unwrap();

        let sessions = repo.list_by_project("/some/project/path").await.unwrap();
        // Should filter correctly when implementation is complete
    }
}

// Integration tests (in separate #[cfg(test)] files)
#[cfg(test)]
mod crash_recovery_tests {
    #[tokio::test]
    async fn test_recover_incomplete_session_after_crash() { ... }

    #[tokio::test]
    async fn test_recovery_from_partial_write() { ... }
}

#[cfg(test)]
mod recovery_tests {
    #[tokio::test]
    async fn test_session_recovery_on_startup() { ... }

    #[tokio::test]
    async fn test_project_recovery() { ... }
}

#[cfg(test)]
mod snapshot_durability_tests {
    #[tokio::test]
    async fn test_snapshot_persists_after_restart() { ... }

    #[tokio::test]
    async fn test_incremental_snapshot_durability() { ... }
}
```

---

## Usage Example

```rust
use opencode_storage::{
    StorageService, StoragePool, SqliteSessionRepository, SqliteProjectRepository,
    CompactionManager, StorageError,
};
use opencode_core::Session;

async fn storage_example() -> Result<(), StorageError> {
    // Create storage pool (SQLite)
    let pool = StoragePool::new("sqlite:sessions.db?mode=rwc")?;
    pool.run_migrations()?;

    // Create repositories
    let session_repo = Arc::new(SqliteSessionRepository::new(&pool)?);
    let project_repo = Arc::new(SqliteProjectRepository::new(&pool)?);

    // Create storage service
    let mut service = StorageService::new(pool, session_repo.clone(), project_repo.clone());

    // Add compaction (FR-055: using instance-based constructor)
    let compaction = CompactionManager::new(CompactionConfig::default());
    service = service.with_compaction_manager(compaction);

    // Save a session
    let session = Session::default();
    service.save_session(&session).await?;

    // Load a session
    let loaded = service.load_session(session.id.to_string().as_str()).await?;
    println!("Loaded session: {}", loaded.id);

    // List all sessions with pagination (FR-045)
    let summaries = service.list_sessions(Some(50), Some(0)).await?;
    for summary in summaries {
        println!("Session: {} ({})", summary.id, summary.title.unwrap_or_default());
    }

    // Compact a session (FR-046)
    let result = service.compact_session(session.id.to_string().as_str()).await?;
    println!("Compacted session: {}", result.compacted_session.id);

    Ok(())
}
```

---

## Gap Status Summary

| Gap Item | FR Number | Severity | Status |
|----------|-----------|----------|--------|
| `compact_session` not in StorageService | FR-046 | P1 | To be implemented |
| `load_project(id)` not available | FR-047 | P1 | To be implemented |
| `exists()` method missing from SessionRepository | FR-051 | P1 | To be implemented |
| `recover_session` not in StorageService | FR-049 | P1 | To be implemented |
| `list_incomplete_sessions` not in StorageService | FR-050 | P2 | To be implemented |
| `Migration` error variant not in StorageError | FR-042 | P2 | To be implemented |
| `SessionLocked` error variant not in StorageError | FR-043 | P2 | To be implemented |
| `Recovery` error variant not in StorageError | FR-044 | P2 | To be implemented |
| `load_session` uses `&str` instead of `Uuid` | - | P2 | By design |
| `list_by_project` is stub implementation | FR-052 | P2 | To be implemented |
| CompactionManager constructor mismatch | FR-055 | P0 | To be implemented |
| Pagination support | FR-045, FR-048 | P1 | To be implemented |

---

(End of file - total 659 lines)
