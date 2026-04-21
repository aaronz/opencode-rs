# OpenCode SDK

Rust SDK for OpenCode RS - Programmatic access to OpenCode capabilities.

## Features

- **Session Management**: Create, load, save, fork, and abort sessions
- **Tool Execution**: Execute and list available tools
- **Auth Integration**: API key authentication
- **Error Handling**: Structured error types with error codes (1xxx-9xxx)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
opencode-sdk = "0.1.0"
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

## Error Codes

| Range | Category |
|-------|----------|
| 1xxx  | Authentication |
| 2xxx  | Authorization |
| 3xxx  | Provider |
| 4xxx  | Tool |
| 5xxx  | Session |
| 6xxx  | Config |
| 7xxx  | Validation |
| 9xxx  | Internal |

## Documentation

- [SDK Guide](../../docs/sdk-guide.md) - Comprehensive guide to using the SDK

## License

MIT