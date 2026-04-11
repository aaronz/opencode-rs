#[cfg(test)]
mod performance_tests {
    use opencode_core::compaction::{CompactionConfig, Compactor};
    use opencode_core::{Message, Session};
    use std::time::{Duration, Instant};

    #[test]
    #[ignore]
    fn test_session_10000_messages_performance() {
        let start = Instant::now();
        let mut session = Session::new();
        for i in 0..10000 {
            session.add_message(Message::user(format!("Message {}", i)));
            if i % 2 == 0 {
                session.add_message(Message::assistant(format!("Response {}", i)));
            }
        }
        let elapsed = start.elapsed();
        println!("Creating session with 10000 messages took: {:?}", elapsed);
        assert!(
            elapsed < Duration::from_secs(10),
            "Session creation took too long: {:?}",
            elapsed
        );
    }

    #[test]
    #[ignore]
    fn test_session_save_large_performance() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("large_session.json");

        let mut session = Session::new();
        for i in 0..5000 {
            session.add_message(Message::user(format!("Message {}", i)));
            session.add_message(Message::assistant(format!("Response {}", i)));
        }

        let start = Instant::now();
        session.save(&path).unwrap();
        let save_elapsed = start.elapsed();
        println!("Saving 5000 message pairs took: {:?}", save_elapsed);
        assert!(
            save_elapsed < Duration::from_secs(300),
            "Save took too long: {:?}",
            save_elapsed
        );

        let start = Instant::now();
        let loaded = Session::load(&path).unwrap();
        let load_elapsed = start.elapsed();
        println!("Loading 5000 message pairs took: {:?}", load_elapsed);
        assert!(
            load_elapsed < Duration::from_secs(300),
            "Load took too long: {:?}",
            load_elapsed
        );
        assert_eq!(loaded.messages.len(), session.messages.len());
    }

    #[test]
    #[ignore]
    fn test_compaction_5000_messages_performance() {
        let mut session = Session::new();
        for i in 0..5000 {
            session.add_message(Message::user(format!("Message {}", i)));
            session.add_message(Message::assistant(format!("Response {}", i)));
        }

        let config = CompactionConfig::default();
        let compactor = Compactor::new(config);

        let start = Instant::now();
        let result = compactor.compact_to_fit(session.messages.clone());
        let elapsed = start.elapsed();
        println!("Compacting 5000 messages took: {:?}", elapsed);
        assert!(
            elapsed < Duration::from_secs(2),
            "Compaction took too long: {:?}",
            elapsed
        );
        println!("Original: {}, Compacted: {}", 10000, result.messages.len());
    }

    #[test]
    #[ignore]
    fn test_context_build_large_session() {
        let mut session = Session::new();
        for i in 0..1000 {
            session.add_message(Message::user(format!("Message {}", i)));
            session.add_message(Message::assistant(format!("Response {}", i)));
        }

        let start = Instant::now();
        let _context = session.build_context();
        let elapsed = start.elapsed();
        println!(
            "Building context for 1000 message pairs took: {:?}",
            elapsed
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "Context build took too long: {:?}",
            elapsed
        );
    }

    #[test]
    #[ignore]
    fn test_token_counting_10000_messages() {
        let counter = opencode_core::TokenCounter::for_model("gpt-4o");
        let messages: Vec<Message> = (0..10000)
            .map(|i| {
                Message::user(format!(
                    "Message number {} with some substantial content for token counting",
                    i
                ))
            })
            .collect();

        let start = Instant::now();
        let _total_tokens = counter.count_messages(&messages);
        let elapsed = start.elapsed();
        println!("Counting tokens for 10000 messages took: {:?}", elapsed);
        assert!(
            elapsed < Duration::from_secs(10),
            "Token counting took too long: {:?}",
            elapsed
        );
    }

    #[test]
    #[ignore]
    fn test_session_fork_performance() {
        let mut session = Session::new();
        for i in 0..1000 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        let start = Instant::now();
        for _ in 0..100 {
            let _child = session.fork(uuid::Uuid::new_v4());
        }
        let elapsed = start.elapsed();
        println!("100 session forks took: {:?}", elapsed);
        assert!(
            elapsed < Duration::from_secs(2),
            "Fork took too long: {:?}",
            elapsed
        );
    }

    #[test]
    #[ignore]
    fn test_session_undo_redo_performance() {
        let mut session = Session::new();
        for i in 0..1000 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        let start = Instant::now();
        for _ in 0..100 {
            session.undo(1).unwrap();
        }
        let elapsed = start.elapsed();
        println!("100 undos took: {:?}", elapsed);
        assert!(
            elapsed < Duration::from_secs(1),
            "Undo took too long: {:?}",
            elapsed
        );
    }

    #[test]
    #[ignore]
    fn test_session_truncate_performance() {
        let mut session = Session::new();
        for i in 0..5000 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        let start = Instant::now();
        session.truncate_for_context(100);
        let elapsed = start.elapsed();
        println!("Truncating 5000 messages to 100 tokens took: {:?}", elapsed);
        assert!(
            elapsed < Duration::from_secs(1),
            "Truncate took too long: {:?}",
            elapsed
        );
    }

    #[test]
    #[ignore]
    fn test_message_serialization_performance() {
        let messages: Vec<Message> = (0..5000)
            .map(|i| {
                Message::user(format!(
                    "Message {} with substantial content for serialization testing",
                    i
                ))
            })
            .collect();

        let start = Instant::now();
        let json = serde_json::to_string(&messages).unwrap();
        let serialize_elapsed = start.elapsed();
        println!("Serializing 5000 messages took: {:?}", serialize_elapsed);
        assert!(
            serialize_elapsed < Duration::from_secs(2),
            "Serialization took too long: {:?}",
            serialize_elapsed
        );

        let start = Instant::now();
        let _deserialized: Vec<Message> = serde_json::from_str(&json).unwrap();
        let deserialize_elapsed = start.elapsed();
        println!(
            "Deserializing 5000 messages took: {:?}",
            deserialize_elapsed
        );
        assert!(
            deserialize_elapsed < Duration::from_secs(2),
            "Deserialization took too long: {:?}",
            deserialize_elapsed
        );
    }
}
