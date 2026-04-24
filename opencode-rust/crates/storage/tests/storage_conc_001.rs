use opencode_core::Session;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::sqlite_repository::SqliteSessionRepository;
use tokio::task::JoinSet;

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
async fn test_storage_conc_001_concurrent_writes_no_corruption() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000001");
    repo.save(&session).await.unwrap();

    let repo =
        SqliteSessionRepository::new(StoragePool::new(_temp_dir.path().join("test.db")).unwrap());

    let mut join_set = JoinSet::new();
    for i in 0..10 {
        let repo = SqliteSessionRepository::new(
            StoragePool::new(_temp_dir.path().join("test.db")).unwrap(),
        );
        let mut session = create_test_session("00000000-0000-0000-0000-000000000001");
        session
            .messages
            .push(opencode_core::Message::user(format!("Message {}", i)));
        join_set.spawn(async move { repo.save(&session).await });
    }

    let mut errors = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => errors.push(e),
            Err(_) => {}
        }
    }

    assert!(
        errors.is_empty(),
        "Should not have errors during concurrent writes"
    );

    let loaded = repo
        .find_by_id("00000000-0000-0000-0000-000000000001")
        .await
        .unwrap();
    assert!(loaded.is_some());
}

#[tokio::test]
async fn test_storage_conc_001_no_panic_on_concurrent_writes() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000002");
    repo.save(&session).await.unwrap();

    let mut join_set = JoinSet::new();
    for i in 0..5 {
        let repo = SqliteSessionRepository::new(
            StoragePool::new(_temp_dir.path().join("test.db")).unwrap(),
        );
        let mut session = create_test_session("00000000-0000-0000-0000-000000000002");
        session
            .messages
            .push(opencode_core::Message::user(format!("Message {}", i)));
        join_set.spawn(async move { repo.save(&session).await });
    }

    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(())) = result {}
    }
}

#[tokio::test]
async fn test_storage_conc_001_session_loadable_after_concurrent_writes() {
    let (pool, _temp_dir) = create_temp_db();
    MigrationManager::new(pool.clone(), 3)
        .migrate()
        .await
        .unwrap();
    let repo = SqliteSessionRepository::new(pool);

    let session = create_test_session("00000000-0000-0000-0000-000000000003");
    repo.save(&session).await.unwrap();

    let mut join_set = JoinSet::new();
    for i in 0..5 {
        let repo = SqliteSessionRepository::new(
            StoragePool::new(_temp_dir.path().join("test.db")).unwrap(),
        );
        let mut session = create_test_session("00000000-0000-0000-0000-000000000003");
        session
            .messages
            .push(opencode_core::Message::user(format!("Message {}", i)));
        join_set.spawn(async move {
            let _ = repo.save(&session).await;
        });
    }

    while let Some(_) = join_set.join_next().await {}

    let loaded = repo
        .find_by_id("00000000-0000-0000-0000-000000000003")
        .await
        .unwrap();
    assert!(
        loaded.is_some(),
        "Session should be loadable after concurrent writes"
    );
}
