use opencode_core::Message;
use opencode_storage::database::StoragePool;
use opencode_storage::{InMemoryProjectRepository, InMemorySessionRepository, StorageService};
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_storage_e2e_004_concurrent_session_access() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let pool = StoragePool::new(&db_path).unwrap();

    let session_repo: Arc<InMemorySessionRepository> = Arc::new(InMemorySessionRepository::new());
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let service = Arc::new(StorageService::new(
        session_repo.clone(),
        project_repo.clone(),
        pool,
    ));

    let num_sessions = 5;
    let mut handles = vec![];

    for i in 0..num_sessions {
        let service = service.clone();
        let handle = tokio::spawn(async move {
            let mut session = opencode_core::Session::new();
            session.add_message(Message::user(format!("Message for session {}", i)));
            session.add_message(Message::assistant(format!("Response for session {}", i)));
            let result = service.save_session(&session).await;
            (i, session.id, result)
        });
        handles.push(handle);
    }

    let mut creation_results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        creation_results.push(result);
    }

    for (i, _, result) in &creation_results {
        assert!(result.is_ok(), "Session {} creation should succeed", i);
    }

    let mut load_handles = vec![];
    for (_, session_id, _) in &creation_results {
        let service = service.clone();
        let id = session_id.clone();
        let handle = tokio::spawn(async move { service.load_session(&id.to_string()).await });
        load_handles.push(handle);
    }

    let mut load_results = vec![];
    for handle in load_handles {
        let result = handle.await.unwrap();
        load_results.push(result);
    }

    for result in &load_results {
        assert!(result.is_ok(), "Session load should succeed");
        let loaded = result.as_ref().unwrap().as_ref().unwrap();
        assert_eq!(
            loaded.messages.len(),
            2,
            "Each session should have 2 messages"
        );
    }

    let ids: Vec<_> = creation_results.iter().map(|(_, id, _)| id).collect();
    let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(
        unique_ids.len(),
        num_sessions,
        "All session IDs should be unique"
    );

    drop(temp_dir);
}
