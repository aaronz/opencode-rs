# Contributing to Agent OS

Thank you for your interest in contributing!

## How to Contribute

1. **Fork the repository** and create your branch from `main`
2. **Clone your fork** locally
3. **Create a feature branch**: `git checkout -b feature/my-new-feature`
4. **Make your changes** and commit with clear, descriptive messages
5. **Push to your fork** and submit a Pull Request
6. **Wait for review** - we aim to respond within 48 hours

## Development Setup

Please refer to the main [README.md](README.md) for project-specific setup instructions.

## Code Style

- Follow the existing code style in the project
- Run `cargo fmt` (Go/Rust projects) or `npm run format` before committing
- Ensure all tests pass before submitting PR

## Commit Messages

- Use clear, descriptive commit messages
- Start with a verb (Add, Fix, Update, Remove)
- Keep the first line under 72 characters
- Add detailed description after a blank line if needed

## Pull Request Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated if applicable
- [ ] Documentation updated if needed
- [ ] PR description clearly describes the changes

## Issues

Feel free to submit issues for:
- Bug reports
- Feature requests
- Documentation improvements

## Plugin Development

OpenCode RS supports a WASM-based plugin system that allows extending functionality through portable, sandboxed plugins.

### Plugin Architecture

Plugins are compiled to WebAssembly (WASM) and run in a WASM runtime (wasmi). Each plugin can:
- Register custom tools that become available to the agent
- Hook into lifecycle events (session start/end, tool calls, messages)
- Define permissions for filesystem and network access

### Directory Structure

```
plugins/
├── hello_world/           # Example plugin
│   ├── Cargo.toml
│   ├── build.sh           # Build script
│   └── src/
│       └── lib.rs         # Plugin source
└── bin/                   # Compiled plugin binaries
    └── opencode_plugin_hello_world.wasm
```

### Creating a New Plugin

#### 1. Initialize Plugin Structure

Create a new directory under `plugins/` with `Cargo.toml`:

```toml
[package]
name = "opencode-plugin-my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[target.wasm32-wasip1]
runner = "wasmtime"

[dependencies]
wasmi = "0.31"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### 2. Implement Plugin API

Your plugin must implement two exported functions:

```rust
#![allow(static_mut_refs)]
#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::ptr;
use std::slice;

