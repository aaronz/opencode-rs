use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeFacadeError {
    #[error("runtime command not yet implemented: {0}")]
    NotImplemented(&'static str),

    #[error("runtime dependency error: {0}")]
    Dependency(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),
}
