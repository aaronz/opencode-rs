use opencode_core::Session;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::sqlite_repository::SqliteSessionRepository;
use std::fs;
use tempfile::tempdir;

fn create_temp_db() -> (StoragePool, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let pool = StoragePool::new(&db_path).expect("Failed to create temp database");
    (pool, temp_dir)
}

#[tokio::test]
async fn test_storage_backup_001_backup_is_valid_sqlite() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let backup_path = temp_dir.path().join("backup.db");

    {
        let pool = StoragePool::new(&db_path).unwrap();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);

        let mut session = Session::new();
        session.id = uuid::Uuid::new_v4();
        repo.save(&session).await.unwrap();
    }

    fs::copy(&db_path, &backup_path).unwrap();

    let backup_pool = StoragePool::new(&backup_path).unwrap();
    let repo = SqliteSessionRepository::new(backup_pool);

    let count = repo.count().await.unwrap();
    assert_eq!(count, 1, "Backup should contain the saved session");

    let loaded = repo.find_all(10, 0).await.unwrap();
    assert!(
        loaded.len() == 1,
        "Should be able to load session from backup"
    );
}

#[tokio::test]
async fn test_storage_backup_001_backup_during_write() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let backup_path = temp_dir.path().join("backup.db");

    {
        let pool = StoragePool::new(&db_path).unwrap();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
    }

    let pool = StoragePool::new(&db_path).unwrap();
    let repo = SqliteSessionRepository::new(pool);

    for i in 0..10 {
        let mut session = Session::new();
        session.id = uuid::Uuid::new_v4();
        repo.save(&session).await.unwrap();
    }

    fs::copy(&db_path, &backup_path).unwrap();

    let mut session = Session::new();
    session.id = uuid::Uuid::new_v4();
    repo.save(&session).await.unwrap();

    let backup_pool = StoragePool::new(&backup_path).unwrap();
    let backup_repo = SqliteSessionRepository::new(backup_pool);

    let backup_count = backup_repo.count().await.unwrap();
    assert_eq!(
        backup_count, 10,
        "Backup should have 10 sessions (before the last write)"
    );

    let current_count = repo.count().await.unwrap();
    assert_eq!(current_count, 11, "Current should have 11 sessions");
}

#[tokio::test]
async fn test_storage_backup_001_backup_consistent_after_multiple_writes() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let backup_path = temp_dir.path().join("backup.db");

    let original_ids: Vec<_> = (0..5).map(|_| uuid::Uuid::new_v4()).collect();

    {
        let pool = StoragePool::new(&db_path).unwrap();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);

        for id in &original_ids {
            let mut session = Session::new();
            session.id = *id;
            repo.save(&session).await.unwrap();
        }
    }

    fs::copy(&db_path, &backup_path).unwrap();

    {
        let pool = StoragePool::new(&db_path).unwrap();
        let repo = SqliteSessionRepository::new(pool);

        for _ in 0..10 {
            let mut session = Session::new();
            session.id = uuid::Uuid::new_v4();
            repo.save(&session).await.unwrap();
        }
    }

    let backup_pool = StoragePool::new(&backup_path).unwrap();
    let backup_repo = SqliteSessionRepository::new(backup_pool);

    let backup_sessions: Vec<_> = backup_repo.find_all(100, 0).await.unwrap();
    let current_count = SqliteSessionRepository::new(StoragePool::new(&db_path).unwrap())
        .count()
        .await
        .unwrap();

    assert_eq!(
        backup_sessions.len(),
        5,
        "Backup should be point-in-time consistent with 5 sessions"
    );

    for id in &original_ids {
        let loaded = backup_repo.find_by_id(&id.to_string()).await.unwrap();
        assert!(
            loaded.is_some(),
            "Each original session should be in backup"
        );
    }

    assert_eq!(current_count, 15, "Current should have all 15 sessions");
}
