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
async fn test_storage_query_001_pagination_basic() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    for i in 0..50 {
        let mut session = Session::new();
        session.id =
            uuid::Uuid::parse_str(&format!("00000000-0000-0000-0000-00000000{:04}", i)).unwrap();
        repo.save(&session).await.unwrap();
    }

    let page1 = repo.find_all(10, 0).await.unwrap();
    assert_eq!(page1.len(), 10, "First page should have 10");

    let page2 = repo.find_all(10, 10).await.unwrap();
    assert_eq!(page2.len(), 10, "Second page should have 10");

    let page5 = repo.find_all(10, 40).await.unwrap();
    assert_eq!(page5.len(), 10, "Fifth page should have 10");
}

#[tokio::test]
async fn test_storage_query_001_pagination_beyond_end() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    for i in 0..25 {
        let mut session = Session::new();
        session.id =
            uuid::Uuid::parse_str(&format!("00000000-0000-0000-0000-00000000{:04}", i)).unwrap();
        repo.save(&session).await.unwrap();
    }

    let empty_page = repo.find_all(10, 100).await.unwrap();
    assert_eq!(empty_page.len(), 0, "Page beyond data should be empty");

    let partial_page = repo.find_all(10, 20).await.unwrap();
    assert_eq!(partial_page.len(), 5, "Partial page at end should have 5");
}

#[tokio::test]
async fn test_storage_query_001_pagination_no_overlap() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    for i in 0..30 {
        let mut session = Session::new();
        session.id =
            uuid::Uuid::parse_str(&format!("00000000-0000-0000-0000-00000000{:04}", i)).unwrap();
        repo.save(&session).await.unwrap();
    }

    let page0 = repo.find_all(10, 0).await.unwrap();
    let page1 = repo.find_all(10, 10).await.unwrap();
    let page2 = repo.find_all(10, 20).await.unwrap();

    let ids0: Vec<_> = page0.iter().map(|s| s.id).collect();
    let ids1: Vec<_> = page1.iter().map(|s| s.id).collect();
    let ids2: Vec<_> = page2.iter().map(|s| s.id).collect();

    for id0 in &ids0 {
        assert!(!ids1.contains(id0), "Page 0 and 1 should not overlap");
        assert!(!ids2.contains(id0), "Page 0 and 2 should not overlap");
    }
    assert!(
        !ids1.iter().any(|id| ids2.contains(id)),
        "Page 1 and 2 should not overlap"
    );
}

#[tokio::test]
async fn test_storage_query_001_pagination_count() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    for i in 0..100 {
        let mut session = Session::new();
        session.id =
            uuid::Uuid::parse_str(&format!("00000000-0000-0000-0000-00000000{:04}", i)).unwrap();
        repo.save(&session).await.unwrap();
    }

    let count = repo.count().await.unwrap();
    assert_eq!(count, 100, "Count should return total sessions");

    let page_counts: Vec<usize> = vec![
        repo.find_all(10, 0).await.unwrap().len(),
        repo.find_all(10, 10).await.unwrap().len(),
        repo.find_all(10, 20).await.unwrap().len(),
        repo.find_all(10, 30).await.unwrap().len(),
        repo.find_all(10, 40).await.unwrap().len(),
        repo.find_all(10, 50).await.unwrap().len(),
        repo.find_all(10, 60).await.unwrap().len(),
        repo.find_all(10, 70).await.unwrap().len(),
        repo.find_all(10, 80).await.unwrap().len(),
        repo.find_all(10, 90).await.unwrap().len(),
    ];

    let total_from_pages: usize = page_counts.iter().sum();
    assert_eq!(
        total_from_pages, 100,
        "Sum of all pages should equal total count"
    );
}
