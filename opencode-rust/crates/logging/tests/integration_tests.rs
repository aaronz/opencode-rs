use opencode_logging::config::LoggingConfig;
use opencode_logging::event::{LogEvent, LogFields, LogLevel};
use opencode_logging::logger::Logger;
use opencode_logging::query::LogQuery;
use opencode_logging::store::LogStore;
use opencode_logging::AgentLogger;

fn create_test_db() -> (tempfile::TempDir, LogStore) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = LogStore::new(&db_path).unwrap();
    (temp_dir, store)
}

#[test]
fn test_log_store_query_by_session_insert_10_logs_session_a_and_5_logs_session_b() {
    let (_temp_dir, store) = create_test_db();

    for i in 1..=10 {
        let event = LogEvent::new(
            i as u64,
            LogLevel::Info,
            "test.target",
            format!("session_a_log_{}", i),
        )
        .with_session_id("session_a");
        store.append(&event).unwrap();
    }

    for i in 1..=5 {
        let event = LogEvent::new(
            (10 + i) as u64,
            LogLevel::Info,
            "test.target",
            format!("session_b_log_{}", i),
        )
        .with_session_id("session_b");
        store.append(&event).unwrap();
    }

    let results = store.query(&LogQuery::new()).unwrap();
    assert_eq!(results.len(), 15, "Should have total 15 logs");
}

#[test]
fn test_log_store_query_by_session_a_returns_exactly_10_logs() {
    let (_temp_dir, store) = create_test_db();

    for i in 1..=10 {
        let event = LogEvent::new(
            i as u64,
            LogLevel::Info,
            "test.target",
            format!("session_a_log_{}", i),
        )
        .with_session_id("session_a");
        store.append(&event).unwrap();
    }

    for i in 1..=5 {
        let event = LogEvent::new(
            (10 + i) as u64,
            LogLevel::Info,
            "test.target",
            format!("session_b_log_{}", i),
        )
        .with_session_id("session_b");
        store.append(&event).unwrap();
    }

    let results = store
        .query(&LogQuery::new().with_session_id("session_a"))
        .unwrap();

    assert_eq!(results.len(), 10, "Query by session_a should return exactly 10 logs");
}

#[test]
fn test_log_store_query_by_session_b_returns_exactly_5_logs() {
    let (_temp_dir, store) = create_test_db();

    for i in 1..=10 {
        let event = LogEvent::new(
            i as u64,
            LogLevel::Info,
            "test.target",
            format!("session_a_log_{}", i),
        )
        .with_session_id("session_a");
        store.append(&event).unwrap();
    }

    for i in 1..=5 {
        let event = LogEvent::new(
            (10 + i) as u64,
            LogLevel::Info,
            "test.target",
            format!("session_b_log_{}", i),
        )
        .with_session_id("session_b");
        store.append(&event).unwrap();
    }

    let results = store
        .query(&LogQuery::new().with_session_id("session_b"))
        .unwrap();

    assert_eq!(results.len(), 5, "Query by session_b should return exactly 5 logs");
}

#[test]
fn test_log_store_query_without_session_id_returns_all_logs() {
    let (_temp_dir, store) = create_test_db();

    for i in 1..=10 {
        let event = LogEvent::new(
            i as u64,
            LogLevel::Info,
            "test.target",
            format!("session_a_log_{}", i),
        )
        .with_session_id("session_a");
        store.append(&event).unwrap();
    }

    for i in 1..=5 {
        let event = LogEvent::new(
            (10 + i) as u64,
            LogLevel::Info,
            "test.target",
            format!("session_b_log_{}", i),
        )
        .with_session_id("session_b");
        store.append(&event).unwrap();
    }

    let results = store.query(&LogQuery::new()).unwrap();

    assert_eq!(results.len(), 15, "Query without session_id should return all 15 logs");
}

#[test]
fn test_log_store_query_session_with_mixed_levels() {
    let (_temp_dir, store) = create_test_db();

    for i in 1..=3 {
        let event = LogEvent::new(i as u64, LogLevel::Error, "test", "error").with_session_id("sess_x");
        store.append(&event).unwrap();
    }
    for i in 4..=6 {
        let event = LogEvent::new(i as u64, LogLevel::Info, "test", "info").with_session_id("sess_x");
        store.append(&event).unwrap();
    }
    for i in 7..=9 {
        let event = LogEvent::new(i as u64, LogLevel::Debug, "test", "debug").with_session_id("sess_y");
        store.append(&event).unwrap();
    }

    let results = store
        .query(&LogQuery::new().with_session_id("sess_x"))
        .unwrap();

    assert_eq!(results.len(), 6);

    let error_only = store
        .query(
            &LogQuery::new()
                .with_session_id("sess_x")
                .with_level(LogLevel::Error),
        )
        .unwrap();

    assert_eq!(error_only.len(), 3);
}

#[test]
fn test_log_store_query_by_nonexistent_session_returns_empty() {
    let (_temp_dir, store) = create_test_db();

    store
        .append(&LogEvent::new(1, LogLevel::Info, "test", "msg").with_session_id("existing"))
        .unwrap();

    let results = store
        .query(&LogQuery::new().with_session_id("nonexistent"))
        .unwrap();

    assert!(results.is_empty());
}

