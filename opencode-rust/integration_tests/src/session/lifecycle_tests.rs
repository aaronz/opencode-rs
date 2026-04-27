use crate::common::TempProject;
use opencode_core::checkpoint::CheckpointManager;
use opencode_core::config::ShareMode;
use opencode_core::message::Message;
use opencode_core::revert::RevertManager;
use opencode_core::session::Session;
use uuid::Uuid;

#[test]
fn test_session_lifecycle_create_fork_share_compact_revert() {
    let project = TempProject::new();

    // Step 1: Create session
    let mut session = Session::new();
    session.add_message(Message::user("Initial message".to_string()));
    session.add_message(Message::assistant("Response to initial".to_string()));

    let original_id = session.id;
    let original_message_count = session.messages.len();

    // Save original state for later verification
    let save_path = project.path().join("original_session.json");
    session
        .save(&save_path)
        .expect("Original session should save");

    // Step 2: Create checkpoint before fork
    let checkpoints_dir = project.path().join("checkpoints");
    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(10);

    let _checkpoint = checkpoint_manager
        .create(&session, "Before fork checkpoint")
        .expect("Checkpoint should be created");

    // Step 3: Fork child session
    let mut child = session.fork(Uuid::new_v4());

    // Verify fork invariants
    assert_ne!(child.id, original_id, "Child should have different ID");
    assert_eq!(
        child.parent_session_id.as_deref(),
        Some(original_id.to_string().as_str()),
        "Child should reference parent"
    );
    assert_eq!(
        child.messages.len(),
        original_message_count,
        "Child should have copied messages"
    );

    // Add messages to child (should not affect parent)
    child.add_message(Message::user("Child message 1".to_string()));
    child.add_message(Message::assistant("Child response 1".to_string()));

    let child_message_count = child.messages.len();
    assert_eq!(
        session.messages.len(),
        original_message_count,
        "Parent should not be affected"
    );

    // Step 4: Share parent session
    let mut shareable_session = Session::load(&save_path).expect("Should reload original");
    shareable_session.set_share_mode(ShareMode::Manual);
    let share_link = shareable_session
        .generate_share_link()
        .expect("Should generate share link");

    assert!(
        share_link.contains("/share/"),
        "Share link should contain /share/ path"
    );
    assert!(
        shareable_session.shared_id.is_some(),
        "Share ID should be set"
    );
    assert_eq!(
        shareable_session.share_mode,
        Some(ShareMode::Manual),
        "Share mode should be Manual"
    );

    let original_share_id = shareable_session.shared_id.clone();

    // Save shared session
    let shared_path = project.path().join("shared_session.json");
    shareable_session
        .save(&shared_path)
        .expect("Shared session should save");

    // Verify share invariants preserved after save/load
    let loaded_shareable = Session::load(&shared_path).expect("Should load shared session");
    assert_eq!(
        loaded_shareable.shared_id, original_share_id,
        "Share ID should persist"
    );
    assert_eq!(
        loaded_shareable.share_mode,
        Some(ShareMode::Manual),
        "Share mode should persist"
    );
    assert!(
        loaded_shareable.is_shared(),
        "Loaded session should still report as shared"
    );

    // Step 5: Compact parent session (context compression)
    let mut compactable_session = Session::load(&save_path).expect("Should reload for compaction");

    // Add many messages to trigger compaction
    for i in 0..30 {
        compactable_session.add_message(Message::assistant(format!(
            "Response {} with substantial content to ensure compaction happens and messages need to be compressed",
            i
        )));
    }

    let pre_compact_message_count = compactable_session.messages.len();
    assert!(
        pre_compact_message_count > 20,
        "Should have many messages before compaction"
    );

    // Perform compaction
    let compaction_result = compactable_session.compact_messages(100);
    assert!(
        compaction_result.was_compacted,
        "Compaction should have occurred"
    );
    assert!(
        compactable_session.messages.len() < pre_compact_message_count,
        "Messages should be reduced"
    );

    // Save compacted session
    let compacted_path = project.path().join("compacted_session.json");
    compactable_session
        .save(&compacted_path)
        .expect("Compacted session should save");

    // Verify compaction preserved message integrity
    let loaded_compacted = Session::load(&compacted_path).expect("Should load compacted");
    assert_eq!(
        loaded_compacted.id, original_id,
        "Session ID should be preserved"
    );
    assert!(
        loaded_compacted.messages.len() < pre_compact_message_count,
        "Message count should be reduced"
    );

    // Verify recent messages preserved
    let has_recent_content = loaded_compacted
        .messages
        .iter()
        .any(|m| m.content.contains("Response 2") || m.content.contains("Response 5"));
    assert!(
        has_recent_content,
        "Recent messages should be preserved after compaction"
    );

    // Step 6: Revert to checkpoint
    let mut revert_manager = RevertManager::new(10);
    let revert_point = revert_manager.create_point(
        original_message_count,
        "Before fork revert point".to_string(),
    );

    let mut session_to_revert = Session::load(&save_path).expect("Should load session for revert");

    // Add some messages before reverting
    session_to_revert.add_message(Message::user("This message should be removed".to_string()));
    session_to_revert.add_message(Message::assistant("This too".to_string()));

    assert!(
        session_to_revert.messages.len() > original_message_count,
        "Should have extra messages"
    );

    // Perform revert
    revert_manager
        .revert_to(&mut session_to_revert, &revert_point.id)
        .expect("Revert should succeed");

    // Verify revert restored correct state
    assert_eq!(
        session_to_revert.messages.len(),
        original_message_count,
        "Message count should be restored to original"
    );

    // Verify message content preserved
    assert_eq!(session_to_revert.messages[0].content, "Initial message");
    assert_eq!(session_to_revert.messages[1].content, "Response to initial");

    // Step 7: Verify session state integrity after all operations

    // 7a: Child session integrity
    let child_save_path = project.path().join("child_session.json");
    child.save(&child_save_path).expect("Child should save");
    let loaded_child = Session::load(&child_save_path).expect("Child should load");

    assert_eq!(loaded_child.id, child.id, "Child ID preserved");
    assert_eq!(
        loaded_child.parent_session_id.as_deref(),
        Some(original_id.to_string().as_str()),
        "Parent reference preserved"
    );
    assert_eq!(
        loaded_child.messages.len(),
        child_message_count,
        "Child message count preserved"
    );

    // Verify child has its own messages plus parent messages
    assert!(loaded_child
        .messages
        .iter()
        .any(|m| m.content == "Child message 1"));
    assert!(loaded_child
        .messages
        .iter()
        .any(|m| m.content == "Initial message"));

    // 7b: Shared session integrity
    let loaded_shared = Session::load(&shared_path).expect("Should reload shared");
    assert!(loaded_shared.is_shared(), "Should still be shared");
    assert_eq!(
        loaded_shared.get_share_id(),
        original_share_id.as_deref(),
        "Share ID preserved"
    );

    // 7c: Compacted session integrity
    let loaded_compacted2 = Session::load(&compacted_path).expect("Should reload compacted");
    assert!(
        loaded_compacted2.messages.len() < pre_compact_message_count,
        "Compaction result preserved"
    );

    // 7d: Reverted session integrity
    let reverted_save_path = project.path().join("reverted_session.json");
    session_to_revert
        .save(&reverted_save_path)
        .expect("Reverted session should save");
    let loaded_reverted = Session::load(&reverted_save_path).expect("Reverted session should load");

    assert_eq!(
        loaded_reverted.messages.len(),
        original_message_count,
        "Reverted state preserved"
    );
    assert_eq!(
        loaded_reverted.id, original_id,
        "Session ID preserved after revert"
    );

    // Step 8: Verify checkpoint functionality
    let checkpoints = checkpoint_manager
        .list(&original_id)
        .expect("Should list checkpoints");
    assert!(!checkpoints.is_empty(), "Checkpoints should exist");

    let checkpointed_session = checkpoint_manager
        .load(&original_id, 0)
        .expect("Should load checkpoint");
    assert_eq!(
        checkpointed_session.id, original_id,
        "Checkpoint preserves session ID"
    );
    assert_eq!(
        checkpointed_session.messages.len(),
        original_message_count,
        "Checkpoint preserves message count"
    );
}

