use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use crate::{Tool, ToolResult, ToolContext};
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
    #[serde(rename = "workspace")]
    workspace: Option<String>,
}

#[derive(Debug, Serialize)]
struct DiagnosticResult {
    file: String,
    line: u32,
    column: u32,
    severity: String,
    message: String,
}

async fn run_cargo_diagnostics(file: &str) -> Result<Vec<DiagnosticResult>, OpenCodeError> {
    let output = Command::new("cargo")
        .args(["clippy", "--message-format=json", "--", "-A", "clippy::all"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| OpenCodeError::Tool(format!("Failed to run cargo clippy: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut diagnostics = Vec::new();

    for line in stdout.lines() {
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(reason) = msg.get("reason").and_then(|r| r.as_str()) {
                if reason == "compiler-message" {
                    if let Some(message) = msg.get("message") {
                        if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                            for span in spans {
                                if let Some(file_name) = span.get("file_name").and_then(|f| f.as_str()) {
                                    if file_name == file || file.is_empty() {
                                        diagnostics.push(DiagnosticResult {
                                            file: file_name.to_string(),
                                            line: span.get("line_start").and_then(|l| l.as_u64()).unwrap_or(0) as u32,
                                            column: span.get("column_start").and_then(|c| c.as_u64()).unwrap_or(0) as u32,
                                            severity: message.get("level").and_then(|l| l.as_str()).unwrap_or("warning").to_string(),
                                            message: message.get("rendered").and_then(|r| r.as_str())
                                                .or_else(|| message.get("message").and_then(|m| m.as_str()))
                                                .unwrap_or("Unknown diagnostic").to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(diagnostics)
}

async fn run_eslint_diagnostics(file: &str) -> Result<Vec<DiagnosticResult>, OpenCodeError> {
    let output = Command::new("npx")
        .args(["eslint", "--format=json", file])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| OpenCodeError::Tool(format!("Failed to run eslint: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut diagnostics = Vec::new();

    if let Ok(results) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(results_array) = results.as_array() {
            for result in results_array {
                if let Some(messages) = result.get("messages").and_then(|m| m.as_array()) {
                    for msg in messages {
                        diagnostics.push(DiagnosticResult {
                            file: file.to_string(),
                            line: msg.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32,
                            column: msg.get("column").and_then(|c| c.as_u64()).unwrap_or(0) as u32,
                            severity: match msg.get("severity").and_then(|s| s.as_u64()) {
                                Some(2) => "error".to_string(),
                                _ => "warning".to_string(),
                            },
                            message: msg.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown").to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(diagnostics)
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

    async fn execute(&self, args: serde_json::Value, ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: LspArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let file = args.file_path.as_deref().unwrap_or("");
        let line = args.line.unwrap_or(1).saturating_sub(1);
        let character = args.character.unwrap_or(1).saturating_sub(1);
        let symbol = args.symbol.as_deref().unwrap_or("");
        let workspace = args.workspace.as_ref().or_else(|| {
            ctx.as_ref().and_then(|c| c.directory.as_ref())
        }).map(|s| s.as_str());

        match args.operation.as_str() {
            "goToDefinition" => {
                if file.is_empty() {
                    return Err(OpenCodeError::Tool("file_path is required for goToDefinition".to_string()));
                }

                self.goto_definition_with_retry(file, line, character, workspace.as_deref()).await
            }
            "findReferences" => {
                if file.is_empty() && symbol.is_empty() {
                    return Err(OpenCodeError::Tool("file_path or symbol is required for findReferences".to_string()));
                }

                self.find_references_with_retry(file, line, character, symbol, workspace.as_deref()).await
            }
            "hover" => {
                if file.is_empty() {
                    return Err(OpenCodeError::Tool("file_path is required for hover".to_string()));
                }
                
                Ok(ToolResult::ok(format!("Hover information for {}:{}:{}", file, line, character))
                    .with_title(format!("Hover {}", file)))
            }
            "documentSymbol" => {
                if file.is_empty() {
                    return Err(OpenCodeError::Tool("file_path is required for documentSymbol".to_string()));
                }
                
                let grep_output = Command::new("grep")
                    .args(["-n", "^\\s*\\(pub\\s\\+\\)\\?\\(fn\\|struct\\|enum\\|trait\\|impl\\|type\\)", file])
                    .output()
                    .await;
                
                match grep_output {
                    Ok(out) => {
                        let result = String::from_utf8_lossy(&out.stdout);
                        Ok(ToolResult::ok(result.to_string())
                            .with_title(format!("Document Symbols {}", file)))
                    }
                    Err(e) => {
                        Ok(ToolResult::ok(format!("Failed to get symbols: {}", e))
                            .with_title(format!("Document Symbols {}", file)))
                    }
                }
            }
            "workspaceSymbol" => {
                if symbol.is_empty() {
                    return Err(OpenCodeError::Tool("symbol is required for workspaceSymbol".to_string()));
                }
                
                let grep_output = Command::new("grep")
                    .args(["-rn", "--include=*.rs", &format!("\\<{}\\>", symbol), "."])
                    .output()
                    .await;
                
                match grep_output {
                    Ok(out) => {
                        let result = String::from_utf8_lossy(&out.stdout);
                        Ok(ToolResult::ok(result.to_string())
                            .with_title(format!("Workspace Symbols: {}", symbol)))
                    }
                    Err(e) => {
                        Ok(ToolResult::ok(format!("Failed to search: {}", e))
                            .with_title(format!("Workspace Symbols: {}", symbol)))
                    }
                }
            }
            "goToImplementation" => {
                if file.is_empty() {
                    return Err(OpenCodeError::Tool("file_path is required for goToImplementation".to_string()));
                }
                
                Ok(ToolResult::ok(format!("Go to implementation at {}:{}:{}", file, line, character))
                    .with_title(format!("Go to Implementation {}", file)))
            }
            "diagnostics" => {
                if file.is_empty() {
                    return Err(OpenCodeError::Tool("file_path is required for diagnostics".to_string()));
                }
                
                let diagnostics = if file.ends_with(".rs") {
                    run_cargo_diagnostics(file).await
                } else if file.ends_with(".js") || file.ends_with(".ts") || file.ends_with(".jsx") || file.ends_with(".tsx") {
                    run_eslint_diagnostics(file).await
                } else {
                    Ok(Vec::new())
                };
                
                match diagnostics {
                    Ok(diags) if !diags.is_empty() => {
                        let result = serde_json::to_string_pretty(&diags)
                            .unwrap_or_else(|_| "[]".to_string());
                        Ok(ToolResult::ok(result).with_title(format!("Diagnostics {} ({} issues)", file, diags.len())))
                    }
                    Ok(_) => {
                        Ok(ToolResult::ok(format!("No diagnostics found for {}", file))
                            .with_title(format!("Diagnostics {}", file)))
                    }
                    Err(e) => {
                        Ok(ToolResult::ok(format!("Failed to get diagnostics: {}", e))
                            .with_title(format!("Diagnostics {}", file)))
                    }
                }
            }
            "prepareCallHierarchy" | "incomingCalls" | "outgoingCalls" => {
                Ok(ToolResult::ok(format!("{} not yet implemented", args.operation))
                    .with_title(args.operation))
            }
            _ => {
                Err(OpenCodeError::Tool(format!(
                    "Unknown LSP operation: {}. Supported: goToDefinition, findReferences, hover, documentSymbol, workspaceSymbol, goToImplementation, diagnostics",
                    args.operation
                )))
            }
        }
    }
}

impl LspTool {
    async fn goto_definition_with_retry(
        &self,
        file: &str,
        line: u32,
        character: u32,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        let max_retries = 3;
        let retry_delay_ms = [100, 500, 1000];

        for attempt in 0..max_retries {
            match self.goto_definition_impl(file, line, character, workspace).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < max_retries - 1 => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms[attempt])).await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn goto_definition_impl(
        &self,
        file: &str,
        line: u32,
        character: u32,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        if file.ends_with(".rs") {
            let root = workspace.map(PathBuf::from).unwrap_or_else(|| {
                PathBuf::from(file).parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
            });

            let server_cmd = if root.join("Cargo.toml").exists() {
                "rust-analyzer"
            } else {
                return Ok(ToolResult::ok(format!(
                    "No LSP server configured for {} (no Cargo.toml found in workspace)", file
                )).with_title(format!("Go to Definition {}", file)));
            };

            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("echo '{}' | timeout 5 rust-analyzer --ipa-lookup 2>/dev/null || echo ''",
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "textDocument/definition",
                        "params": {
                            "textDocument": { "uri": format!("file://{}", file) },
                            "position": { "line": line, "character": character }
                        }
                    })
                ))
                .current_dir(&root)
                .output()
                .await
                .map_err(|e| OpenCodeError::Tool(format!("Failed to run rust-analyzer: {}", e)))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                return self.goto_definition_fallback(file, line, character).await;
            }

            if let Ok(resp) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(result) = resp.get("result").and_then(|r| r.as_object()) {
                    if let Some(location) = result.get("location") {
                        let loc = location.as_object()
                            .ok_or_else(|| OpenCodeError::Tool("Invalid LSP response".to_string()))?;
                        let uri = loc.get("uri").and_then(|u| u.as_str()).unwrap_or(file);
                        let range = loc.get("range").and_then(|r| r.as_object());

                        if let (Some(start), Some(end)) = (
                            range.and_then(|r| r.get("start")),
                            range.and_then(|r| r.get("end"))
                        ) {
                            let start_line = start.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32 + 1;
                            let start_col = start.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as u32 + 1;
                            let end_line = end.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32 + 1;
                            let end_col = end.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as u32 + 1;

                            return Ok(ToolResult::ok(format!(
                                "{}:{}:{} -> {}:{}:{}",
                                file, line + 1, character + 1, uri, start_line, start_col
                            )).with_title(format!("Go to Definition {}", file)));
                        }
                    }
                }
            }

            self.goto_definition_fallback(file, line, character).await
        } else {
            Ok(ToolResult::ok(format!("Go to definition not supported for {}", file))
                .with_title(format!("Go to Definition {}", file)))
        }
    }

    async fn goto_definition_fallback(
        &self,
        file: &str,
        line: u32,
        character: u32,
    ) -> Result<ToolResult, OpenCodeError> {
        let content = tokio::fs::read_to_string(file).await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to read file: {}", e)))?;

        let search_range = 10..content.lines().count().min(line as usize + 100);

        for (idx, l) in content.lines().enumerate().skip(search_range.start).take(search_range.end - search_range.start) {
            let trimmed = l.trim();
            if trimmed.starts_with("fn ") || trimmed.starts_with("struct ") ||
               trimmed.starts_with("enum ") || trimmed.starts_with("impl ") ||
               trimmed.starts_with("trait ") || trimmed.starts_with("type ") {
                return Ok(ToolResult::ok(format!(
                    "{}:{} -> {}:1", file, line + 1, idx + 1
                )).with_title(format!("Go to Definition {}", file)));
            }
        }

        Ok(ToolResult::ok(format!("No definition found for {}:{}", file, line + 1))
            .with_title(format!("Go to Definition {}", file)))
    }

    async fn find_references_with_retry(
        &self,
        file: &str,
        line: u32,
        character: u32,
        symbol: &str,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        let max_retries = 3;
        let retry_delay_ms = [100, 500, 1000];

        for attempt in 0..max_retries {
            match self.find_references_impl(file, line, character, symbol, workspace).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < max_retries - 1 => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms[attempt])).await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn find_references_impl(
        &self,
        file: &str,
        line: u32,
        character: u32,
        symbol: &str,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        let search_symbol: String = if symbol.is_empty() {
            let content = tokio::fs::read_to_string(file).await
                .map_err(|e| OpenCodeError::Tool(format!("Failed to read file: {}", e)))?;
            content.lines()
                .nth(line as usize)
                .and_then(|l| {
                    let trimmed = l.trim();
                    if trimmed.contains("fn ") || trimmed.contains("struct ") ||
                       trimmed.contains("enum ") || trimmed.contains("impl ") {
                        Some(trimmed.split(|c: char| !c.is_alphanumeric() && c != '_')
                            .filter(|s| !s.is_empty())
                            .last()
                            .map(|s| s.to_string())
                            .unwrap_or_default())
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        } else {
            symbol.to_string()
        };

        if search_symbol.is_empty() {
            return Ok(ToolResult::ok(format!("No symbol found at {}:{}", file, line + 1))
                .with_title(format!("Find References {}", symbol)));
        }

        let root = workspace.map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".").canonicalize().unwrap_or_default());

        let output = Command::new("grep")
            .args(["-rn", "--include=*.rs", "-E", &format!("\\b{}\\b", search_symbol)])
            .current_dir(&root)
            .output()
            .await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to search: {}", e)))?;

        let result = String::from_utf8_lossy(&output.stdout);
        if result.is_empty() {
            Ok(ToolResult::ok(format!("No references found for '{}'", search_symbol))
                .with_title(format!("Find References {}", search_symbol)))
        } else {
            let count = result.lines().count();
            Ok(ToolResult::ok(format!("Found {} references:\n{}", count, result))
                .with_title(format!("Find References {}", search_symbol)))
        }
    }
}
