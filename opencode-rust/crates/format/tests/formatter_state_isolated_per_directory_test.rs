use std::collections::HashMap;
use std::path::Path;

use opencode_config::{FormatterConfig, FormatterEntry};
use opencode_format::{FormatService, InstanceStateManager};

#[tokio::test]
async fn formatter_state_isolated_per_directory() {
    let service = FormatService::new();

    let mut project_a_formatters = HashMap::new();
    project_a_formatters.insert(
        "prettier".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["prettier".to_string(), "--write".to_string()]),
            environment: None,
            extensions: Some(vec![".js".to_string(), ".ts".to_string()]),
        },
    );

    let mut project_b_formatters = HashMap::new();
    project_b_formatters.insert(
        "rustfmt".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["rustfmt".to_string()]),
            environment: None,
            extensions: Some(vec![".rs".to_string()]),
        },
    );

    let _ = service
        .init(
            Path::new("/project-a"),
            FormatterConfig::Formatters(project_a_formatters),
        )
        .await;
    let _ = service
        .init(
            Path::new("/project-b"),
            FormatterConfig::Formatters(project_b_formatters),
        )
        .await;

    let manager = service.instance_manager().lock().await;
    assert_eq!(
        manager.instances_count(),
        2,
        "Should have 2 separate instances"
    );

    let project_a_instance = manager
        .get(Path::new("/project-a"))
        .expect("project-a instance should exist");
    let project_b_instance = manager
        .get(Path::new("/project-b"))
        .expect("project-b instance should exist");

    match project_a_instance.formatter_config() {
        FormatterConfig::Formatters(map) => {
            assert!(
                map.contains_key("prettier"),
                "project-a should have prettier"
            );
            assert!(
                !map.contains_key("rustfmt"),
                "project-a should NOT have rustfmt"
            );
        }
        _ => panic!("Expected Formatters variant for project-a"),
    }

    match project_b_instance.formatter_config() {
        FormatterConfig::Formatters(map) => {
            assert!(
                !map.contains_key("prettier"),
                "project-b should NOT have prettier"
            );
            assert!(map.contains_key("rustfmt"), "project-b should have rustfmt");
        }
        _ => panic!("Expected Formatters variant for project-b"),
    }
}

#[tokio::test]
async fn instance_state_manager_tracks_multiple_directories() {
    let service = FormatService::new();

    let _ = service
        .init(
            Path::new("/workspace/project-1"),
            FormatterConfig::Disabled(false),
        )
        .await;
    let _ = service
        .init(
            Path::new("/workspace/project-2"),
            FormatterConfig::Disabled(false),
        )
        .await;
    let _ = service
        .init(
            Path::new("/workspace/project-3"),
            FormatterConfig::Disabled(false),
        )
        .await;

    let manager = service.instance_manager().lock().await;
    assert_eq!(
        manager.instances_count(),
        3,
        "Should have 3 separate instances for 3 directories"
    );

    assert!(
        manager.get(Path::new("/workspace/project-1")).is_some(),
        "project-1 instance should exist"
    );
    assert!(
        manager.get(Path::new("/workspace/project-2")).is_some(),
        "project-2 instance should exist"
    );
    assert!(
        manager.get(Path::new("/workspace/project-3")).is_some(),
        "project-3 instance should exist"
    );
    assert!(
        manager.get(Path::new("/workspace/project-4")).is_none(),
        "project-4 instance should NOT exist"
    );
}

#[tokio::test]
async fn instance_state_manager_get_or_create_behavior() {
    let mut manager = InstanceStateManager::new();
    let config = FormatterConfig::Disabled(false);

    manager.get_or_create(Path::new("/dir1"), config.clone());
    assert_eq!(manager.instances_count(), 1);

    {
        let instance1 = manager.get(Path::new("/dir1")).unwrap();
        assert_eq!(*instance1.directory(), Path::new("/dir1"));
    }

    manager.get_or_create(Path::new("/dir2"), FormatterConfig::Disabled(true));
    assert_eq!(manager.instances_count(), 2);

    manager.get_or_create(Path::new("/dir1"), FormatterConfig::Disabled(true));
    assert_eq!(
        manager.instances_count(),
        2,
        "Should not create duplicate for same directory"
    );
}

#[tokio::test]
async fn instance_state_manager_remove_behavior() {
    let mut manager = InstanceStateManager::new();
    let config = FormatterConfig::Disabled(false);

    manager.get_or_create(Path::new("/dir1"), config);
    assert_eq!(manager.instances_count(), 1);

    let removed = manager.remove(Path::new("/dir1"));
    assert!(removed.is_some());
    assert_eq!(manager.instances_count(), 0);

    let removed_after = manager.remove(Path::new("/nonexistent"));
    assert!(removed_after.is_none());
}

#[tokio::test]
async fn different_directories_get_different_configs() {
    let service = FormatService::new();

    let mut config_a = HashMap::new();
    config_a.insert(
        "prettier".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["prettier".to_string(), "--write".to_string()]),
            environment: None,
            extensions: Some(vec![".js".to_string()]),
        },
    );

    let mut config_b = HashMap::new();
    config_b.insert(
        "rustfmt".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["rustfmt".to_string()]),
            environment: None,
            extensions: Some(vec![".rs".to_string()]),
        },
    );

    let _ = service
        .init(
            Path::new("/project-a"),
            FormatterConfig::Formatters(config_a),
        )
        .await;
    let _ = service
        .init(
            Path::new("/project-b"),
            FormatterConfig::Formatters(config_b),
        )
        .await;

    let manager = service.instance_manager().lock().await;

    let instance_a = manager.get(Path::new("/project-a")).unwrap();
    let instance_b = manager.get(Path::new("/project-b")).unwrap();

    match instance_a.formatter_config() {
        FormatterConfig::Formatters(map) => {
            assert!(map.contains_key("prettier"));
            assert!(!map.contains_key("rustfmt"));
        }
        _ => panic!("Expected Formatters for instance_a"),
    }

    match instance_b.formatter_config() {
        FormatterConfig::Formatters(map) => {
            assert!(!map.contains_key("prettier"));
            assert!(map.contains_key("rustfmt"));
        }
        _ => panic!("Expected Formatters for instance_b"),
    }
}
