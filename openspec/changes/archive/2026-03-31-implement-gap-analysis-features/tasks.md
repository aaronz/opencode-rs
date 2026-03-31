## 1. WebSocket Streaming

- [x] 1.1 Add WebSocket handler to server router in `crates/server/src/router.rs`
- [x] 1.2 Implement WebSocket connection management and session binding
- [x] 1.3 Create JSON message protocol types for WebSocket communication
- [x] 1.4 Implement agent output streaming over WebSocket
- [x] 1.5 Add WebSocket authentication using existing session auth
- [x] 1.6 Implement connection timeout and cleanup

## 2. SSE Streaming

- [x] 2.1 Add SSE endpoint handler to server router
- [x] 2.2 Implement SSE event formatting (token, tool_call, done, error)
- [x] 2.3 Add companion POST endpoint for client input
- [x] 2.4 Implement `Last-Event-ID` reconnection support
- [x] 2.5 Add SSE authentication using existing session auth

## 3. TUI Input Syntax

- [x] 3.1 Create input parser module in `crates/tui/src/input_parser.rs`
- [x] 3.2 Implement `@filename` file inclusion syntax
- [x] 3.3 Implement `!command` shell execution syntax
- [x] 3.4 Implement `/command` TUI command syntax
- [x] 3.5 Add escape syntax support (`\@`, `\!`)
- [x] 3.6 Implement autocompletion for syntax prefixes
- [x] 3.7 Integrate parser with existing TUI input flow

## 4. LSP Diagnostics

- [x] 4.1 Extend LSP tool in `crates/tools/src/lsp_tool.rs` with diagnostics action
- [x] 4.2 Connect LSP tool to LSP client in `crates/lsp/`
- [x] 4.3 Implement structured diagnostic response format
- [x] 4.4 Add hover, definition, references, symbols actions
- [x] 4.5 Implement LSP server auto-start and lifecycle management
- [x] 4.6 Add LSP timeout and error handling

## 5. MCP Protocol

- [x] 5.1 Create `crates/mcp/` crate with MCP protocol types
- [x] 5.2 Implement MCP JSON-RPC 2.0 message parsing
- [x] 5.3 Implement initialize/initialized handshake
- [x] 5.4 Implement tools/list for tool discovery
- [x] 5.5 Implement tools/call for tool execution
- [x] 5.6 Bridge MCP tools to existing ToolRegistry
- [x] 5.7 Implement resources/list and resources/read
- [x] 5.8 Add MCP streaming support for long-running tools
- [x] 5.9 Add MCP endpoint to server router

## 6. Testing and Integration

- [x] 6.1 Write unit tests for WebSocket handler
- [x] 6.2 Write unit tests for SSE handler
- [x] 6.3 Write unit tests for input parser
- [x] 6.4 Write unit tests for LSP tool extensions
- [x] 6.5 Write unit tests for MCP protocol
- [x] 6.6 Integration test: WebSocket streaming with agent
- [x] 6.7 Integration test: SSE streaming with agent
- [x] 6.8 Integration test: TUI input syntax end-to-end
- [x] 6.9 Update documentation for new features
