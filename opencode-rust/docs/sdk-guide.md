# OpenCode RS SDK Guide

The `opencode-sdk` crate provides a public async/await Rust API for programmatic access to OpenCode RS functionality.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
opencode-sdk = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

Or use `cargo add`:

```bash
cargo add opencode-sdk tokio --features full
```

## Examples

See [`crates/sdk/examples/`](https://github.com/opencode-rs/opencode-rs/tree/main/opencode-rust/crates/sdk/examples) for complete examples:

- [`basic.rs`](https://github.com/opencode-rs/opencode-rs/tree/main/opencode-rust/crates/sdk/examples/basic.rs) - Basic SDK usage
- [`basic_usage.rs`](https://github.com/opencode-rs/opencode-rs/tree/main/opencode-rust/crates/sdk/examples/basic_usage.rs) - Common usage patterns
- [`async_session.rs`](https://github.com/opencode-rs/opencode-rs/tree/main/opencode-rust/crates/sdk/examples/async_session.rs) - Async session management
- [`tool_execution.rs`](https://github.com/opencode-rs/opencode-rs/tree/main/opencode-rust/crates/sdk/examples/tool_execution.rs) - Custom tool registration and execution

## API Reference

Full API documentation available at:
- [`docs.rs/opencode-sdk`](https://docs.rs/opencode-sdk) - Main SDK documentation
- [`docs.rs/opencode-core`](https://docs.rs/opencode-core) - Core API reference

## Core Concepts

### Session

A `Session` represents a conversation with the agent, containing messages and metadata.

```rust
use opencode_sdk::{Session, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default()
        .with_llm_provider("openai")
        .with_api_key(std::env::var("OPENAI_API_KEY")?);

    let session = Session::new(config).await?;

    let response = session.execute("Hello, help me with Rust!").await?;
    println!("{}", response);

    Ok(())
}
```

### Tool Registry

Register and manage custom tools:

```rust
use opencode_sdk::{Tool, ToolRegistry};

struct MyTool;

#[opencode_sdk::async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "A custom tool" }

    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({ "result": "success" }))
    }
}

let mut registry = ToolRegistry::new();
registry.register(MyTool).await?;
```

### LLM Providers

Supported providers:
- OpenAI (GPT-4, GPT-3.5, GPT-4o)
- Anthropic (Claude Opus, Sonnet, Haiku)
- Ollama (local models)
- Azure, Google Gemini, AWS Bedrock, OpenRouter, Groq, Cohere, Mistral

```rust
use opencode_sdk::{LLMProvider, OpenAI};

let provider = OpenAI::new()
    .with_model("gpt-4")
    .with_temperature(0.7)
    .with_max_tokens(2048);
```

## Configuration

```rust
use opencode_sdk::Config;

let config = Config::builder()
    .with_llm_provider("openai")
    .with_api_key("sk-...")
    .with_database_path("./opencode.db")
    .with_server_port(3000)
    .build()?;
```

## Session Management

```rust
use opencode_sdk::{Session, ExportFormat};

// Create a new session
let session = Session::new(config).await?;

// Resume an existing session
let session = Session::resume(session_id, config).await?;

// Export session to JSON
let json = session.export(ExportFormat::Json).await?;

// Import session from JSON
let session = Session::import(json, config).await?;
```

## Async/Await Pattern

All SDK operations are async:

```rust
#[tokio::main]
async fn main() {
    let session = Session::new(config).await.unwrap();
    let result = session.execute("task").await.unwrap();
}
```