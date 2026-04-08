#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;
    use crate::lsp_tool::LspTool;
    use crate::tool::Tool;

    fn create_rust_project(tmp: &std::path::Path) -> std::path::PathBuf {
        fs::write(
            tmp.join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )
        .unwrap();

        let src_dir = tmp.join("src");
        fs::create_dir(&src_dir).unwrap();

        let main_rs = r#"
pub struct Calculator {
    value: i64,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn add(&mut self, n: i64) -> i64 {
        self.value += n;
        self.value
    }
}

pub fn helper_function(x: i64) -> i64 {
    x * 2
}

pub fn public_function() -> String {
    String::from("hello")
}
"#;

        fs::write(src_dir.join("main.rs"), main_rs).unwrap();
        tmp.to_path_buf()
    }

    #[tokio::test]
    async fn test_lsp_tool_name_and_description() {
        let tool = LspTool;
        assert_eq!(tool.name(), "lsp");
        assert!(tool.description().contains("LSP"));
    }

    #[tokio::test]
    async fn test_goto_definition_requires_file_path() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "goToDefinition"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_references_requires_file_or_symbol() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "findReferences"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_hover_requires_file_path() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "hover"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_document_symbol_requires_file_path() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "documentSymbol"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_document_symbol_rust_file() {
        let tmp = TempDir::new().unwrap();
        let project_path = create_rust_project(tmp.path());
        let main_rs = project_path.join("src/main.rs");

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "documentSymbol",
            "filePath": main_rs.to_str().unwrap()
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("Calculator") || result.content.contains("fn"));
    }

    #[tokio::test]
    async fn test_workspace_symbol_requires_symbol() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "workspaceSymbol"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_go_to_implementation_requires_file_path() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "goToImplementation"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_go_to_implementation_returns_placeholder() {
        let tmp = TempDir::new().unwrap();
        let project_path = create_rust_project(tmp.path());
        let main_rs = project_path.join("src/main.rs");

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "goToImplementation",
            "filePath": main_rs.to_str().unwrap(),
            "line": 10,
            "character": 5
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("Go to implementation") || result.content.contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_diagnostics_requires_file_path() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "diagnostics"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_diagnostics_rust_file() {
        let tmp = TempDir::new().unwrap();
        let bad_rs = tmp.path().join("bad.rs");
        fs::write(&bad_rs, "fn main() { let x = undefined_var; }").unwrap();

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "diagnostics",
            "filePath": bad_rs.to_str().unwrap()
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_diagnostics_non_rust_file() {
        let tmp = TempDir::new().unwrap();
        let txt_file = tmp.path().join("readme.txt");
        fs::write(&txt_file, "Just some text").unwrap();

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "diagnostics",
            "filePath": txt_file.to_str().unwrap()
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("No diagnostics") || result.content.contains("0 issues"));
    }

    #[tokio::test]
    async fn test_unknown_operation_returns_error() {
        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "unknownOperation",
            "filePath": "/some/path.rs"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_goto_definition_non_rust_file() {
        let tmp = TempDir::new().unwrap();
        let txt_file = tmp.path().join("readme.txt");
        fs::write(&txt_file, "Just some text").unwrap();

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "goToDefinition",
            "filePath": txt_file.to_str().unwrap(),
            "line": 1,
            "character": 1
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("not supported") || result.content.contains("Go to Definition"));
    }

    #[tokio::test]
    async fn test_tool_clone_works() {
        let tool = LspTool;
        let cloned = tool.clone_tool();

        assert_eq!(cloned.name(), "lsp");
        assert_eq!(cloned.description(), tool.description());
    }

    #[tokio::test]
    async fn test_find_references_with_symbol() {
        let tmp = TempDir::new().unwrap();
        let project_path = create_rust_project(tmp.path());
        let main_rs = project_path.join("src/main.rs");

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "findReferences",
            "filePath": main_rs.to_str().unwrap(),
            "line": 5,
            "character": 10,
            "symbol": "Calculator"
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("references") || result.content.contains("Calculator"));
    }

    #[tokio::test]
    async fn test_prepare_call_hierarchy_not_implemented() {
        let tmp = TempDir::new().unwrap();
        let project_path = create_rust_project(tmp.path());
        let main_rs = project_path.join("src/main.rs");

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "prepareCallHierarchy",
            "filePath": main_rs.to_str().unwrap(),
            "line": 10,
            "character": 5
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_incoming_calls_not_implemented() {
        let tmp = TempDir::new().unwrap();
        let project_path = create_rust_project(tmp.path());
        let main_rs = project_path.join("src/main.rs");

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "incomingCalls",
            "filePath": main_rs.to_str().unwrap(),
            "line": 10,
            "character": 5
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_outgoing_calls_not_implemented() {
        let tmp = TempDir::new().unwrap();
        let project_path = create_rust_project(tmp.path());
        let main_rs = project_path.join("src/main.rs");

        let tool = LspTool;
        let args = serde_json::json!({
            "operation": "outgoingCalls",
            "filePath": main_rs.to_str().unwrap(),
            "line": 10,
            "character": 5
        });

        let result = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("not yet implemented"));
    }
}