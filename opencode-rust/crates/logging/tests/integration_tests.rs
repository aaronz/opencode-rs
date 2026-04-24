use opencode_logging::config::LoggingConfig;
use opencode_logging::event::{
    LogEvent, LogFields, LogLevel, ReasoningLog, SanitizedValue, ToolConsideration,
    ToolExecutionLog, ToolResult,
};
use opencode_logging::logger::Logger;
use opencode_logging::query::LogQuery;
use opencode_logging::sanitizer::Sanitizer;
use opencode_logging::store::{
    LogStore, ReasoningLogQuery, ReasoningLogStore, ToolExecutionLogStore,
};
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

    assert_eq!(
        results.len(),
        10,
        "Query by session_a should return exactly 10 logs"
    );
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

    assert_eq!(
        results.len(),
        5,
        "Query by session_b should return exactly 5 logs"
    );
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

    assert_eq!(
        results.len(),
        15,
        "Query without session_id should return all 15 logs"
    );
}

#[test]
fn test_log_store_query_session_with_mixed_levels() {
    let (_temp_dir, store) = create_test_db();

    for i in 1..=3 {
        let event =
            LogEvent::new(i as u64, LogLevel::Error, "test", "error").with_session_id("sess_x");
        store.append(&event).unwrap();
    }
    for i in 4..=6 {
        let event =
            LogEvent::new(i as u64, LogLevel::Info, "test", "info").with_session_id("sess_x");
        store.append(&event).unwrap();
    }
    for i in 7..=9 {
        let event =
            LogEvent::new(i as u64, LogLevel::Debug, "test", "debug").with_session_id("sess_y");
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
        let event =
            LogEvent::new(i as u64, LogLevel::Info, "tool.read", "read").with_session_id("sess_a");
        store.append(&event).unwrap();
    }
    for i in 6..=10 {
        let event = LogEvent::new(i as u64, LogLevel::Error, "tool.read", "error")
            .with_session_id("sess_a");
        store.append(&event).unwrap();
    }
    for i in 11..=15 {
        let event =
            LogEvent::new(i as u64, LogLevel::Info, "tool.read", "read").with_session_id("sess_b");
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

    let config = LoggingConfig {
        file_path: Some(log_path.clone()),
        max_file_size_mb: 1,
        max_rotated_files: 3,
        ..Default::default()
    };

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
    assert!(
        rotated_1_exists,
        "opencode.log.1 should exist after rotation triggered"
    );
}

#[tokio::test]
async fn test_log_rotation_creates_second_file_on_more_writes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().join("opencode.log");

    let config = LoggingConfig {
        file_path: Some(log_path.clone()),
        max_file_size_mb: 1,
        max_rotated_files: 3,
        ..Default::default()
    };

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
                logger_clone.info(
                    "test",
                    &format!("first batch {:05}", i),
                    LogFields::default(),
                );
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
                logger_clone.info(
                    "test",
                    &format!("second batch {:05}", i),
                    LogFields::default(),
                );
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
    assert!(
        rotated_2_exists,
        "opencode.log.2 should exist after more writes"
    );
}

#[tokio::test]
async fn test_log_rotation_oldest_deleted_when_max_exceeded() {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().join("opencode.log");

    let config = LoggingConfig {
        file_path: Some(log_path.clone()),
        max_file_size_mb: 1,
        max_rotated_files: 3,
        ..Default::default()
    };

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
                    logger_clone.info(
                        "test",
                        &format!("batch {} msg {:05}", batch_num, i),
                        LogFields::default(),
                    );
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
    assert!(
        !log_4_exists,
        "opencode.log.4 should NOT exist (oldest deleted)"
    );
}

fn create_reasoning_test_db() -> (tempfile::TempDir, ReasoningLogStore) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("reasoning.db");
    let store = ReasoningLogStore::new(&db_path).unwrap();
    (temp_dir, store)
}

#[test]
fn test_reasoning_log_persistence_create_with_3_tool_considerations() {
    let (_temp_dir, store) = create_reasoning_test_db();

    let reasoning = ReasoningLog {
        step_id: "step_001".to_string(),
        session_id: "sess_reasoning_001".to_string(),
        timestamp: chrono::Utc::now(),
        prompt: "What tool should I use to read a file?".to_string(),
        response: "I should use the read tool".to_string(),
        tools_considered: vec![
            ToolConsideration {
                tool_name: "read".to_string(),
                reason: "Most appropriate for reading file contents".to_string(),
                selected: true,
            },
            ToolConsideration {
                tool_name: "grep".to_string(),
                reason: "Can read but grep is for searching".to_string(),
                selected: false,
            },
            ToolConsideration {
                tool_name: "bash".to_string(),
                reason: "Too complex for simple file reading".to_string(),
                selected: false,
            },
        ],
        decision: "Selected read tool as the most appropriate for the task".to_string(),
        prompt_tokens: 1500,
        completion_tokens: 100,
        latency_ms: 250,
    };

    store.append(&reasoning).unwrap();

    let retrieved = store.get("step_001").unwrap().unwrap();
    assert_eq!(retrieved.tools_considered.len(), 3);
}

