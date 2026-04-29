//! Event projections — converts domain events to [`StreamMessage`].
//!
//! These functions are the **single source of truth** for converting
//! [`DomainEvent`] → [`RuntimeFacadeEvent`] → [`StreamMessage`].
//!
//! All SSE/WS event routing must go through these functions to ensure
//! consistent projection behavior.

use opencode_core::events::DomainEvent;
use opencode_runtime::RuntimeFacadeEvent;

use crate::streaming::StreamMessage;

/// Project a [`DomainEvent`] → [`RuntimeFacadeEvent`] → [`StreamMessage`] for SSE/WS streaming.
///
/// Returns `None` if the event has no stream projection (task lifecycle
/// and token streaming events are intentionally excluded).
pub fn event_to_stream_message(event: &DomainEvent, session_id: &str) -> Option<StreamMessage> {
    let candidate = StreamMessage::from_domain_event(event)?;
    match candidate.session_id() {
        Some(source_session) if source_session == session_id => Some(candidate),
        Some(_) => None,
        None => Some(candidate),
    }
}

/// Project a [`DomainEvent`] to a [`StreamMessage`] without session filtering.
/// Use this when broadcasting to all sessions.
pub fn broadcast_stream_message(event: &DomainEvent) -> Option<StreamMessage> {
    StreamMessage::from_domain_event(event)
}
