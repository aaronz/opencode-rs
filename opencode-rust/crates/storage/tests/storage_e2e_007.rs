use opencode_core::Message;
use opencode_storage::{InMemorySessionRepository, SessionRepository};

#[tokio::test]
async fn test_storage_e2e_007_inmemory_repository_save_and_load() {
    let repo = InMemorySessionRepository::new();

    let mut session = opencode_core::Session::new();
    session.add_message(Message::user("Test message".to_string()));

    repo.save(&session).await.unwrap();

    let loaded = repo.find_by_id(&session.id.to_string()).await.unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().messages.len(), 1);
}

#[tokio::test]
async fn test_storage_e2e_007_inmemory_repository_list_all() {
    let repo = InMemorySessionRepository::new();

    for i in 0..5 {
        let mut session = opencode_core::Session::new();
        session.add_message(Message::user(format!("Message {}", i)));
        repo.save(&session).await.unwrap();
    }

    let all = repo.find_all(10, 0).await.unwrap();
    assert_eq!(all.len(), 5);

    let count = repo.count().await.unwrap();
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_storage_e2e_007_inmemory_repository_delete() {
    let repo = InMemorySessionRepository::new();

    let mut session = opencode_core::Session::new();
    session.add_message(Message::user("To be deleted".to_string()));
    repo.save(&session).await.unwrap();

    let id = session.id.to_string();
    assert!(repo.exists(&id).await.unwrap());

    repo.delete(&id).await.unwrap();

    assert!(!repo.exists(&id).await.unwrap());
    assert!(repo.find_by_id(&id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_storage_e2e_007_inmemory_repository_multiple_operations() {
    let repo = InMemorySessionRepository::new();

    let mut session1 = opencode_core::Session::new();
    session1.add_message(Message::user("First session".to_string()));
    repo.save(&session1).await.unwrap();

    let mut session2 = opencode_core::Session::new();
    session2.add_message(Message::user("Second session".to_string()));
    repo.save(&session2).await.unwrap();

    assert_eq!(repo.count().await.unwrap(), 2);

    repo.delete(&session1.id.to_string()).await.unwrap();
    assert_eq!(repo.count().await.unwrap(), 1);

    let remaining = repo.find_all(10, 0).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, session2.id);
}
