use opencode_core::Session;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::sqlite_repository::SqliteSessionRepository;
use tempfile::tempdir;

fn create_temp_db() -> (StoragePool, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let pool = StoragePool::new(&db_path).expect("Failed to create temp database");
    (pool, temp_dir)
}

#[tokio::test]
async fn test_storage_query_002_find_all_returns_sessions() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let mut session1 = Session::new();
    session1.id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let mut session2 = Session::new();
    session2.id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    repo.save(&session1).await.unwrap();
    repo.save(&session2).await.unwrap();

    let all = repo.find_all(10, 0).await.unwrap();
    assert_eq!(all.len(), 2, "Should return all saved sessions");
}

#[tokio::test]
async fn test_storage_query_002_find_all_ordered_by_updated_at() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let mut session1 = Session::new();
    session1.id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let mut session2 = Session::new();
    session2.id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    repo.save(&session1).await.unwrap();
    repo.save(&session2).await.unwrap();

    let all = repo.find_all(10, 0).await.unwrap();
    assert_eq!(all.len(), 2, "Should return all sessions");

    let first = &all[0];
    let second = &all[1];
    assert!(
        first.updated_at >= second.updated_at,
        "Sessions should be ordered by updated_at descending (most recent first)"
    );
}

#[tokio::test]
async fn test_storage_query_002_find_all_with_pagination_preserves_ordering() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    for i in 0..20 {
        let mut session = Session::new();
        session.id =
            uuid::Uuid::parse_str(&format!("00000000-0000-0000-0000-00000000{:04}", i)).unwrap();
        repo.save(&session).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }

    let all_ordered: Vec<_> = repo.find_all(100, 0).await.unwrap();

    let page1 = repo.find_all(10, 0).await.unwrap();
    let page2 = repo.find_all(10, 10).await.unwrap();

    assert_eq!(page1[0].id, all_ordered[0].id);
    assert_eq!(page2[0].id, all_ordered[10].id);
    assert_eq!(page1.last().unwrap().id, all_ordered[9].id);
    assert_eq!(page2.last().unwrap().id, all_ordered[19].id);
}

#[tokio::test]
async fn test_storage_query_002_no_false_positives_in_find_all() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = Session::new();
    repo.save(&session).await.unwrap();

    let all = repo.find_all(100, 0).await.unwrap();
    assert_eq!(all.len(), 1, "Should only return the saved session");

    let found = repo.find_by_id(&session.id.to_string()).await.unwrap();
    assert!(found.is_some(), "Should find by id");
}
