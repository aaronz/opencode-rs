use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct WebfetchTool;

#[derive(Deserialize)]
struct WebfetchArgs {
    url: String,
    #[serde(default = "default_format")]
    format: Option<String>,
    timeout: Option<u64>,
}

fn default_format() -> Option<String> {
    Some("markdown".to_string())
}

const MAX_RESPONSE_SIZE: usize = 5 * 1024 * 1024;
const DEFAULT_TIMEOUT: u64 = 30_000;
const MAX_TIMEOUT: u64 = 120_000;

#[async_trait]
impl Tool for WebfetchTool {
    fn name(&self) -> &str {
        "webfetch"
    }

    fn description(&self) -> &str {
        "Fetch web content from URL"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(WebfetchTool)
    }

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: WebfetchArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        if !args.url.starts_with("http://") && !args.url.starts_with("https://") {
            return Ok(ToolResult::err("URL must start with http:// or https://".to_string()));
        }

        let timeout = std::cmp::min(args.timeout.unwrap_or(DEFAULT_TIMEOUT / 1000) * 1000, MAX_TIMEOUT);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout))
            .build()
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let format = args.format.as_deref().unwrap_or("markdown");
        let accept_header = match format {
            "markdown" => "text/markdown;q=1.0, text/x-markdown;q=0.9, text/plain;q=0.8, text/html;q=0.7, */*;q=0.1",
            "text" => "text/plain;q=1.0, text/markdown;q=0.9, text/html;q=0.8, */*;q=0.1",
            "html" => "text/html;q=1.0, application/xhtml+xml;q=0.9, text/plain;q=0.8, text/markdown;q=0.7, */*;q=0.1",
            _ => "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        };

        let response = client
            .get(&args.url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Accept", accept_header)
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
            .map_err(|e| OpenCodeError::Tool(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(ToolResult::err(format!("Request failed with status code: {}", response.status())));
        }

        let content_type = response.headers().get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let content_length = response.headers().get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok());

        if let Some(len) = content_length {
            if len > MAX_RESPONSE_SIZE {
                return Ok(ToolResult::err("Response too large (exceeds 5MB limit)".to_string()));
            }
        }

        let body = response.bytes().await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to read response body: {}", e)))?;

        if body.len() > MAX_RESPONSE_SIZE {
            return Ok(ToolResult::err("Response too large (exceeds 5MB limit)".to_string()));
        }

        let mime = content_type.split(';').next().unwrap_or("").trim().to_lowercase();
        let is_image = mime.starts_with("image/") && mime != "image/svg+xml";

        if is_image {
            return Ok(ToolResult::ok(format!(
                "Image fetched successfully ({} bytes)",
                body.len()
            )));
        }

        let content = String::from_utf8_lossy(&body).to_string();

        if format == "markdown" && content_type.contains("text/html") {
            let markdown = convert_html_to_markdown(&content);
            return Ok(ToolResult::ok(markdown));
        }

        Ok(ToolResult::ok(content))
    }
}

fn convert_html_to_markdown(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script_style = false;
    let mut buffer = String::new();

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                let tag = buffer.trim().to_lowercase();
                if tag == "script" || tag == "style" {
                    in_script_style = true;
                }
                buffer.clear();
            }
            '>' => {
                in_tag = false;
                let tag = buffer.trim().to_lowercase();
                buffer.clear();

                if tag == "/script" || tag == "/style" {
                    in_script_style = false;
                }

                if tag.starts_with("h") && tag.len() > 1 && tag.starts_with('/') {
                    result.push('\n');
                }
            }
            _ => {
                if in_tag {
                    buffer.push(c);
                } else if !in_script_style {
                    result.push(c);
                }
            }
        }
    }

    result.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
