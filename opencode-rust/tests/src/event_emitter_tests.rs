use opencode_agent::{AgentEvent, AgentEventEmitter, BroadcastEventEmitter};
use serde_json::json;

#[test]
fn test_agent_event_emitter_trait_is_object_safe() {
    fn assert_object_safe(_: &dyn AgentEventEmitter) {}
    let emitter = BroadcastEventEmitter::new(100);
    assert_object_safe(&emitter);
}

#[test]
fn test_broadcast_event_emitter_implements_trait() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let _: &dyn AgentEventEmitter = &emitter;
}

#[tokio::test]
async fn test_agent_event_emitter_subscribers_receive_events() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::with_default_capacity();
    let mut receiver = emitter.subscribe();

    let event = AgentEvent::tool_call("read", json!({"path": "/tmp/test"}));
    emitter.emit(event.clone());

    let received = receiver.recv().await.unwrap();
    match received {
        AgentEvent::ToolCall { tool, params } => {
            assert_eq!(tool, "read");
            assert_eq!(params["path"], "/tmp/test");
        }
        _ => panic!("Expected ToolCall variant"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_multiple_subscribers_all_receive() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let mut receiver1 = emitter.subscribe();
    let mut receiver2 = emitter.subscribe();
    let mut receiver3 = emitter.subscribe();

    let event = AgentEvent::thinking(" 分析中...");
    emitter.emit(event.clone());

    let received1 = receiver1.recv().await.unwrap();
    let received2 = receiver2.recv().await.unwrap();
    let received3 = receiver3.recv().await.unwrap();

    match (&received1, &received2, &received3) {
        (
            AgentEvent::Thinking { content: c1 },
            AgentEvent::Thinking { content: c2 },
            AgentEvent::Thinking { content: c3 },
        ) => {
            assert_eq!(c1, " 分析中...");
            assert_eq!(c2, " 分析中...");
            assert_eq!(c3, " 分析中...");
        }
        _ => panic!("Expected Thinking variant"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_error_event() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let mut receiver = emitter.subscribe();

    let event = AgentEvent::error("Tool execution failed");
    emitter.emit(event);

    let received = receiver.recv().await.unwrap();
    match received {
        AgentEvent::Error { error } => {
            assert_eq!(error, "Tool execution failed");
        }
        _ => panic!("Expected Error variant"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_complete_event() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let mut receiver = emitter.subscribe();

    let event = AgentEvent::complete("Task finished successfully");
    emitter.emit(event);

    let received = receiver.recv().await.unwrap();
    match received {
        AgentEvent::Complete { summary } => {
            assert_eq!(summary, "Task finished successfully");
        }
        _ => panic!("Expected Complete variant"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_token_event() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let mut receiver = emitter.subscribe();

    let event = AgentEvent::token("Hello, world!");
    emitter.emit(event);

    let received = receiver.recv().await.unwrap();
    match received {
        AgentEvent::Token { content } => {
            assert_eq!(content, "Hello, world!");
        }
        _ => panic!("Expected Token variant"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_message_event() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let mut receiver = emitter.subscribe();

    let event = AgentEvent::message("assistant", "How can I help you?");
    emitter.emit(event);

    let received = receiver.recv().await.unwrap();
    match received {
        AgentEvent::Message { role, content } => {
            assert_eq!(role, "assistant");
            assert_eq!(content, "How can I help you?");
        }
        _ => panic!("Expected Message variant"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_tool_result_event() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let mut receiver = emitter.subscribe();

    let event = AgentEvent::tool_result(
        "read",
        json!({"content": "file contents here", "lines": 10}),
    );
    emitter.emit(event);

    let received = receiver.recv().await.unwrap();
    match received {
        AgentEvent::ToolResult { tool, result } => {
            assert_eq!(tool, "read");
            assert_eq!(result["content"], "file contents here");
            assert_eq!(result["lines"], 10);
        }
        _ => panic!("Expected ToolResult variant"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_sequence_of_events() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);
    let mut receiver = emitter.subscribe();

    let events = vec![
        AgentEvent::tool_call("grep", json!({"pattern": "fn", "path": "/src"})),
        AgentEvent::tool_result("grep", json!({"matches": 5, "files": ["a.rs", "b.rs"]})),
        AgentEvent::token("Found "),
        AgentEvent::token("5 "),
        AgentEvent::token("matches"),
        AgentEvent::message("assistant", "I found 5 matches in 2 files."),
        AgentEvent::complete("Search completed"),
    ];

    for event in events {
        emitter.emit(event);
    }

    let mut received_events = Vec::new();
    while let Ok(event) = receiver.recv().await {
        received_events.push(event);
        if received_events.len() == 7 {
            break;
        }
    }

    assert_eq!(received_events.len(), 7);

    match &received_events[0] {
        AgentEvent::ToolCall { tool, params } => {
            assert_eq!(tool, "grep");
            assert_eq!(params["pattern"], "fn");
        }
        _ => panic!("First event should be ToolCall"),
    }

    match &received_events[6] {
        AgentEvent::Complete { summary } => {
            assert_eq!(summary, "Search completed");
        }
        _ => panic!("Last event should be Complete"),
    }
}

#[tokio::test]
async fn test_agent_event_emitter_broadcast_channel_capacity() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(10);
    let mut receiver = emitter.subscribe();

    for i in 0..5 {
        let event = AgentEvent::token(format!("token-{}", i));
        emitter.emit(event);
    }

    let mut count = 0;
    while receiver.recv().await.is_ok() {
        count += 1;
        if count >= 5 {
            break;
        }
    }
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_broadcast_event_emitter_clone_preserves_sender() {
    let emitter1 = BroadcastEventEmitter::new(100);
    let emitter2 = emitter1.clone();

    let _receiver1 = emitter1.subscribe();
    let mut receiver2 = emitter2.subscribe();

    emitter1.emit(AgentEvent::thinking("test"));

    let received = receiver2.recv().await.unwrap();
    match received {
        AgentEvent::Thinking { content } => {
            assert_eq!(content, "test");
        }
        _ => panic!("Expected Thinking variant"),
    }

    drop(emitter1);

    emitter2.emit(AgentEvent::complete("after drop"));

    let received = receiver2.recv().await.unwrap();
    match received {
        AgentEvent::Complete { summary } => {
            assert_eq!(summary, "after drop");
        }
        _ => panic!("Expected Complete variant"),
    }

    drop(emitter2);
    drop(receiver2);
}

#[tokio::test]
async fn test_agent_event_emitter_send_sync() {
    let emitter: BroadcastEventEmitter = BroadcastEventEmitter::new(100);

    fn assert_send_sync<T: Send + Sync>(_: T) {}
    assert_send_sync(emitter);

    let emitter2: BroadcastEventEmitter = BroadcastEventEmitter::with_default_capacity();
    assert_send_sync(emitter2);
}

#[test]
fn test_broadcast_event_emitter_default_capacity() {
    let emitter = BroadcastEventEmitter::default();
    let mut receiver = emitter.subscribe();
    emitter.emit(AgentEvent::token("test"));

    let rt = tokio::runtime::Runtime::new().unwrap();
    let received = rt.block_on(async { receiver.recv().await });
    assert!(received.is_ok());
}
