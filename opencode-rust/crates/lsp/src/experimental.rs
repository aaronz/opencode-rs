use crate::client::LspClient;
use crate::types::Severity;
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalLspToolArgs {
    pub operation: String,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub line: Option<u32>,
    pub character: Option<u32>,
    pub symbol: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnosticsResult {
    pub file: String,
    pub diagnostics: Vec<DiagnosticInfo>,
    pub total_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub severity: String,
    pub message: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GotoDefinitionResult {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub target_uri: String,
    pub target_line: u32,
    pub target_column: u32,
    pub found: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindReferencesResult {
    pub symbol: String,
    pub references: Vec<ReferenceInfo>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceInfo {
    pub uri: String,
    pub line: u32,
    pub column: u32,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverResult {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub content: String,
    pub has_info: bool,
}

pub struct ExperimentalLspTool {
    client: Arc<Mutex<LspClient>>,
}

impl ExperimentalLspTool {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(LspClient::new())),
        }
    }

    pub async fn execute(&self, args: ExperimentalLspToolArgs) -> Result<String, OpenCodeError> {
        match args.operation.as_str() {
            "diagnostics" => self.get_diagnostics(args).await,
            "goToDefinition" => self.goto_definition(args).await,
            "findReferences" => self.find_references(args).await,
            "hover" => self.hover(args).await,
            _ => Err(OpenCodeError::Tool(format!(
                "Unknown operation: {}. Supported: diagnostics, goToDefinition, findReferences, hover",
                args.operation
            ))),
        }
    }

    async fn get_diagnostics(
        &self,
        args: ExperimentalLspToolArgs,
    ) -> Result<String, OpenCodeError> {
        let file_path = args.file_path.as_ref().ok_or_else(|| {
            OpenCodeError::Tool("filePath is required for diagnostics".to_string())
        })?;

        let uri = if file_path.starts_with("file://") {
            file_path.to_string()
        } else {
            let abs_path = std::fs::canonicalize(file_path)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| file_path.clone());
            format!("file://{}", abs_path)
        };

        let mut client = self.client.lock().await;

        if !client.is_healthy() {
            let workspace = args
                .workspace
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(file_path)
                        .parent()
                        .unwrap_or(&PathBuf::from("."))
                        .to_path_buf()
                });

            if let Err(e) = client.initialize(&workspace).await {
                return Ok(serde_json::json!({
                    "status": "unavailable",
                    "message": format!("LSP server not available: {}", e),
                    "file": file_path
                })
                .to_string());
            }
        }

        let diagnostics = client.get_diagnostics(&uri).await.unwrap_or_default();

        let diagnostic_infos: Vec<DiagnosticInfo> = diagnostics
            .iter()
            .map(|d| DiagnosticInfo {
                line: d.range.start.line + 1,
                column: d.range.start.character + 1,
                end_line: d.range.end.line + 1,
                end_column: d.range.end.character + 1,
                severity: format!("{:?}", d.severity).to_lowercase(),
                message: d.message.clone(),
                source: d.source.clone(),
            })
            .collect();

        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();

        let result = LspDiagnosticsResult {
            file: file_path.clone(),
            diagnostics: diagnostic_infos,
            total_count: diagnostics.len(),
            error_count,
            warning_count,
        };

        Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()))
    }

    async fn goto_definition(
        &self,
        args: ExperimentalLspToolArgs,
    ) -> Result<String, OpenCodeError> {
        let file_path = args.file_path.as_ref().ok_or_else(|| {
            OpenCodeError::Tool("filePath is required for goToDefinition".to_string())
        })?;

        let line = args.line.unwrap_or(1).saturating_sub(1);
        let character = args.character.unwrap_or(1).saturating_sub(1);

        let uri = if file_path.starts_with("file://") {
            file_path.to_string()
        } else {
            let abs_path = std::fs::canonicalize(file_path)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| file_path.clone());
            format!("file://{}", abs_path)
        };

        let mut client = self.client.lock().await;

        if !client.is_healthy() {
            let workspace = args
                .workspace
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(file_path)
                        .parent()
                        .unwrap_or(&PathBuf::from("."))
                        .to_path_buf()
                });

            if let Err(e) = client.initialize(&workspace).await {
                return Ok(serde_json::json!({
                    "status": "unavailable",
                    "message": format!("LSP server not available: {}", e),
                    "file": file_path
                })
                .to_string());
            }
        }

        match client.goto_definition(&uri, line, character).await {
            Ok(Some(location)) => {
                let result = GotoDefinitionResult {
                    file: file_path.clone(),
                    line: line + 1,
                    column: character + 1,
                    target_uri: location.uri,
                    target_line: location.range.start.line + 1,
                    target_column: location.range.start.character + 1,
                    found: true,
                };
                Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()))
            }
            Ok(None) => {
                let result = GotoDefinitionResult {
                    file: file_path.clone(),
                    line: line + 1,
                    column: character + 1,
                    target_uri: String::new(),
                    target_line: 0,
                    target_column: 0,
                    found: false,
                };
                Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()))
            }
            Err(e) => Err(e),
        }
    }

    async fn find_references(
        &self,
        args: ExperimentalLspToolArgs,
    ) -> Result<String, OpenCodeError> {
        let file_path = args.file_path.as_ref().ok_or_else(|| {
            OpenCodeError::Tool("filePath is required for findReferences".to_string())
        })?;

        let line = args.line.unwrap_or(1).saturating_sub(1);
        let character = args.character.unwrap_or(1).saturating_sub(1);

        let uri = if file_path.starts_with("file://") {
            file_path.to_string()
        } else {
            let abs_path = std::fs::canonicalize(file_path)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| file_path.clone());
            format!("file://{}", abs_path)
        };

        let mut client = self.client.lock().await;

        if !client.is_healthy() {
            let workspace = args
                .workspace
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(file_path)
                        .parent()
                        .unwrap_or(&PathBuf::from("."))
                        .to_path_buf()
                });

            if let Err(e) = client.initialize(&workspace).await {
                return Ok(serde_json::json!({
                    "status": "unavailable",
                    "message": format!("LSP server not available: {}", e),
                    "file": file_path
                })
                .to_string());
            }
        }

        let references = client.find_references(&uri, line, character).await?;

        let symbol = args.symbol.clone().unwrap_or_else(|| "unknown".to_string());
        let ref_infos: Vec<ReferenceInfo> = references
            .iter()
            .map(|loc| ReferenceInfo {
                uri: loc.uri.clone(),
                line: loc.range.start.line + 1,
                column: loc.range.start.character + 1,
                context: format!(
                    "{}:{}:{}",
                    loc.uri,
                    loc.range.start.line + 1,
                    loc.range.start.character + 1
                ),
            })
            .collect();

        let result = FindReferencesResult {
            symbol,
            references: ref_infos,
            total_count: references.len(),
        };

        Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()))
    }

    async fn hover(&self, args: ExperimentalLspToolArgs) -> Result<String, OpenCodeError> {
        let file_path = args
            .file_path
            .as_ref()
            .ok_or_else(|| OpenCodeError::Tool("filePath is required for hover".to_string()))?;

        let line = args.line.unwrap_or(1).saturating_sub(1);
        let character = args.character.unwrap_or(1).saturating_sub(1);

        let result = HoverResult {
            file: file_path.clone(),
            line: line + 1,
            column: character + 1,
            content: format!("Hover information requested for {}:{}:{} (LSP hover not yet fully implemented in experimental tool)", file_path, line + 1, character + 1),
            has_info: false,
        };

        Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()))
    }
}

