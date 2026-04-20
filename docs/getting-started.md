# Getting Started with OpenCode RS

OpenCode RS is a Rust-based AI coding agent providing interactive developer assistance via TUI, HTTP API, and SDK.

## Installation

### From Source

```bash
git clone https://github.com/opencode-ai/opencode-rs.git
cd opencode-rs
cargo build --release
```

### Using the Build Script

```bash
./build.sh
```

## Quick Start

### TUI Mode (Default)

```bash
opencode
```

### Web Interface Mode

```bash
opencode web --port 3000
```

### API Server Mode

```bash
opencode serve --port 8080
```

## Configuration

Create a `config.toml` or `config.json` file:

```toml
[server]
port = 3000
hostname = "127.0.0.1"

[server.desktop]
enabled = true
auto_open_browser = true

[server.acp]
enabled = true
server_id = "local"
version = "1.0"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENCODE_LLM_PROVIDER` | LLM provider name | `openai` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `OLLAMA_BASE_URL` | Ollama server URL | `http://localhost:11434` |
| `OPENCODE_DB_PATH` | SQLite database path | `./opencode.db` |

## Agent Modes

- **Build Mode**: Full read/write tool access for code implementation
- **Plan Mode**: Read-only tool access for planning and investigation
- **General Mode**: Search and investigation capabilities

## Available Tools

| Tool | Description |
|------|-------------|
| `Read` | Read file contents with line range support |
| `Write` | Create or overwrite files |
| `Edit` | Targeted file edits |
| `Grep` | Regex-based search |
| `Glob` | File pattern matching |
| `Git` | Git operations (status, diff, log, commit) |
| `Bash` | Shell command execution |
| `WebSearch` | Internet search |

## Session Management

Sessions are automatically persisted to SQLite. Use `/session` commands to manage conversation history.