use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenCodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("TUI error: {0}")]
    Tui(String),
}