#[test]
fn test_reasoning_log_persistence_store_in_logstore() {
    let (_temp_dir, store) = create_reasoning_test_db();

    let reasoning = ReasoningLog {
        step_id: "step_002".to_string(),
        session_id: "sess_reasoning_002".to_string(),
        timestamp: chrono::Utc::now(),
        prompt: "Analyze the codebase".to_string(),
        response: "Found 5 issues".to_string(),
        tools_considered: vec![
            ToolConsideration {
                tool_name: "grep".to_string(),
                reason: "Good for searching patterns".to_string(),
                selected: true,
            },
            ToolConsideration {
                tool_name: "read".to_string(),
                reason: "Good for reading files".to_string(),
                selected: false,
            },
        ],
        decision: "Using grep to search for issues".to_string(),
        prompt_tokens: 2000,
        completion_tokens: 150,
        latency_ms: 300,
    };

    let result = store.append(&reasoning);
    assert!(result.is_ok(), "Store append should succeed");
}

#[test]
fn test_reasoning_log_persistence_query_returns_reasoning_log() {
    let (_temp_dir, store) = create_reasoning_test_db();

    let reasoning = ReasoningLog {
        step_id: "step_003".to_string(),
        session_id: "sess_query_test".to_string(),
        timestamp: chrono::Utc::now(),
        prompt: "Find all test files".to_string(),
        response: "Found 3 test files".to_string(),
        tools_considered: vec![ToolConsideration {
            tool_name: "grep".to_string(),
            reason: "Good pattern matching".to_string(),
            selected: true,
        }],
        decision: "Using grep with pattern *_test.rs".to_string(),
        prompt_tokens: 1800,
        completion_tokens: 80,
        latency_ms: 200,
    };

    store.append(&reasoning).unwrap();

    let results = store.get_by_session("sess_query_test").unwrap();
    assert_eq!(results.len(), 1);

    let retrieved = &results[0];
    assert_eq!(retrieved.step_id, "step_003");
    assert_eq!(retrieved.session_id, "sess_query_test");
}

#[test]
fn test_reasoning_log_persistence_verify_tools_considered_preserved_with_selected_flags() {
    let (_temp_dir, store) = create_reasoning_test_db();

    let reasoning = ReasoningLog {
        step_id: "step_004".to_string(),
        session_id: "sess_tools_flag_test".to_string(),
        timestamp: chrono::Utc::now(),
        prompt: "Which tool is best?".to_string(),
        response: "The bash tool".to_string(),
        tools_considered: vec![
            ToolConsideration {
                tool_name: "read".to_string(),
                reason: "For reading".to_string(),
                selected: false,
            },
            ToolConsideration {
                tool_name: "bash".to_string(),
                reason: "Can execute commands".to_string(),
                selected: true,
            },
            ToolConsideration {
                tool_name: "write".to_string(),
                reason: "For writing".to_string(),
                selected: false,
            },
        ],
        decision: "Selected bash".to_string(),
        prompt_tokens: 500,
        completion_tokens: 30,
        latency_ms: 100,
    };

    store.append(&reasoning).unwrap();

    let retrieved = store.get("step_004").unwrap().unwrap();
    let selected_tools: Vec<_> = retrieved
        .tools_considered
        .iter()
        .filter(|t| t.selected)
        .collect();
    let unselected_tools: Vec<_> = retrieved
        .tools_considered
        .iter()
        .filter(|t| !t.selected)
        .collect();

    assert_eq!(selected_tools.len(), 1);
    assert_eq!(selected_tools[0].tool_name, "bash");
    assert_eq!(unselected_tools.len(), 2);
    assert!(unselected_tools.iter().all(|t| t.tool_name != "bash"));
}

#[test]
fn test_reasoning_log_persistence_verify_prompt_tokens_completion_tokens_latency_ms_correct() {
    let (_temp_dir, store) = create_reasoning_test_db();

    let reasoning = ReasoningLog {
        step_id: "step_005".to_string(),
        session_id: "sess_token_test".to_string(),
        timestamp: chrono::Utc::now(),
        prompt: "Count lines in file".to_string(),
        response: "The file has 100 lines".to_string(),
        tools_considered: vec![],
        decision: "No tools needed".to_string(),
        prompt_tokens: 12345,
        completion_tokens: 67890,
        latency_ms: 999,
    };

    store.append(&reasoning).unwrap();

    let retrieved = store.get("step_005").unwrap().unwrap();
    assert_eq!(retrieved.prompt_tokens, 12345);
    assert_eq!(retrieved.completion_tokens, 67890);
    assert_eq!(retrieved.latency_ms, 999);
}

