use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Thinking,
    AwaitingPermission,
    Executing,
    Streaming,
    Completed,
    Error,
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionError {
    pub from: SessionState,
    pub to: SessionState,
}

impl std::fmt::Display for StateTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid state transition from {:?} to {:?}",
            self.from, self.to
        )
    }
}

impl std::error::Error for StateTransitionError {}

/// Validates if a state transition is allowed
pub fn is_valid_transition(from: SessionState, to: SessionState) -> bool {
    matches!(
        (from, to),
        (SessionState::Idle, SessionState::Thinking)
            | (SessionState::Thinking, SessionState::AwaitingPermission)
            | (SessionState::Thinking, SessionState::Streaming)
            | (SessionState::AwaitingPermission, SessionState::Executing)
            | (SessionState::Executing, SessionState::Thinking)
            | (SessionState::Streaming, SessionState::Completed)
            | (SessionState::Executing, SessionState::Error)
            | (SessionState::Thinking, SessionState::Error)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_idle_to_thinking() {
        assert!(is_valid_transition(
            SessionState::Idle,
            SessionState::Thinking
        ));
    }

    #[test]
    fn test_invalid_idle_to_completed() {
        assert!(!is_valid_transition(
            SessionState::Idle,
            SessionState::Completed
        ));
    }

    #[test]
    fn test_default_state_is_idle() {
        assert_eq!(SessionState::default(), SessionState::Idle);
    }
}
