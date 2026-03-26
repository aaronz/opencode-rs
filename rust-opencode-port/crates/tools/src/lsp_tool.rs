use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct LspTool;

#[derive(Deserialize)]
struct LspArgs {
    action: String,
    symbol: Option<String>,
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
}

#[async_trait]
impl Tool for LspTool {
    fn name(&self) -> &str {
        "lsp"
    }

    fn description(&self) -> &str {
        "Query LSP for symbols"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(LspTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: LspArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        match args.action.as_str() {
            "goto_definition" => {
                let file = args.file.unwrap_or_default();
                let line = args.line.unwrap_or(0);
                let column = args.column.unwrap_or(0);

                Ok(ToolResult::ok(format!(
                    "Go to definition at {}:{}:{} (placeholder)",
                    file, line, column
                )))
            }
            "find_references" => {
                let symbol = args.symbol.unwrap_or_default();

                Ok(ToolResult::ok(format!(
                    "Find references for '{}' (placeholder)",
                    symbol
                )))
            }
            "completion" => {
                let file = args.file.unwrap_or_default();
                let line = args.line.unwrap_or(0);
                let column = args.column.unwrap_or(0);

                Ok(ToolResult::ok(format!(
                    "Code completion at {}:{}:{} (placeholder)",
                    file, line, column
                )))
            }
            "diagnostics" => {
                let file = args.file.unwrap_or_default();

                Ok(ToolResult::ok(format!(
                    "Diagnostics for '{}' (placeholder)",
                    file
                )))
            }
            _ => Err(OpenCodeError::Tool(format!(
                "Unknown LSP action: {}",
                args.action
            ))),
        }
    }
}