#[test]
fn test_session_lifecycle_fork_preserves_message_history() {
    let project = TempProject::new();

    // Create session with varied message types
    let mut session = Session::new();
    session.add_message(Message::system("System prompt".to_string()));
    session.add_message(Message::user("User message 1".to_string()));
    session.add_message(Message::assistant("Assistant response 1".to_string()));
    session.add_message(Message::user("User message 2".to_string()));
    session.add_message(Message::assistant("Assistant response 2".to_string()));

    let parent_message_count = session.messages.len();
    let parent_id = session.id;

    // Fork at specific message index
    let mut child = session
        .fork_at_message(2)
        .expect("Should fork at message 2");

    // Verify child has correct messages (0, 1, 2)
    assert_eq!(child.messages.len(), 3, "Child should have 3 messages");
    assert_eq!(child.messages[0].content, "System prompt");
    assert_eq!(child.messages[1].content, "User message 1");
    assert_eq!(child.messages[2].content, "Assistant response 1");

    // Verify parent unchanged
    assert_eq!(
        session.messages.len(),
        parent_message_count,
        "Parent should have 5 messages"
    );

    // Add messages to child
    child.add_message(Message::assistant("Child continuation".to_string()));

    // Fork again from child (grandchild)
    let grandchild = child.fork(Uuid::new_v4());

    // Verify lineage
    assert_eq!(
        grandchild.parent_session_id.as_deref(),
        Some(child.id.to_string().as_str()),
        "Grandchild should reference child as parent"
    );

    // Verify grandchild has correct lineage_path
    let expected_lineage = Some(parent_id.to_string());
    assert_eq!(child.compute_lineage_path(), expected_lineage);

    // Verify grandchild has child messages
    assert!(grandchild
        .messages
        .iter()
        .any(|m| m.content.contains("Child continuation")));

    // Save and verify persistence
    let child_path = project.path().join("child_session.json");
    child.save(&child_path).expect("Child should save");

    let loaded_child = Session::load(&child_path).expect("Child should load");
    assert_eq!(
        loaded_child.parent_session_id.as_deref(),
        Some(parent_id.to_string().as_str())
    );
    assert!(loaded_child
        .messages
        .iter()
        .any(|m| m.content.contains("Child continuation")));
}

