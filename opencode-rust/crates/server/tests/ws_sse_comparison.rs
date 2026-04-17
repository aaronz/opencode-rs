use opencode_core::bus::InternalEvent;
use opencode_server::routes::ws::SessionHub;
use opencode_server::streaming::{ReconnectionStore, StreamMessage};

#[tokio::test]
async fn test_ws_and_sse_share_same_stream_message_types() {
    let msg = StreamMessage::Message {
        session_id: "shared-session".to_string(),
        content: "test message".to_string(),
        role: "assistant".to_string(),
    };

    let json = serde_json::to_string(&msg).expect("should serialize");
    let parsed: StreamMessage = serde_json::from_str(&json).expect("should deserialize");

    match parsed {
        StreamMessage::Message {
            session_id,
            content,
            role,
        } => {
            assert_eq!(session_id, "shared-session");
            assert_eq!(content, "test message");
            assert_eq!(role, "assistant");
        }
        _ => panic!("expected Message variant"),
    }
}

#[tokio::test]
async fn test_ws_and_sse_reconnection_store_shared() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "ws-session",
        StreamMessage::Message {
            session_id: "ws-session".to_string(),
            content: "ws message".to_string(),
            role: "user".to_string(),
        },
    );

    store.record_message(
        "sse-session",
        StreamMessage::Message {
            session_id: "sse-session".to_string(),
            content: "sse message".to_string(),
            role: "user".to_string(),
        },
    );

    let ws_entries = store.replay_from("ws-session", 0);
    let sse_entries = store.replay_from("sse-session", 0);

    assert_eq!(ws_entries.len(), 1);
    assert_eq!(sse_entries.len(), 1);

    match &ws_entries[0].message {
        StreamMessage::Message { content, .. } => assert_eq!(content, "ws message"),
        _ => panic!("expected Message variant"),
    }

    match &sse_entries[0].message {
        StreamMessage::Message { content, .. } => assert_eq!(content, "sse message"),
        _ => panic!("expected Message variant"),
    }
}

#[tokio::test]
async fn test_ws_session_hub_supports_multiple_protocols() {
    let hub = SessionHub::new(256);
    let session_id = "multi-protocol-session";

    let _ws_client = hub.register_client(session_id, "ws-client-1").await;
    let _sse_client = hub.register_client(session_id, "sse-client-1").await;

    assert_eq!(hub.get_session_client_count(session_id).await, 2);

    let msg = StreamMessage::SessionUpdate {
        session_id: session_id.to_string(),
        status: "protocol_agnostic".to_string(),
    };

    hub.broadcast(session_id, msg).await;

    assert_eq!(hub.total_client_count().await, 2);
    assert_eq!(hub.session_count().await, 1);
}

#[test]
fn test_stream_message_all_variants_work_for_both_protocols() {
    let variants = vec![
        StreamMessage::Message {
            session_id: "test".to_string(),
            content: "hello".to_string(),
            role: "user".to_string(),
        },
        StreamMessage::ToolCall {
            session_id: "test".to_string(),
            tool_name: "read".to_string(),
            args: serde_json::json!({"path": "/test"}),
            call_id: "call-1".to_string(),
        },
        StreamMessage::ToolResult {
            session_id: "test".to_string(),
            call_id: "call-1".to_string(),
            output: "file content".to_string(),
            success: true,
        },
        StreamMessage::SessionUpdate {
            session_id: "test".to_string(),
            status: "running".to_string(),
        },
        StreamMessage::Heartbeat {
            timestamp: 1234567890,
        },
        StreamMessage::Error {
            session_id: Some("test".to_string()),
            error: "ERROR".to_string(),
            code: "ERR".to_string(),
            message: "error occurred".to_string(),
        },
        StreamMessage::Connected {
            session_id: Some("test".to_string()),
        },
    ];

    for msg in variants {
        let json = serde_json::to_string(&msg).expect("should serialize for both WS and SSE");
        let parsed: StreamMessage =
            serde_json::from_str(&json).expect("should deserialize for both WS and SSE");
        assert_eq!(serde_json::to_string(&parsed).expect("roundtrip"), json);
    }
}

#[tokio::test]
async fn test_event_bus_integration_works_for_both_protocols() {
    use opencode_core::bus::SharedEventBus;

    let event_bus = SharedEventBus::default();

    let tool_started = InternalEvent::ToolCallStarted {
        session_id: "shared-event-session".to_string(),
        tool_name: "grep".to_string(),
        call_id: "call-shared".to_string(),
    };

    event_bus.publish(tool_started.clone());

    let msg = StreamMessage::from_internal_event(&tool_started);
    assert!(msg.is_some());

    match msg.unwrap() {
        StreamMessage::ToolCall {
            tool_name, call_id, ..
        } => {
            assert_eq!(tool_name, "grep");
            assert_eq!(call_id, "call-shared");
        }
        _ => panic!("expected ToolCall variant"),
    }
}

#[test]
fn test_query_parsing_works_for_both_protocols() {
    fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
        query
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next().unwrap_or_default();
                Some((key.to_string(), value.to_string()))
            })
            .collect()
    }

    let params = parse_query("session_id=test-session&token=abc123");

    assert_eq!(params.get("session_id"), Some(&"test-session".to_string()));
    assert_eq!(params.get("token"), Some(&"abc123".to_string()));

    let empty_params = parse_query("");
    assert!(empty_params.contains_key(""));
}

#[tokio::test]
async fn test_reconnection_token_works_for_both_protocols() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "reconnect-test",
        StreamMessage::Message {
            session_id: "reconnect-test".to_string(),
            content: "first".to_string(),
            role: "user".to_string(),
        },
    );

    store.record_message(
        "reconnect-test",
        StreamMessage::Message {
            session_id: "reconnect-test".to_string(),
            content: "second".to_string(),
            role: "assistant".to_string(),
        },
    );

    let token = store.generate_token("reconnect-test", None);
    assert!(!token.is_empty());

    let sequence = store.validate_token("reconnect-test", &token);
    assert!(sequence.is_some());
    assert_eq!(sequence.unwrap(), 2);

    let replayed = store.replay_from("reconnect-test", 0);
    assert_eq!(replayed.len(), 2);

    let invalid_token_seq = store.validate_token("reconnect-test", "invalid-token");
    assert!(invalid_token_seq.is_none());
}
