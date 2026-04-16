# LSP Integration Documentation

OpenCode provides Language Server Protocol (LSP) integration for enhanced code intelligence in IDEs and editors.

## Overview

The `opencode-lsp` crate provides:
- Built-in LSP server detection and auto-launch
- LSP client for connecting to external language servers
- Custom LSP server implementation for OpenCode-specific features
- Multi-language support (Rust, TypeScript, Python, Go, JavaScript)

## Supported Languages

| Language | Server | Command | File Indicators |
|----------|--------|---------|-----------------|
| Rust | rust-analyzer | `rust-analyzer` | `Cargo.toml` |
| TypeScript | TypeScript Language Server | `typescript-language-server --stdio` | `tsconfig.json`, `package.json` |
| JavaScript | JavaScript Language Server | `javascript-language-server --stdio` | `package.json` |
| Python | Python Language Server (pylsp) | `pylsp` | `pyproject.toml`, `setup.py`, `requirements.txt` |
| Go | gopls | `gopls` | `go.mod` |

## Supported LSP Features

### Text Document Synchronization
- `textDocument/didOpen` - Document open notification
- `textDocument/didChange` - Document change notification
- `textDocument/didSave` - Document save notification
- `textDocument/didClose` - Document close notification

### Code Completion
- `textDocument/completion` - Triggered on typing (via completion provider)
- Returns completion items with labels, kinds, and details
- Built-in keywords: `fn`, `let`, `mut`, `pub`, `use`, `struct`, `enum`, `impl`, `trait`, `match`, etc.

### Hover Information
- `textDocument/hover` - Returns type information for symbols
- Displays keyword, type, function, struct, enum, trait, impl block, constant, and variable classifications

### Go to Definition
- `textDocument/definition` - Jump to symbol definitions
- Supports functions, structs, enums, and traits

### Find References
- `textDocument/references` - Find all occurrences of a symbol
- Returns array of `Location` objects

### Code Actions
- `textDocument/codeAction` - Provides quick fixes and refactorings
- Built-in actions:
  - "Ignore this diagnostic" (QUICKFIX)
  - "Extract to function" (REFACTOR_EXTRACT)

### Diagnostic Aggregation
- `DiagnosticAggregator` for collecting and presenting LSP diagnostics
- Severity levels: Error, Warning, Information, Hint

## VSCode LSP Client Setup

### Prerequisites
- VSCode with LSP extension support
- Language server installed for your language

### Manual LSP Configuration

1. Create a `.vscode/settings.json` in your project:

```json
{
  "languageServer": {
    "rust-analyzer": {
      "command": "rust-analyzer",
      "args": [],
      "rootPatterns": ["Cargo.toml"]
    },
    "typescript": {
      "command": "typescript-language-server",
      "args": ["--stdio"],
      "rootPatterns": ["tsconfig.json", "package.json"]
    },
    "python": {
      "command": "pylsp",
      "args": [],
      "rootPatterns": ["pyproject.toml", "setup.py"]
    },
    "gopls": {
      "command": "gopls",
      "args": [],
      "rootPatterns": ["go.mod"]
    }
  }
}
```

### Using VSCode Remote Development

For remote development with VSCode Remote SSH:

1. Ensure language servers are installed on the remote machine
2. Configure `.vscode/settings.json` on the remote:

```json
{
  "rust-analyzer.server.path": "/path/to/rust-analyzer",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.cargo.buildScripts.enable": true
}
```

### VSCode Extension Recommendations

| Language | Recommended Extension |
|----------|----------------------|
| Rust | rust-analyzer |
| TypeScript/JavaScript | TypeScript Language Server |
| Python | Python (Microsoft) with Pylance |
| Go | Go (Microsoft) |

### Troubleshooting VSCode LSP

1. **Server not starting**: Check that the language server command is in your PATH
2. **Diagnostics not showing**: Ensure the language server supports diagnostics and is properly configured
3. **Slow performance**: Disable unnecessary language server features in settings

## Neovim LSP Setup

### Using nvim-lspconfig

The recommended way to configure LSP in Neovim is using `nvim-lspconfig`:

```lua
-- ~/.config/nvim/lua/lsp.lua

local lspconfig = require('lspconfig')

-- Rust
lspconfig.rust_analyzer.setup({
  cmd = { "rust-analyzer" },
  root_pattern = lspconfig.util.root_pattern("Cargo.toml"),
  settings = {
    ['rust-analyzer'] = {
      cargo = {
        loadOutDirsFromCheck = true,
        buildScripts = { enable = true }
      }
    }
  }
})

-- TypeScript/JavaScript
lspconfig.tsserver.setup({
  cmd = { "typescript-language-server", "--stdio" },
  root_pattern = lspconfig.util.root_pattern("tsconfig.json", "package.json")
})

-- Python
lspconfig.pylsp.setup({
  cmd = { "pylsp" },
  root_pattern = lspconfig.util.root_pattern("pyproject.toml", "setup.py")
})

-- Go
lspconfig.gopls.setup({
  cmd = { "gopls" },
  root_pattern = lspconfig.util.root_pattern("go.mod")
})
```

### Using Mason for Automatic Installation

With `williamboman/mason.nvim`:

```lua
-- Install language servers
require('mason').setup()
require('mason-lspconfig').setup({
  ensure_installed = { 'rust_analyzer', 'tsserver', 'pylsp', 'gopls' }
})
```

### Neovim LSP Keybindings

```lua
-- ~/.config/nvim/lua/lsp.lua

local opts = { noremap = true, silent = true }

-- See hover information
vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)

-- Go to definition
vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)

-- Find references
vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)

-- Show diagnostics
vim.keymap.set('n', 'gl', vim.diagnostic.open_float, opts)

-- Code actions
vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)

-- Rename symbol
vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
```

### Troubleshooting Neovim LSP

1. **Server not found**: Verify the command is in your PATH or provide full path
2. **Diagnostics missing**: Run `:LspInfo` to check server status
3. **Slow startup**: Use `nvim-lspconfig` with minimal configuration

## Programmatic LSP Usage

### Using BuiltInRegistry

```rust
use opencode_lsp::{BuiltInRegistry, Language};

let registry = BuiltInRegistry::new();

// Detect servers for a Rust project
let servers = registry.detect_for_root(std::path::Path::new("/path/to/project"));
for server in servers {
    println!("Found server: {} ({})", server.name, server.id);
}

// Check if a server is available
if registry.is_available("rust-analyzer") {
    println!("rust-analyzer is available");
}
```

### Launching an LSP Client

```rust
use opencode_lsp::{LspClient, LaunchConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = LspClient::new();
    
    client.start("rust-analyzer", &std::path::PathBuf::from("/path/to/project")).await?;
    
    // Client is now running and ready to handle requests
    Ok(())
}
```

### Custom LSP Server

```rust
use opencode_lsp::{LspServer, MockLspServer};

let mock_server = MockLspServer::new();
let server = LspServer::new(mock_server.client().clone());
```

## Configuration Options

### BundledConfig

```rust
use opencode_lsp::BundledConfig;

let config = BundledConfig {
    detection_enabled: true,
    excluded_servers: vec!["pylsp".to_string()],
    custom_paths: std::collections::HashMap::new(),
};
```

### FailureHandlingConfig

```rust
use opencode_lsp::FailureHandlingConfig;

let config = FailureHandlingConfig {
    max_consecutive_errors: 5,
    restart_on crash: true,
    crash_backoff_ms: 1000,
};
```

## Error Handling

The crate provides structured errors via the `LspError` enum:

```rust
use opencode_lsp::LspError;

// Handle specific error types
match error {
    LspError::ConnectionFailed(msg) => { /* retry or report */ }
    LspError::RequestFailed(id, msg) => { /* handle failure */ }
    LspError::ProtocolViolation(violation) => { /* log and continue */ }
    _ => { /* unknown error */ }
}
```

## Testing

Run LSP tests:

```bash
cargo test -p opencode-lsp
```

Run with output:

```bash
cargo test -p opencode-lsp -- --nocapture
```

## Dependencies

Key dependencies:
- `tower-lsp` (0.20) - Language Server Protocol implementation
- `lsp-types` (0.95) - LSP type definitions
- `tokio` - Async runtime

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      OpenCode                           │
├─────────────────────────────────────────────────────────┤
│  LspManager                                             │
│  ├── BuiltInRegistry (auto-detection)                  │
│  ├── LspClient (external server connection)             │
│  └── CustomLspServer (OpenCode-specific features)      │
├─────────────────────────────────────────────────────────┤
│  Tower LSP                                              │
│  └── LanguageServer trait implementation               │
├─────────────────────────────────────────────────────────┤
│  External LSP Servers                                   │
│  ├── rust-analyzer                                     │
│  ├── typescript-language-server                        │
│  ├── pylsp                                             │
│  └── gopls                                              │
└─────────────────────────────────────────────────────────┘
```