#[test]
fn test_session_lifecycle_share_preserves_across_operations() {
    let project = TempProject::new();

    // Create and share session
    let mut session = Session::new();
    session.add_message(Message::user("Share me".to_string()));
    session.set_share_mode(ShareMode::Auto);
    let _share_link = session.generate_share_link().expect("Should generate link");
    let original_share_id = session.shared_id.clone();

    assert!(session.is_shared());

    // Fork should NOT inherit shared_id
    let mut child = session.fork(Uuid::new_v4());
    assert!(
        child.shared_id.is_none(),
        "Forked session should not inherit shared_id"
    );
    assert!(!child.is_shared(), "Forked session should not be shared");

    // But child can generate its own share link
    child.set_share_mode(ShareMode::Manual);
    let _child_share_link = child
        .generate_share_link()
        .expect("Child should generate share link");
    assert_ne!(
        child.shared_id, original_share_id,
        "Child should have different share_id"
    );

    // Save original session
    let original_path = project.path().join("shared_original.json");
    session.save(&original_path).expect("Should save");

    // Compact original (should preserve shareability)
    for i in 0..20 {
        session.add_message(Message::assistant(format!("Content {}", i)));
    }

    session.compact_messages(100);

    let shared_path = project.path().join("shared_compacted.json");
    session.save(&shared_path).expect("Should save compacted");

    // Load and verify share still works
    let mut loaded = Session::load(&shared_path).expect("Should load");
    assert_eq!(
        loaded.shared_id, original_share_id,
        "Share ID preserved after compaction"
    );
    assert!(loaded.is_shared(), "Should still report as shared");

    // Disable sharing and verify
    loaded.set_share_mode(ShareMode::Disabled);
    assert!(!loaded.is_shared(), "Should not be shared when disabled");
    assert!(loaded.shared_id.is_none(), "shared_id should be cleared");
}

#[test]
fn test_session_lifecycle_compact_preserves_recent_messages() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::user("First message".to_string()));

    // Add many messages
    for i in 0..50 {
        session.add_message(Message::assistant(format!("Important response {}", i)));
    }

    let original_count = session.messages.len();
    assert!(original_count > 30, "Should have many messages");

    // Compact aggressively
    let result = session.compact_messages(50); // Very small token budget

    assert!(result.was_compacted, "Should be compacted");

    // Verify some recent messages are preserved
    let has_recent = session
        .messages
        .iter()
        .any(|m| m.content.contains("Important response"));
    assert!(has_recent, "Recent messages should be preserved");

    // Save and load
    let save_path = project.path().join("compacted.json");
    session.save(&save_path).expect("Should save");

    let loaded = Session::load(&save_path).expect("Should load");
    assert_eq!(
        loaded.messages.len(),
        session.messages.len(),
        "Message count preserved"
    );

    // Verify compaction result persists
    let recent_content = loaded
        .messages
        .iter()
        .filter(|m| m.content.contains("Important response"))
        .count();
    assert!(
        recent_content > 0,
        "Recent messages should be in loaded session"
    );
}

