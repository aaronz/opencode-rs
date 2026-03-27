use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenCodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

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

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_io() {
        let err = OpenCodeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn test_error_display_json() {
        let err =
            OpenCodeError::Json(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err());
        assert!(err.to_string().contains("JSON error"));
    }

    #[test]
    fn test_error_display_network() {
        let err = OpenCodeError::Network("connection refused".to_string());
        assert!(err.to_string().contains("Network error"));
    }

    #[test]
    fn test_error_display_config() {
        let err = OpenCodeError::Config("missing key".to_string());
        assert!(err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_error_display_session() {
        let err = OpenCodeError::Session("not found".to_string());
        assert!(err.to_string().contains("Session error"));
    }

    #[test]
    fn test_error_display_tool() {
        let err = OpenCodeError::Tool("execution failed".to_string());
        assert!(err.to_string().contains("Tool error"));
    }

    #[test]
    fn test_error_display_llm() {
        let err = OpenCodeError::Llm("api error".to_string());
        assert!(err.to_string().contains("LLM error"));
    }

    #[test]
    fn test_error_display_provider() {
        let err = OpenCodeError::Provider("invalid response".to_string());
        assert!(err.to_string().contains("Provider error"));
    }
}
