# PRD: lsp Module

## Module Overview

**Module Name:** `lsp`
**Type:** Integration
**Source:** `/packages/opencode/src/lsp/`

## Purpose

Language Server Protocol integration for code intelligence. Provides go-to-definition, find-references, symbol search, and rename capabilities.

## Functionality

### Core Features

1. **LSP Client**
   - Connects to LSP servers (TypeScript, Python, Rust, etc.)
   - Handles server lifecycle
   - Manages multiple language servers

2. **Queries**

   | Query | Description |
   |-------|-------------|
   | `gotoDefinition` | Go to symbol definition |
   | `findReferences` | Find all references to symbol |
   | `documentSymbols` | List symbols in document |
   | `workspaceSymbols` | Search symbols across workspace |
   | `hover` | Get hover information |
   | `rename` | Rename symbol across workspace |

3. **Supported Languages**

   | Language | Server |
   |----------|--------|
   | TypeScript/JavaScript | `typescript-language-server` |
   | Python | `python-lsp-server` |
   | Rust | `rust-analyzer` |
   | Go | `gopls` |
   | And more... | |

### API Surface

```typescript
interface LSPClient {
  initialize(): Promise<void>
  gotoDefinition(file: string, position: Position): Promise<Location | null>
  findReferences(file: string, position: Position): Promise<Location[]>
  documentSymbols(file: string): Promise<Symbol[]>
  workspaceSymbols(query: string): Promise<Symbol[]>
  hover(file: string, position: Position): Promise<Hover | null>
  rename(file: string, position: Position, newName: string): Promise<WorkspaceEdit | null>
  shutdown(): Promise<void>
}

interface Position {
  line: number
  character: number
}

interface Location {
  uri: string
  range: Range
}

interface Symbol {
  name: string
  kind: SymbolKind
  location: Location
}
```

### Key Files

- LSP client implementation
- Server management
- Protocol handling
- Error handling

### Dependencies

- `vscode-languageserver` - LSP protocol
- Language-specific LSP servers (external)

## Acceptance Criteria

1. LSP client connects to servers correctly
2. All query types return correct results
3. Server lifecycle is managed properly
4. Errors are handled gracefully
5. Multiple servers can run concurrently

## Rust Implementation Guidance

The Rust equivalent should:
- Use `lsp-server` crate for LSP protocol
- Use `tokio` for async operations
- Implement proper server management
- Consider using `serde` for JSON-RPC
