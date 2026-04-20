//! Basic usage example for the OpenCode SDK.
//!
//! This example demonstrates:
//! - Creating a client with ClientConfig
//! - Creating and executing a session
//! - Printing the response
//! - Handling errors with anyhow

use anyhow::{Context, Result};
use opencode_sdk::{ClientConfig, OpenCodeClient};

#[tokio::main]
async fn main() -> Result<()> {
    println!("OpenCode SDK - Basic Usage Example");
    println!("==================================\n");

    let config = ClientConfig {
        base_url: std::env::var("OPENCODE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080/api".to_string()),
        auth: opencode_sdk::ApiKeyAuth::new(
            std::env::var("OPENCODE_API_KEY").unwrap_or_else(|_| "sk-dev-key".to_string()),
        ),
        timeout: std::time::Duration::from_secs(30),
        skip_tls_verification: false,
    };

    println!("Creating client with config...");
    let client = OpenCodeClient::builder()
        .base_url(&config.base_url)
        .api_key(std::env::var("OPENCODE_API_KEY").unwrap_or_else(|_| "sk-dev-key".to_string()))
        .timeout(config.timeout)
        .build()
        .context("Failed to create OpenCode client")?;

    println!("Client created successfully!");
    println!("  Base URL: {}", client.config().base_url);

    println!("\nCreating a new session...");
    let session = client
        .create_session(Some("Hello, OpenCode!"))
        .await
        .context("Failed to create session")?;

    println!("Session created!");
    println!("  Session ID: {}", session.session_id);
    println!("  Status: {}", session.status);
    println!("  Message count: {}", session.message_count);

    let response = client
        .get_session(&session.session_id.to_string())
        .await
        .context("Failed to get session")?;

    println!("\nSession response retrieved!");
    println!("  Session ID: {}", response.id);
    println!("  Messages: {}", response.messages.len());

    println!("\nDone!");

    Ok(())
}
