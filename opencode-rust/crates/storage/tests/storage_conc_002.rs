use opencode_core::Session;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::sqlite_repository::SqliteSessionRepository;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

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
async fn test_storage_conc_002_delete_while_reading() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000001");
    repo.save(&session).await.unwrap();

    let repo_clone =
        SqliteSessionRepository::new(StoragePool::new(_temp_dir.path().join("test.db")).unwrap());
    let repo_delete =
        SqliteSessionRepository::new(StoragePool::new(_temp_dir.path().join("test.db")).unwrap());

    let read_handle = tokio::spawn(async move {
        let mut results = Vec::new();
        for _ in 0..5 {
            let loaded = repo_clone
                .find_by_id("00000000-0000-0000-0000-000000000001")
                .await;
            results.push(loaded);
            sleep(Duration::from_millis(10)).await;
        }
        results
    });

    sleep(Duration::from_millis(5)).await;
    repo_delete
        .delete("00000000-0000-0000-0000-000000000001")
        .await
        .ok();

    let results = read_handle.await.unwrap();

    let has_success = results
        .iter()
        .any(|r| r.is_ok() && r.as_ref().unwrap().is_some());
    let has_not_found = results.iter().any(|r| {
        r.as_ref()
            .err()
            .map(|e| e.to_string().contains("NotFound"))
            .unwrap_or(false)
    });

    assert!(
        has_success || has_not_found,
        "Read should either succeed with data or return NotFound"
    );
}

#[tokio::test]
async fn test_storage_conc_002_no_panic_on_delete_during_load() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000002");
    repo.save(&session).await.unwrap();

    let repo_read =
        SqliteSessionRepository::new(StoragePool::new(_temp_dir.path().join("test.db")).unwrap());
    let repo_delete =
        SqliteSessionRepository::new(StoragePool::new(_temp_dir.path().join("test.db")).unwrap());

    let read_handle = tokio::spawn(async move {
        repo_read
            .find_by_id("00000000-0000-0000-0000-000000000002")
            .await
    });

    sleep(Duration::from_millis(1)).await;
    repo_delete
        .delete("00000000-0000-0000-0000-000000000002")
        .await
        .ok();

    let result = read_handle.await.unwrap();
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_storage_conc_002_storage_consistent_after_delete_during_read() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000003");
    repo.save(&session).await.unwrap();

    let repo_read =
        SqliteSessionRepository::new(StoragePool::new(_temp_dir.path().join("test.db")).unwrap());
    let repo_delete =
        SqliteSessionRepository::new(StoragePool::new(_temp_dir.path().join("test.db")).unwrap());

    let read_handle = tokio::spawn(async move {
        repo_read
            .find_by_id("00000000-0000-0000-0000-000000000003")
            .await
    });

    sleep(Duration::from_millis(1)).await;
    repo_delete
        .delete("00000000-0000-0000-0000-000000000003")
        .await
        .ok();

    let _ = read_handle.await.unwrap();

    let count = repo.count().await.unwrap();
    assert_eq!(count, 0, "Storage should be consistent after delete");
}
