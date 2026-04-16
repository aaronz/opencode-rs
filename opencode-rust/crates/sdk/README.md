# OpenCode SDK

Rust SDK for OpenCode RS - Programmatic access to OpenCode AI coding agent capabilities.

[![crates.io](https://img.shields.io/crates/v/opencode-sdk.svg)](https://crates.io/crates/opencode-sdk)
[![Documentation](https://docs.rs/opencode-sdk/badge.svg)](https://docs.rs/opencode-sdk)

## Features

- **Session Management**: Create, load, save, fork, and abort conversation sessions
- **Tool Execution**: Execute and list available tools from the OpenCode tool registry
- **Auth Integration**: API key authentication for secure API access
- **Error Handling**: Structured error types with error codes (1xxx-9xxx ranges)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
opencode-sdk = "0.1"
```

### TLS Support

By default, the SDK uses `native-tls` for TLS. If you prefer `rustls`:

```toml
[dependencies]
opencode-sdk = { version = "0.1", default-features = false, features = ["rustls-tls"] }
```

## Quick Start

```rust
use opencode_sdk::OpenCodeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenCodeClient::builder()
        .api_key("your-api-key")
        .build()?;

    let session = client.create_session(Some("Hello, world!")).await?;
    println!("Created session: {}", session.session_id);

    Ok(())
}
```

## Usage

### Client Configuration

```rust
use opencode_sdk::{OpenCodeClient, ClientConfig};
use std::time::Duration;

let client = OpenCodeClient::builder()
    .base_url("http://localhost:8080/api")  // Default: http://localhost:8080/api
    .api_key("sk-your-api-key")
    .timeout(Duration::from_secs(60))       // Default: 30 seconds
    .skip_tls_verification(true)           // Default: false (for development only)
    .build()?;
```

### Session Management

```rust
// Create a new session
let session = client.create_session(Some("Initial prompt")).await?;

// Get a session by ID
let session = client.get_session("session-id").await?;

// List all sessions
let sessions = client.list_sessions(Some(10), Some(0)).await?;

// Fork a session
let forked = client.fork_session("session-id", 0).await?;

// Abort a session
client.abort_session("session-id").await?;

// Delete a session
client.delete_session("session-id").await?;
```

### Messages

```rust
// Add a message to a session
let response = client
    .add_message("session-id", Some("user"), "Hello, can you help me?")
    .await?;
```

### Tool Execution

```rust
use opencode_sdk::tools::ToolCall;

// List available tools
let tools = client.list_tools().await?;
for tool in &tools {
    println!("{}: {}", tool.name, tool.description);
}

// Execute a tool
let tool_call = ToolCall::new("read", serde_json::json!({
    "file_path": "Cargo.toml"
}));
let result = client.execute_tool(tool_call).await?;
```

### Local Session (Offline Mode)

The SDK also supports offline local sessions without requiring a server:

```rust
// Create a local session
client.create_local_session(Some("Hello")).await?;

// Get the local session
if let Some(session) = client.get_local_session().await? {
    println!("Local session: {:?}", session);
}

// Add messages to local session
client.add_local_message("user", "Hello!").await?;
```

## Error Handling

The SDK uses structured errors with error codes:

```rust
use opencode_sdk::{OpenCodeClient, SdkError};

match client.create_session(None).await {
    Ok(session) => println!("Created: {}", session.session_id),
    Err(SdkError::AuthenticationFailed { detail, code }) => {
        eprintln!("Auth failed ({}): {}", code, detail);
    }
    Err(SdkError::SessionNotFound { id, code }) => {
        eprintln!("Session not found ({}): {}", code, id);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Error Code Ranges

| Range | Category |
|-------|----------|
| 1xxx | Authentication |
| 2xxx | Authorization |
| 3xxx | Provider |
| 4xxx | Tool |
| 5xxx | Session |
| 6xxx | Config |
| 7xxx | Validation |
| 9xxx | Internal |

## Examples

See the `examples/` directory for working examples:

- `basic.rs` - Complete usage example covering sessions, messages, and tools

Run the example:

```bash
OPENCODE_API_KEY=your-api-key cargo run --example basic
```

## Minimum Supported Rust Version

This crate requires Rust 1.75 or later.

## License

MIT OR Apache-2.0