#[test]
fn test_log_store_query_session_id_isolation_with_other_filters() {
    let (_temp_dir, store) = create_test_db();

    for i in 1..=5 {
        let event = LogEvent::new(i as u64, LogLevel::Info, "tool.read", "read").with_session_id("sess_a");
        store.append(&event).unwrap();
    }
    for i in 6..=10 {
        let event = LogEvent::new(i as u64, LogLevel::Error, "tool.read", "error").with_session_id("sess_a");
        store.append(&event).unwrap();
    }
    for i in 11..=15 {
        let event = LogEvent::new(i as u64, LogLevel::Info, "tool.read", "read").with_session_id("sess_b");
        store.append(&event).unwrap();
    }

    let results = store
        .query(
            &LogQuery::new()
                .with_session_id("sess_a")
                .with_level(LogLevel::Error),
        )
        .unwrap();

    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_log_rotation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().join("opencode.log");

    let mut config = LoggingConfig::default();
    config.file_path = Some(log_path.clone());
    config.max_file_size_mb = 1;
    config.max_rotated_files = 3;

    let logger = Logger::new(config).unwrap();

    let msg_len = 50usize;
    let msgs_to_fill = (1024 * 1024) / msg_len;
    let batch_size = 500;

    for batch_start in (0..msgs_to_fill).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, msgs_to_fill);
        let mut handles = vec![];

        for i in batch_start..batch_end {
            let logger_clone = logger.clone();
            handles.push(tokio::spawn(async move {
                logger_clone.info("test", &format!("message {:05}", i), LogFields::default());
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    let main_log_exists = log_path.exists();
    assert!(main_log_exists, "opencode.log should exist after writing");

    let rotated_1_exists = temp_dir.path().join("opencode.log.1").exists();
    assert!(rotated_1_exists, "opencode.log.1 should exist after rotation triggered");
}

#[tokio::test]
async fn test_log_rotation_creates_second_file_on_more_writes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().join("opencode.log");

    let mut config = LoggingConfig::default();
    config.file_path = Some(log_path.clone());
    config.max_file_size_mb = 1;
    config.max_rotated_files = 3;

    let logger = Logger::new(config).unwrap();

    let msg_len = 50usize;
    let msgs_to_fill = (1024 * 1024) / msg_len;
    let batch_size = 500;

    for batch_start in (0..msgs_to_fill).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, msgs_to_fill);
        let mut handles = vec![];

        for i in batch_start..batch_end {
            let logger_clone = logger.clone();
            handles.push(tokio::spawn(async move {
                logger_clone.info("test", &format!("first batch {:05}", i), LogFields::default());
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    for batch_start in (0..msgs_to_fill).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, msgs_to_fill);
        let mut handles = vec![];

        for i in batch_start..batch_end {
            let logger_clone = logger.clone();
            handles.push(tokio::spawn(async move {
                logger_clone.info("test", &format!("second batch {:05}", i), LogFields::default());
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    let rotated_1_exists = temp_dir.path().join("opencode.log.1").exists();
    let rotated_2_exists = temp_dir.path().join("opencode.log.2").exists();

    assert!(rotated_1_exists, "opencode.log.1 should exist");
    assert!(rotated_2_exists, "opencode.log.2 should exist after more writes");
}

#[tokio::test]
async fn test_log_rotation_oldest_deleted_when_max_exceeded() {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().join("opencode.log");

    let mut config = LoggingConfig::default();
    config.file_path = Some(log_path.clone());
    config.max_file_size_mb = 1;
    config.max_rotated_files = 3;

    let logger = Logger::new(config).unwrap();

    let msg_len = 50usize;
    let msgs_per_rotation = (1024 * 1024) / msg_len;
    let batches = 6;
    let batch_size = 500;

    for batch_num in 0..batches {
        for batch_start in (0..msgs_per_rotation).step_by(batch_size) {
            let batch_end = std::cmp::min(batch_start + batch_size, msgs_per_rotation);
            let mut handles = vec![];

            for i in batch_start..batch_end {
                let logger_clone = logger.clone();
                handles.push(tokio::spawn(async move {
                    logger_clone.info("test", &format!("batch {} msg {:05}", batch_num, i), LogFields::default());
                }));
            }

            for handle in handles {
                let _ = handle.await;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    let log_1_exists = temp_dir.path().join("opencode.log.1").exists();
    let log_2_exists = temp_dir.path().join("opencode.log.2").exists();
    let log_3_exists = temp_dir.path().join("opencode.log.3").exists();
    let log_4_exists = temp_dir.path().join("opencode.log.4").exists();

    assert!(log_1_exists, "opencode.log.1 should exist");
    assert!(log_2_exists, "opencode.log.2 should exist");
    assert!(log_3_exists, "opencode.log.3 should exist");
    assert!(!log_4_exists, "opencode.log.4 should NOT exist (oldest deleted)");
}