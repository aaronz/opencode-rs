# Getting Started with OpenCode RS

OpenCode RS is a Rust-based AI coding agent providing interactive developer assistance via TUI, HTTP API, and SDK.

## Prerequisites

- **Rust 1.70+**: OpenCode RS is built with Rust. Install via [rustup](https://rustup.rs/).
- **SQLite**: Required for session persistence (usually pre-installed on macOS/Linux).
- **LLM Provider API Key**: For cloud providers (OpenAI, Anthropic) or local Ollama server.

## Installation

### From Source

```bash
git clone https://github.com/opencode-ai/opencode-rs.git
cd opencode-rs
cargo build --release
```

The binary will be at `target/release/opencode`.

### Using the Build Script

```bash
./build.sh
```

### Verify Installation

```bash
opencode --version
```

## Configuration

Create a `config.toml` or `config.json` file in your project directory or home folder (`~/.config/opencode/config.toml`).

### Minimal Configuration

```toml
[llm]
provider = "openai"
model = "gpt-4"

[llm.openai]
api_key = "${OPENAI_API_KEY}"
```

### Full Configuration Example

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

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.7
max_tokens = 4096

[llm.openai]
api_key = "${OPENAI_API_KEY}"

[llm.ollama]
base_url = "http://localhost:11434"
model = "llama2"

[storage]
db_path = "./opencode.db"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENCODE_LLM_PROVIDER` | LLM provider name | `openai` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `OLLAMA_BASE_URL` | Ollama server URL | `http://localhost:11434` |
| `OPENCODE_DB_PATH` | SQLite database path | `./opencode.db` |

## First Session Walkthrough

### 1. Start OpenCode RS

```bash
opencode
```

This launches the TUI mode. You should see the welcome screen and prompt.

### 2. Set Your API Key

If using OpenAI, set your API key:

```bash
export OPENAI_API_KEY=sk-...
```

Or configure it in `config.toml`:

```toml
[llm.openai]
api_key = "sk-..."
```

### 3. Start Your First Task

Type your request at the prompt. For example:

```
> Help me create a new Rust project with a hello world binary
```

OpenCode RS will:
1. Analyze your request
2. Use tools to create files and run commands
3. Show you the results in real-time
4. Ask for confirmation before making changes

### 4. Approve or Modify

- Type `y` to approve changes
- Type `n` to cancel
- Type `/edit` to modify the proposed solution

### 5. Review the Results

After completion, you can:
- Ask follow-up questions
- Request modifications
- Start a new task with `/new`

## Basic Commands

### Interactive Mode Commands

| Command | Description |
|---------|-------------|
| `/new` | Start a new session |
| `/session` | Show current session info |
| `/mode build` | Switch to Build mode (full read/write) |
| `/mode plan` | Switch to Plan mode (read-only) |
| `/mode general` | Switch to General mode (search/investigate) |
| `/exit` | Exit OpenCode RS |

### Command Line Options

```bash
opencode                    # Start TUI mode (default)
opencode web --port 3000     # Start web interface
opencode serve --port 8080   # Start API server
opencode desktop --port 3000 # Start desktop mode (TUI + server + browser)
opencode --version          # Show version
opencode --help             # Show help
```

### ACP Commands

```bash
opencode acp status                     # Show ACP status
opencode acp handshake --client-id <id> # Perform handshake
opencode acp connect --url <url>         # Connect to server
```

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

### Session Commands

| Command | Description |
|---------|-------------|
| `/session` | Show current session |
| `/sessions` | List all sessions |
| `/session export` | Export session to JSON |
| `/session import` | Import session from JSON |

## Next Steps

- Read the [SDK Guide](sdk-guide.md) for programmatic access
- Read the [Plugin Development](plugin-dev.md) guide to extend functionality
- Check out example plugins in `plugins/hello_world/`