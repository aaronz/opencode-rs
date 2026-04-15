use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;

pub struct WebSearchTool;

#[derive(Deserialize)]
struct SearchArgs {
    query: String,
    #[serde(default = "default_num_results")]
    num_results: Option<usize>,
    #[serde(default)]
    livecrawl: Option<String>,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default = "default_context_chars")]
    context_max_characters: Option<usize>,
}

fn default_num_results() -> Option<usize> {
    Some(8)
}

fn default_context_chars() -> Option<usize> {
    Some(10000)
}

impl sealed::Sealed for WebSearchTool {}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "websearch"
    }

    fn description(&self) -> &str {
        "Search the web for information using Exa API"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(WebSearchTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: SearchArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let num_results = args.num_results.unwrap_or(8);
        let livecrawl = args.livecrawl.unwrap_or_else(|| "fallback".to_string());
        let search_type = args.r#type.unwrap_or_else(|| "auto".to_string());
        let context_chars = args.context_max_characters.unwrap_or(10000);

        let api_key = std::env::var("EXA_API_KEY")
            .or_else(|_| std::env::var("OPENCODE_EXA_API_KEY"))
            .ok();

        if api_key.is_none() {
            return Ok(ToolResult::ok(format!(
                "Web search for '{}':\n\nTo enable web search, set EXA_API_KEY environment variable.\nFor now, please manually navigate to search engines.",
                args.query
            )));
        }

        let api_key = api_key.expect("api_key was validated above");

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "web_search_exa",
                "arguments": {
                    "query": args.query,
                    "type": search_type,
                    "numResults": num_results,
                    "livecrawl": livecrawl,
                    "contextMaxCharacters": context_chars
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
            return Ok(ToolResult::err(format!(
                "Search error ({}): {}",
                status, text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to read response: {}", e)))?;

        // Parse SSE response
        for line in response_text.lines() {
            if let Some(data_str) = line.strip_prefix("data: ") {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(data_str) {
                    if let Some(result) = data
                        .get("result")
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
            "No search results found for '{}'. Please try a different query.",
            args.query
        )))
    }
}