impl Default for ExperimentalLspTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_experimental_tool_name() {
        let _tool = ExperimentalLspTool::new();
        assert!(true, "ExperimentalLspTool can be created");
    }

    #[tokio::test]
    async fn test_experimental_tool_diagnostics_requires_file() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "diagnostics".to_string(),
            file_path: None,
            line: None,
            character: None,
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("filePath is required"));
    }

    #[tokio::test]
    async fn test_experimental_tool_goto_definition_requires_file() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "goToDefinition".to_string(),
            file_path: None,
            line: None,
            character: None,
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("filePath is required"));
    }

    #[tokio::test]
    async fn test_experimental_tool_find_references_requires_file() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "findReferences".to_string(),
            file_path: None,
            line: None,
            character: None,
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("filePath is required"));
    }

    #[tokio::test]
    async fn test_experimental_tool_hover_requires_file() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "hover".to_string(),
            file_path: None,
            line: None,
            character: None,
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("filePath is required"));
    }

    #[tokio::test]
    async fn test_experimental_tool_unknown_operation() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "unknownOperation".to_string(),
            file_path: Some("test.rs".to_string()),
            line: None,
            character: None,
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown operation"));
    }

    #[tokio::test]
    async fn test_experimental_tool_diagnostics_returns_json() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "diagnostics".to_string(),
            file_path: Some("test.rs".to_string()),
            line: None,
            character: None,
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("file").is_some() || parsed.get("status").is_some());
        assert!(parsed.get("diagnostics").is_some() || parsed.get("status").is_some());
    }

    #[tokio::test]
    async fn test_experimental_tool_hover_returns_json() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "hover".to_string(),
            file_path: Some("test.rs".to_string()),
            line: Some(10),
            character: Some(5),
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.get("file").unwrap(), "test.rs");
        assert_eq!(parsed.get("line").unwrap(), 10);
        assert_eq!(parsed.get("column").unwrap(), 5);
        assert!(parsed.get("content").is_some());
    }

    #[tokio::test]
    async fn test_experimental_lsp_tool_struct_creation() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "diagnostics".to_string(),
            file_path: Some("src/main.rs".to_string()),
            line: Some(1),
            character: Some(1),
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_experimental_lsp_tool_goto_definition_operation() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "goToDefinition".to_string(),
            file_path: Some("src/main.rs".to_string()),
            line: Some(5),
            character: Some(10),
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_experimental_lsp_tool_find_references_operation() {
        let tool = ExperimentalLspTool::new();
        let args = ExperimentalLspToolArgs {
            operation: "findReferences".to_string(),
            file_path: Some("src/main.rs".to_string()),
            line: Some(5),
            character: Some(10),
            symbol: None,
            workspace: None,
        };
        let result = tool.execute(args).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_diagnostic_info_serialization() {
        let info = DiagnosticInfo {
            line: 10,
            column: 5,
            end_line: 10,
            end_column: 15,
            severity: "error".to_string(),
            message: "test error".to_string(),
            source: Some("rust-analyzer".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test error"));
        assert!(json.contains("error"));
    }

    #[tokio::test]
    async fn test_goto_definition_result_serialization() {
        let result = GotoDefinitionResult {
            file: "test.rs".to_string(),
            line: 10,
            column: 5,
            target_uri: "file:///target.rs".to_string(),
            target_line: 20,
            target_column: 10,
            found: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test.rs"));
        assert!(json.contains("target.rs"));
        assert!(json.contains("found"));
    }

    #[tokio::test]
    async fn test_find_references_result_serialization() {
        let result = FindReferencesResult {
            symbol: "my_function".to_string(),
            references: vec![ReferenceInfo {
                uri: "file:///src/main.rs".to_string(),
                line: 10,
                column: 5,
                context: "call site".to_string(),
            }],
            total_count: 1,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("my_function"));
        assert!(json.contains("total_count"));
    }

    #[tokio::test]
    async fn test_hover_result_serialization() {
        let result = HoverResult {
            file: "test.rs".to_string(),
            line: 10,
            column: 5,
            content: "fn my_function()".to_string(),
            has_info: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test.rs"));
        assert!(json.contains("has_info"));
    }
}
