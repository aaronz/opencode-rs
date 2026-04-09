#[cfg(test)]
mod tests {
    use crate::session_tools::{SessionLoadTool, SessionSaveTool};
    use crate::tool::Tool;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_session_load_invalid_uuid() {
        let args = serde_json::json!({
            "session_id": "not-a-valid-uuid"
        });

        let tool = SessionLoadTool;
        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid session_id format"));
    }

    #[tokio::test]
    async fn test_session_load_missing_session_id() {
        let args = serde_json::json!({});

        let tool = SessionLoadTool;
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("session_id required"));
    }

    #[tokio::test]
    async fn test_session_save_requires_permission() {
        let args = serde_json::json!({
            "session_id": "not-a-valid-uuid"
        });

        let tool = SessionSaveTool;
        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Permission denied"));
    }

    #[tokio::test]
    async fn test_session_save_missing_session_id_permission_denied() {
        let args = serde_json::json!({});

        let tool = SessionSaveTool;
        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Permission denied"));
    }

    #[tokio::test]
    async fn test_session_tool_name_and_description() {
        let load_tool = SessionLoadTool;
        assert_eq!(load_tool.name(), "session_load");
        assert!(!load_tool.description().is_empty());

        let save_tool = SessionSaveTool;
        assert_eq!(save_tool.name(), "session_save");
        assert!(!save_tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_session_tool_clone() {
        let load_tool = SessionLoadTool;
        let cloned = load_tool.clone_tool();
        assert_eq!(cloned.name(), "session_load");

        let save_tool = SessionSaveTool;
        let cloned = save_tool.clone_tool();
        assert_eq!(cloned.name(), "session_save");
    }

    #[tokio::test]
    async fn test_session_load_nonexistent_session() {
        let non_existent_id = Uuid::new_v4();
        let args = serde_json::json!({
            "session_id": non_existent_id.to_string()
        });

        let tool = SessionLoadTool;
        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Failed to load session"));
    }
}
