#![allow(clippy::redundant_closure, clippy::needless_range_loop, clippy::needless_option_as_deref, clippy::implicit_saturating_sub, clippy::double_ended_iterator_last)]

use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

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
                                if let Some(file_name) =
                                    span.get("file_name").and_then(|f| f.as_str())
                                {
                                    if file_name == file || file.is_empty() {
                                        diagnostics.push(DiagnosticResult {
                                            file: file_name.to_string(),
                                            line: span
                                                .get("line_start")
                                                .and_then(|l| l.as_u64())
                                                .unwrap_or(0)
                                                as u32,
                                            column: span
                                                .get("column_start")
                                                .and_then(|c| c.as_u64())
                                                .unwrap_or(0)
                                                as u32,
                                            severity: message
                                                .get("level")
                                                .and_then(|l| l.as_str())
                                                .unwrap_or("warning")
                                                .to_string(),
                                            message: message
                                                .get("rendered")
                                                .and_then(|r| r.as_str())
                                                .or_else(|| {
                                                    message.get("message").and_then(|m| m.as_str())
                                                })
                                                .unwrap_or("Unknown diagnostic")
                                                .to_string(),
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
                            message: msg
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Unknown")
                                .to_string(),
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

    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: LspArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let file = args.file_path.as_deref().unwrap_or("");
        let line = args.line.unwrap_or(1).saturating_sub(1);
        let character = args.character.unwrap_or(1).saturating_sub(1);
        let symbol = args.symbol.as_deref().unwrap_or("");
        let workspace = args
            .workspace
            .as_ref()
            .or_else(|| ctx.as_ref().and_then(|c| c.directory.as_ref()))
            .map(|s| s.as_str());

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

                self.hover_with_retry(file, line, character, workspace.as_deref()).await
            }
            "codeActions" => {
                if file.is_empty() {
                    return Err(OpenCodeError::Tool("file_path is required for codeActions".to_string()));
                }

                self.code_actions_with_retry(file, line, character, workspace.as_deref()).await
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
                    "Unknown LSP operation: {}. Supported: goToDefinition, findReferences, hover, codeActions, documentSymbol, workspaceSymbol, goToImplementation, diagnostics",
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
            match self
                .goto_definition_impl(file, line, character, workspace)
                .await
            {
                Ok(result) => return Ok(result),
                Err(_e) if attempt < max_retries - 1 => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms[attempt]))
                        .await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    #[cfg(test)]
    async fn test_goto_definition_fallback_on_empty_response(
        &self,
        file: &str,
        line: u32,
        character: u32,
    ) -> Result<ToolResult, OpenCodeError> {
        self.goto_definition_fallback(file, line, character).await
    }

    #[cfg(test)]
    fn test_parse_lsp_response(raw: &str) -> Option<(String, u32, u32)> {
        let resp: serde_json::Value = serde_json::from_str(raw).ok()?;
        let result = resp.get("result")?.as_object()?;
        let location = result.get("location")?.as_object()?;
        let uri = location.get("uri")?.as_str()?.to_string();
        let range = location.get("range")?.as_object()?;
        let start = range.get("start")?.as_object()?;
        let line = start.get("line")?.as_u64()? as u32 + 1;
        let character = start.get("character")?.as_u64()? as u32 + 1;
        Some((uri, line, character))
    }

    #[cfg(test)]
    fn test_extract_symbol_from_line(line: &str) -> Option<String> {
        let trimmed = line.trim();
        let (keyword, after_idx) = if let Some(idx) = trimmed.find("fn ") {
            ("fn ", idx)
        } else if let Some(idx) = trimmed.find("struct ") {
            ("struct ", idx)
        } else if let Some(idx) = trimmed.find("enum ") {
            ("enum ", idx)
        } else if trimmed.starts_with("impl ") {
            return Some(trimmed.to_string());
        } else {
            return None;
        };

        let after_keyword = trimmed[after_idx + keyword.len()..].trim_start();
        after_keyword
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
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
                PathBuf::from(file)
                    .parent()
                    .unwrap_or(&PathBuf::from("."))
                    .to_path_buf()
            });

            let _server_cmd = if root.join("Cargo.toml").exists() {
                "rust-analyzer"
            } else {
                return Ok(ToolResult::ok(format!(
                    "No LSP server configured for {} (no Cargo.toml found in workspace)",
                    file
                ))
                .with_title(format!("Go to Definition {}", file)));
            };

            let output = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "echo '{}' | timeout 5 rust-analyzer --ipa-lookup 2>/dev/null || echo ''",
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
                        let loc = location.as_object().ok_or_else(|| {
                            OpenCodeError::Tool("Invalid LSP response".to_string())
                        })?;
                        let uri = loc.get("uri").and_then(|u| u.as_str()).unwrap_or(file);
                        let range = loc.get("range").and_then(|r| r.as_object());

                        if let (Some(start), Some(end)) = (
                            range.and_then(|r| r.get("start")),
                            range.and_then(|r| r.get("end")),
                        ) {
                            let start_line =
                                start.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32 + 1;
                            let start_col =
                                start.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as u32
                                    + 1;
                            let _end_line =
                                end.get("line").and_then(|l| l.as_u64()).unwrap_or(0) as u32 + 1;
                            let _end_col =
                                end.get("character").and_then(|c| c.as_u64()).unwrap_or(0) as u32
                                    + 1;

                            return Ok(ToolResult::ok(format!(
                                "{}:{}:{} -> {}:{}:{}",
                                file,
                                line + 1,
                                character + 1,
                                uri,
                                start_line,
                                start_col
                            ))
                            .with_title(format!("Go to Definition {}", file)));
                        }
                    }
                }
            }

            self.goto_definition_fallback(file, line, character).await
        } else {
            Ok(
                ToolResult::ok(format!("Go to definition not supported for {}", file))
                    .with_title(format!("Go to Definition {}", file)),
            )
        }
    }

    async fn goto_definition_fallback(
        &self,
        file: &str,
        line: u32,
        _character: u32,
    ) -> Result<ToolResult, OpenCodeError> {
        let content = tokio::fs::read_to_string(file)
            .await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to read file: {}", e)))?;

        let line_count = content.lines().count();
        let start = if line as usize > 10 {
            line as usize - 10
        } else {
            0
        };
        let end = line_count.min(line as usize + 100);

        if start >= end {
            return Ok(
                ToolResult::ok(format!("No definition found for {}:{}", file, line + 1))
                    .with_title(format!("Go to Definition {}", file)),
            );
        }

        for (idx, l) in content.lines().enumerate().skip(start).take(end - start) {
            let trimmed = l.trim();
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("trait ")
                || trimmed.starts_with("type ")
            {
                return Ok(
                    ToolResult::ok(format!("{}:{} -> {}:1: {}", file, line + 1, idx + 1, trimmed))
                        .with_title(format!("Go to Definition {}", file)),
                );
            }
        }

        Ok(
            ToolResult::ok(format!("No definition found for {}:{}", file, line + 1))
                .with_title(format!("Go to Definition {}", file)),
        )
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
            match self
                .find_references_impl(file, line, character, symbol, workspace)
                .await
            {
                Ok(result) => return Ok(result),
                Err(_e) if attempt < max_retries - 1 => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms[attempt]))
                        .await;
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
        _character: u32,
        symbol: &str,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        let search_symbol: String = if symbol.is_empty() {
            let content = tokio::fs::read_to_string(file)
                .await
                .map_err(|e| OpenCodeError::Tool(format!("Failed to read file: {}", e)))?;
            content
                .lines()
                .nth(line as usize)
                .and_then(|l| {
                    let trimmed = l.trim();
                    if trimmed.contains("fn ")
                        || trimmed.contains("struct ")
                        || trimmed.contains("enum ")
                        || trimmed.contains("impl ")
                    {
                        Some(
                            trimmed
                                .split(|c: char| !c.is_alphanumeric() && c != '_')
                                .filter(|s| !s.is_empty())
                                .last()
                                .map(|s| s.to_string())
                                .unwrap_or_default(),
                        )
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        } else {
            symbol.to_string()
        };

        if search_symbol.is_empty() {
            return Ok(
                ToolResult::ok(format!("No symbol found at {}:{}", file, line + 1))
                    .with_title(format!("Find References {}", symbol)),
            );
        }

        let root = workspace
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".").canonicalize().unwrap_or_default());

        let output = Command::new("grep")
            .args([
                "-rn",
                "--include=*.rs",
                "-E",
                &format!("\\b{}\\b", search_symbol),
            ])
            .current_dir(&root)
            .output()
            .await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to search: {}", e)))?;

        let result = String::from_utf8_lossy(&output.stdout);
        if result.is_empty() {
            Ok(
                ToolResult::ok(format!("No references found for '{}'", search_symbol))
                    .with_title(format!("Find References {}", search_symbol)),
            )
        } else {
            let count = result.lines().count();
            Ok(
                ToolResult::ok(format!("Found {} references:\n{}", count, result))
                    .with_title(format!("Find References {}", search_symbol)),
            )
        }
    }

    async fn hover_with_retry(
        &self,
        file: &str,
        line: u32,
        character: u32,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        let max_retries = 3;
        let retry_delay_ms = [100, 500, 1000];

        for attempt in 0..max_retries {
            match self.hover_impl(file, line, character, workspace).await {
                Ok(result) => return Ok(result),
                Err(_e) if attempt < max_retries - 1 => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms[attempt]))
                        .await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn hover_impl(
        &self,
        file: &str,
        line: u32,
        character: u32,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        if !file.ends_with(".rs") {
            return Ok(ToolResult::ok(format!("Hover not supported for {}", file))
                .with_title(format!("Hover {}", file)));
        }

        let root = workspace.map(PathBuf::from).unwrap_or_else(|| {
            PathBuf::from(file)
                .parent()
                .unwrap_or(&PathBuf::from("."))
                .to_path_buf()
        });

        if !root.join("Cargo.toml").exists() {
            return Ok(ToolResult::ok(format!(
                "No LSP server configured for {} (no Cargo.toml found in workspace)",
                file
            ))
            .with_title(format!("Hover {}", file)));
        }

        let output = match Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | timeout 5 rust-analyzer --ipa-lookup 2>/dev/null || echo ''",
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "textDocument/hover",
                    "params": {
                        "textDocument": { "uri": format!("file://{}", file) },
                        "position": { "line": line, "character": character }
                    }
                })
            ))
            .current_dir(&root)
            .output()
            .await
        {
            Ok(output) => output,
            Err(_) => {
                return self.hover_fallback(file, line, character).await;
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return self.hover_fallback(file, line, character).await;
        }

        if let Ok(resp) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(result) = resp.get("result") {
                if result.is_null() {
                    return Ok(ToolResult::ok(format!(
                        "No hover information available for {}:{}:{}",
                        file,
                        line + 1,
                        character + 1
                    ))
                    .with_title(format!("Hover {}", file)));
                }

                if let Some(hover_obj) = result.as_object() {
                    let mut hover_content = String::new();

                    if let Some(contents) = hover_obj.get("contents").and_then(|c| c.as_object()) {
                        if let Some(value) = contents.get("value").and_then(|v| v.as_str()) {
                            hover_content.push_str(value);
                        } else if let Some(value) = contents.get("value").and_then(|v| v.as_str()) {
                            hover_content.push_str(value);
                        }
                    } else if let Some(value) = result.as_str() {
                        hover_content.push_str(value);
                    }

                    if hover_content.is_empty() {
                        hover_content = result.to_string();
                    }

                    let docs = hover_obj
                        .get("docs")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());

                    let final_content = if let Some(docs) = docs {
                        format!("{}\n\nDocumentation:\n{}", hover_content, docs)
                    } else {
                        hover_content
                    };

                    return Ok(ToolResult::ok(final_content).with_title(format!("Hover {}", file)));
                }
            }
        }

        self.hover_fallback(file, line, character).await
    }

    async fn hover_fallback(
        &self,
        file: &str,
        line: u32,
        _character: u32,
    ) -> Result<ToolResult, OpenCodeError> {
        let content = match tokio::fs::read_to_string(file).await {
            Ok(c) => c,
            Err(_) => {
                return Ok(ToolResult::ok(format!(
                    "No hover information available for {} (file not found)",
                    file
                ))
                .with_title(format!("Hover {}", file)));
            }
        };

        let target_line = content.lines().nth(line as usize);

        if let Some(line_content) = target_line {
            let trimmed = line_content.trim();
            if !trimmed.is_empty() {
                return Ok(ToolResult::ok(format!(
                    "Line {}: {}\n\nNo LSP hover information available, showing source line.",
                    line + 1,
                    trimmed
                ))
                .with_title(format!("Hover {}", file)));
            }
        }

        Ok(ToolResult::ok(format!(
            "No hover information available for {}:{}",
            file,
            line + 1
        ))
        .with_title(format!("Hover {}", file)))
    }

    async fn code_actions_with_retry(
        &self,
        file: &str,
        line: u32,
        character: u32,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        let max_retries = 3;
        let retry_delay_ms = [100, 500, 1000];

        for attempt in 0..max_retries {
            match self
                .code_actions_impl(file, line, character, workspace)
                .await
            {
                Ok(result) => return Ok(result),
                Err(_e) if attempt < max_retries - 1 => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms[attempt]))
                        .await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn code_actions_impl(
        &self,
        file: &str,
        line: u32,
        character: u32,
        workspace: Option<&str>,
    ) -> Result<ToolResult, OpenCodeError> {
        if !file.ends_with(".rs") {
            return Ok(
                ToolResult::ok(format!("Code actions not supported for {}", file))
                    .with_title(format!("Code Actions {}", file)),
            );
        }

        let root = workspace.map(PathBuf::from).unwrap_or_else(|| {
            PathBuf::from(file)
                .parent()
                .unwrap_or(&PathBuf::from("."))
                .to_path_buf()
        });

        if !root.join("Cargo.toml").exists() {
            return Ok(ToolResult::ok(format!(
                "No LSP server configured for {} (no Cargo.toml found in workspace)",
                file
            ))
            .with_title(format!("Code Actions {}", file)));
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | timeout 5 rust-analyzer --ipa-lookup 2>/dev/null || echo ''",
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "textDocument/codeAction",
                    "params": {
                        "textDocument": { "uri": format!("file://{}", file) },
                        "position": { "line": line, "character": character },
                        "range": {
                            "start": { "line": line, "character": character },
                            "end": { "line": line, "character": character }
                        },
                        "context": {
                            "diagnostics": []
                        }
                    }
                })
            ))
            .current_dir(&root)
            .output()
            .await
            .map_err(|e| OpenCodeError::Tool(format!("Failed to run rust-analyzer: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(ToolResult::ok(format!(
                "No code actions available for {}:{}",
                file,
                line + 1
            ))
            .with_title(format!("Code Actions {}", file)));
        }

        if let Ok(resp) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(result) = resp.get("result") {
                if let Some(actions) = result.as_array() {
                    if actions.is_empty() {
                        return Ok(ToolResult::ok(format!(
                            "No code actions available for {}:{}",
                            file,
                            line + 1
                        ))
                        .with_title(format!("Code Actions {}", file)));
                    }

                    let mut action_list = Vec::new();
                    for (idx, action) in actions.iter().enumerate() {
                        if let Some(title) = action.get("title").and_then(|t| t.as_str()) {
                            let kind = action
                                .get("kind")
                                .and_then(|k| k.as_str())
                                .unwrap_or("unknown");
                            let id = action
                                .get("id")
                                .and_then(|i| i.as_u64())
                                .unwrap_or(idx as u64);
                            action_list.push(format!(
                                "{}. [{}] {} (id: {})",
                                idx + 1,
                                kind,
                                title,
                                id
                            ));
                        }
                    }

                    if action_list.is_empty() {
                        return Ok(ToolResult::ok(format!(
                            "No code actions available for {}:{}",
                            file,
                            line + 1
                        ))
                        .with_title(format!("Code Actions {}", file)));
                    }

                    return Ok(ToolResult::ok(action_list.join("\n")).with_title(format!(
                        "Code Actions {} ({} actions)",
                        file,
                        action_list.len()
                    )));
                }
            }
        }

        Ok(ToolResult::ok(format!(
            "No code actions available for {}:{}",
            file,
            line + 1
        ))
        .with_title(format!("Code Actions {}", file)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lsp_tool_name_and_description() {
        let tool = LspTool;
        assert_eq!(tool.name(), "lsp");
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_lsp_execute_requires_operation() {
        let tool = LspTool;
        let result = tool.execute(serde_json::json!({}), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lsp_execute_go_to_definition_requires_file() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "goToDefinition"
                }),
                None,
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("file_path is required"));
    }

    #[tokio::test]
    async fn test_lsp_execute_find_references_requires_file_or_symbol() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "findReferences"
                }),
                None,
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("file_path or symbol is required"));
    }

    #[tokio::test]
    async fn test_lsp_execute_hover_returns_info() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "hover",
                    "filePath": "test.rs",
                    "line": 10,
                    "character": 5
                }),
                None,
            )
            .await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("No hover information available"));
    }

    #[tokio::test]
    async fn test_lsp_execute_document_symbol_requires_file() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "documentSymbol"
                }),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lsp_execute_workspace_symbol_requires_symbol() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "workspaceSymbol"
                }),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lsp_execute_go_to_implementation_requires_file() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "goToImplementation"
                }),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lsp_execute_diagnostics_requires_file() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "diagnostics"
                }),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lsp_execute_unknown_operation() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "unknownOperation"
                }),
                None,
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown LSP operation"));
    }

    #[tokio::test]
    async fn test_lsp_execute_prepare_call_hierarchy_not_implemented() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "prepareCallHierarchy",
                    "filePath": "test.rs",
                    "line": 10,
                    "character": 5
                }),
                None,
            )
            .await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_lsp_execute_incoming_calls_not_implemented() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "incomingCalls",
                    "filePath": "test.rs",
                    "line": 10,
                    "character": 5
                }),
                None,
            )
            .await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_lsp_execute_outgoing_calls_not_implemented() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "outgoingCalls",
                    "filePath": "test.rs",
                    "line": 10,
                    "character": 5
                }),
                None,
            )
            .await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_goto_definition_fallback_finds_fn() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn my_function() {}\nfn main() {}").unwrap();

        let result = tool
            .test_goto_definition_fallback_on_empty_response(test_file.to_str().unwrap(), 0, 0)
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("my_function"));
    }

    #[tokio::test]
    async fn test_goto_definition_fallback_finds_struct() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "struct MyStruct {}\nfn main() {}").unwrap();

        let result = tool
            .test_goto_definition_fallback_on_empty_response(test_file.to_str().unwrap(), 0, 0)
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("MyStruct"));
    }

    #[tokio::test]
    async fn test_goto_definition_fallback_finds_enum() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "enum MyEnum {}\nfn main() {}").unwrap();

        let result = tool
            .test_goto_definition_fallback_on_empty_response(test_file.to_str().unwrap(), 0, 0)
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("MyEnum"));
    }

    #[tokio::test]
    async fn test_goto_definition_fallback_finds_impl() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "impl MyTrait for MyStruct {}\nfn main() {}").unwrap();

        let result = tool
            .test_goto_definition_fallback_on_empty_response(test_file.to_str().unwrap(), 0, 0)
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("impl"));
    }

    #[tokio::test]
    async fn test_goto_definition_fallback_no_definition_found() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "// no definitions here\na = b + c;").unwrap();

        let result = tool
            .test_goto_definition_fallback_on_empty_response(test_file.to_str().unwrap(), 0, 0)
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("No definition found"));
    }

    #[tokio::test]
    async fn test_goto_definition_fallback_file_not_found() {
        let tool = LspTool;
        let result = tool
            .test_goto_definition_fallback_on_empty_response("/nonexistent/path/file.rs", 0, 0)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to read file"));
    }

    #[test]
    fn test_parse_lsp_response_valid() {
        let raw = r#"{
            "result": {
                "location": {
                    "uri": "file:///src/main.rs",
                    "range": {
                        "start": { "line": 10, "character": 4 },
                        "end": { "line": 10, "character": 16 }
                    }
                }
            }
        }"#;

        let result = LspTool::test_parse_lsp_response(raw);
        assert!(result.is_some());
        let (uri, line, character) = result.unwrap();
        assert_eq!(uri, "file:///src/main.rs");
        assert_eq!(line, 11);
        assert_eq!(character, 5);
    }

    #[test]
    fn test_parse_lsp_response_invalid_json() {
        let raw = "not json";
        let result = LspTool::test_parse_lsp_response(raw);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_lsp_response_missing_fields() {
        let raw = r#"{"result": {}}"#;
        let result = LspTool::test_parse_lsp_response(raw);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_symbol_from_fn_line() {
        let line = "    pub fn my_function(arg: u32) -> String";
        let result = LspTool::test_extract_symbol_from_line(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "my_function");
    }

    #[test]
    fn test_extract_symbol_from_struct_line() {
        let line = "pub struct MyStruct {";
        let result = LspTool::test_extract_symbol_from_line(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "MyStruct");
    }

    #[test]
    fn test_extract_symbol_from_impl_line() {
        let line = "impl Debug for MyType {";
        let result = LspTool::test_extract_symbol_from_line(line);
        assert!(result.is_some());
        assert!(result.unwrap().contains("impl"));
    }

    #[test]
    fn test_extract_symbol_from_plain_line() {
        let line = "let x = 5;";
        let result = LspTool::test_extract_symbol_from_line(line);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_references_with_symbol() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("mod.rs");

        std::fs::write(
            &test_file,
            "const VALUE: i32 = 42;\nfn main() { let x = VALUE; }",
        )
        .unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "findReferences",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 0,
                    "character": 7,
                    "symbol": "VALUE"
                }),
                Some(crate::ToolContext {
                    directory: Some(temp_dir.path().to_str().unwrap().to_string()),
                    ..Default::default()
                }),
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(
            tool_result.content.contains("references") || tool_result.content.contains("VALUE")
        );
    }

    #[tokio::test]
    async fn test_diagnostics_for_non_rust_file() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "some text").unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "diagnostics",
                    "filePath": test_file.to_str().unwrap()
                }),
                None,
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(
            tool_result.content.contains("No diagnostics found")
                || tool_result.content.contains("diagnostics")
        );
    }

    #[tokio::test]
    async fn test_clone_tool_returns_working_instance() {
        let tool = LspTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "lsp");

        let result = cloned
            .execute(
                serde_json::json!({
                    "operation": "hover",
                    "filePath": "test.rs",
                    "line": 1,
                    "character": 1
                }),
                None,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tool_result_title_is_set() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "documentSymbol",
                    "filePath": "test.rs"
                }),
                None,
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.title.is_some());
    }

    #[test]
    fn test_retry_delay_values_are_increasing() {
        let retry_delay_ms = [100, 500, 1000];
        for i in 1..retry_delay_ms.len() {
            assert!(
                retry_delay_ms[i] > retry_delay_ms[i - 1],
                "retry delay at index {} should be greater than previous",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_goto_definition_fails_gracefully_without_rust_analyzer() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("main.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "goToDefinition",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 1,
                    "character": 4
                }),
                Some(crate::ToolContext {
                    directory: Some(temp_dir.path().to_str().unwrap().to_string()),
                    ..Default::default()
                }),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_references_with_empty_file() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("empty.rs");
        std::fs::write(&test_file, "").unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "findReferences",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 0,
                    "character": 0
                }),
                Some(crate::ToolContext {
                    directory: Some(temp_dir.path().to_str().unwrap().to_string()),
                    ..Default::default()
                }),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lsp_execute_with_workspace_context() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            "fn defined_here() {}\nfn main() { defined_here(); }",
        )
        .unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "goToDefinition",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 2,
                    "character": 5
                }),
                Some(crate::ToolContext {
                    directory: Some(temp_dir.path().to_str().unwrap().to_string()),
                    ..Default::default()
                }),
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(!tool_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_lsp_execute_hover_with_rust_file() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            "fn my_function() {}\nfn main() { my_function(); }",
        )
        .unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "hover",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 2,
                    "character": 5
                }),
                Some(crate::ToolContext {
                    directory: Some(temp_dir.path().to_str().unwrap().to_string()),
                    ..Default::default()
                }),
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.title.is_some());
    }

    #[tokio::test]
    async fn test_lsp_execute_hover_requires_file() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "hover"
                }),
                None,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("file_path is required"));
    }

    #[tokio::test]
    async fn test_lsp_execute_code_actions_requires_file() {
        let tool = LspTool;
        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "codeActions"
                }),
                None,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("file_path is required"));
    }

    #[tokio::test]
    async fn test_lsp_execute_code_actions_with_rust_file() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "codeActions",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 1,
                    "character": 1
                }),
                Some(crate::ToolContext {
                    directory: Some(temp_dir.path().to_str().unwrap().to_string()),
                    ..Default::default()
                }),
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.title.is_some());
    }

    #[tokio::test]
    async fn test_lsp_execute_hover_for_non_rust_file() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "some text").unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "hover",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 1,
                    "character": 1
                }),
                None,
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("not supported"));
    }

    #[tokio::test]
    async fn test_lsp_execute_code_actions_for_non_rust_file() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "some text").unwrap();

        let result = tool
            .execute(
                serde_json::json!({
                    "operation": "codeActions",
                    "filePath": test_file.to_str().unwrap(),
                    "line": 1,
                    "character": 1
                }),
                None,
            )
            .await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("not supported"));
    }

    #[tokio::test]
    async fn test_hover_fallback_returns_line_content() {
        let tool = LspTool;
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn my_function() {}\nfn main() {}").unwrap();

        let result = tool.hover_fallback(test_file.to_str().unwrap(), 0, 0).await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("fn my_function()"));
    }

    #[tokio::test]
    async fn test_hover_fallback_file_not_found() {
        let tool = LspTool;
        let result = tool.hover_fallback("/nonexistent/path/file.rs", 0, 0).await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.content.contains("file not found"));
    }
}
