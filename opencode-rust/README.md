# OpenCode RS

Rust implementation of the AI coding agent.

## Data Directory

opencode-rs stores configuration, data, and cache files in the following locations:

| Type | Linux/macOS | Windows |
|------|-------------|---------|
| Config | `~/.config/opencode-rs/` | `%APPDATA%/opencode-rs/` |
| Data | `~/.local/share/opencode-rs/` | `%LOCALAPPDATA%/opencode-rs/` |
| Cache | `~/.cache/opencode-rs/` | `%LOCALAPPDATA%/opencode-rs/cache/` |
| Logs | `~/.config/opencode-rs/logs/` | `%APPDATA%/opencode-rs/logs/` |

### Environment Variables

You can override these paths with environment variables:

- `OPENCODE_RS_CONFIG_DIR` - Override config directory
- `OPENCODE_RS_DATA_DIR` - Override data directory
- `OPENCODE_RS_CACHE_DIR` - Override cache directory
- `OPENCODE_RS_LOG_DIR` - Override log directory

### Project Local Configuration

opencode-rs looks for project-specific configuration in `.opencode-rs/` directory in your project root.

**Note:** opencode-rs does NOT use or modify the `.opencode/` directory used by the original opencode project. They are completely separate.

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
