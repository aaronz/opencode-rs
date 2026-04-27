use crate::common::TempProject;
use chrono::Utc;
use opencode_core::config::ShareMode;
use opencode_core::{Message, Session};
use opencode_storage::compaction::{CompactionManager, ShareabilityVerifier};

fn create_shareable_session() -> Session {
    let mut session = Session::new();
    session.add_message(Message::user("Test message".to_string()));
    session.set_share_mode(ShareMode::Manual);
    session.generate_share_link().unwrap();
    session
}

fn create_auto_shareable_session() -> Session {
    let mut session = Session::new();
    session.add_message(Message::user("Test message".to_string()));
    session.set_share_mode(ShareMode::Auto);
    session.generate_share_link().unwrap();
    session
}

fn create_non_shareable_session() -> Session {
    let mut session = Session::new();
    session.add_message(Message::user("Non-shareable message".to_string()));
    session
}

fn create_readonly_shareable_session() -> Session {
    let mut session = Session::new();
    session.add_message(Message::user("Read-only message".to_string()));
    session.set_share_mode(ShareMode::ReadOnly);
    session.generate_share_link().unwrap();
    session
}

#[test]
fn test_compaction_shareable_output_can_be_saved_and_restored() {
    let project = TempProject::new();

    let mut session = create_shareable_session();
    for i in 0..30 {
        session.add_message(Message::assistant(format!(
            "Response {} with substantial content to ensure compaction happens",
            i
        )));
    }

    let original_share_id = session.shared_id.clone();
    let original_share_mode = session.share_mode.clone();

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(result.shareability_preserved);
    assert_eq!(session.shared_id, original_share_id);
    assert_eq!(session.share_mode, original_share_mode);

    let save_path = project.path().join("shareable_compacted.json");
    session
        .save(&save_path)
        .expect("Should save shareable compacted session");
    assert!(save_path.exists());

    let loaded = Session::load(&save_path).expect("Should load shareable compacted session");
    assert_eq!(loaded.id, session.id);
    assert_eq!(loaded.shared_id, original_share_id);
    assert_eq!(loaded.share_mode, original_share_mode);
    assert_eq!(loaded.messages.len(), session.messages.len());
}

#[test]
fn test_compaction_non_shareable_output_can_be_saved_and_restored() {
    let project = TempProject::new();

    let mut session = create_non_shareable_session();
    for i in 0..30 {
        session.add_message(Message::assistant(format!(
            "Response {} with content to trigger compaction",
            i
        )));
    }

    let original_message_count = session.messages.len();

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(!result.original_was_shareable);

    let save_path = project.path().join("non_shareable_compacted.json");
    session
        .save(&save_path)
        .expect("Should save non-shareable compacted session");
    assert!(save_path.exists());

    let loaded = Session::load(&save_path).expect("Should load non-shareable compacted session");
    assert_eq!(loaded.id, session.id);
    assert!(loaded.shared_id.is_none());
    assert!(loaded.messages.len() < original_message_count);
}

#[test]
fn test_compaction_auto_shareable_output_preserved() {
    let project = TempProject::new();

    let mut session = create_auto_shareable_session();
    for i in 0..25 {
        session.add_message(Message::assistant(format!(
            "Auto shareable response {} with content",
            i
        )));
    }

    let original_share_id = session.shared_id.clone();

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(result.shareability_preserved);
    assert_eq!(result.verification.share_mode, Some("Auto".to_string()));
    assert_eq!(session.shared_id, original_share_id);

    let save_path = project.path().join("auto_shareable_compacted.json");
    session
        .save(&save_path)
        .expect("Should save auto-shareable compacted session");

    let loaded = Session::load(&save_path).expect("Should load auto-shareable session");
    assert_eq!(loaded.shared_id, original_share_id);
    assert_eq!(loaded.share_mode, Some(ShareMode::Auto));
}

#[test]
fn test_compaction_readonly_shareable_output_preserved() {
    let project = TempProject::new();

    let mut session = create_readonly_shareable_session();
    for i in 0..25 {
        session.add_message(Message::assistant(format!(
            "Read-only response {} with content",
            i
        )));
    }

    let original_share_id = session.shared_id.clone();

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(result.shareability_preserved);

    let save_path = project.path().join("readonly_shareable_compacted.json");
    session
        .save(&save_path)
        .expect("Should save readonly-shareable compacted session");

    let loaded = Session::load(&save_path).expect("Should load readonly session");
    assert_eq!(loaded.shared_id, original_share_id);
    assert_eq!(loaded.share_mode, Some(ShareMode::ReadOnly));
}

