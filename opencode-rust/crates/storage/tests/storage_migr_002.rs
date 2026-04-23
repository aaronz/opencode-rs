use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::sqlite_repository::SqliteSessionRepository;
use tempfile::tempdir;

#[tokio::test]
async fn test_storage_migr_002_migration_with_many_sessions() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).unwrap();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();

    let repo = SqliteSessionRepository::new(pool);

    for i in 0..100 {
        let mut session = opencode_core::Session::new();
        session.id =
            uuid::Uuid::parse_str(&format!("00000000-0000-0000-0000-00000000{:04}", i)).unwrap();
        repo.save(&session).await.unwrap();
    }

    let count = repo.count().await.unwrap();
    assert_eq!(count, 100, "Should have 100 sessions");

    let loaded = repo
        .find_by_id("00000000-0000-0000-0000-000000000050")
        .await
        .unwrap();
    assert!(loaded.is_some(), "Should be able to load session 50");
}

#[tokio::test]
async fn test_storage_migr_002_migration_idempotent_with_data() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).unwrap();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();

    let repo = SqliteSessionRepository::new(pool.clone());
    let mut session = opencode_core::Session::new();
    session.id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    repo.save(&session).await.unwrap();

    drop(repo);

    let pool2 = StoragePool::new(&db_path).unwrap();
    let manager = MigrationManager::new(pool2, 3);
    let result = manager.migrate().await;
    assert!(result.is_ok(), "Migration should be idempotent");

    let repo2 = SqliteSessionRepository::new(StoragePool::new(&db_path).unwrap());
    let count = repo2.count().await.unwrap();
    assert_eq!(count, 1, "Should still have 1 session after re-migration");
}

#[tokio::test]
async fn test_storage_migr_002_pagination_works_after_migration() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).unwrap();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();

    let repo = SqliteSessionRepository::new(pool);

    for i in 0..50 {
        let mut session = opencode_core::Session::new();
        session.id =
            uuid::Uuid::parse_str(&format!("00000000-0000-0000-0000-00000000{:04}", i)).unwrap();
        repo.save(&session).await.unwrap();
    }

    let all = repo.find_all(100, 0).await.unwrap();
    assert_eq!(all.len(), 50, "Should retrieve all 50 sessions");

    let page1 = repo.find_all(20, 0).await.unwrap();
    assert_eq!(page1.len(), 20, "First page should have 20");

    let page2 = repo.find_all(20, 20).await.unwrap();
    assert_eq!(page2.len(), 20, "Second page should have 20");

    let page3 = repo.find_all(20, 40).await.unwrap();
    assert_eq!(page3.len(), 10, "Third page should have 10");

    let empty_page = repo.find_all(20, 60).await.unwrap();
    assert_eq!(empty_page.len(), 0, "Beyond should be empty");
}
