//! Basic example of using the OpenCode SDK.
//!
//! This example demonstrates:
//! - Creating a client
//! - Creating a session
//! - Adding messages
//! - Executing tools
//! - Listing sessions

use opencode_sdk::tools::ToolCall;
use opencode_sdk::OpenCodeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("OpenCode SDK Basic Example");
    eprintln!("Note: This example requires a running OpenCode server");

    // Create the client
    println!("Creating OpenCode client...");
    let client = OpenCodeClient::builder()
        .base_url("http://localhost:8080/api")
        .api_key("sk-your-api-key-here")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    println!("Client created successfully!");
    println!("  Base URL: {}", client.config().base_url);

    // Create a new session with an initial prompt
    println!("\nCreating a new session...");
    let session = client
        .create_session(Some("Hello, OpenCode! Can you help me with a task?"))
        .await?;

    println!("Session created!");
    println!("  Session ID: {}", session.session_id);
    println!("  Status: {}", session.status);
    println!("  Message count: {}", session.message_count);

    // Add a follow-up message
    println!("\nAdding a follow-up message...");
    let response = client
        .add_message(
            &session.session_id.to_string(),
            Some("user"),
            "Can you list the files in the current directory?",
        )
        .await?;

    println!("Message added!");
    println!("  New message count: {}", response.message_count);

    // List all sessions
    println!("\nListing all sessions...");
    let sessions = client.list_sessions(Some(10), Some(0)).await?;

    println!("Found {} sessions:", sessions.len());
    for s in &sessions {
        println!("  - {} ({} messages)", s.id, s.message_count);
    }

    // List available tools
    println!("\nListing available tools...");
    let tools = client.list_tools().await?;

    println!("Found {} tools:", tools.len());
    for tool in &tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Execute a tool (if the server supports it)
    println!("\nAttempting to execute 'read' tool...");
    let tool_call = ToolCall::new("read", serde_json::json!({ "file_path": "Cargo.toml" }));

    match client.execute_tool(tool_call).await {
        Ok(result) => {
            println!("Tool executed successfully!");
            println!("  Success: {}", result.is_success());
            if result.is_success() {
                let result_str = result.result.as_deref().unwrap_or("");
                println!("  Result: {}", &result_str[..result_str.len().min(200)]);
            }
        }
        Err(e) => {
            println!("Tool execution failed: {}", e);
            println!("  (This is expected if the server is not running or tool is not available)");
        }
    }

    // Fork the session (create a branch)
    println!("\nForking the session at message index 0...");
    match client
        .fork_session(&session.session_id.to_string(), 0)
        .await
    {
        Ok(forked) => {
            println!("Session forked!");
            println!("  New Session ID: {}", forked.id);
            println!("  Parent Session ID: {:?}", forked.parent_session_id);
        }
        Err(e) => {
            println!("Fork failed: {}", e);
        }
    }

    // Abort the original session
    println!("\nAborting the original session...");
    match client.abort_session(&session.session_id.to_string()).await {
        Ok(()) => {
            println!("Session aborted successfully!");
        }
        Err(e) => {
            println!("Abort failed: {}", e);
        }
    }

    println!("\nDone!");

    Ok(())
}
