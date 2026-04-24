use chrono::Utc;
use opencode_storage::models::ProjectModel;
use opencode_storage::{InMemoryProjectRepository, ProjectRepository};
use uuid::Uuid;

#[tokio::test]
async fn test_storage_e2e_005_project_save_and_load() {
    let repo = InMemoryProjectRepository::new();

    let original = ProjectModel {
        id: Uuid::new_v4().to_string(),
        path: "/tmp/test_project".to_string(),
        name: Some("Test Project".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: Some(r#"{"key": "value"}"#.to_string()),
    };

    repo.save(&original).await.unwrap();

    let loaded = repo.find_by_id(&original.id).await.unwrap();
    assert!(loaded.is_some());

    let loaded = loaded.unwrap();
    assert_eq!(loaded.id, original.id);
    assert_eq!(loaded.path, original.path);
    assert_eq!(loaded.name, original.name);
    assert_eq!(loaded.data, original.data);
}

#[tokio::test]
async fn test_storage_e2e_005_project_find_by_path() {
    let repo = InMemoryProjectRepository::new();

    let project = ProjectModel {
        id: Uuid::new_v4().to_string(),
        path: "/tmp/test_project".to_string(),
        name: Some("Test Project".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: None,
    };

    repo.save(&project).await.unwrap();

    let loaded = repo.find_by_path("/tmp/test_project").await.unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().id, project.id);
}

#[tokio::test]
async fn test_storage_e2e_005_project_delete() {
    let repo = InMemoryProjectRepository::new();

    let project = ProjectModel {
        id: Uuid::new_v4().to_string(),
        path: "/tmp/test_project".to_string(),
        name: Some("Test Project".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: None,
    };

    repo.save(&project).await.unwrap();
    assert!(repo.find_by_id(&project.id).await.unwrap().is_some());

    repo.delete(&project.id).await.unwrap();
    assert!(repo.find_by_id(&project.id).await.unwrap().is_none());
    assert!(repo.find_by_id(&project.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_storage_e2e_005_project_persistence() {
    let repo = InMemoryProjectRepository::new();

    let project1 = ProjectModel {
        id: Uuid::new_v4().to_string(),
        path: "/tmp/project1".to_string(),
        name: Some("Project 1".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: Some(r#"{"type": "web"}"#.to_string()),
    };

    let project2 = ProjectModel {
        id: Uuid::new_v4().to_string(),
        path: "/tmp/project2".to_string(),
        name: Some("Project 2".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: Some(r#"{"type": "api"}"#.to_string()),
    };

    repo.save(&project1).await.unwrap();
    repo.save(&project2).await.unwrap();

    assert_eq!(repo.count().await.unwrap(), 2);

    let all = repo.find_all(10, 0).await.unwrap();
    assert_eq!(all.len(), 2);

    let by_path1 = repo.find_by_path("/tmp/project1").await.unwrap();
    let by_path2 = repo.find_by_path("/tmp/project2").await.unwrap();

    assert_eq!(by_path1.unwrap().name, Some("Project 1".to_string()));
    assert_eq!(by_path2.unwrap().name, Some("Project 2".to_string()));
}
