mod common;

use opencode_core::config::ShareMode;
use opencode_server::routes::share::{ShareOperation, ShareServer, ShortShareConfig};

#[tokio::test]
async fn test_read_only_sharing_mode_works() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let session_id = "test-session-123".to_string();
    let link = server
        .create_short_link_with_mode(
            session_id.clone(),
            ShareMode::ReadOnly,
            None,
            None,
        )
        .await;

    assert_eq!(link.share_mode, ShareMode::ReadOnly);
    assert!(link.allowed_operations.contains(&ShareOperation::Read));
    assert!(!link.allowed_operations.contains(&ShareOperation::Write));
    assert!(!link.allowed_operations.contains(&ShareOperation::Delete));
    assert!(!link.allowed_operations.contains(&ShareOperation::Fork));
}

#[tokio::test]
async fn test_collaborative_mode_supports_concurrent_access() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let session_id = "test-session-456".to_string();
    let link = server
        .create_short_link_with_mode(
            session_id.clone(),
            ShareMode::Collaborative,
            None,
            None,
        )
        .await;

    assert_eq!(link.share_mode, ShareMode::Collaborative);
    assert!(link.allowed_operations.contains(&ShareOperation::Read));
    assert!(link.allowed_operations.contains(&ShareOperation::Write));
    assert!(link.allowed_operations.contains(&ShareOperation::Fork));
    assert!(!link.allowed_operations.contains(&ShareOperation::Delete));
}

#[tokio::test]
async fn test_controlled_mode_access_control() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let session_id = "test-session-789".to_string();
    let link = server
        .create_short_link_with_mode(
            session_id.clone(),
            ShareMode::Controlled,
            None,
            None,
        )
        .await;

    assert_eq!(link.share_mode, ShareMode::Controlled);
    assert!(link.allowed_operations.contains(&ShareOperation::Read));
    assert!(!link.allowed_operations.contains(&ShareOperation::Write));
}

#[tokio::test]
async fn test_access_control_is_properly_enforced_read_only() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let short_code = {
        let link = server
            .create_short_link_with_mode(
                "test-session-access".to_string(),
                ShareMode::ReadOnly,
                None,
                None,
            )
            .await;
        link.short_code.clone()
    };

    assert!(server.check_permission(&short_code, ShareOperation::Read).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Write).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Delete).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Fork).await);
}

#[tokio::test]
async fn test_access_control_is_properly_enforced_collaborative() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let short_code = {
        let link = server
            .create_short_link_with_mode(
                "test-session-collab".to_string(),
                ShareMode::Collaborative,
                None,
                None,
            )
            .await;
        link.short_code.clone()
    };

    assert!(server.check_permission(&short_code, ShareOperation::Read).await);
    assert!(server.check_permission(&short_code, ShareOperation::Write).await);
    assert!(server.check_permission(&short_code, ShareOperation::Fork).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Delete).await);
}

#[tokio::test]
async fn test_access_control_is_properly_enforced_controlled() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let short_code = {
        let link = server
            .create_short_link_with_mode(
                "test-session-controlled".to_string(),
                ShareMode::Controlled,
                None,
                None,
            )
            .await;
        link.short_code.clone()
    };

    assert!(server.check_permission(&short_code, ShareOperation::Read).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Write).await);
}

#[tokio::test]
async fn test_disabled_mode_denies_all_operations() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let short_code = {
        let link = server
            .create_short_link_with_mode(
                "test-session-disabled".to_string(),
                ShareMode::Disabled,
                None,
                None,
            )
            .await;
        link.short_code.clone()
    };

    assert!(!server.check_permission(&short_code, ShareOperation::Read).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Write).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Delete).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Fork).await);
}

#[tokio::test]
async fn test_manual_mode_allows_read_only() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let short_code = {
        let link = server
            .create_short_link_with_mode(
                "test-session-manual".to_string(),
                ShareMode::Manual,
                None,
                None,
            )
            .await;
        link.short_code.clone()
    };

    assert!(server.check_permission(&short_code, ShareOperation::Read).await);
    assert!(!server.check_permission(&short_code, ShareOperation::Write).await);
}

#[tokio::test]
async fn test_update_share_mode_changes_permissions() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let short_code = {
        let link = server
            .create_short_link_with_mode(
                "test-session-update".to_string(),
                ShareMode::ReadOnly,
                None,
                None,
            )
            .await;
        link.short_code.clone()
    };

    assert!(!server.check_permission(&short_code, ShareOperation::Write).await);

    let updated = server.update_share_mode(&short_code, ShareMode::Collaborative).await;
    assert!(updated);

    assert!(server.check_permission(&short_code, ShareOperation::Write).await);
    assert!(server.check_permission(&short_code, ShareOperation::Fork).await);
}

#[tokio::test]
async fn test_add_allowed_operation_extends_permissions() {
    let config = ShortShareConfig::default();
    let server = ShareServer::new(config);

    let short_code = {
        let link = server
            .create_short_link_with_mode(
                "test-session-add-op".to_string(),
                ShareMode::ReadOnly,
                None,
                None,
            )
            .await;
        link.short_code.clone()
    };

    assert!(!server.check_permission(&short_code, ShareOperation::Write).await);

    let added = server.add_allowed_operation(&short_code, ShareOperation::Write).await;
    assert!(added);

    assert!(server.check_permission(&short_code, ShareOperation::Write).await);
}

#[tokio::test]
async fn test_sharing_modes_serialization() {
    use opencode_core::config::ShareMode;
    
    let modes = vec![
        ShareMode::Manual,
        ShareMode::Auto,
        ShareMode::Disabled,
        ShareMode::ReadOnly,
        ShareMode::Collaborative,
        ShareMode::Controlled,
    ];

    for mode in modes {
        let json = serde_json::to_string(&mode).unwrap();
        let loaded: ShareMode = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, mode);
    }
}
