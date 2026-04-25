#[cfg(test)]
mod tests {
    use crate::read::ReadTool;
    use crate::tool::Tool;
    use crate::ToolResult;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tempfile::TempDir;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_test_file_name() -> String {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("test_read_{}.txt", id)
    }

    #[tokio::test]
    async fn test_read_file_basic() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        let file_name = unique_test_file_name();

        fs::write(path.join(&file_name), "line 1\nline 2\nline 3\n").unwrap();

        let args = serde_json::json!({
            "path": path.join(&file_name).to_str().unwrap()
        });

        let tool = ReadTool::new();
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok(), "execute failed: {:?}", result);
        let result = result.unwrap();
        assert!(
            result.content.contains("line 1"),
            "content should contain 'line 1': {}",
            result.content
        );
        assert!(
            result.content.contains("line 2"),
            "content should contain 'line 2': {}",
            result.content
        );
    }

    #[tokio::test]
    async fn test_read_with_offset_limit() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        let file_name = unique_test_file_name();

        fs::write(
            path.join(&file_name),
            "line 1\nline 2\nline 3\nline 4\nline 5\n",
        )
        .unwrap();

        let args = serde_json::json!({
            "path": path.join(&file_name).to_str().unwrap(),
            "offset": 1,
            "limit": 2
        });

        let tool = ReadTool::new();
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok(), "execute failed: {:?}", result);
        let result = result.unwrap();
        assert!(
            result.content.contains("2: line 2"),
            "content should contain '2: line 2': {}",
            result.content
        );
        assert!(
            result.content.contains("3: line 3"),
            "content should contain '3: line 3': {}",
            result.content
        );
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
