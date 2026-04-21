//! Structured error types for the storage crate.
//!
//! Error code ranges:
//! - 51xx: Session operations (not found, corrupted)
//! - 52xx: Project operations (not found, corrupted)
//! - 53xx: Account operations (not found, auth)
//! - 54xx: Database connection/pool errors
//! - 55xx: Serialization errors
//! - 59xx: Internal storage errors

//! - 56xx: Migration errors

use thiserror::Error;

/// Structured storage error type with specific variants for different failure modes.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StorageError {
    // --- Session errors (51xx) ---
    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("session corrupted: {0}")]
    SessionCorrupted(String),

    #[error("session expired: {0}")]
    SessionExpired(String),

    // --- Project errors (52xx) ---
    #[error("project not found: {0}")]
    ProjectNotFound(String),

    #[error("project corrupted: {0}")]
    ProjectCorrupted(String),

    // --- Account errors (53xx) ---
    #[error("account not found: {0}")]
    AccountNotFound(String),

    #[error("account auth failed: {0}")]
    AccountAuthFailed(String),

    // --- Database errors (54xx) ---
    #[error("database error: {0}")]
    Database(String),

    #[error("database pool error: {0}")]
    PoolError(String),

    // --- Serialization errors (55xx) ---
    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    // --- Migration errors (56xx) ---
    #[error("migration error: {0}")]
    Migration(String),

    // --- Internal errors (59xx) ---
    #[error("internal storage error: {0}")]
    Internal(String),
}

impl StorageError {
    /// Returns the numeric error code per FR-118 storage specification.
    pub fn code(&self) -> u16 {
        match self {
            Self::SessionNotFound(_) => 5101,
            Self::SessionCorrupted(_) => 5102,
            Self::SessionExpired(_) => 5103,
            Self::ProjectNotFound(_) => 5201,
            Self::ProjectCorrupted(_) => 5202,
            Self::AccountNotFound(_) => 5301,
            Self::AccountAuthFailed(_) => 5302,
            Self::Database(_) => 5401,
            Self::PoolError(_) => 5402,
            Self::Serialization(_) => 5501,
            Self::Deserialization(_) => 5502,
            Self::Migration(_) => 5601,
            Self::Internal(_) => 5901,
        }
    }
}

// --- From implementations for ergonomic error conversion ---

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        use rusqlite::Error;
        match &err {
            Error::QueryReturnedNoRows => {
                // This is handled at a higher level - caller should convert to NotFound
                Self::Database(err.to_string())
            }
            Error::SqliteFailure(_, _) => {
                // Constraint violations are just database errors at this level
                Self::Database(err.to_string())
            }
            _ => Self::Database(err.to_string()),
        }
    }
}