#[test]
fn test_session_lifecycle_revert_with_checkpoints() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::user("Message 1".to_string()));

    // Record state after each step (accounting for auto-compaction)
    let state_after_msg1 = session.messages.len();

    session.add_message(Message::assistant("Response 1".to_string()));
    session.add_message(Message::user("Message 2".to_string()));

    let _state_after_msg2 = session.messages.len();

    // Create checkpoint at current state
    let checkpoints_dir = project.path().join("checkpoints");
    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(10);

    let cp1 = checkpoint_manager
        .create(&session, "Checkpoint 1")
        .expect("cp1");
    assert_eq!(cp1.sequence_number, 0);

    // Add more messages (auto-compaction may occur)
    session.add_message(Message::assistant("Response 2".to_string()));
    session.add_message(Message::user("Message 3".to_string()));

    let _cp2 = checkpoint_manager
        .create(&session, "Checkpoint 2")
        .expect("cp2");
    let state_after_cp2 = session.messages.len();

    session.add_message(Message::assistant("Response 3".to_string()));
    session.add_message(Message::user("Message 4".to_string()));

    // Create revert point at current state
    let mut revert_manager = RevertManager::new(5);
    let revert_point =
        revert_manager.create_point(session.messages.len(), "Revert point".to_string());

    // Add more messages
    session.add_message(Message::assistant("Response 4".to_string()));

    let pre_revert_count = session.messages.len();
    assert!(pre_revert_count >= state_after_cp2);

    // Revert to point
    revert_manager
        .revert_to(&mut session, &revert_point.id)
        .expect("Should revert");

    // Verify we have content from before the revert
    assert!(session.messages.iter().any(|m| m.content == "Message 1"));

    // Revert to checkpoint (restore full session state from cp1)
    let restored = checkpoint_manager
        .load(&session.id, 0)
        .expect("Should load cp1");

    // Verify checkpoint has the message we expect
    assert!(restored.messages.iter().any(|m| m.content == "Message 1"));

    // Can load checkpoint at different points
    let _restored_cp2 = checkpoint_manager
        .load(&session.id, 1)
        .expect("Should load cp2");
    assert!(session.messages.len() >= state_after_msg1);
}

#[test]
fn test_session_lifecycle_full_integration_sequence() {
    let project = TempProject::new();

    // Phase 1: Create session with history
    let mut session = Session::new();
    session.add_message(Message::system("You are a helpful assistant".to_string()));
    session.add_message(Message::user("Start a new task".to_string()));
    session.add_message(Message::assistant("I'll help you with that.".to_string()));
    session.add_message(Message::user("Add more details".to_string()));
    session.add_message(Message::assistant("Here's more detail.".to_string()));

    let initial_id = session.id;

    // Phase 2: Create checkpoint
    let checkpoints_dir = project.path().join("checkpoints");
    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(5);

    let _checkpoint = checkpoint_manager
        .create(&session, "Initial checkpoint")
        .unwrap();

    // Phase 3: Fork for exploration
    let mut exploration = session.fork_at_message(2).expect("Fork at response 1");
    exploration.add_message(Message::user("Explore alternative".to_string()));
    exploration.add_message(Message::assistant("Alternative approach".to_string()));

    // Phase 4: Share main session
    let main_path = project.path().join("main_session.json");
    session.save(&main_path).expect("Save main");

    let mut shareable = Session::load(&main_path).expect("Load for sharing");
    shareable.set_share_mode(ShareMode::Manual);
    let _share_link = shareable.generate_share_link().expect("Generate link");
    let _original_share_id = shareable.shared_id.clone();
    shareable.save(&main_path).expect("Save shared");

    // Phase 5: Compact main session
    let mut main = Session::load(&main_path).expect("Load for compaction");
    for i in 0..25 {
        main.add_message(Message::assistant(format!("Additional response {}", i)));
    }

    let pre_compact_count = main.messages.len();
    let _compact_result = main.compact_messages(100);

    main.save(&main_path).expect("Save compacted");

    // Phase 6: Revert to checkpoint
    let mut revert_manager = RevertManager::new(5);
    let revert_point = revert_manager.create_point(3, "Before compaction".to_string());

    let mut to_revert = Session::load(&main_path).expect("Load for revert");
    to_revert.add_message(Message::user("Extra message".to_string()));

    revert_manager
        .revert_to(&mut to_revert, &revert_point.id)
        .expect("Revert");

    // Verify all operations worked
    let checkpoints = checkpoint_manager
        .list(&initial_id)
        .expect("List checkpoints");
    assert!(!checkpoints.is_empty());

    let loaded_main = Session::load(&main_path).expect("Load main");
    assert_eq!(loaded_main.id, initial_id);

    let loaded_exploration = {
        let exploration_path = project.path().join("exploration.json");
        exploration
            .save(&exploration_path)
            .expect("Save exploration");
        Session::load(&exploration_path).expect("Load exploration")
    };

    assert_eq!(
        loaded_exploration.parent_session_id.as_deref(),
        Some(initial_id.to_string().as_str()),
        "Exploration should reference main as parent"
    );

    // Final verification: all sessions load correctly
    assert!(loaded_main.messages.len() <= pre_compact_count);
    assert!(to_revert.messages.len() <= pre_compact_count);
}
