# PRD: mcp Module

## Module Overview

**Module Name:** `mcp`
**Type:** Integration
**Source:** `/packages/opencode/src/mcp/`

## Purpose

Model Context Protocol implementation. Integrates with MCP servers for extended capabilities like Exa search.

## Functionality

### Core Features

1. **MCP Client**
   - Connects to MCP servers via stdio or HTTP
   - Handles JSON-RPC communication
   - Manages server lifecycle

2. **MCP Tools**
   - `mcp-exa` - Exa web search integration

3. **Protocol Support**
   - JSON-RPC 2.0
   - Tool execution
   - Resource handling
   - Sampling

### API Surface

```typescript
interface MCPClient {
  connect(serverPath: string): Promise<void>
  listTools(): Promise<Tool[]>
  executeTool(name: string, args: Record<string, any>): Promise<ToolResult>
  disconnect(): Promise<void>
}

interface Tool {
  name: string
  description: string
  inputSchema: JSONSchema
}

interface ToolResult {
  content: ContentBlock[]
  isError?: boolean
}
```

### Key Files

- MCP client implementation
- Protocol handling
- Tool integration

### Configuration

```json
{
  "mcp": {
    "servers": [
      {
        "name": "exa",
        "command": "npx",
        "args": ["-y", "@exa/mcp-server"]
      }
    ]
  }
}
```

## Dependencies

- `@modelcontextprotocol/sdk` - MCP SDK

## Acceptance Criteria

1. MCP client connects to servers
2. Tools are discovered and usable
3. JSON-RPC communication works correctly
4. Errors are handled gracefully

## Rust Implementation Guidance

The Rust equivalent should:
- Implement JSON-RPC 2.0 client
- Use `tokio` for async I/O
- Consider using `serde` for serialization
- Implement proper error handling

## Test Design

### Unit Tests
- `json_rpc_client`: Test request ID generation, response matching, and timeout handling.
- `tool_parsing`: Validate that MCP Tool schemas properly map to the internal Tool interface.

### Integration Tests
- `mcp_server_execution`: Spawn a simple Python or Node script acting as an MCP server over stdio and test `listTools` and `executeTool`.

### Rust Specifics
- Use `tokio::process::Command` with `Stdio::piped()` to test stdio MCP server interaction.
- Use `serde_json` to assert exact payload shapes.
