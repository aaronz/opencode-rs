//! Session sharing functionality.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareError {
    SharingDisabled,
    InvalidShareMode,
    AccessDenied,
}

impl std::fmt::Display for ShareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareError::SharingDisabled => write!(f, "sharing is disabled for this session"),
            ShareError::InvalidShareMode => write!(f, "invalid share mode for this operation"),
            ShareError::AccessDenied => write!(f, "access denied for this operation"),
        }
    }
}

impl std::error::Error for ShareError {}