#[test]
fn test_compaction_expired_shareable_session_becomes_non_shareable() {
    let project = TempProject::new();

    let mut session = create_shareable_session();
    session.set_share_expiry(Some(Utc::now() - chrono::Duration::hours(1)));

    for i in 0..20 {
        session.add_message(Message::assistant(format!("Response {} with content", i)));
    }

    let verification = ShareabilityVerifier::verify(&session);
    assert!(!verification.is_shareable);
    assert!(verification.is_expired);

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(!result.original_was_shareable);

    let save_path = project.path().join("expired_shareable_compacted.json");
    session
        .save(&save_path)
        .expect("Should save expired shareable compacted session");

    let loaded = Session::load(&save_path).expect("Should load expired session");
    let loaded_verification = ShareabilityVerifier::verify(&loaded);
    assert!(
        !loaded_verification.is_shareable,
        "Expired session should not be shareable"
    );
    assert!(loaded_verification.is_expired);
}

#[test]
fn test_compaction_disabled_share_mode_preserved() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::user("Test".to_string()));
    session.set_share_mode(ShareMode::Disabled);

    for i in 0..20 {
        session.add_message(Message::assistant(format!("Response {} with content", i)));
    }

    let verification = ShareabilityVerifier::verify(&session);
    assert!(!verification.is_shareable);

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert_eq!(result.verification.share_mode, Some("Disabled".to_string()));

    let save_path = project.path().join("disabled_share_compacted.json");
    session
        .save(&save_path)
        .expect("Should save disabled share compacted session");

    let loaded = Session::load(&save_path).expect("Should load disabled share session");
    assert_eq!(loaded.share_mode, Some(ShareMode::Disabled));
}

#[test]
fn test_compaction_preserves_share_id_across_save_load_cycle() {
    let project = TempProject::new();

    let mut session = create_shareable_session();
    for i in 0..25 {
        session.add_message(Message::assistant(format!(
            "Content {} to ensure compaction is triggered properly",
            i
        )));
    }

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());

    let original_share_id = session.shared_id.clone();
    let first_save_path = project.path().join("cycle_1.json");
    session
        .save(&first_save_path)
        .expect("First save should work");

    let loaded1 = Session::load(&first_save_path).expect("First load should work");
    assert_eq!(loaded1.shared_id, original_share_id);

    for i in 0..10 {
        session.add_message(Message::assistant(format!(
            "Additional response {} after first compaction",
            i
        )));
    }

    let result2 = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result2.is_ok());

    let second_save_path = project.path().join("cycle_2.json");
    session
        .save(&second_save_path)
        .expect("Second save should work");

    let loaded2 = Session::load(&second_save_path).expect("Second load should work");
    assert_eq!(loaded2.shared_id, original_share_id);
    assert!(loaded2.messages.len() < session.messages.len() + 5);
}

#[test]
fn test_compaction_shareability_verification_after_export() {
    let mut session = create_shareable_session();
    for i in 0..20 {
        session.add_message(Message::assistant(format!(
            "Response {} with content to trigger compaction",
            i
        )));
    }

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.verification.is_shareable);

    let export_json = session.export_json().expect("Export should succeed");
    assert!(!export_json.is_empty());
    assert!(serde_json::from_str::<serde_json::Value>(&export_json).is_ok());
}

#[test]
fn test_compaction_multiple_share_modes_all_preserve_correctly() {
    let project = TempProject::new();

    let share_modes = vec![ShareMode::Manual, ShareMode::Auto, ShareMode::ReadOnly];

    for share_mode in share_modes {
        let mut session = Session::new();
        session.add_message(Message::user(format!("Test for {:?}", share_mode)));
        session.set_share_mode(share_mode.clone());
        session.generate_share_link().unwrap();

        for i in 0..25 {
            session.add_message(Message::assistant(format!(
                "Response {} for {:?}",
                i, share_mode
            )));
        }

        let original_share_id = session.shared_id.clone();

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
        assert!(
            result.is_ok(),
            "Compaction should succeed for {:?}",
            share_mode
        );
        let result = result.unwrap();
        assert!(result.compaction_result.was_compacted);
        assert!(result.shareability_preserved);
        assert_eq!(
            result.verification.share_mode,
            Some(format!("{:?}", share_mode))
        );

        let save_path = project.path().join(format!("{:?}.json", share_mode));
        session.save(&save_path).expect("Save should work");

        let loaded = Session::load(&save_path).expect("Load should work");
        assert_eq!(
            loaded.shared_id, original_share_id,
            "Share ID should be preserved for {:?}",
            share_mode
        );
        assert_eq!(
            loaded.share_mode,
            Some(share_mode.clone()),
            "Share mode should be preserved for {:?}",
            share_mode
        );
    }
}

