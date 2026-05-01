use std::sync::{Arc, RwLock};

use crate::events::RuntimeFacadeEvent;

#[derive(Clone, Default)]
pub struct RecordingEventSink {
    events: Arc<RwLock<Vec<RuntimeFacadeEvent>>>,
}

impl RecordingEventSink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn record(&self, event: RuntimeFacadeEvent) {
        self.events.write().unwrap().push(event);
    }

    pub fn recorded_events(&self) -> Vec<RuntimeFacadeEvent> {
        self.events.read().unwrap().clone()
    }

    pub fn clear(&self) {
        self.events.write().unwrap().clear();
    }

    pub fn event_count(&self) -> usize {
        self.events.read().unwrap().len()
    }

    pub fn contains(&self, event: &RuntimeFacadeEvent) -> bool {
        self.events.read().unwrap().contains(event)
    }

    pub fn last_event(&self) -> Option<RuntimeFacadeEvent> {
        self.events.read().unwrap().last().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_session_started_event() -> RuntimeFacadeEvent {
        RuntimeFacadeEvent::SessionStarted {
            session_id: "test-session".to_string(),
        }
    }

    fn create_tool_call_event() -> RuntimeFacadeEvent {
        RuntimeFacadeEvent::ToolCallStarted {
            session_id: "test-session".to_string(),
            tool_name: "test_tool".to_string(),
            call_id: "call-123".to_string(),
        }
    }

    #[tokio::test]
    async fn test_recording_sink_records_events() {
        let sink = RecordingEventSink::new();
        assert_eq!(sink.event_count(), 0);

        let event = create_session_started_event();
        sink.record(event);

        assert_eq!(sink.event_count(), 1);
    }

    #[tokio::test]
    async fn test_recording_sink_clear() {
        let sink = RecordingEventSink::new();
        let event = create_session_started_event();

        sink.record(event);
        assert_eq!(sink.event_count(), 1);

        sink.clear();
        assert_eq!(sink.event_count(), 0);
    }

    #[tokio::test]
    async fn test_recording_sink_recorded_events() {
        let sink = RecordingEventSink::new();

        let event1 = create_session_started_event();
        let event2 = create_tool_call_event();

        sink.record(event1);
        sink.record(event2);

        let events = sink.recorded_events();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_recording_sink_contains() {
        let sink = RecordingEventSink::new();
        let event = create_session_started_event();

        assert!(!sink.contains(&event));
        sink.record(event);
        assert!(sink.contains(&create_session_started_event()));
    }

    #[tokio::test]
    async fn test_recording_sink_last_event() {
        let sink = RecordingEventSink::new();

        assert!(sink.last_event().is_none());

        let event1 = create_session_started_event();
        let event2 = create_tool_call_event();

        sink.record(event1);
        sink.record(event2.clone());

        assert_eq!(sink.last_event(), Some(event2));
    }
}