static mut REGISTERED_TOOLS: Vec<ToolDefinition> = Vec::new();
static mut INITIALIZED: bool = false;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginCommand {
    pub action: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

fn register_tool(name: &str, description: &str, input_schema: serde_json::Value) {
    unsafe {
        REGISTERED_TOOLS.push(ToolDefinition {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        });
    }
}

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    unsafe {
        if INITIALIZED {
            return 0;
        }
        // Register your tools here
        register_tool(
            "my_tool",
            "Description of my tool",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "First parameter"
                    }
                },
                "required": ["param1"]
            }),
        );
        INITIALIZED = true;
    }
    0
}

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const u8, len: usize) -> i32 {
    if command.is_null() || len == 0 {
        return -1;
    }
    let cmd_slice = unsafe { slice::from_raw_parts(command, len) };
    let cmd: PluginCommand = match serde_json::from_slice(cmd_slice) {
        Ok(c) => c,
        Err(_) => return -2,
    };
    match cmd.action.as_str() {
        "my_tool" => { /* execute tool */ 0 }
        "list_tools" => { /* list registered tools */ 0 }
        _ => -3,
    }
}
```

#### 3. Create Metadata File

Create a `*.plugin.json` file in your plugin directory:

```json
{
    "name": "my-plugin",
    "version": "0.1.0",
    "description": "My custom plugin for OpenCode RS",
    "main": "target/wasm32-wasip1/release/opencode_plugin_my_plugin.wasm",
    "enabled": true,
    "priority": 0,
    "capabilities": ["AddTools"],
    "allowed_events": [],
    "filesystem_scope": null,
    "network_allowed": false,
    "domain": "runtime"
}
```

#### 4. Create Build Script

Create `build.sh`:

```bash
#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
PLUGIN_NAME="opencode_plugin_my_plugin"
OUTPUT_DIR="$PROJECT_DIR/plugins/bin"
WASM_SOURCE="$SCRIPT_DIR/target/wasm32-wasip1/release/${PLUGIN_NAME}.wasm"
WASM_DEST="$OUTPUT_DIR/${PLUGIN_NAME}.wasm"
mkdir -p "$OUTPUT_DIR"
cd "$SCRIPT_DIR"
cargo build --target wasm32-wasip1 --release -p opencode-plugin-my-plugin
cp "$WASM_SOURCE" "$WASM_DEST"
echo "Built $WASM_DEST"
```

### Building Plugins

Run the build script:

```bash
cd plugins/hello_world
./build.sh
```

Or build manually:

```bash
cargo build --target wasm32-wasip1 --release -p opencode-plugin-my-plugin
```

### Loading Plugins

Plugins are discovered from two locations:
- **Global**: `~/.config/opencode/plugins/`
- **Project**: `./.opencode/plugins/` (project root)

Place your plugin metadata and WASM binary in either location. The plugin system will automatically discover and load plugins on startup.

### Plugin API Reference

#### Exported Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `plugin_init` | `fn() -> i32` | Initializes the plugin. Returns 0 on success. Call once before any execute calls. |
| `plugin_execute` | `fn(*const u8, usize) -> i32` | Executes a plugin action. Takes JSON command pointer and length. Returns 0 on success, negative on error. |

#### `plugin_init` Return Values

| Value | Meaning |
|-------|---------|
| 0 | Success |
| 1+ | Error code (plugin-specific) |

#### `plugin_execute` Return Values

| Value | Meaning |
|-------|---------|
| 0 | Success |
| -1 | Null command pointer or zero length |
| -2 | Invalid JSON in command |
| -3 | Unknown action |

#### PluginCommand Structure

```json
{
    "action": "string",   // Action name (tool name or special command)
    "args": {}            // JSON object with action arguments
}
```

#### Special Actions

| Action | Description |
|--------|-------------|
| `list_tools` | Returns list of registered tool definitions |
| `<tool_name>` | Executes the specified tool |

### Lifecycle Hooks

Plugins can implement lifecycle hooks (when using the Plugin trait):

- `on_init()` - Called after `init()` during plugin startup
- `on_start()` - Called when runtime starts or new session begins
- `on_tool_call(tool_name, args, session_id)` - Called before each tool execution; return `Err` to block
- `on_message(content, session_id)` - Called when message received
- `on_session_end(session_id)` - Called when session ends

### Plugin Capabilities

| Capability | Description |
|------------|-------------|
| `AddTools` | Plugin can register custom tools |
| `ListenEvents` | Plugin can receive lifecycle event notifications |
| `RewritePrompt` | Plugin can modify prompts before LLM processing |
| `InjectShellEnv` | Plugin can inject environment variables |
| `AddContextSources` | Plugin can add context to agent requests |
| `InterceptSensitiveRead` | Plugin can intercept file read operations |
| `SendNotification` | Plugin can send notifications |

### Plugin Configuration

See [`crates/plugin/src/lib.rs`](crates/plugin/src/lib.rs) for the full Plugin API.

For WASM runtime details, see the [wasmi documentation](https://github.com/wasmi/wasmi).

### Example: Hello World Plugin

See [`plugins/hello_world/`](plugins/hello_world/) for a complete working example:

- Source: [`plugins/hello_world/src/lib.rs`](plugins/hello_world/src/lib.rs)
- Build script: [`plugins/hello_world/build.sh`](plugins/hello_world/build.sh)
- Tests: [`opencode-rust/crates/plugin/tests/hook_determinism.rs`](opencode-rust/crates/plugin/tests/hook_determinism.rs)

## License

By contributing, you agree that your contributions will be licensed under the project's LICENSE file.