impl From<deadpool_sqlite::PoolError> for StorageError {
    fn from(err: deadpool_sqlite::PoolError) -> Self {
        Self::PoolError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        Self::Deserialization(err.to_string())
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<opencode_core::OpenCodeError> for StorageError {
    fn from(err: opencode_core::OpenCodeError) -> Self {
        Self::Database(err.to_string())
    }
}

// Conversion from StorageError to OpenCodeError for compatibility
impl From<StorageError> for opencode_core::OpenCodeError {
    fn from(err: StorageError) -> Self {
        opencode_core::OpenCodeError::Storage(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::SessionNotFound("abc".into());
        assert!(err.to_string().contains("session not found"));
        assert_eq!(err.code(), 5101);
    }

    #[test]
    fn test_storage_error_code_database() {
        let err = StorageError::Database("constraint violation".into());
        assert_eq!(err.code(), 5401);
    }

    #[test]
    fn test_storage_error_code_serialization() {
        let err = StorageError::Serialization("invalid json".into());
        assert_eq!(err.code(), 5501);
    }

    #[test]
    fn test_from_rusqlite_error() {
        let rusqlite_err = rusqlite::Error::InvalidParameterName("test".into());
        let storage_err: StorageError = rusqlite_err.into();
        match storage_err {
            StorageError::Database(_) => {}
            other => panic!("expected Database variant, got {:?}", other),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let storage_err: StorageError = json_err.into();
        match storage_err {
            StorageError::Deserialization(_) => {}
            other => panic!("expected Deserialization variant, got {:?}", other),
        }
    }

    #[test]
    fn test_from_pool_error() {
        let _ = StorageError::PoolError("pool closed".into());
    }

    #[test]
    fn test_storage_error_all_codes() {
        assert_eq!(StorageError::SessionNotFound("test".into()).code(), 5101);
        assert_eq!(StorageError::SessionCorrupted("test".into()).code(), 5102);
        assert_eq!(StorageError::SessionExpired("test".into()).code(), 5103);
        assert_eq!(StorageError::ProjectNotFound("test".into()).code(), 5201);
        assert_eq!(StorageError::ProjectCorrupted("test".into()).code(), 5202);
        assert_eq!(StorageError::AccountNotFound("test".into()).code(), 5301);
        assert_eq!(StorageError::AccountAuthFailed("test".into()).code(), 5302);
        assert_eq!(StorageError::Database("test".into()).code(), 5401);
        assert_eq!(StorageError::PoolError("test".into()).code(), 5402);
        assert_eq!(StorageError::Serialization("test".into()).code(), 5501);
        assert_eq!(StorageError::Deserialization("test".into()).code(), 5502);
        assert_eq!(StorageError::Migration("test".into()).code(), 5601);
        assert_eq!(StorageError::Internal("test".into()).code(), 5901);
    }

    #[test]
    fn test_storage_error_display_all_variants() {
        let variants = [
            (
                StorageError::SessionNotFound("test".into()),
                "session not found",
            ),
            (
                StorageError::SessionCorrupted("test".into()),
                "session corrupted",
            ),
            (
                StorageError::SessionExpired("test".into()),
                "session expired",
            ),
            (
                StorageError::ProjectNotFound("test".into()),
                "project not found",
            ),
            (
                StorageError::ProjectCorrupted("test".into()),
                "project corrupted",
            ),
            (
                StorageError::AccountNotFound("test".into()),
                "account not found",
            ),
            (
                StorageError::AccountAuthFailed("test".into()),
                "account auth failed",
            ),
            (StorageError::Database("test".into()), "database error"),
            (
                StorageError::PoolError("test".into()),
                "database pool error",
            ),
            (
                StorageError::Serialization("test".into()),
                "serialization error",
            ),
            (
                StorageError::Deserialization("test".into()),
                "deserialization error",
            ),
            (
                StorageError::Migration("test".into()),
                "migration error",
            ),
            (
                StorageError::Internal("test".into()),
                "internal storage error",
            ),
        ];

        for (err, expected_substring) in variants {
            let display = err.to_string();
            assert!(
                display.contains(expected_substring),
                "Expected '{}' to contain '{}'",
                display,
                expected_substring
            );
        }
    }

    #[test]
    fn test_from_io_error() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let storage_err: StorageError = io_err.into();
        match storage_err {
            StorageError::Database(_) => {}
            other => panic!("expected Database variant, got {:?}", other),
        }
    }

    #[test]
    fn test_from_opencode_error() {
        let core_err = opencode_core::OpenCodeError::Storage("test".into());
        let storage_err: StorageError = core_err.into();
        match storage_err {
            StorageError::Database(_) => {}
            other => panic!("expected Database variant, got {:?}", other),
        }
    }

    #[test]
    fn test_storage_error_to_opencore_error() {
        let storage_err = StorageError::SessionNotFound("test".into());
        let core_err: opencode_core::OpenCodeError = storage_err.into();
        match core_err {
            opencode_core::OpenCodeError::Storage(_) => {}
            other => panic!("expected Storage variant, got {:?}", other),
        }
    }

    #[test]
    fn test_rusqlite_error_query_returned_no_rows() {
        let rusqlite_err = rusqlite::Error::QueryReturnedNoRows;
        let storage_err: StorageError = rusqlite_err.into();
        match storage_err {
            StorageError::Database(_) => {}
            other => panic!("expected Database variant, got {:?}", other),
        }
    }

    // FR-042: Migration error variant tests
    #[test]
    fn test_migration_error_format() {
        let err = StorageError::Migration("schema version mismatch".into());
        let display = err.to_string();
        assert!(display.contains("migration error"));
        assert!(display.contains("schema version mismatch"));
    }

    #[test]
    fn test_migration_error_code() {
        let err = StorageError::Migration("test".into());
        assert_eq!(err.code(), 5601);
    }

    #[test]
    fn test_migration_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<StorageError>();
    }

    #[test]
    fn test_migration_error_source() {
        let err = StorageError::Migration("test migration failure".into());
        let error_string = err.to_string();
        assert!(error_string.contains("test migration failure"));
    }
}
