# OpenCode RS

Rust implementation of the AI coding agent.

## Installation

```bash
cargo build --release
./target/release/opencode-rs
```

## Configuration

Create `~/.config/opencode-rs/config.toml`:

```toml
provider = "openai"
model = "gpt-4o"
api_key = "your-api-key"
temperature = 0.7
```

Or use environment variables:
- `OPENCODE_PROVIDER` - openai, anthropic, or ollama
- `OPENCODE_MODEL` - model name
- `OPENCODE_API_KEY` - API key
- `OPENCODE_TEMPERATURE` - temperature setting

## Features

- **Multi-provider LLM**: OpenAI, Anthropic Claude, Ollama (local)
- **Tool System**: File operations, grep, git, web search
- **Agent System**: Build (full access), Plan (read-only), General (search)
- **Terminal UI**: Interactive TUI with ratatui
- **Session Management**: Save and resume conversations

## Usage

```bash
# Start interactive mode
opencode-rs

# List sessions
opencode-rs list

# Resume session
opencode-rs session --id <session-id>
```
