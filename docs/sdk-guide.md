# SDK Guide

OpenCode RS provides a Rust SDK for programmatic access to OpenCode capabilities. This guide covers installation, core concepts, and practical examples.

## Installation

### Add to Cargo.toml

```toml
[dependencies]
opencode-sdk = "0.1.0"
```

### Version Requirements

- Rust 1.70+
- TLS backend (native-tls or rustls)

### TLS Backend

By default, the SDK uses `native-tls`. To use `rustls`:

```toml
[dependencies]
opencode-sdk = { version = "0.1.0", default-features = false, features = ["rustls-tls"] }
```

## Core Concepts

### Client

The `OpenCodeClient` is the main entry point for SDK usage:

```rust
use opencode_sdk::OpenCodeClient;

let client = OpenCodeClient::builder()
    .api_key("your-api-key")
    .build()?;
```

### Session

A `Session` represents a conversation with the agent. Sessions can be:
- Created with an optional initial message
- Saved and resumed
- Exported to/imported from JSON
- Forked for parallel exploration

### Tool Registry

The SDK provides access to OpenCode's built-in tools:
- `Read` - Read file contents
- `Write` - Create or overwrite files
- `Edit` - Targeted file edits
- `Grep` - Regex-based search
- `Glob` - File pattern matching
- `Bash` - Shell command execution

### LLM Providers

Supported providers:
- **OpenAI** - GPT-4, GPT-3.5-turbo
- **Anthropic** - Claude 3 Opus, Sonnet, Haiku
- **Ollama** - Local LLM server

## Configuration

### Basic Configuration

```rust
use opencode_sdk::OpenCodeClient;

let client = OpenCodeClient::builder()
    .api_key("sk-...")
    .build()?;
```

### Provider Configuration

```rust
use opencode_sdk::{OpenCodeClient, LLMProvider};

let client = OpenCodeClient::builder()
    .provider(LLMProvider::OpenAI)
    .model("gpt-4")
    .api_key("sk-...")
    .temperature(0.7)
    .max_tokens(4096)
    .build()?;
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OLLAMA_BASE_URL` | Ollama server URL |

## Session Management

### Create a Session

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

### Save and Resume

```rust
// Save session
let session_json = session.export_to_json()?;

// Resume later
let restored = client.import_from_json(&session_json).await?;
```

### Session ID

Sessions are identified by UUID:

```rust
let session_id = session.session_id;
println!("Session ID: {}", session_id);
```

## Tool Execution

### Register Custom Tools

```rust
use opencode_sdk::{Tool, ToolRegistry};

struct EchoTool;

#[opencode_sdk::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes the input back"
    }

    async fn execute(&self, input: &str) -> Result<String, opencode_sdk::Error> {
        Ok(input.to_string())
    }
}

let mut registry = ToolRegistry::new();
registry.register(Box::new(EchoTool));
```

### Execute Built-in Tools

```rust
use opencode_sdk::OpenCodeClient;

let client = OpenCodeClient::builder()
    .api_key("your-api-key")
    .build()?;

let session = client.create_session(None).await?;

// Read a file
let content = session.execute_tool("read", serde_json::json!({
    "path": "Cargo.toml"
})).await?;
```

### List Available Tools

```rust
let tools = session.list_tools().await?;
for tool in tools {
    println!("{}: {}", tool.name, tool.description);
}
```

## Async/Await Pattern

The SDK is fully async and works with Tokio:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenCodeClient::builder()
        .api_key("your-api-key")
        .build()?;

    // Concurrent session creation
    let (session1, session2) = tokio::join!(
        client.create_session(Some("Task 1".to_string())),
        client.create_session(Some("Task 2".to_string()))
    );

    Ok(())
}
```

## Error Handling

The SDK uses structured error types with error codes:

```rust
use opencode_sdk::OpenCodeClient;

match client.create_session(None).await {
    Ok(session) => println!("Created: {}", session.session_id),
    Err(e) => {
        eprintln!("Error: {}", e);
        match e.code() {
            1001 => println!("Authentication failed"),
            5001 => println!("Session not found"),
            _ => println!("Unknown error"),
        }
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

The SDK includes several examples demonstrating key features:

- [`basic_usage`](examples/basic_usage.rs) - Simple client setup and session creation
- [`async_session`](examples/async_session.rs) - Session save/resume and JSON import/export
- [`tool_execution`](examples/tool_execution.rs) - Custom tool registration and execution
- [`provider_config`](examples/provider_config.rs) - Multiple LLM provider configuration

Run examples with:

```bash
cargo run --example basic_usage
cargo run --example async_session
cargo run --example tool_execution
cargo run --example provider_config
```

## API Reference

For detailed API documentation, see:
- [opencode-sdk on docs.rs](https://docs.rs/opencode-sdk)
- [crates/sdk/src/lib.rs](../../opencode-rust/crates/sdk/src/lib.rs)

## License

MIT