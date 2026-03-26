use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct CodesearchTool;

#[derive(Deserialize)]
struct CodeSearchArgs {
    query: String,
    #[serde(default = "default_tokens")]
    tokens_num: Option<usize>,
}

fn default_tokens() -> Option<usize> {
    Some(5000)
}

#[async_trait]
impl Tool for CodesearchTool {
    fn name(&self) -> &str {
        "codesearch"
    }

    fn description(&self) -> &str {
        "Search for relevant code context using Exa API"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(CodesearchTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: CodeSearchArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let tokens_num = args.tokens_num.unwrap_or(5000).clamp(1000, 50000);

        let api_key = std::env::var("EXA_API_KEY")
            .or_else(|_| std::env::var("OPENCODE_EXA_API_KEY"))
            .ok();

        if api_key.is_none() {
            return Ok(ToolResult::ok(format!(
                "Code search for '{}':\n\nTo enable code search, set EXA_API_KEY environment variable.\nPlease try a different query or check the spelling.",
                args.query
            )));
        }

        let api_key = api_key.unwrap();

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "get_code_context_exa",
                "arguments": {
                    "query": args.query,
                    "tokensNum": tokens_num
                }
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://mcp.exa.ai/mcp")
            .header("accept", "application/json, text/event-stream")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Tool(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Ok(ToolResult::err(format!("Code search error ({}): {}", status, text)));
        }

        let response_text = response.text().await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to read response: {}", e)))?;

        for line in response_text.lines() {
            if let Some(data_str) = line.strip_prefix("data: ") {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(data_str) {
                    if let Some(result) = data.get("result")
                        .and_then(|r| r.get("content"))
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|item| item.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        return Ok(ToolResult::ok(result.to_string()));
                    }
                }
            }
        }

        Ok(ToolResult::ok(format!(
            "No code snippets or documentation found for '{}'. Please try a different query or be more specific.",
            args.query
        )))
    }
}