#[test]
fn test_compaction_shareability_preserved_with_system_messages() {
    let project = TempProject::new();

    let mut session = create_shareable_session();
    session.add_message(Message::system("System prompt to preserve".to_string()));
    for i in 0..25 {
        session.add_message(Message::assistant(format!(
            "Response {} with substantial content to trigger compaction",
            i
        )));
    }

    let original_share_id = session.shared_id.clone();

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(result.shareability_preserved);

    let has_system = session
        .messages
        .iter()
        .any(|m| m.content.contains("System prompt"));
    assert!(has_system, "System message should be preserved");

    let save_path = project.path().join("system_shareable_compacted.json");
    session.save(&save_path).expect("Should save");

    let loaded = Session::load(&save_path).expect("Should load");
    assert_eq!(loaded.shared_id, original_share_id);
    assert_eq!(loaded.share_mode, Some(ShareMode::Manual));
}

#[test]
fn test_compaction_edge_case_very_small_max_tokens() {
    let mut session = create_shareable_session();
    for i in 0..50 {
        session.add_message(Message::assistant(format!("Content {}", i)));
    }

    let original_share_id = session.shared_id.clone();

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 10);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(result.shareability_preserved);
    assert_eq!(session.shared_id, original_share_id);
}

#[test]
fn test_compaction_shareable_with_disabled_mode_switch() {
    let project = TempProject::new();

    let mut session = create_shareable_session();
    for i in 0..20 {
        session.add_message(Message::assistant(format!("Response {} with content", i)));
    }

    let result1 = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result1.is_ok());
    assert!(result1.unwrap().shareability_preserved);

    session.set_share_mode(ShareMode::Disabled);

    for i in 0..10 {
        session.add_message(Message::assistant(format!(
            "Additional response {} after disabling share",
            i
        )));
    }

    let result2 = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result2.is_ok());
    let result2 = result2.unwrap();
    assert!(result2.compaction_result.was_compacted);
    assert_eq!(
        result2.verification.share_mode,
        Some("Disabled".to_string())
    );

    let save_path = project.path().join("disabled_after_compact.json");
    session.save(&save_path).expect("Should save");

    let loaded = Session::load(&save_path).expect("Should load");
    assert_eq!(loaded.share_mode, Some(ShareMode::Disabled));
}

#[test]
fn test_compaction_can_compact_without_breaking_shareability() {
    let mut session = create_shareable_session();
    for i in 0..20 {
        session.add_message(Message::assistant(format!("Response {}", i)));
    }

    let can_compact = CompactionManager::can_compact_without_breaking_shareability(&session, 1000);
    assert!(can_compact);

    let non_shareable = create_non_shareable_session();
    let can_compact_non_shareable =
        CompactionManager::can_compact_without_breaking_shareability(&non_shareable, 1000);
    assert!(can_compact_non_shareable);
}

#[test]
fn test_compaction_shareability_verification_edge_cases() {
    let mut session = create_shareable_session();
    let verification = ShareabilityVerifier::verify_and_check_export(&session)
        .expect("verify_and_check_export should succeed");
    assert!(verification.is_shareable);
    assert!(verification.has_share_id);
    assert!(!verification.is_expired);
    assert!(verification.export_verified);

    session.set_share_mode(ShareMode::Disabled);
    let verification_disabled = ShareabilityVerifier::verify(&session);
    assert!(!verification_disabled.is_shareable);

    session.set_share_mode(ShareMode::Manual);
    session.set_share_expiry(Some(Utc::now() - chrono::Duration::hours(1)));
    let verification_expired = ShareabilityVerifier::verify(&session);
    assert!(!verification_expired.is_shareable);
    assert!(verification_expired.is_expired);
}

#[test]
fn test_compaction_preserves_recent_messages_count() {
    let mut session = create_shareable_session();
    for i in 0..30 {
        session.add_message(Message::user(format!("User message {}", i)));
        session.add_message(Message::assistant(format!("Assistant response {}", i)));
    }

    let original_count = session.messages.len();

    let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.compaction_result.was_compacted);
    assert!(result.shareability_preserved);

    let recent_content = session
        .messages
        .iter()
        .filter(|m| m.content.contains("message"))
        .count();
    assert!(
        recent_content >= 5,
        "Should preserve some recent messages with content"
    );
    assert!(session.messages.len() < original_count);
}