#[test]
fn test_reasoning_log_persistence_multiple_sessions_isolated() {
    let (_temp_dir, store) = create_reasoning_test_db();

    for i in 1..=3 {
        let reasoning = ReasoningLog {
            step_id: format!("step_session_a_{}", i),
            session_id: "sess_a".to_string(),
            timestamp: chrono::Utc::now(),
            prompt: format!("Prompt A {}", i),
            response: format!("Response A {}", i),
            tools_considered: vec![],
            decision: format!("Decision A {}", i),
            prompt_tokens: 100 * i,
            completion_tokens: 50 * i,
            latency_ms: 10 * i,
        };
        store.append(&reasoning).unwrap();
    }

    for i in 1..=2 {
        let reasoning = ReasoningLog {
            step_id: format!("step_session_b_{}", i),
            session_id: "sess_b".to_string(),
            timestamp: chrono::Utc::now(),
            prompt: format!("Prompt B {}", i),
            response: format!("Response B {}", i),
            tools_considered: vec![],
            decision: format!("Decision B {}", i),
            prompt_tokens: 200 * i,
            completion_tokens: 75 * i,
            latency_ms: 20 * i,
        };
        store.append(&reasoning).unwrap();
    }

    let results_a = store.get_by_session("sess_a").unwrap();
    let results_b = store.get_by_session("sess_b").unwrap();

    assert_eq!(results_a.len(), 3);
    assert_eq!(results_b.len(), 2);

    let all_results = store
        .query(&ReasoningLogQuery::for_session("sess_a"))
        .unwrap();
    assert_eq!(all_results.len(), 3);
}

fn create_tool_exec_test_db() -> (tempfile::TempDir, ToolExecutionLogStore) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("tool_exec.db");
    let store = ToolExecutionLogStore::new(&db_path).unwrap();
    (temp_dir, store)
}

