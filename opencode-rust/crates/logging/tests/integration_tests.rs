use opencode_logging::event::{LogEvent, LogLevel};
use opencode_logging::query::LogQuery;
use opencode_logging::store::LogStore;

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

#[test]
fn test_log_store_cleanup_after_test() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("cleanup_test.db");
    let store = LogStore::new(&db_path).unwrap();

    for i in 1..=5 {
        let event = LogEvent::new(i as u64, LogLevel::Info, "test", format!("msg{}", i))
            .with_session_id("cleanup_sess");
        store.append(&event).unwrap();
    }

    drop(store);

    let store2 = LogStore::new(&db_path).unwrap();
    let results = store2
        .query(&LogQuery::new().with_session_id("cleanup_sess"))
        .unwrap();

    assert_eq!(results.len(), 5);
}