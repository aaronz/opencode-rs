#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;
    use crate::grep_tool::GrepTool;
    use crate::tool::Tool;
    use crate::ToolResult;
    
    #[tokio::test]
    async fn test_grep_finds_pattern() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        
        fs::write(path.join("test.txt"), "hello world\ntest line\n").unwrap();
        
        let tmp_path = path.to_str().unwrap().to_string();
        
        let args = serde_json::json!({
            "pattern": "hello",
            "path": tmp_path
        });
        
        let tool = GrepTool;
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("hello"));
    }
    
    #[tokio::test]
    async fn test_grep_no_matches() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        
        fs::write(path.join("test.txt"), "hello world\n").unwrap();
        
        let tmp_path = path.to_str().unwrap().to_string();
        
        let args = serde_json::json!({
            "pattern": "notfound",
            "path": tmp_path
        });
        
        let tool = GrepTool;
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("No matches"));
    }
    
    #[tokio::test]
    async fn test_grep_with_file_type() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        
        fs::write(path.join("test.rs"), "fn main() {}").unwrap();
        fs::write(path.join("test.txt"), "hello").unwrap();
        
        let tmp_path = path.to_str().unwrap().to_string();
        
        let args = serde_json::json!({
            "pattern": "fn",
            "path": tmp_path,
            "file_type": "rs"
        });
        
        let tool = GrepTool;
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("fn"));
    }
}
