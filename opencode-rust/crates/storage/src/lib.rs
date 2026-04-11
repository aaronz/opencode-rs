use opencode_core::OpenCodeError;
pub mod compaction;
pub mod database;
pub mod migration;
pub mod models;
pub mod service;

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
pub use migration::MigrationManager;
pub use service::StorageService;
