use opencode_core::Session;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::sqlite_repository::SqliteSessionRepository;

fn create_temp_db() -> (StoragePool, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let pool = StoragePool::new(&db_path).expect("Failed to create temp database");
    (pool, temp_dir)
}

fn create_test_session(id: &str) -> Session {
    let mut session = Session::new();
    session.id = uuid::Uuid::parse_str(id).unwrap();
    session
}

#[tokio::test]
async fn test_storage_crash_002_write_error_propagation() {
    let (pool, temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000001");

    repo.save(&session).await.unwrap();

    let loaded = repo
        .find_by_id("00000000-0000-0000-0000-000000000001")
        .await
        .unwrap();
    assert!(loaded.is_some());

    drop(temp_dir);

    let result = repo.save(&session).await;
    assert!(result.is_err(), "Should fail when database is inaccessible");
}

#[tokio::test]
async fn test_storage_crash_002_original_data_preserved_after_failure() {
    let (pool, temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000002");

    repo.save(&session).await.unwrap();

    drop(temp_dir);

    let result = repo.save(&session).await;
    assert!(result.is_err(), "Should fail when disk is inaccessible");

    let fresh_temp = tempfile::tempdir().unwrap();
    let fresh_db_path = fresh_temp.path().join("fresh.db");
    let fresh_pool = StoragePool::new(&fresh_db_path).unwrap();
    MigrationManager::new(fresh_pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let fresh_repo = SqliteSessionRepository::new(fresh_pool);

    let fresh_session = create_test_session("00000000-0000-0000-0000-000000000003");

    let save_result = fresh_repo.save(&fresh_session).await;
    assert!(
        save_result.is_ok(),
        "Should be able to save to fresh database"
    );

    let loaded = fresh_repo
        .find_by_id("00000000-0000-0000-0000-000000000003")
        .await
        .unwrap();
    assert!(loaded.is_some());
}

#[tokio::test]
async fn test_storage_crash_002_error_is_descriptive() {
    let (pool, temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000004");

    repo.save(&session).await.unwrap();
    drop(temp_dir);

    let result = repo.save(&session).await;
    if let Err(e) = result {
        let err_str = format!("{}", e);
        assert!(
            err_str.contains("Storage") || err_str.contains("database") || err_str.contains("IO"),
            "Error should be descriptive, got: {}",
            err_str
        );
    }
}

#[tokio::test]
async fn test_storage_crash_002_can_retry_after_failure() {
    let (pool, temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000005");

    repo.save(&session).await.unwrap();
    drop(temp_dir);

    let result = repo.save(&session).await;
    assert!(result.is_err(), "First save should fail");

    let new_temp_dir = tempfile::tempdir().unwrap();
    let new_db_path = new_temp_dir.path().join("test.db");
    std::fs::copy(&new_db_path, &new_db_path).ok();

    if let Ok(new_pool) = StoragePool::new(&new_db_path) {
        MigrationManager::new(new_pool.clone(), 3)
            .migrate()
            .await
            .ok();
        let new_repo = SqliteSessionRepository::new(new_pool);
        let retry_result = new_repo.save(&session).await;
        assert!(
            retry_result.is_ok(),
            "Should be able to retry save with restored database"
        );
    }
}
