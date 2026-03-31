use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct LspTool;

#[derive(Deserialize)]
struct LspArgs {
    #[serde(rename = "operation")]
    operation: String,
    #[serde(rename = "filePath")]
    file_path: Option<String>,
    #[serde(rename = "line")]
    line: Option<u32>,
    #[serde(rename = "character")]
    character: Option<u32>,
    #[serde(rename = "symbol")]
    symbol: Option<String>,
}

#[async_trait]
impl Tool for LspTool {
    fn name(&self) -> &str {
        "lsp"
    }

    fn description(&self) -> &str {
        "Query LSP for code navigation and analysis"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(LspTool)
    }

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: LspArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let file = args.file_path.as_deref().unwrap_or("");
        let line = args.line.unwrap_or(1);
        let character = args.character.unwrap_or(1);
        let symbol = args.symbol.as_deref().unwrap_or("");

        match args.operation.as_str() {
            "goToDefinition" => {
                let result = format!("Go to definition at {}:{}:{} (LSP placeholder)", file, line, character);
                Ok(ToolResult::ok(result).with_title(format!("Go to Definition {}", file)))
            }
            "findReferences" => {
                let result = format!("Find references for '{}' (LSP placeholder)", symbol);
                Ok(ToolResult::ok(result).with_title(format!("Find References {}", symbol)))
            }
            "hover" => {
                let result = format!("Hover at {}:{}:{} (LSP placeholder)", file, line, character);
                Ok(ToolResult::ok(result).with_title(format!("Hover {}", file)))
            }
            "documentSymbol" => {
                let result = format!("Document symbols in {} (LSP placeholder)", file);
                Ok(ToolResult::ok(result).with_title(format!("Document Symbols {}", file)))
            }
            "workspaceSymbol" => {
                let result = format!("Workspace symbol '{}' (LSP placeholder)", symbol);
                Ok(ToolResult::ok(result).with_title("Workspace Symbols"))
            }
            "goToImplementation" => {
                let result = format!("Go to implementation at {}:{}:{} (LSP placeholder)", file, line, character);
                Ok(ToolResult::ok(result).with_title(format!("Go to Implementation {}", file)))
            }
            "prepareCallHierarchy" => {
                let result = format!("Prepare call hierarchy at {}:{}:{} (LSP placeholder)", file, line, character);
                Ok(ToolResult::ok(result).with_title(format!("Call Hierarchy {}", file)))
            }
            "incomingCalls" => {
                let result = format!("Incoming calls at {}:{}:{} (LSP placeholder)", file, line, character);
                Ok(ToolResult::ok(result).with_title(format!("Incoming Calls {}", file)))
            }
            "outgoingCalls" => {
                let result = format!("Outgoing calls at {}:{}:{} (LSP placeholder)", file, line, character);
                Ok(ToolResult::ok(result).with_title(format!("Outgoing Calls {}", file)))
            }
            "diagnostics" => {
                if file.is_empty() {
                    return Err(OpenCodeError::Tool("file_path is required for diagnostics".to_string()));
                }
                let result = format!("Diagnostics for {} (LSP placeholder - use cargo clippy or build for real diagnostics)", file);
                Ok(ToolResult::ok(result).with_title(format!("Diagnostics {}", file)))
            }
            _ => {
                Err(OpenCodeError::Tool(format!(
                    "Unknown LSP operation: {}. Supported: goToDefinition, findReferences, hover, documentSymbol, workspaceSymbol, goToImplementation, prepareCallHierarchy, incomingCalls, outgoingCalls, diagnostics",
                    args.operation
                )))
            }
        }
    }
}
