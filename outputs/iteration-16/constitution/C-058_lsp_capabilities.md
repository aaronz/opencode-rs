# Constitution C-058: LSP Capabilities

**Version**: 1.0  
**Date**: 2026-04-07  
**Iteration**: v16  
**Status**: Adopted

---

## Preamble

This Constitution documents the Language Server Protocol (LSP) capabilities required for OpenCode-RS IDE integration, specifically the implementation of `textDocument/definition`, `textDocument/references`, and related features.

## Article 1: LSP Architecture

### Section 1.1: Dual LSP Components

OpenCode-RS operates in two LSP modes:

| Component | Role | Implementation |
|-----------|------|----------------|
| **LSP Server** | Serves LSP to IDE/editor | `tower_lsp` based, opencode as server |
| **LSP Client** | Uses external LSP servers | Spawns rust-analyzer, tsserver, etc. |

### Section 1.2: Client/Server Responsibilities

**LSP Server** (what opencode provides to editor):
- Receives LSP requests from editor
- Provides code intelligence for opencode's internal code
- Capabilities: definition, references, hover, completion

**LSP Client** (what opencode uses internally):
- Spawns external language servers (rust-analyzer, tsserver, gopls, pylsp)
- Proxies requests to external servers
- Aggregates results from multiple servers

## Article 2: Required Capabilities

### Section 2.1: Server Capabilities

The LSP server MUST advertise these capabilities in `initialize` response:

```rust
ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Kind(FULL)),
    hover_provider: Some(HoverProviderCapability::Simple(true)),
    definition_provider: Some(OneOf::Left(true)),
    references_provider: Some(OneOf::Left(true)),
    completion_provider: Some(CompletionOptions::default()),
    // ... other capabilities
}
```

### Section 2.2: Client Capabilities

The LSP client MUST implement:

| Method | Description |
|--------|-------------|
| `goto_definition(uri, line, col)` | Jump to symbol definition |
| `find_references(uri, line, col)` | Find all references to symbol |
| `completion(uri, line, col)` | Get completion items |
| `get_diagnostics(uri)` | Get diagnostic messages |

## Article 3: JSON-RPC Protocol

### Section 3.1: Message Format

All LSP communication uses JSON-RPC 2.0 over stdin/stdout:

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "textDocument/definition",
  "params": {
    "textDocument": { "uri": "file:///path/to/file.rs" },
    "position": { "line": 10, "character": 5 }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "uri": "file:///path/to/definition.rs",
    "range": {
      "start": { "line": 20, "character": 0 },
      "end": { "line": 20, "character": 15 }
    }
  }
}
```

### Section 3.2: Content-Length Header

All messages MUST be preceded by `Content-Length` header:
```
Content-Length: 123\r\n
\r\n
{"jsonrpc": "2.0", ...}
```

### Section 3.3: Response Correlation

Requests MUST include an `id` for response correlation. The client maintains a pending request map:

```rust
pending: HashMap<u64, oneshot::Sender<String>>
request_id: u64  // monotonically increasing
```

## Article 4: Language Server Detection

### Section 4.1: Auto-Detection

The client MUST auto-detect the appropriate language server based on project files:

| File | Language Server |
|------|-----------------|
| `Cargo.toml` | `rust-analyzer` |
| `package.json` + `tsconfig.json` | `typescript-language-server --stdio` |
| `go.mod` | `gopls` |
| `pyproject.toml` or `setup.py` | `pylsp` |

### Section 4.2: Fallback

If no language server detected, return empty results gracefully without error.

## Article 5: Error Handling

### Section 5.1: Timeout Handling

LSP requests MUST have a timeout (recommended: 5 seconds). On timeout:
- Return empty result (`None` for definition, `[]` for references)
- Do NOT crash or panic
- Log warning for debugging

### Section 5.2: Process Management

On `shutdown()`:
1. Send `shutdown` notification
2. Kill the LSP server process
3. Clean up stdin/stdout handles

On drop:
- Ensure process is killed to prevent zombies

## Article 6: Testing Requirements

### Section 6.1: Unit Tests

| Test | Description |
|------|-------------|
| `test_language_detection_rust` | Verify rust-analyzer for Cargo.toml |
| `test_language_detection_typescript` | Verify tsserver for package.json + tsconfig |
| `test_jsonrpc_message_parsing` | Verify Content-Length parsing |
| `test_location_parsing` | Verify Location/Range JSON parsing |

### Section 6.2: Integration Tests

External LSP server communication should be tested with mocked responses.

## Article 7: Adoption

This Constitution is effective immediately upon merge to main branch.

---

**Ratified**: 2026-04-07  
**Expires**: Never  
**Amendments**: Requires RFC process
