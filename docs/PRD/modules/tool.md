# PRD: tool Module

## Module Overview

**Module Name:** `tool`
**Type:** Core
**Source:** `/packages/opencode/src/tool/`

## Purpose

Implements all tools available to the agent for interacting with the filesystem, running commands, and searching. Tools are the primary way the agent interacts with the outside world.

## Functionality

### Core Features

The tool module implements **26 tools** that provide the agent's capabilities:

#### File Operations

| Tool | Description | Key Params |
|------|-------------|------------|
| `read` | Read file contents | `filePath`, `offset`, `limit` |
| `write` | Write new file | `filePath`, `content` |
| `edit` | In-place file editing | `filePath`, `oldString`, `newString` |
| `truncate` | Truncate file | `filePath`, `length` |
| `glob` | Find files by pattern | `pattern`, `path` |
| `codesearch` | Code search | `pattern`, `path` |

#### Command Execution

| Tool | Description | Key Params |
|------|-------------|------------|
| `bash` | Execute bash commands | `command`, `timeout` |
| `shell` | Shell operations | `command` |

#### Search

| Tool | Description | Key Params |
|------|-------------|------------|
| `grep` | Search file contents | `pattern`, `path`, `include` |
| `webfetch` | Fetch web content | `url` |
| `websearch` | Web search | `query` |
| `mcp-exa` | MCP Exa search | `query` |

#### Code Intelligence

| Tool | Description | Key Params |
|------|-------------|------------|
| `lsp` | LSP queries | `command`, `params` |

#### Editor Operations

| Tool | Description | Key Params |
|------|-------------|------------|
| `multiedit` | Multiple edit operations | `edits: [{filePath, oldString, newString}]` |
| `apply_patch` | Apply patches | `patch` |

#### Planning & Task

| Tool | Description | Key Params |
|------|-------------|------------|
| `plan` | Plan mode operations | `action` |
| `task` | Task management | `action`, `taskId` |
| `todowrite` | Todo list management | `todos` |
| `question` | Interactive questions | `question`, `options` |

#### Special

| Tool | Description | Key Params |
|------|-------------|------------|
| `skill` | Load skill | `name` |
| `external-directory` | External dir access | `path` |
| `invalid` | Invalid tool marker | - |

### Tool Registry

```typescript
// tool/registry.ts contains the tool registry
interface ToolRegistry {
  register(tool: Tool): void
  get(name: string): Tool | undefined
  list(): Tool[]
  execute(name: string, params: Record<string, unknown>): Promise<ToolResult>
}

interface Tool {
  name: string
  description: string
  parameters: JSONSchema
  execute(params: Record<string, unknown>): Promise<ToolResult>
  schema?: ToolSchema  // For UI display
}
```

### Tool Execution

```typescript
// Tool execution result
interface ToolResult {
  success: boolean
  output?: string
  error?: string
  metadata?: {
    duration?: number
    cacheHit?: boolean
  }
}
```

### Tool Schema

Each tool has a schema for UI display and validation:
- `*.txt` files contain tool descriptions for UI
- Tool registry provides metadata for documentation

### Key Files

| File | Purpose |
|------|---------|
| `registry.ts` | Tool registry implementation |
| `tool.ts` | Base tool interface |
| `schema.ts` | Tool schema definitions |
| `index.ts` | Exports |
| `bash.ts` | Bash command execution |
| `read.ts` | File reading |
| `write.ts` | File writing |
| `edit.ts` | File editing |
| `grep.ts` | Content search |
| `glob.ts` | File pattern matching |
| `lsp.ts` | LSP integration |
| `webfetch.ts` | Web fetching |
| `websearch.ts` | Web search |

### Dependencies

- `lsp` - For code intelligence
- `mcp` - For MCP tool integration
- `config` - For tool configuration
- `util` - For common utilities

## Implementation Notes

- All tools return structured results
- Proper error handling with meaningful messages
- Timeout support for long-running operations
- Dry-run support where applicable
- Tool execution can be parallelized

## Acceptance Criteria

1. All 26 tools function correctly
2. Tool results are properly structured
3. Errors provide meaningful feedback
4. Timeouts are enforced
5. Tool schema is properly documented

## Rust Implementation Guidance

The Rust equivalent should:
- Use `tokio::process` for command execution
- Use `tokio::fs` for file operations
- Implement tool trait for each tool
- Use serde for parameter validation
- Consider using `async_trait`
- Implement proper error types

## Test Design

### Unit Tests
- `bash_tool`: Mock process execution to verify argument formatting and timeout enforcement.
- `edit_tool`: Test partial replacement, invalid oldString detection, and multiple occurrence failures on in-memory strings.
- `read/write_tool`: Use in-memory filesystems or `tempfile` to ensure read limits, offsets, and writes work.

### Integration Tests
- `tool_registry`: Register multiple tools, pass a JSON-RPC-style invocation map, and ensure the correct tool executes and returns the schema-compliant result.

### Rust Specifics
- Use `tempfile` crate for isolated file system testing.
- Test asynchronous shell execution boundaries handling stdout/stderr capturing efficiently using `tokio::process::Command`.
