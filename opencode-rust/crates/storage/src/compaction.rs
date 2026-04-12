use chrono::Utc;
use opencode_core::compaction::{CompactionResult, Compactor};
use opencode_core::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareabilityError {
    SessionNotShareable(String),
    ShareabilityCheckFailed(String),
    CompactionPreservedShareabilityFailed(String),
    ExportVerificationFailed(String),
}

impl std::fmt::Display for ShareabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareabilityError::SessionNotShareable(msg) => {
                write!(f, "Session is not shareable: {}", msg)
            }
            ShareabilityError::ShareabilityCheckFailed(msg) => {
                write!(f, "Shareability check failed: {}", msg)
            }
            ShareabilityError::CompactionPreservedShareabilityFailed(msg) => {
                write!(f, "Compaction did not preserve shareability: {}", msg)
            }
            ShareabilityError::ExportVerificationFailed(msg) => {
                write!(f, "Export verification failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for ShareabilityError {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShareabilityVerification {
    pub is_shareable: bool,
    pub has_share_id: bool,
    pub share_mode: Option<String>,
    pub is_expired: bool,
    pub export_verified: bool,
}

pub struct ShareabilityVerifier;

impl ShareabilityVerifier {
    pub fn verify(session: &Session) -> ShareabilityVerification {
        let is_expired = session
            .share_expires_at
            .map(|expiry| Utc::now() > expiry)
            .unwrap_or(false);

        let is_shareable = session.shared_id.is_some()
            && !matches!(
                session.share_mode,
                Some(opencode_core::config::ShareMode::Disabled)
            )
            && !is_expired;

        ShareabilityVerification {
            is_shareable,
            has_share_id: session.shared_id.is_some(),
            share_mode: session.share_mode.as_ref().map(|m| format!("{:?}", m)),
            is_expired,
            export_verified: false,
        }
    }

    pub fn verify_and_check_export(
        session: &Session,
    ) -> Result<ShareabilityVerification, ShareabilityError> {
        let mut verification = Self::verify(session);

        if verification.is_shareable {
            match session.export_json() {
                Ok(_) => {
                    verification.export_verified = true;
                    Ok(verification)
                }
                Err(e) => Err(ShareabilityError::ExportVerificationFailed(e.to_string())),
            }
        } else {
            Ok(verification)
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompactionWithShareabilityResult {
    pub compaction_result: CompactionResult,
    pub shareability_preserved: bool,
    pub verification: ShareabilityVerification,
    pub original_was_shareable: bool,
}

pub struct CompactionManager;

impl CompactionManager {
    pub fn compact_with_shareability_verification(
        session: &mut Session,
        max_tokens: usize,
    ) -> Result<CompactionWithShareabilityResult, ShareabilityError> {
        let original_verification = ShareabilityVerifier::verify(session);
        let original_was_shareable = original_verification.is_shareable;

        if original_was_shareable {
            let export_check = ShareabilityVerifier::verify_and_check_export(session);
            if export_check.is_err() {
                return Err(ShareabilityError::ShareabilityCheckFailed(
                    "Pre-compaction shareability check failed".to_string(),
                ));
            }
        }

        let original_messages = session.messages.clone();
        let original_shared_id = session.shared_id.clone();
        let original_share_mode = session.share_mode.clone();
        let original_share_expires_at = session.share_expires_at;

        let compaction_result = {
            let config = opencode_core::compaction::CompactionConfig {
                max_tokens,
                preserve_system_messages: true,
                preserve_recent_messages: 10,
                ..Default::default()
            };
            let compactor = Compactor::new(config);
            let messages = std::mem::take(&mut session.messages);
            compactor.compact_to_fit(messages)
        };

        session.messages = compaction_result.messages.clone();

        let post_verification = ShareabilityVerifier::verify(session);
        let shareability_preserved = if original_was_shareable {
            post_verification.is_shareable
        } else {
            !post_verification.is_shareable
        };

        if !shareability_preserved && original_was_shareable {
            session.messages = original_messages;
            session.shared_id = original_shared_id;
            session.share_mode = original_share_mode;
            session.share_expires_at = original_share_expires_at;

            return Err(ShareabilityError::CompactionPreservedShareabilityFailed(
                "Compaction would break shareability".to_string(),
            ));
        }

        Ok(CompactionWithShareabilityResult {
            compaction_result,
            shareability_preserved,
            verification: post_verification,
            original_was_shareable,
        })
    }

    pub fn can_compact_without_breaking_shareability(
        session: &Session,
        _max_tokens: usize,
    ) -> bool {
        let verification = ShareabilityVerifier::verify(session);

        if !verification.is_shareable {
            return true;
        }

        if let Ok(export_data) = session.export_json() {
            if export_data.is_empty() {
                return false;
            }

            let test_session = Session::new();
            if test_session.export_json().is_err() {
                return false;
            }

            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_core::config::ShareMode;
    use opencode_core::Message;

    fn create_shareable_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Test message".to_string()));
        session.set_share_mode(ShareMode::Manual);
        session.generate_share_link().unwrap();
        session
    }

    fn create_non_shareable_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Test message".to_string()));
        session
    }

    #[tokio::test]
    async fn test_shareability_verification_shareable_session() {
        let session = create_shareable_session();
        let verification = ShareabilityVerifier::verify(&session);

        assert!(verification.is_shareable);
        assert!(verification.has_share_id);
        assert_eq!(verification.share_mode, Some("Manual".to_string()));
        assert!(!verification.is_expired);
    }

    #[tokio::test]
    async fn test_shareability_verification_non_shareable_session() {
        let session = create_non_shareable_session();
        let verification = ShareabilityVerifier::verify(&session);

        assert!(!verification.is_shareable);
        assert!(!verification.has_share_id);
        assert!(verification.share_mode.is_none());
    }

    #[tokio::test]
    async fn test_shareability_verification_expired_session() {
        let mut session = create_shareable_session();
        session.set_share_expiry(Some(Utc::now() - chrono::Duration::hours(1)));

        let verification = ShareabilityVerifier::verify(&session);

        assert!(!verification.is_shareable);
        assert!(verification.is_expired);
    }

    #[tokio::test]
    async fn test_shareability_verification_disabled_mode() {
        let mut session = create_shareable_session();
        session.set_share_mode(ShareMode::Disabled);

        let verification = ShareabilityVerifier::verify(&session);

        assert!(!verification.is_shareable);
    }

    #[tokio::test]
    async fn test_compaction_preserves_shareability() {
        let mut session = create_shareable_session();

        for i in 0..20 {
            session.add_message(Message::assistant(format!("Response {}", i)));
        }

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 1000);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.shareability_preserved);
        assert!(result.original_was_shareable);
        assert!(result.verification.is_shareable);
    }

    #[tokio::test]
    async fn test_compaction_non_shareable_session() {
        let mut session = create_non_shareable_session();

        for i in 0..20 {
            session.add_message(Message::assistant(format!("This is a longer response number {} with more content to ensure compaction happens", i)));
        }

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.compaction_result.was_compacted);
        assert!(!result.original_was_shareable);
    }

    #[tokio::test]
    async fn test_compaction_verification_before_compaction() {
        let mut session = create_shareable_session();
        session.set_share_mode(ShareMode::Disabled);

        let verification = ShareabilityVerifier::verify_and_check_export(&session);

        assert!(verification.is_ok());
        let verification = verification.unwrap();
        assert!(!verification.is_shareable);
        assert!(!verification.export_verified);
    }

    #[tokio::test]
    async fn test_compaction_verification_passes_for_shareable() {
        let session = create_shareable_session();

        let verification = ShareabilityVerifier::verify_and_check_export(&session);

        assert!(verification.is_ok());
        let verification = verification.unwrap();
        assert!(verification.is_shareable);
        assert!(verification.export_verified);
    }

    #[tokio::test]
    async fn test_can_compact_without_breaking_shareability_shareable() {
        let session = create_shareable_session();

        let can_compact =
            CompactionManager::can_compact_without_breaking_shareability(&session, 1000);

        assert!(can_compact);
    }

    #[tokio::test]
    async fn test_can_compact_without_breaking_shareability_non_shareable() {
        let session = create_non_shareable_session();

        let can_compact =
            CompactionManager::can_compact_without_breaking_shareability(&session, 1000);

        assert!(can_compact);
    }

    #[tokio::test]
    async fn test_compaction_shareable_auto_mode() {
        let mut session = Session::new();
        session.add_message(Message::user("Test".to_string()));
        session.set_share_mode(ShareMode::Auto);
        session.generate_share_link().unwrap();

        for i in 0..20 {
            session.add_message(Message::assistant(format!("Response {}", i)));
        }

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 1000);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.shareability_preserved);
        assert_eq!(result.verification.share_mode, Some("Auto".to_string()));
    }

    #[tokio::test]
    async fn test_compaction_respects_max_tokens() {
        let mut session = Session::new();
        session.add_message(Message::system("System prompt".to_string()));

        for i in 0..50 {
            session.add_message(Message::user(format!(
                "Long message number {} with content",
                i
            )));
            session.add_message(Message::assistant(format!(
                "Long response number {} with content",
                i
            )));
        }

        let max_tokens = 500;
        let original_count = session.messages.len();

        let result =
            CompactionManager::compact_with_shareability_verification(&mut session, max_tokens);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.compaction_result.was_compacted);
        assert!(session.messages.len() < original_count);
    }

    #[tokio::test]
    async fn test_compaction_does_not_compact_when_not_needed() {
        let mut session = Session::new();
        session.add_message(Message::user("Short".to_string()));
        session.add_message(Message::assistant("Short".to_string()));

        let original_len = session.messages.len();

        let result =
            CompactionManager::compact_with_shareability_verification(&mut session, 100000);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.compaction_result.was_compacted);
        assert_eq!(session.messages.len(), original_len);
    }

    #[tokio::test]
    async fn test_shareability_verification_auto_mode() {
        let mut session = Session::new();
        session.add_message(Message::user("Test".to_string()));
        session.set_share_mode(ShareMode::Auto);
        session.generate_share_link().unwrap();

        let verification = ShareabilityVerifier::verify(&session);

        assert!(verification.is_shareable);
        assert_eq!(verification.share_mode, Some("Auto".to_string()));
    }
}
