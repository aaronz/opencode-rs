# storage.md — Storage Module

## Module Overview

- **Crate**: `opencode-storage`
- **Source**: `crates/storage/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Session and project persistence layer with SQLite backend, in-memory repositories for testing, crash recovery, compaction management, and snapshot durability.

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
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
anyhow = "1.0"
tokio = { version = "1.45", features = ["full"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

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
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Session locked: {0}")]
    SessionLocked(String),
    #[error("Compaction error: {0}")]
    Compaction(String),
    #[error("Recovery error: {0}")]
    Recovery(String),
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
    pub async fn load_session(&self, id: Uuid) -> Result<Session, StorageError>;
    pub async fn list_sessions(&self) -> Result<Vec<SessionSummary>, StorageError>;
    pub async fn delete_session(&self, id: Uuid) -> Result<(), StorageError>;
    pub async fn compact_session(&self, id: Uuid) -> Result<CompactionResult, StorageError>;

    // Project operations
    pub async fn save_project(&self, project: &Project) -> Result<(), StorageError>;
    pub async fn load_project(&self, id: Uuid) -> Result<Project, StorageError>;
    pub async fn list_projects(&self) -> Result<Vec<ProjectSummary>, StorageError>;

    // Recovery
    pub async fn recover_session(&self, id: Uuid) -> Result<Session, StorageError>;
    pub async fn list_incomplete_sessions(&self) -> Result<Vec<Uuid>, StorageError>;
}
```

### Repository Traits

```rust
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &Session) -> Result<(), StorageError>;
    async fn load(&self, id: Uuid) -> Result<Session, StorageError>;
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
    async fn list(&self) -> Result<Vec<SessionSummary>, StorageError>;
    async fn exists(&self, id: Uuid) -> Result<bool, StorageError>;
}

pub trait ProjectRepository: Send + Sync {
    async fn save(&self, project: &Project) -> Result<(), StorageError>;
    async fn load(&self, id: Uuid) -> Result<Project, StorageError>;
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
    async fn list(&self) -> Result<Vec<ProjectSummary>, StorageError>;
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
    async fn exists(&self, id: Uuid) -> Result<bool, StorageError> { ... }
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
```

### CompactionManager

```rust
pub struct CompactionManager {
    config: CompactionConfig,
    summarizer: SessionSummarizer,
}

impl CompactionManager {
    pub fn new(config: CompactionConfig) -> Self { ... }
    
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
pub struct SessionSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub model: Option<String>,
    pub last_message_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
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
    
    // Add compaction
    let compaction = CompactionManager::new(CompactionConfig::default());
    service = service.with_compaction_manager(compaction);
    
    // Save a session
    let session = Session::default();
    service.save_session(&session).await?;
    
    // Load a session
    let loaded = service.load_session(session.id).await?;
    println!("Loaded session: {}", loaded.id);
    
    // List all sessions
    let summaries = service.list_sessions().await?;
    for summary in summaries {
        println!("Session: {} ({})", summary.id, summary.title.unwrap_or_default());
    }
    
    Ok(())
}
```
