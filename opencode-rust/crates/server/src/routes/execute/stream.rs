//! SSE streaming implementation for execute endpoint.
//!
//! Provides SSE (Server-Sent Events) formatting and streaming for agent execution events.
//! Follows the actix-web SSE patterns using futures::StreamExt for async streaming.

use std::pin::Pin;

use actix_web::web;
use futures::Stream;
use serde::Serialize;

use super::types::ExecuteEvent;

/// SSE formatted event ready for transmission.
#[derive(Debug, Clone)]
pub struct SseFormattedEvent {
    /// Optional event ID for client reconnection support.
    pub id: Option<u64>,
    /// The SSE event type name (e.g., "tool_call", "message").
    pub event_type: &'static str,
    /// The serialized JSON data payload.
    pub data: String,
}

#[allow(dead_code)]
impl SseFormattedEvent {
    /// Creates a new SSE formatted event.
    pub(crate) fn new(event_type: &'static str, data: String, id: Option<u64>) -> Self {
        Self {
            id,
            event_type,
            data,
        }
    }

    /// Formats the event as an SSE string according to the Server-Sent Events spec.
    ///
    /// Format:
    /// ```text
    /// id: <event_id>\n
    /// event: <event_type>\n
    /// data: <payload>\n\n
    /// ```
    ///
    /// or without id:
    /// ```text
    /// event: <event_type>\n
    /// data: <payload>\n\n
    /// ```
    pub(crate) fn to_sse_string(&self) -> String {
        let mut output = String::new();

        if let Some(id) = self.id {
            output.push_str(&format!("id: {}\n", id));
        }

        output.push_str(&format!("event: {}\n", self.event_type));
        output.push_str(&format!("data: {}\n\n", self.data));

        output
    }
}

/// Extracts the event type name from an ExecuteEvent variant.
fn event_type_name(event: &ExecuteEvent) -> &'static str {
    match event {
        ExecuteEvent::ToolCall { .. } => "tool_call",
        ExecuteEvent::ToolResult { .. } => "tool_result",
        ExecuteEvent::Message { .. } => "message",
        ExecuteEvent::Token { .. } => "token",
        ExecuteEvent::Error { .. } => "error",
        ExecuteEvent::Complete { .. } => "complete",
    }
}

/// Serialize event data to JSON, handling serialization errors gracefully.
fn serialize_event<T: Serialize>(event: &T) -> String {
    serde_json::to_string(event).unwrap_or_else(|_| {
        serde_json::json!({
            "type": "error",
            "code": "SERIALIZATION_ERROR",
            "message": "failed to serialize event payload"
        })
        .to_string()
    })
}

/// Formats an ExecuteEvent into SSE format with optional event ID.
///
/// # Arguments
/// * `event` - The ExecuteEvent to format
/// * `id` - Optional event ID for reconnection support
///
/// # Returns
/// A formatted SSE string ready for transmission.
pub(crate) fn format_sse_event(event: &ExecuteEvent, id: Option<u64>) -> String {
    let event_type = event_type_name(event);
    let data = serialize_event(event);

    let mut output = String::new();

    if let Some(event_id) = id {
        output.push_str(&format!("id: {}\n", event_id));
    }

    output.push_str(&format!("event: {}\n", event_type));
    output.push_str(&format!("data: {}\n\n", data));

    output
}

/// Formats an ExecuteEvent into an SseFormattedEvent struct.
///
/// This is useful when you need to inspect the components before serialization.
#[allow(dead_code)]
pub(crate) fn format_sse_event_struct(event: &ExecuteEvent, id: Option<u64>) -> SseFormattedEvent {
    let event_type = event_type_name(event);
    let data = serialize_event(event);

    SseFormattedEvent::new(event_type, data, id)
}

/// Creates a streaming response from an iterator of ExecuteEvents.
///
/// Returns a `Box<str>` for each event formatted as SSE.
/// This is useful for integration with actix-web's `HttpResponse::streaming()`.
pub(crate) fn execute_event_stream(
    events: impl IntoIterator<Item = ExecuteEvent>,
) -> Pin<Box<dyn Stream<Item = std::result::Result<web::Bytes, actix_web::Error>> + Send>> {
    use futures::stream::{self, StreamExt};

    // Collect into Vec to ensure Send - required for the stream to be Send
    let events: Vec<ExecuteEvent> = events.into_iter().collect();
    let stream = stream::iter(events.into_iter().map(|event| {
        let formatted = format_sse_event(&event, None);
        Ok(web::Bytes::from(formatted))
    }));

    Box::pin(stream)
}

