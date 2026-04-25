# cli.md — CLI Module

> **User Documentation**: [cli.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx)
>
> This document describes the CLI implementation. For user-facing CLI documentation and command reference, see the user docs linked above.

## Module Overview

- **Crate**: `opencode-cli`
- **Source**: `crates/cli/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Command-line interface with subcommands, NDJSON output serialization for piping to other tools, and webview integration.

---

## Crate Layout

```
crates/cli/src/
├── lib.rs              ← Re-exports, tests
├── cmd/                ← Command implementations
├── output/             ← NdjsonSerializer for streaming output
└── webview.rs          ← Webview integration
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1.45", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

opencode-core = { path = "../core" }
opencode-config = { path = "../config" }
opencode-agent = { path = "../agent" }
```

---

## Core Types

### NdjsonSerializer

```rust
// Newline-delimited JSON (NDJSON) output for CLI streaming
pub struct NdjsonSerializer<W: Write> {
    writer: W,
}

impl<W: Write> NdjsonSerializer<W> {
    pub fn new(writer: W) -> Self;
    pub fn write_message(&mut self, role: &str, content: &str) -> Result<(), io::Error>;
    pub fn write_start(&mut self, model: &str) -> Result<(), io::Error>;
    pub fn write_chunk(&mut self, content: &str) -> Result<(), io::Error>;
    pub fn write_done(&mut self) -> Result<(), io::Error>;
    pub fn write_error(&mut self, error: &str) -> Result<(), io::Error>;
    pub fn write_tool_call(&mut self, tool: &str, args: &str) -> Result<(), io::Error>;
    pub fn write_tool_result(&mut self, tool: &str, result: &str) -> Result<(), io::Error>;
    pub fn flush(&mut self) -> Result<(), io::Error>;
}
```

### Output Events (NDJSON format)

```json
{"event": "start", "model": "gpt-4o"}
{"event": "message", "role": "user", "content": "Hello"}
{"event": "chunk", "content": "Hello"}
{"event": "tool_call", "tool": "read", "args": {"path": "foo.txt"}}
{"event": "tool_result", "tool": "read", "result": "file contents"}
{"event": "error", "error": "something went wrong"}
{"event": "done"}
```

### CLI Commands

```rust
// From cmd/
pub enum CliCommand {
    Run { model: Option<String>, prompt: Option<String> },
    Session { subcommand: SessionSubcommand },
    List,
    Info,
    Config { subcommand: ConfigSubcommand },
    Agent { agent_type: String, prompt: String },
    Web { port: Option<u16> },
    Serve { port: Option<u16> },
    Desktop { port: Option<u16> },
}

pub enum SessionSubcommand {
    List,
    Show { id: String },
    Delete { id: String },
    Export { id: String, format: String },
}

pub enum ConfigSubcommand {
    Get { key: String },
    Set { key: String, value: String },
    Show,
}
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-cli` |
|---|---|
| `opencode-core` (binary) | CLI argument parsing, command dispatch |

**Dependencies of `opencode-cli`**:
| Crate | Usage |
|---|---|
| `opencode-core` | `Config`, `Session` |
| `opencode-config` | `Config` loading |
| `opencode-agent` | `Agent` implementations for CLI mode |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::output::NdjsonSerializer;

    #[test]
    fn test_ndjson_serializer_write_message() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_message("user", "Hello").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"message\""));
        assert!(output.contains("\"role\":\"user\""));
        assert!(output.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_ndjson_serializer_write_start() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_start("gpt-4").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"start\""));
        assert!(output.contains("\"model\":\"gpt-4\""));
    }

    #[test]
    fn test_ndjson_serializer_write_error() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_error("something went wrong").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"error\""));
        assert!(output.contains("\"error\":\"something went wrong\""));
    }

    #[test]
    fn test_ndjson_serializer_write_tool_call() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_tool_call("read", r#"{"path": "foo.txt"}"#).unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"tool_call\""));
        assert!(output.contains("\"tool\":\"read\""));
    }

    #[test]
    fn test_ndjson_serializer_write_done() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_done().unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"done\""));
    }
}
```

---

## CLI Usage

```bash
# Start interactive TUI
opencode

# Run single prompt
opencode run "Explain this codebase"

# Session management
opencode session list
opencode session show <id>
opencode session delete <id>

# Config management
opencode config show
opencode config get agent.model

# Run specific agent type
opencode agent plan "What should I work on next?"

# Start server/webview
opencode serve --port 8080
opencode web --port 3000
opencode desktop --port 3000
```

## User-Facing CLI Commands Reference

| Command | Description | User Doc Section |
|---------|-------------|------------------|
| `opencode` | Start TUI (default) | [cli.mdx#tui](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#tui) |
| `opencode run` | Non-interactive mode with prompt | [cli.mdx#run](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#run) |
| `opencode serve` | Headless API server | [cli.mdx#serve](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#serve) |
| `opencode web` | Web interface server | [cli.mdx#web](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#web) |
| `opencode attach` | Connect TUI to remote server | [cli.mdx#attach](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#attach) |
| `opencode agent create` | Create custom agent | [cli.mdx#agent-create](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#agent-create) |
| `opencode agent list` | List available agents | [cli.mdx#agent-list](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#agent-list) |
| `opencode models` | List available models | [cli.mdx#models](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#models) |
| `opencode auth login` | Add provider credentials | [cli.mdx#auth-login](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#auth-login) |
| `opencode session list` | List sessions | [cli.mdx#session-list](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#session-list) |
| `opencode session export` | Export session to JSON | [cli.mdx#export](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#export) |
| `opencode mcp add` | Add MCP server | [cli.mdx#mcp-add](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#mcp-add) |
| `opencode github install` | Install GitHub agent | [cli.mdx#github-install](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/cli.mdx#github-install) |

## Environment Variables (from user docs)

| Variable | Description |
|----------|-------------|
| `OPENCODE_AUTO_SHARE` | Auto-share sessions |
| `OPENCODE_CONFIG` | Config file path |
| `OPENCODE_CONFIG_DIR` | Config directory path |
| `OPENCODE_DISABLE_AUTOUPDATE` | Disable auto-update check |
| `OPENCODE_ENABLE_EXA` | Enable Exa web search |
| `OPENCODE_SERVER_PASSWORD` | Server basic auth password |
| `OPENCODE_EXPERIMENTAL_*` | Experimental features |
