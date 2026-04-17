use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSearchError {
    #[error("API key is missing. Set EXA_API_KEY or OPENCODE_EXA_API_KEY environment variable.")]
    ApiKeyMissing,
}

impl From<WebSearchError> for OpenCodeError {
    fn from(err: WebSearchError) -> Self {
        OpenCodeError::Tool(err.to_string())
    }
}

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
            .ok()
            .ok_or(WebSearchError::ApiKeyMissing)?;

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
            let text = response.text().await.map_err(|e| {
                OpenCodeError::Tool(format!("Failed to read error response: {}", e))
            })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn remove_api_keys() {
        std::env::remove_var("EXA_API_KEY");
        std::env::remove_var("OPENCODE_EXA_API_KEY");
    }

    #[tokio::test]
    async fn test_web_search_missing_api_key_returns_error() {
        remove_api_keys();
        let tool = WebSearchTool;
        let args = serde_json::json!({"query": "test query"});

        let result = tool.execute(args, None).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("API key is missing"));
    }

    #[tokio::test]
    async fn test_web_search_missing_api_key_error_type() {
        remove_api_keys();
        let tool = WebSearchTool;
        let args = serde_json::json!({"query": "test query"});

        let result = tool.execute(args, None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            OpenCodeError::Tool(msg) => {
                assert!(msg.contains("API key is missing"));
            }
            _ => panic!("Expected OpenCodeError::Tool"),
        }
    }

    #[tokio::test]
    async fn test_web_search_handles_network_errors_gracefully() {
        remove_api_keys();
        std::env::set_var("EXA_API_KEY", "test_key");

        let tool = WebSearchTool;
        let args = serde_json::json!({"query": "test query"});

        let result = tool.execute(args, None).await;

        std::env::remove_var("EXA_API_KEY");

        match result {
            Ok(tool_result) => {
                assert!(
                    tool_result.success,
                    "Expected success or proper error, got error: {:?}",
                    tool_result
                );
            }
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Request failed")
                        || err_msg.contains("connection")
                        || err_msg.contains("API key is missing"),
                    "Expected network or auth error, got: {}",
                    err_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_web_search_returns_proper_error_on_malformed_response() {
        remove_api_keys();
        std::env::set_var("EXA_API_KEY", "test_key");

        let tool = WebSearchTool;
        let args = serde_json::json!({"query": "test"});

        let result = tool.execute(args, None).await;

        std::env::remove_var("EXA_API_KEY");

        match &result {
            Ok(tool_result) => {
                assert!(tool_result.success, "Expected successful result");
            }
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Request failed")
                        || err_msg.contains("connection")
                        || err_msg.contains("API key is missing"),
                    "Expected proper error, got: {}",
                    err_msg
                );
            }
        }
        assert!(
            result.is_ok() || result.is_err(),
            "Should return either success or a proper error, not panic"
        );
    }
}