/// Creates a streaming response from a stream of ExecuteEvents.
///
/// This version accepts a `Stream` of `ExecuteEvent` for lazy/eager evaluation.
#[allow(dead_code)]
pub(crate) fn execute_event_stream_from_stream<S>(
    stream: S,
) -> Pin<Box<dyn Stream<Item = std::result::Result<web::Bytes, actix_web::Error>> + Send>>
where
    S: Stream<Item = ExecuteEvent> + Send + 'static,
{
    use futures::StreamExt;

    let stream = stream.map(|event| {
        let formatted = format_sse_event(&event, None);
        Ok(web::Bytes::from(formatted))
    });

    Box::pin(stream)
}

/// Creates an SSE stream with event IDs from a stream of ExecuteEvents.
///
/// Each event is assigned a sequential ID starting from 1.
#[allow(dead_code)]
pub(crate) fn execute_event_stream_with_ids<S>(
    stream: S,
) -> Pin<Box<dyn Stream<Item = std::result::Result<web::Bytes, actix_web::Error>> + Send>>
where
    S: Stream<Item = ExecuteEvent> + Send + 'static,
{
    use futures::StreamExt;

    let stream = stream.enumerate().map(|(idx, event)| {
        let id = idx as u64 + 1; // SSE IDs are typically 1-indexed
        let formatted = format_sse_event(&event, Some(id));
        Ok(web::Bytes::from(formatted))
    });

    Box::pin(stream)
}

