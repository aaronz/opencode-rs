#[cfg(test)]
mod tests {
    use crate::read::ReadTool;
    use crate::tool::Tool;
    use crate::ToolResult;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file_basic() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();

        fs::write(path.join("test.txt"), "line 1\nline 2\nline 3\n").unwrap();

        let args = serde_json::json!({
            "path": path.join("test.txt").to_str().unwrap()
        });

        let tool = ReadTool::new();
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("line 1"));
        assert!(result.content.contains("line 2"));
    }

    #[tokio::test]
    async fn test_read_with_offset_limit() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();

        fs::write(
            path.join("test.txt"),
            "line 1\nline 2\nline 3\nline 4\nline 5\n",
        )
        .unwrap();

        let args = serde_json::json!({
            "path": path.join("test.txt").to_str().unwrap(),
            "offset": 1,
            "limit": 2
        });

        let tool = ReadTool::new();
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("2: line 2"));
        assert!(result.content.contains("3: line 3"));
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let args = serde_json::json!({
            "path": "/nonexistent/file.txt"
        });

        let tool = ReadTool::new();
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }
}
