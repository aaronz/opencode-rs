//! Tool execution example for the OpenCode SDK.
//!
//! This example demonstrates:
//! - Creating a local tool registry
//! - Registering a custom echo tool
//! - Executing the Read tool (built-in)
//! - Executing the Write tool (built-in)
//! - Handling tool results and errors

use anyhow::{Context, Result};
use opencode_sdk::tools::{ToolCall, ToolParameter, ToolRegistry, ToolResult};

#[tokio::main]
async fn main() -> Result<()> {
    println!("OpenCode SDK - Tool Execution Example");
    println!("=====================================\n");

    let mut registry = ToolRegistry::new();

    println!("1. Registering a custom 'echo' tool...");
    registry.register_tool(
        "echo",
        "Echoes the input message back",
        vec![ToolParameter {
            name: "message".to_string(),
            description: "Message to echo back".to_string(),
            required: true,
            schema: serde_json::json!({"type": "string"}),
        }],
        |args| {
            let message = args["message"]
                .as_str()
                .ok_or("Missing required parameter: message")?;
            Ok(format!("Echo: {}", message))
        },
    );
    println!("   Custom 'echo' tool registered successfully!");
    println!("   Registered tools: {:?}\n", registry.list_tools());

    println!("2. Executing the custom 'echo' tool...");
    let echo_result =
        registry.execute_tool("echo", serde_json::json!({ "message": "Hello, OpenCode!" }));
    handle_tool_result("echo", &echo_result);

    println!("3. Executing 'echo' tool with missing parameter...");
    let echo_error = registry.execute_tool("echo", serde_json::json!({}));
    handle_tool_result("echo", &echo_error);

    println!("\n4. Executing 'echo' tool with wrong name (tool not found)...");
    let not_found_result =
        registry.execute_tool("nonexistent", serde_json::json!({ "message": "test" }));
    handle_tool_result("nonexistent", &not_found_result);

    println!("\n5. Simulating Read tool execution (local filesystem)...");
    let test_file = "/tmp/opencode_sdk_test_read.txt";
    std::fs::write(test_file, "Hello from OpenCode SDK!").context("Failed to write test file")?;
    let read_executor = opencode_sdk::ToolExecutor::new(|args| {
        let file_path = args["file_path"]
            .as_str()
            .ok_or("Missing required parameter: file_path")?;
        std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))
    });
    match read_executor.execute(serde_json::json!({ "file_path": test_file })) {
        Ok(content) => println!("   Read result: {}", content.trim()),
        Err(e) => eprintln!("   Read error: {}", e),
    }

    println!("\n6. Simulating Write tool execution (local filesystem)...");
    let write_executor = opencode_sdk::ToolExecutor::new(|args| {
        let file_path = args["file_path"]
            .as_str()
            .ok_or("Missing required parameter: file_path")?;
        let content = args["content"]
            .as_str()
            .ok_or("Missing required parameter: content")?;
        std::fs::write(file_path, content).map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(format!("Successfully wrote to {}", file_path))
    });
    let output_file = "/tmp/opencode_sdk_test_write.txt";
    match write_executor.execute(serde_json::json!({
        "file_path": output_file,
        "content": "Written via OpenCode SDK!"
    })) {
        Ok(msg) => println!("   Write result: {}", msg),
        Err(e) => eprintln!("   Write error: {}", e),
    }

    println!("\n7. Executing tools via OpenCodeClient (requires server)...");
    println!("   Note: This demonstrates the API-based tool execution flow");
    let client = opencode_sdk::OpenCodeClient::builder()
        .base_url("http://localhost:8080/api")
        .api_key(std::env::var("OPENCODE_API_KEY").unwrap_or_else(|_| "sk-dev-key".to_string()))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create client")?;

    let tool_call = ToolCall::new("read", serde_json::json!({ "file_path": "Cargo.toml" }));
    match client.execute_tool(tool_call).await {
        Ok(result) => {
            println!("   Tool executed via API!");
            println!("   Success: {}", result.is_success());
            if result.is_success() {
                let content = result.result.as_deref().unwrap_or("");
                if content.len() > 100 {
                    println!("   Result (truncated): {}...", &content[..100]);
                } else {
                    println!("   Result: {}", content);
                }
            } else {
                println!(
                    "   Error: {}",
                    result.error.as_deref().unwrap_or("Unknown error")
                );
            }
        }
        Err(e) => {
            println!(
                "   Tool execution via API failed (expected if server not running): {}",
                e
            );
        }
    }

    let _ = std::fs::remove_file(test_file);
    let _ = std::fs::remove_file(output_file);

    println!("\nDone!");

    Ok(())
}

fn handle_tool_result(name: &str, result: &ToolResult) {
    if result.is_success() {
        println!("   [{}] Success: {}", name, result.unwrap_result());
    } else {
        println!("   [{}] Error: {}", name, result.unwrap_error());
    }
}