#[test]
fn tool_sanitization() {
    let (_temp_dir, store) = create_tool_exec_test_db();

    let sanitizer = Sanitizer::new();

    let mut params = std::collections::HashMap::new();
    params.insert("api_key".to_string(), serde_json::json!("secret123"));
    params.insert("password".to_string(), serde_json::json!("pass456"));
    params.insert(
        "file_path".to_string(),
        serde_json::json!("/some/safe/path.txt"),
    );
    params.insert("action".to_string(), serde_json::json!("read"));

    let sanitized_params = sanitizer.sanitize_params(&params);

    let tool_log = ToolExecutionLog {
        execution_id: "exec_sanitization_test_001".to_string(),
        session_id: "sess_sanitization_test".to_string(),
        tool_name: "test_tool".to_string(),
        timestamp: chrono::Utc::now(),
        parameters: sanitized_params,
        result: ToolResult {
            success: true,
            message: "Success".to_string(),
            output: None,
        },
        latency_ms: 100,
        error: None,
    };

    store.append(&tool_log).unwrap();

    let retrieved = store.get("exec_sanitization_test_001").unwrap().unwrap();

    assert!(matches!(retrieved.parameters, SanitizedValue::Nested(_)));

    if let SanitizedValue::Nested(nested) = &retrieved.parameters {
        assert!(matches!(
            nested.get("api_key"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("password"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("file_path"),
            Some(SanitizedValue::Safe(_))
        ));
        assert!(matches!(
            nested.get("action"),
            Some(SanitizedValue::Safe(_))
        ));

        if let SanitizedValue::Redacted(api_key_redacted) = nested.get("api_key").unwrap() {
            assert_eq!(api_key_redacted, "[REDACTED]");
        }
        if let SanitizedValue::Redacted(password_redacted) = nested.get("password").unwrap() {
            assert_eq!(password_redacted, "[REDACTED]");
        }
        if let SanitizedValue::Safe(file_path) = nested.get("file_path").unwrap() {
            assert_eq!(file_path, "/some/safe/path.txt");
        }
        if let SanitizedValue::Safe(action) = nested.get("action").unwrap() {
            assert_eq!(action, "read");
        }
    }
}

#[test]
fn tool_sanitization_query_by_session() {
    let (_temp_dir, store) = create_tool_exec_test_db();

    let sanitizer = Sanitizer::new();

    let mut params1 = std::collections::HashMap::new();
    params1.insert("api_key".to_string(), serde_json::json!("key1"));
    params1.insert("data".to_string(), serde_json::json!("value1"));
    let sanitized1 = sanitizer.sanitize_params(&params1);

    let tool_log1 = ToolExecutionLog {
        execution_id: "exec_session_test_001".to_string(),
        session_id: "sess_api_keys".to_string(),
        tool_name: "api_tool".to_string(),
        timestamp: chrono::Utc::now(),
        parameters: sanitized1,
        result: ToolResult {
            success: true,
            message: "OK".to_string(),
            output: None,
        },
        latency_ms: 50,
        error: None,
    };
    store.append(&tool_log1).unwrap();

    let mut params2 = std::collections::HashMap::new();
    params2.insert("password".to_string(), serde_json::json!("secret2"));
    params2.insert("name".to_string(), serde_json::json!("test2"));
    let sanitized2 = sanitizer.sanitize_params(&params2);

    let tool_log2 = ToolExecutionLog {
        execution_id: "exec_session_test_002".to_string(),
        session_id: "sess_api_keys".to_string(),
        tool_name: "auth_tool".to_string(),
        timestamp: chrono::Utc::now(),
        parameters: sanitized2,
        result: ToolResult {
            success: true,
            message: "OK".to_string(),
            output: None,
        },
        latency_ms: 30,
        error: None,
    };
    store.append(&tool_log2).unwrap();

    let mut params3 = std::collections::HashMap::new();
    params3.insert("token".to_string(), serde_json::json!("abc123"));
    params3.insert("action".to_string(), serde_json::json!("login"));
    let sanitized3 = sanitizer.sanitize_params(&params3);

    let tool_log3 = ToolExecutionLog {
        execution_id: "exec_session_test_003".to_string(),
        session_id: "sess_other".to_string(),
        tool_name: "login_tool".to_string(),
        timestamp: chrono::Utc::now(),
        parameters: sanitized3,
        result: ToolResult {
            success: true,
            message: "OK".to_string(),
            output: None,
        },
        latency_ms: 40,
        error: None,
    };
    store.append(&tool_log3).unwrap();

    let sess_api_results = store.get_by_session("sess_api_keys").unwrap();
    assert_eq!(sess_api_results.len(), 2);

    for log in &sess_api_results {
        if let SanitizedValue::Nested(nested) = &log.parameters {
            if log.tool_name == "api_tool" {
                assert!(matches!(
                    nested.get("api_key"),
                    Some(SanitizedValue::Redacted(_))
                ));
                assert!(matches!(nested.get("data"), Some(SanitizedValue::Safe(_))));
            }
            if log.tool_name == "auth_tool" {
                assert!(matches!(
                    nested.get("password"),
                    Some(SanitizedValue::Redacted(_))
                ));
                assert!(matches!(nested.get("name"), Some(SanitizedValue::Safe(_))));
            }
        }
    }
}

#[test]
fn tool_sanitization_all_secret_patterns() {
    let (_temp_dir, store) = create_tool_exec_test_db();

    let sanitizer = Sanitizer::new();

    let mut params = std::collections::HashMap::new();
    params.insert("api_key".to_string(), serde_json::json!("sk-12345"));
    params.insert("password".to_string(), serde_json::json!("my_pass"));
    params.insert("token".to_string(), serde_json::json!("tok_abc"));
    params.insert("secret".to_string(), serde_json::json!("my_secret"));
    params.insert("authorization".to_string(), serde_json::json!("Bearer xyz"));
    params.insert("private_key".to_string(), serde_json::json!("key_data"));
    params.insert("access_key".to_string(), serde_json::json!("access_data"));
    params.insert("username".to_string(), serde_json::json!("john"));
    params.insert("safe_field".to_string(), serde_json::json!("visible_value"));

    let sanitized_params = sanitizer.sanitize_params(&params);

    let tool_log = ToolExecutionLog {
        execution_id: "exec_all_patterns_001".to_string(),
        session_id: "sess_patterns".to_string(),
        tool_name: "multi_tool".to_string(),
        timestamp: chrono::Utc::now(),
        parameters: sanitized_params,
        result: ToolResult {
            success: true,
            message: "OK".to_string(),
            output: None,
        },
        latency_ms: 60,
        error: None,
    };

    store.append(&tool_log).unwrap();

    let retrieved = store.get("exec_all_patterns_001").unwrap().unwrap();

    if let SanitizedValue::Nested(nested) = &retrieved.parameters {
        assert!(matches!(
            nested.get("api_key"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("password"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("token"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("secret"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("authorization"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("private_key"),
            Some(SanitizedValue::Redacted(_))
        ));
        assert!(matches!(
            nested.get("access_key"),
            Some(SanitizedValue::Redacted(_))
        ));

        assert!(matches!(
            nested.get("username"),
            Some(SanitizedValue::Safe(_))
        ));
        assert!(matches!(
            nested.get("safe_field"),
            Some(SanitizedValue::Safe(_))
        ));
    }
}
