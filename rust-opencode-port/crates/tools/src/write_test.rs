#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;
    use crate::write::WriteTool;
    use crate::tool::Tool;
    use crate::ToolResult;
    
    #[tokio::test]
    async fn test_write_creates_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        let file_path = path.join("new_file.txt");
        
        let args = serde_json::json!({
            "path": file_path.to_str().unwrap(),
            "content": "Hello, World!"
        });
        
        let tool = WriteTool;
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }
    
    #[tokio::test]
    async fn test_write_creates_directories() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        let file_path = path.join("subdir").join("nested").join("file.txt");
        
        let args = serde_json::json!({
            "path": file_path.to_str().unwrap(),
            "content": "nested content"
        });
        
        let tool = WriteTool;
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "nested content");
    }
    
    #[tokio::test]
    async fn test_write_overwrites_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        let file_path = path.join("existing.txt");
        
        fs::write(&file_path, "original").unwrap();
        
        let args = serde_json::json!({
            "path": file_path.to_str().unwrap(),
            "content": "updated"
        });
        
        let tool = WriteTool;
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "updated");
    }
}
