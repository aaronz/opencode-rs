use opencode_core::OpenCodeError;
pub mod compaction;
pub mod database;
pub mod error;
pub mod memory_repository;
pub mod migration;
pub mod models;
pub mod repository;
pub mod service;
pub mod sqlite_repository;

#[cfg(test)]
mod crash_recovery_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod snapshot_durability_tests;

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
pub use sqlite_repository::{
    SqliteAccountRepository, SqlitePluginStateRepository, SqliteProjectRepository,
    SqliteSessionRepository,
};
