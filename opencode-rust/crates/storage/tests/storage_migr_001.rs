use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::sqlite_repository::SqliteSessionRepository;
use tempfile::tempdir;

#[tokio::test]
async fn test_storage_migr_001_migration_creates_schema() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).unwrap();
    let manager = MigrationManager::new(pool, 3);

    let result = manager.migrate().await;
    assert!(result.is_ok(), "Migration should succeed");

    let repo = SqliteSessionRepository::new(StoragePool::new(&db_path).unwrap());
    let count = repo.count().await.unwrap();
    assert_eq!(count, 0, "Empty database should have 0 sessions");
}

#[tokio::test]
async fn test_storage_migr_001_idempotent_migration() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).unwrap();
    let manager = MigrationManager::new(pool.clone(), 3);

    manager.migrate().await.unwrap();

    let manager2 = MigrationManager::new(pool, 3);
    let result = manager2.migrate().await;
    assert!(
        result.is_ok(),
        "Second migration should also succeed (idempotent)"
    );
}

#[tokio::test]
async fn test_storage_migr_001_database_works_after_migration() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).unwrap();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();

    let repo = SqliteSessionRepository::new(pool);
    let mut session = opencode_core::Session::new();
    session.id = uuid::Uuid::new_v4();

    let save_result = repo.save(&session).await;
    assert!(
        save_result.is_ok(),
        "Should be able to save after migration"
    );

    let loaded = repo.find_by_id(&session.id.to_string()).await.unwrap();
    assert!(loaded.is_some(), "Should be able to load after migration");
}

#[tokio::test]
async fn test_storage_migr_001_schema_persists_after_close() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    {
        let pool = StoragePool::new(&db_path).unwrap();
        MigrationManager::new(pool, 3).migrate().await.unwrap();
    }

    {
        let pool = StoragePool::new(&db_path).unwrap();
        let repo = SqliteSessionRepository::new(pool);

        let mut session = opencode_core::Session::new();
        session.id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        repo.save(&session).await.unwrap();

        let loaded = repo
            .find_by_id("00000000-0000-0000-0000-000000000001")
            .await
            .unwrap();
        assert!(loaded.is_some());
    }
}
