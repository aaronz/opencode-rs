use opencode_server::routes::ws::SessionHub;
use opencode_server::streaming::{ReconnectionStore, StreamMessage};
use serde_json::json;

#[test]
fn test_stream_message_serialization() {
    let msg = StreamMessage::Message {
        session_id: "test-session".to_string(),
        content: "Hello, WebSocket!".to_string(),
        role: "assistant".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"message""#));
    assert!(json_str.contains("test-session"));
    assert!(json_str.contains("Hello, WebSocket!"));
}

#[test]
fn test_stream_message_tool_call_serialization() {
    let msg = StreamMessage::ToolCall {
        session_id: "test-session".to_string(),
        tool_name: "read".to_string(),
        args: json!({"path": "/test/file.txt"}),
        call_id: "call-123".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"tool_call""#));
    assert!(json_str.contains("read"));
    assert!(json_str.contains("call-123"));
}

#[test]
fn test_stream_message_tool_result_serialization() {
    let msg = StreamMessage::ToolResult {
        session_id: "test-session".to_string(),
        call_id: "call-123".to_string(),
        output: "file contents here".to_string(),
        success: true,
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"tool_result""#));
    assert!(json_str.contains("call-123"));
    assert!(json_str.contains(r#""success":true"#));
}

#[test]
fn test_stream_message_session_update_serialization() {
    let msg = StreamMessage::SessionUpdate {
        session_id: "test-session".to_string(),
        status: "processing".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"session_update""#));
    assert!(json_str.contains("processing"));
}

#[test]
fn test_stream_message_error_serialization() {
    let msg = StreamMessage::Error {
        session_id: Some("test-session".to_string()),
        error: "TOOL_NOT_FOUND".to_string(),
        code: "TOOL_NOT_FOUND".to_string(),
        message: "The tool 'foo' was not found".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"error""#));
    assert!(json_str.contains("TOOL_NOT_FOUND"));
}

#[test]
fn test_stream_message_heartbeat_serialization() {
    let msg = StreamMessage::Heartbeat {
        timestamp: 1713200000,
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"heartbeat""#));
    assert!(json_str.contains("1713200000"));
}

#[test]
fn test_stream_message_connected_serialization() {
    let msg = StreamMessage::Connected {
        session_id: Some("test-session".to_string()),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"connected""#));
}

#[test]
fn test_stream_message_session_id_extraction() {
    let msg = StreamMessage::Message {
        session_id: "session-abc".to_string(),
        content: "test".to_string(),
        role: "user".to_string(),
    };

    assert_eq!(msg.session_id(), Some("session-abc"));

    let error_msg = StreamMessage::Error {
        session_id: Some("session-xyz".to_string()),
        error: "err".to_string(),
        code: "err".to_string(),
        message: "error".to_string(),
    };

    assert_eq!(error_msg.session_id(), Some("session-xyz"));

    let heartbeat = StreamMessage::Heartbeat { timestamp: 0 };
    assert_eq!(heartbeat.session_id(), None);
}

#[test]
fn test_reconnection_store_record_and_replay() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "session-replay-test",
        StreamMessage::Message {
            session_id: "session-replay-test".to_string(),
            content: "message 1".to_string(),
            role: "assistant".to_string(),
        },
    );

    store.record_message(
        "session-replay-test",
        StreamMessage::Message {
            session_id: "session-replay-test".to_string(),
            content: "message 2".to_string(),
            role: "assistant".to_string(),
        },
    );

    let entries = store.replay_from("session-replay-test", 0);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].sequence, 1);
    assert_eq!(entries[1].sequence, 2);
}

#[test]
fn test_reconnection_store_token_generation_and_validation() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "session-token-test",
        StreamMessage::Connected { session_id: None },
    );

    let token = store.generate_token("session-token-test", None);
    assert!(!token.is_empty());

    let validated_seq = store.validate_token("session-token-test", &token);
    assert!(validated_seq.is_some());
    assert_eq!(validated_seq.unwrap(), 1);

    let invalid = store.validate_token("session-token-test", "invalid-token");
    assert!(invalid.is_none());
}

#[test]
fn test_reconnection_store_different_sessions_independent() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "session-A",
        StreamMessage::Message {
            session_id: "session-A".to_string(),
            content: "A's message".to_string(),
            role: "assistant".to_string(),
        },
    );

    store.record_message(
        "session-B",
        StreamMessage::Message {
            session_id: "session-B".to_string(),
            content: "B's message".to_string(),
            role: "assistant".to_string(),
        },
    );

    let token_a = store.generate_token("session-A", None);
    let token_b = store.generate_token("session-B", None);

    assert!(store.validate_token("session-A", &token_a).is_some());
    assert!(store.validate_token("session-B", &token_b).is_some());
    assert!(store.validate_token("session-A", &token_b).is_none());
    assert!(store.validate_token("session-B", &token_a).is_none());
}

#[test]
fn test_reconnection_store_replay_limit() {
    let store = ReconnectionStore::new(3);

    for i in 0..5 {
        store.record_message(
            "session-limit",
            StreamMessage::Message {
                session_id: "session-limit".to_string(),
                content: format!("message {}", i),
                role: "assistant".to_string(),
            },
        );
    }

    let entries = store.replay_from("session-limit", 0);
    assert_eq!(entries.len(), 3);
    assert!(entries[0].message.session_id().is_some());
}

#[tokio::test]
async fn test_session_hub_register_and_unregister_client() {
    let hub = SessionHub::new(256);

    let session_id = "test-session-hub";
    let client_id = "test-client";

    assert_eq!(hub.get_session_client_count(session_id).await, 0);

    let _receiver = hub.register_client(session_id, client_id).await;

    assert_eq!(hub.get_session_client_count(session_id).await, 1);
    assert_eq!(hub.total_client_count().await, 1);
    assert_eq!(hub.session_count().await, 1);

    hub.unregister_client(session_id, client_id).await;

    assert_eq!(hub.get_session_client_count(session_id).await, 0);
    assert_eq!(hub.total_client_count().await, 0);
    assert_eq!(hub.session_count().await, 0);
}

#[tokio::test]
async fn test_session_hub_multiple_clients_same_session() {
    let hub = SessionHub::new(256);

    let session_id = "shared-session";

    let _r1 = hub.register_client(session_id, "client-1").await;
    let _r2 = hub.register_client(session_id, "client-2").await;
    let _r3 = hub.register_client(session_id, "client-3").await;

    assert_eq!(hub.get_session_client_count(session_id).await, 3);
    assert_eq!(hub.total_client_count().await, 3);
    assert_eq!(hub.session_count().await, 1);
}

#[tokio::test]
async fn test_session_hub_broadcast_to_session() {
    let hub = SessionHub::new(256);

    let session_id = "broadcast-session";
    let mut receiver1 = hub.register_client(session_id, "client-1").await;
    let mut receiver2 = hub.register_client(session_id, "client-2").await;

    let msg = StreamMessage::Message {
        session_id: session_id.to_string(),
        content: "Broadcast message".to_string(),
        role: "assistant".to_string(),
    };

    hub.broadcast(session_id, msg).await;

    let received1 = receiver1.recv().await.expect("client-1 should receive");
    let received2 = receiver2.recv().await.expect("client-2 should receive");

    match (&received1, &received2) {
        (
            StreamMessage::Message { content: c1, .. },
            StreamMessage::Message { content: c2, .. },
        ) => {
            assert_eq!(c1, "Broadcast message");
            assert_eq!(c2, "Broadcast message");
        }
        _ => panic!("Expected Message variant"),
    }
}

#[tokio::test]
async fn test_session_hub_broadcast_all() {
    let hub = SessionHub::new(256);

    let mut r1 = hub.register_client("session-1", "client-1").await;
    let mut r2 = hub.register_client("session-2", "client-2").await;

    let msg = StreamMessage::SessionUpdate {
        session_id: "all".to_string(),
        status: "broadcast_all_test".to_string(),
    };

    hub.broadcast_all(msg).await;

    let received1 = r1.recv().await.expect("client-1 should receive");
    let received2 = r2.recv().await.expect("client-2 should receive");

    match (&received1, &received2) {
        (
            StreamMessage::SessionUpdate { status: s1, .. },
            StreamMessage::SessionUpdate { status: s2, .. },
        ) => {
            assert_eq!(s1, "broadcast_all_test");
            assert_eq!(s2, "broadcast_all_test");
        }
        _ => panic!("Expected SessionUpdate variant"),
    }
}

#[tokio::test]
async fn test_session_hub_client_disconnect_remaining_clients() {
    let hub = SessionHub::new(256);

    let session_id = "disconnect-test";
    let mut receiver1 = hub.register_client(session_id, "client-1").await;
    let mut receiver2 = hub.register_client(session_id, "client-2").await;

    hub.unregister_client(session_id, "client-1").await;

    let msg = StreamMessage::Message {
        session_id: session_id.to_string(),
        content: "After disconnect".to_string(),
        role: "assistant".to_string(),
    };

    hub.broadcast(session_id, msg).await;

    let received2 = receiver2.recv().await.expect("client-2 should receive");
    match received2 {
        StreamMessage::Message { content, .. } => {
            assert_eq!(content, "After disconnect");
        }
        _ => panic!("Expected Message variant"),
    }

    let err = receiver1.try_recv();
    assert!(err.is_err(), "disconnected client should not receive");
}

#[test]
fn test_connection_type_variants() {
    use opencode_server::streaming::ConnectionType;

    let ws_type = ConnectionType::WebSocket;
    let sse_type = ConnectionType::Sse;

    assert_eq!(format!("{:?}", ws_type), "WebSocket");
    assert_eq!(format!("{:?}", sse_type), "Sse");
}

#[test]
fn test_stream_message_from_tool_call_to_result_sequence() {
    let tool_call = StreamMessage::ToolCall {
        session_id: "seq-test".to_string(),
        tool_name: "read".to_string(),
        args: json!({"path": "/test.txt"}),
        call_id: "call-seq-1".to_string(),
    };

    let tool_result = StreamMessage::ToolResult {
        session_id: "seq-test".to_string(),
        call_id: "call-seq-1".to_string(),
        output: "file contents".to_string(),
        success: true,
    };

    let call_json = serde_json::to_string(&tool_call).expect("should serialize");
    let result_json = serde_json::to_string(&tool_result).expect("should serialize");

    assert!(call_json.contains("tool_call"));
    assert!(call_json.contains("read"));
    assert!(result_json.contains("tool_result"));
    assert!(result_json.contains("call-seq-1"));
}

#[test]
fn test_stream_message_token_sequence_for_streaming() {
    let tokens = vec![
        StreamMessage::Message {
            session_id: "stream-test".to_string(),
            content: "He".to_string(),
            role: "assistant".to_string(),
        },
        StreamMessage::Message {
            session_id: "stream-test".to_string(),
            content: "llo".to_string(),
            role: "assistant".to_string(),
        },
        StreamMessage::Message {
            session_id: "stream-test".to_string(),
            content: " world".to_string(),
            role: "assistant".to_string(),
        },
    ];

    for (i, token) in tokens.iter().enumerate() {
        let json = serde_json::to_string(token).expect("should serialize");
        assert!(json.contains("stream-test"));
        if i == 0 {
            assert!(json.contains("He"));
        }
    }
}

#[test]
fn test_error_stream_message_with_null_session_id() {
    let msg = StreamMessage::Error {
        session_id: None,
        error: "INTERNAL_ERROR".to_string(),
        code: "INTERNAL_ERROR".to_string(),
        message: "An internal error occurred".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"error""#));
    assert!(json_str.contains("INTERNAL_ERROR"));

    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse");
    assert!(parsed["session_id"].is_null());
}

#[test]
fn test_connected_stream_message_without_session_id() {
    let msg = StreamMessage::Connected { session_id: None };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"connected""#));

    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse");
    assert!(parsed["session_id"].is_null());
}

#[test]
fn test_stream_message_all_variants_have_type_field() {
    let variants = vec![
        StreamMessage::Message {
            session_id: "s".to_string(),
            content: "c".to_string(),
            role: "r".to_string(),
        },
        StreamMessage::ToolCall {
            session_id: "s".to_string(),
            tool_name: "t".to_string(),
            args: json!({}),
            call_id: "c".to_string(),
        },
        StreamMessage::ToolResult {
            session_id: "s".to_string(),
            call_id: "c".to_string(),
            output: "o".to_string(),
            success: true,
        },
        StreamMessage::SessionUpdate {
            session_id: "s".to_string(),
            status: "st".to_string(),
        },
        StreamMessage::Heartbeat { timestamp: 0 },
        StreamMessage::Error {
            session_id: None,
            error: "e".to_string(),
            code: "e".to_string(),
            message: "m".to_string(),
        },
        StreamMessage::Connected { session_id: None },
    ];

    for variant in variants {
        let json_str = serde_json::to_string(&variant).expect("should serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse");
        assert!(
            parsed.get("type").is_some(),
            "All StreamMessage variants should have 'type' field"
        );
    }
}

#[tokio::test]
async fn test_session_hub_multiple_sessions_independent() {
    let hub = SessionHub::new(256);

    let mut r1 = hub.register_client("session-A", "client-A1").await;
    let _r2 = hub.register_client("session-B", "client-B1").await;
    let _r3 = hub.register_client("session-A", "client-A2").await;

    assert_eq!(hub.session_count().await, 2);
    assert_eq!(hub.get_session_client_count("session-A").await, 2);
    assert_eq!(hub.get_session_client_count("session-B").await, 1);

    let msg_a = StreamMessage::Message {
        session_id: "session-A".to_string(),
        content: "A only".to_string(),
        role: "assistant".to_string(),
    };
    hub.broadcast("session-A", msg_a).await;

    let received = r1.recv().await.expect("client-A1 should receive");
    match received {
        StreamMessage::Message { content, .. } => {
            assert_eq!(content, "A only");
        }
        _ => panic!("Expected Message variant"),
    }

    let msg_b = StreamMessage::SessionUpdate {
        session_id: "session-B".to_string(),
        status: "B status".to_string(),
    };
    hub.broadcast("session-B", msg_b).await;

    hub.unregister_client("session-A", "client-A1").await;
    hub.unregister_client("session-A", "client-A2").await;
    hub.unregister_client("session-B", "client-B1").await;

    assert_eq!(hub.session_count().await, 0);
}

#[test]
fn test_reconnection_store_replay_from_specific_sequence() {
    let store = ReconnectionStore::new(100);

    for i in 1..=5 {
        store.record_message(
            "replay-seq",
            StreamMessage::Message {
                session_id: "replay-seq".to_string(),
                content: format!("msg-{}", i),
                role: "assistant".to_string(),
            },
        );
    }

    let entries_after_2 = store.replay_from("replay-seq", 2);
    assert_eq!(entries_after_2.len(), 3);
    assert_eq!(entries_after_2[0].sequence, 3);
    assert_eq!(entries_after_2[1].sequence, 4);
    assert_eq!(entries_after_2[2].sequence, 5);

    let entries_after_4 = store.replay_from("replay-seq", 4);
    assert_eq!(entries_after_4.len(), 1);
    assert_eq!(entries_after_4[0].sequence, 5);

    let entries_after_5 = store.replay_from("replay-seq", 5);
    assert!(entries_after_5.is_empty());
}

#[test]
fn test_reconnection_store_empty_for_unknown_session() {
    let store = ReconnectionStore::new(100);

    let entries = store.replay_from("unknown-session", 0);
    assert!(entries.is_empty());

    let token = store.generate_token("unknown-session", None);
    let validated = store.validate_token("unknown-session", &token);
    assert!(validated.is_some());
    assert_eq!(validated.unwrap(), 0);
}
