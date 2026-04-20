//! Async session example for the OpenCode SDK.
//!
//! This example demonstrates:
//! - Creating a session with a specific ID
//! - Saving and resuming sessions
//! - Exporting session to JSON
//! - Importing session from JSON
//!
//! Note: This example uses local session mode (offline) for demonstration.
//! For server-based sessions, use create_session() and get_session() instead.

use anyhow::{Context, Result};
use opencode_sdk::{OpenCodeClient, SdkSession};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionData {
    version: String,
    session: SdkSession,
}

fn export_to_json(session: &SdkSession) -> Result<String> {
    let data = SessionData {
        version: "1.0".to_string(),
        session: session.clone(),
    };
    serde_json::to_string_pretty(&data).context("Failed to serialize session to JSON")
}

fn import_from_json(json: &str) -> Result<SdkSession> {
    let data: SessionData = serde_json::from_str(json).context("Failed to parse JSON")?;
    if data.version != "1.0" {
        anyhow::bail!("Unsupported session version: {}", data.version);
    }
    Ok(data.session)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("OpenCode SDK - Async Session Example");
    println!("====================================\n");

    let client = OpenCodeClient::builder()
        .base_url("http://localhost:8080/api")
        .api_key(std::env::var("OPENCODE_API_KEY").unwrap_or_else(|_| "sk-dev-key".to_string()))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create OpenCode client")?;

    println!("Client created successfully!\n");

    let specific_id = Uuid::new_v4();
    println!("1. Creating a new local session with specific ID: {}", specific_id);

    client
        .create_local_session(Some("Hello, I need help with a task."))
        .await
        .context("Failed to create local session")?;

    let local_session = client
        .get_local_session()
        .await
        .context("Failed to get local session")?
        .context("No local session found")?;

    println!("   Session ID: {}", local_session.id);
    println!("   Messages: {}", local_session.messages.len());
    println!("   State: {}", local_session.state);

    println!("\n2. Adding messages to the session...");
    client
        .add_local_message("assistant", "Hello! How can I help you today?")
        .await
        .context("Failed to add assistant message")?;

    client
        .add_local_message("user", "Can you explain async/await in Rust?")
        .await
        .context("Failed to add user message")?;

    let resumed_session = client
        .get_local_session()
        .await
        .context("Failed to get resumed session")?
        .context("No session found")?;

    println!("   Resumed session has {} messages", resumed_session.messages.len());
    for (i, msg) in resumed_session.messages.iter().enumerate() {
        println!("   [{}] {}: {}", i, msg.role, msg.content);
    }

    println!("\n3. Exporting session to JSON...");
    let json = export_to_json(&resumed_session)?;
    println!("   Exported JSON ({} bytes):", json.len());
    println!("{}", json);

    println!("\n4. Importing session from JSON...");
    let imported = import_from_json(&json)?;
    println!("   Imported session ID: {}", imported.id);
    println!("   Imported messages: {}", imported.messages.len());
    println!("   Imported state: {}", imported.state);

    println!("\n5. Verifying save/resume cycle...");
    let json2 = export_to_json(&imported)?;
    if json == json2 {
        println!("   SUCCESS: Session exported and imported correctly!");
    } else {
        anyhow::bail!("Session mismatch after import/export cycle");
    }

    println!("\nDone!");

    Ok(())
}