/// Validates that a string is valid SSE format.
///
/// Returns true if the string follows the SSE specification:
/// - Each line is formatted as "field: value\n"

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::execute::types::{ExecuteEvent, ExecuteMode, ExecuteRequest};
    use futures::StreamExt;

    /// Validates SSE format. Only used in tests.
    fn is_valid_sse_format(sse_string: &str) -> bool {
        // Must have at least one "data:" line
        let mut has_data = false;

        for line in sse_string.lines() {
            // Empty line terminates the event
            if line.is_empty() {
                break;
            }

            // Each line must start with a valid field name followed by ": "
            if let Some(colon_pos) = line.find(':') {
                let field = &line[..colon_pos];
                let remainder = &line[colon_pos + 1..];

                if remainder.is_empty() {
                    // Empty remainder is only valid for "data" field
                    if field != "data" {
                        return false;
                    }
                } else if !remainder.starts_with(' ') {
                    // Non-empty remainder must start with space
                    return false;
                }

                // Valid field names according to SSE spec
                match field {
                    "id" | "event" | "data" | "retry" => {
                        if field == "data" {
                            has_data = true;
                        }
                    }
                    _ => return false, // Unknown field
                }
            } else {
                // No colon found - invalid format
                return false;
            }
        }

        has_data
    }

    // ============== SSE Event Formatting Tests ==============

    #[test]
    fn test_format_sse_event_tool_call() {
        let event = ExecuteEvent::tool_call(
            "read",
            serde_json::json!({"path": "/test/file.txt"}),
            "call-123",
        );
        let formatted = format_sse_event(&event, Some(1));

        // Verify SSE format
        assert!(formatted.starts_with("id: 1\n"));
        assert!(formatted.contains("event: tool_call\n"));
        assert!(formatted.contains(r#""event":"tool_call""#));
        assert!(formatted.contains(r#""tool":"read""#));
        assert!(formatted.contains(r#""call_id":"call-123""#));
        assert!(formatted.ends_with("\n\n"));
    }

    #[test]
    fn test_format_sse_event_tool_result() {
        let event = ExecuteEvent::tool_result(
            "read",
            serde_json::json!({"content": "file contents"}),
            "call-123",
            true,
        );
        let formatted = format_sse_event(&event, Some(2));

        assert!(formatted.starts_with("id: 2\n"));
        assert!(formatted.contains("event: tool_result\n"));
        assert!(formatted.contains(r#""event":"tool_result""#));
        assert!(formatted.contains(r#""success":true"#));
    }

    #[test]
    fn test_format_sse_event_message() {
        let event = ExecuteEvent::message("assistant", "Hello, world!");
        let formatted = format_sse_event(&event, None); // No ID

        // Without ID, should not have id line
        assert!(!formatted.starts_with("id:"));
        assert!(formatted.starts_with("event: message\n"));
        assert!(formatted.contains("event: message\n"));
        assert!(formatted.contains(r#""role":"assistant""#));
        assert!(formatted.contains("Hello, world!"));
    }

    #[test]
    fn test_format_sse_event_token() {
        let event = ExecuteEvent::token("Hello");
        let formatted = format_sse_event(&event, Some(5));

        assert!(formatted.starts_with("id: 5\n"));
        assert!(formatted.contains("event: token\n"));
    }

    #[test]
    fn test_format_sse_event_error() {
        let event = ExecuteEvent::error("TOOL_NOT_FOUND", "The tool 'foo' was not found");
        let formatted = format_sse_event(&event, Some(99));

        assert!(formatted.starts_with("id: 99\n"));
        assert!(formatted.contains("event: error\n"));
        assert!(formatted.contains(r#""code":"TOOL_NOT_FOUND""#));
        assert!(formatted.contains("The tool 'foo' was not found"));
    }

    #[test]
    fn test_format_sse_event_complete() {
        let state = serde_json::json!({
            "messages": 5,
            "tools_used": ["read", "write"]
        });
        let event = ExecuteEvent::complete(state);
        let formatted = format_sse_event(&event, Some(100));

        assert!(formatted.starts_with("id: 100\n"));
        assert!(formatted.contains("event: complete\n"));
        assert!(formatted.contains(r#""event":"complete""#));
        assert!(formatted.contains(r#""session_state""#));
    }

    // ============== SseFormattedEvent Tests ==============

    #[test]
    fn test_sse_formatted_event_to_string_with_id() {
        let event = SseFormattedEvent::new(
            "tool_call",
            r#"{"event":"tool_call","tool":"read"}"#.to_string(),
            Some(42),
        );
        let sse = event.to_sse_string();

        assert_eq!(
            sse,
            "id: 42\nevent: tool_call\ndata: {\"event\":\"tool_call\",\"tool\":\"read\"}\n\n"
        );
    }

    #[test]
    fn test_sse_formatted_event_to_string_without_id() {
        let event = SseFormattedEvent::new(
            "message",
            r#"{"event":"message","content":"hi"}"#.to_string(),
            None,
        );
        let sse = event.to_sse_string();

        assert_eq!(
            sse,
            "event: message\ndata: {\"event\":\"message\",\"content\":\"hi\"}\n\n"
        );
    }

    #[test]
    fn test_sse_formatted_event_clone() {
        let event1 = SseFormattedEvent::new("test", "test data".to_string(), Some(1));
        let event2 = event1.clone();

        assert_eq!(event1.id, event2.id);
        assert_eq!(event1.event_type, event2.event_type);
        assert_eq!(event1.data, event2.data);
    }

    // ============== SSE Validation Tests ==============

    #[test]
    fn test_is_valid_sse_format_valid() {
        // Valid SSE format
        assert!(is_valid_sse_format("data: hello\n\n"));
        assert!(is_valid_sse_format(
            "id: 1\nevent: message\ndata: hello\n\n"
        ));
        assert!(is_valid_sse_format(
            "id: 1\nevent: tool_call\ndata: {\"tool\":\"read\"}\n\n"
        ));
    }

    #[test]
    fn test_is_valid_sse_format_invalid() {
        // Missing data field
        assert!(!is_valid_sse_format("id: 1\n\n"));
        assert!(!is_valid_sse_format("event: message\n\n"));

        // Invalid field (no colon)
        assert!(!is_valid_sse_format("invalid line\ndata: hello\n\n"));

        // Invalid field name
        assert!(!is_valid_sse_format("unknown: value\ndata: hello\n\n"));

        // Missing space after colon
        assert!(!is_valid_sse_format("data:hello\n\n"));
    }

    #[test]
    fn test_is_valid_sse_format_empty_data() {
        // Empty data is valid (just means empty payload)
        assert!(is_valid_sse_format("data: \n\n"));
        assert!(is_valid_sse_format("data:\n\n"));
    }

    // ============== Stream Tests ==============

    #[tokio::test]
    async fn test_execute_event_stream_sequential_events() {
        let events = vec![
            ExecuteEvent::message("user", "Hello"),
            ExecuteEvent::message("assistant", "Hi there!"),
            ExecuteEvent::tool_call("read", serde_json::json!({}), "call-1"),
            ExecuteEvent::tool_result("read", serde_json::json!({}), "call-1", true),
            ExecuteEvent::message("assistant", "Done!"),
        ];

        let stream = execute_event_stream(events);
        let collected: Vec<_> = stream.collect().await;

        // Should have collected all 5 events
        assert_eq!(collected.len(), 5);

        // Each item should be a valid SSE-formatted Bytes
        for (idx, item) in collected.iter().enumerate() {
            let bytes = item.as_ref().expect("should be Ok");
            let sse_str = String::from_utf8_lossy(bytes);

            // Verify it's valid SSE format
            assert!(
                is_valid_sse_format(&sse_str),
                "Event {} has invalid SSE format: {:?}",
                idx,
                sse_str
            );
        }

        // Verify event sequence
        let first_sse = String::from_utf8_lossy(&collected[0].as_ref().expect("Ok"));
        assert!(first_sse.contains("event: message\n"));
        assert!(first_sse.contains("Hello"));

        let tool_call_sse = String::from_utf8_lossy(&collected[2].as_ref().expect("Ok"));
        assert!(tool_call_sse.contains("event: tool_call\n"));
    }

    #[tokio::test]
    async fn test_execute_event_stream_from_stream_multiple_events() {
        use futures::stream;

        // Create a stream of events
        let stream = stream::iter(vec![
            ExecuteEvent::token("H"),
            ExecuteEvent::token("i"),
            ExecuteEvent::token("!"),
        ]);

        let streaming = execute_event_stream_from_stream(stream);
        let collected: Vec<_> = streaming.collect().await;

        assert_eq!(collected.len(), 3);

        // Verify each token event
        for (idx, item) in collected.iter().enumerate() {
            let bytes = item.as_ref().expect("should be Ok");
            let sse_str = String::from_utf8_lossy(bytes);

            assert!(
                sse_str.contains("event: token\n"),
                "Event {} should be token event: {:?}",
                idx,
                sse_str
            );
        }
    }

    #[tokio::test]
    async fn test_execute_event_stream_with_ids() {
        use futures::stream;

        let events = vec![
            ExecuteEvent::message("user", "First"),
            ExecuteEvent::message("user", "Second"),
            ExecuteEvent::message("user", "Third"),
        ];

        let stream = execute_event_stream_with_ids(stream::iter(events));
        let collected: Vec<_> = stream.collect().await;

        assert_eq!(collected.len(), 3);

        // Verify sequential IDs
        for (idx, item) in collected.iter().enumerate() {
            let bytes = item.as_ref().expect("should be Ok");
            let sse_str = String::from_utf8_lossy(bytes);
            let expected_id = (idx + 1).to_string();

            assert!(
                sse_str.starts_with(&format!("id: {}\n", expected_id)),
                "Event {} should have id {}: {:?}",
                idx,
                expected_id,
                sse_str
            );
        }
    }

    #[tokio::test]
    async fn test_execute_event_stream_empty() {
        let events: Vec<ExecuteEvent> = vec![];
        let stream = execute_event_stream(events);
        let collected: Vec<_> = stream.collect().await;

        assert!(collected.is_empty());
    }

    #[tokio::test]
    async fn test_execute_event_stream_single_event() {
        let events = vec![ExecuteEvent::complete(serde_json::json!({}))];
        let stream = execute_event_stream(events);
        let collected: Vec<_> = stream.collect().await;

        assert_eq!(collected.len(), 1);

        let bytes = collected[0].as_ref().expect("should be Ok");
        let sse_str = String::from_utf8_lossy(bytes);

        assert!(sse_str.contains("event: complete\n"));
        assert!(sse_str.ends_with("\n\n"));
    }

    // ============== Edge Cases ==============

    #[test]
    fn test_format_sse_event_with_special_characters_in_content() {
        // Test content with special characters that could break SSE parsing
        let event = ExecuteEvent::message("assistant", "Hello\nWorld\r\nTest");
        let formatted = format_sse_event(&event, None);

        // Should still be valid SSE - newlines in data are fine
        assert!(is_valid_sse_format(&formatted));
        // The JSON serialized string will have escaped newlines (\\n, \\r\\n)
        assert!(formatted.contains("Hello\\nWorld"));
    }

    #[test]
    fn test_format_sse_event_with_empty_content() {
        let event = ExecuteEvent::message("assistant", "");
        let formatted = format_sse_event(&event, None);

        assert!(is_valid_sse_format(&formatted));
    }

    #[test]
    fn test_format_sse_event_with_unicode_content() {
        let event = ExecuteEvent::message("assistant", "Hello 世界! 🌍");
        let formatted = format_sse_event(&event, Some(1));

        assert!(is_valid_sse_format(&formatted));
        assert!(formatted.contains("Hello 世界! 🌍"));
    }

    #[test]
    fn test_format_sse_event_with_json_params() {
        let complex_params = serde_json::json!({
            "path": "/test/file.txt",
            "options": {
                "encoding": "utf-8",
                "line_numbers": true
            },
            "array": [1, 2, 3]
        });

        let event = ExecuteEvent::tool_call("read", complex_params, "call-complex");
        let formatted = format_sse_event(&event, Some(1));

        assert!(is_valid_sse_format(&formatted));
        assert!(formatted.contains("\"path\":\"/test/file.txt\""));
        assert!(formatted.contains("\"encoding\":\"utf-8\""));
    }

    #[test]
    fn test_execute_request_deserialization() {
        // Test that ExecuteRequest deserializes correctly (used by the execute endpoint)
        let json = r#"{"prompt": "Hello, world!"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(req.prompt, "Hello, world!");
        assert_eq!(req.stream, Some(true));

        let json_with_mode = r#"{"prompt": "Test", "mode": "build", "stream": false}"#;
        let req: ExecuteRequest = serde_json::from_str(json_with_mode).expect("should deserialize");
        assert_eq!(req.mode, Some(ExecuteMode::Build));
        assert_eq!(req.stream, Some(false));
    }
}
