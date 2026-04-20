#!/bin/bash

append_test() {
  FILE=$1
  CONTENT=$2
  if [ -f "$FILE" ]; then
    echo -e "\n## Test Design\n\n$CONTENT" >> "$FILE"
    echo "Appended to $FILE"
  fi
}

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/agent.md" "### Unit Tests
- \`agent_loop\`: Mock the LLM provider and Tool Registry to verify the agent's state machine handles tool calls, stop signals, and user responses correctly.
- \`context_truncation\`: Verify that the agent correctly triggers session compaction when context limits are reached.

### Integration Tests
- \`end_to_end_agent\`: Run the agent with a mock server or local dummy LLM and real basic tools (e.g., read/write file in a temp directory) to ensure full loop resolution.

### Rust Specifics
- Use \`mockall\` to mock the \`Session\`, \`ToolRegistry\`, and \`LLMProvider\` traits.
- Use \`tokio::test\` for testing async agent loops."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/session.md" "### Unit Tests
- \`prompt_generation\`: Validate that \`toPrompt()\` produces the exact expected string formats for various message sequences.
- \`compaction_logic\`: Provide a dummy session exceeding limits and ensure compaction (summarization + pruning) correctly reduces token count without losing system messages.
- \`message_v2_parsing\`: Test parsing/serialization of messages with metadata and attachments.

### Integration Tests
- \`session_persistence\`: Save a session, reload it from SQLite, and assert structural equality.
- \`retry_mechanisms\`: Mock an LLM provider that fails with 429/5xx errors and verify exponential backoff is triggered correctly.

### Rust Specifics
- Test Serialization/Deserialization with \`serde_json::to_string\` and \`from_str\`.
- Use \`insta\` crate for snapshot testing of complex prompts."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/tool.md" "### Unit Tests
- \`bash_tool\`: Mock process execution to verify argument formatting and timeout enforcement.
- \`edit_tool\`: Test partial replacement, invalid oldString detection, and multiple occurrence failures on in-memory strings.
- \`read/write_tool\`: Use in-memory filesystems or \`tempfile\` to ensure read limits, offsets, and writes work.

### Integration Tests
- \`tool_registry\`: Register multiple tools, pass a JSON-RPC-style invocation map, and ensure the correct tool executes and returns the schema-compliant result.

### Rust Specifics
- Use \`tempfile\` crate for isolated file system testing.
- Test asynchronous shell execution boundaries handling stdout/stderr capturing efficiently using \`tokio::process::Command\`."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/provider.md" "### Unit Tests
- \`model_transformation\`: Test parsing of models.dev JSON output into OpenCode's internal \`Model\` structs.
- \`custom_loaders\`: Specifically test the AWS Bedrock region prefixing logic (\`us.*\`, \`eu.*\`) and Cloudflare Gateway routing based on mock configuration.

### Integration Tests
- \`dynamic_discovery\`: Mock the \`https://models.dev/api.json\` endpoint using \`wiremock\` and verify the module fetches, caches, and parses models.
- \`fallback_snapshot\`: Test that the provider falls back to the bundled snapshot if the network request fails.

### Rust Specifics
- Use \`wiremock\` or \`httptest\` to simulate the models.dev HTTP API.
- Test JSON parsing strictly using \`serde_json\`."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/cli.md" "### Unit Tests
- \`argument_parsing\`: Test \`clap\` configurations to ensure subcommands (\`run\`, \`models\`, \`serve\`, etc.) route correctly.
- \`flag_overrides\`: Test that CLI flags correctly override matching \`opencode.json\` or environment variable settings.

### Integration Tests
- \`cli_execution\`: Execute the compiled binary (or use \`assert_cmd\`) with basic flags (e.g., \`--help\`, \`--version\`) to verify output and exit codes.
- \`db_migration_trigger\`: Simulate a first run to ensure database migration logs/logic are triggered.

### Rust Specifics
- Use the \`assert_cmd\` crate to spawn the CLI process and assert on \`stdout\`, \`stderr\`, and exit codes.
- Use \`rexpect\` for testing expected terminal interactions."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/server.md" "### Unit Tests
- \`route_handlers\`: Test individual handler functions directly bypassing the HTTP server wrapper.
- \`middleware\`: Test rate limiting, auth, and CORS headers via request/response mocking.

### Integration Tests
- \`http_server\`: Start the server on a random open port and make actual HTTP requests (GET /health, POST /session) using a client like \`reqwest\`.
- \`proxy_routing\`: Test proxy behavior with a mock downstream server.

### Rust Specifics
- If using \`axum\`, use \`axum::test_helpers\` or \`tower::ServiceExt\` to test routes without binding to a network port.
- Use \`reqwest\` for end-to-end server integration tests."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/storage.md" "### Unit Tests
- \`sql_generation\`: Validate that queries for session fetching, message insertion, and auth updates produce expected SQL.
- \`migration_logic\`: Test that migrations apply sequentially and fail safely on malformed updates.

### Integration Tests
- \`sqlite_crud\`: Create an in-memory SQLite database, apply migrations, and run full CRUD cycles for sessions, messages, and settings.
- \`persistence_check\`: Write to a file-backed SQLite database, close the connection, reopen it, and verify data integrity.

### Rust Specifics
- Use \`rusqlite\` with in-memory databases (\`sqlite::memory:\`) for fast, isolated tests.
- Use \`tempfile\` to test file-backed SQLite databases."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/config.md" "### Unit Tests
- \`config_parsing\`: Test parsing \`opencode.json\` defaults, overrides, and malformed files.
- \`precedence_logic\`: Verify that environment variables > local config > global config > defaults.

### Integration Tests
- \`file_loading\`: Create temporary configuration files and verify the module loads and merges them correctly.

### Rust Specifics
- Use \`figment\` or \`config\` crates and test their merge capabilities.
- Use \`temp_env\` crate to safely test environment variable overrides in isolation."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/lsp.md" "### Unit Tests
- \`protocol_parsing\`: Test serialization/deserialization of LSP messages (JSON-RPC).
- \`lifecycle_management\`: Test initialize, initialized, and shutdown sequence messages.

### Integration Tests
- \`dummy_lsp_server\`: Spawn a mock script that replies to standard LSP requests and verify the client processes goto-definition and hover responses correctly.

### Rust Specifics
- Use \`lsp-types\` to validate structure.
- Test async bidirectional channels (\`tokio::sync::mpsc\`) extensively."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/mcp.md" "### Unit Tests
- \`json_rpc_client\`: Test request ID generation, response matching, and timeout handling.
- \`tool_parsing\`: Validate that MCP Tool schemas properly map to the internal Tool interface.

### Integration Tests
- \`mcp_server_execution\`: Spawn a simple Python or Node script acting as an MCP server over stdio and test \`listTools\` and \`executeTool\`.

### Rust Specifics
- Use \`tokio::process::Command\` with \`Stdio::piped()\` to test stdio MCP server interaction.
- Use \`serde_json\` to assert exact payload shapes."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/plugin.md" "### Unit Tests
- \`hook_execution\`: Mock plugins and verify that \`config\`, \`provider\`, and \`tool\` hooks are called in the correct order.
- \`error_isolation\`: Ensure that a panicking or erroring plugin does not crash the core application.

### Integration Tests
- \`dylib_loading\`: Compile a dummy plugin to a \`.so\`/\`.dylib\`, load it dynamically, and verify hook execution.

### Rust Specifics
- Test FFI boundaries and `libloading` safety carefully.
- Alternatively, if plugins are WASM, test WASM host execution via \`wasmtime\`."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/auth.md" "### Unit Tests
- \`credential_encryption\`: Test that keys are securely encrypted before storage and correctly decrypted.
- \`oauth_flow\`: Test token refresh logic calculation based on expiry times.

### Integration Tests
- \`keyring_integration\`: Test saving to and fetching from the OS keyring (may require mocking in CI environments).

### Rust Specifics
- Use \`keyring-rs\` mock features if available, or fallback to file-based mocks in CI.
- Use \`chrono\` or \`time\` crates with mockable clocks to test expiry logic."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/project.md" "### Unit Tests
- \`type_detection\`: Pass mock directory structures (e.g., presence of \`package.json\`, \`Cargo.toml\`) and verify the correct project type is returned.

### Integration Tests
- \`workspace_parsing\`: Point the module at real monorepo structures to verify accurate root resolution.

### Rust Specifics
- Use \`tempfile\` and \`std::fs::create_dir_all\` to scaffold mock project trees for tests."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/acp.md" "### Unit Tests
- \`protocol_messages\`: Verify formatting of handshake, ack, and status messages.
- \`state_machine\`: Test the transition from disconnected -> handshaking -> connected.

### Integration Tests
- \`local_agent_communication\`: Spin up two instances of the ACP server locally and test end-to-end handshake and message passing.

### Rust Specifics
- Use \`tokio::net::TcpStream\` or IPC sockets for integration testing.
- Test task cancellation on disconnect."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/util.md" "### Unit Tests
- \`logging\`: Test log level filtering and format.
- \`error_formatting\`: Test conversion of nested errors into structured \`NamedError\` equivalents.

### Rust Specifics
- Test \`tracing\` output using \`tracing-test\` or capturing subscribers.
- Test error trait implementations using \`anyhow\`."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/effect.md" "### Unit Tests
- \`monad_composition\`: Test chaining of successful and failing effects.
- \`service_injection\`: Test that a service can be registered and accessed from context.

### Rust Specifics
- In Rust, this usually maps to testing generic trait bounds and async result chaining. Test \`Result<T, E>\` combinators and custom context injection structs."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/flag.md" "### Unit Tests
- \`flag_resolution\`: Test parsing strings like \"1\", \"true\", \"0\", \"false\" into boolean flags.

### Rust Specifics
- Use \`temp_env\` to mock environment variables for test isolation."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/global.md" "### Unit Tests
- \`path_resolution\`: Test fallback paths for different OS targets (Windows, macOS, Linux).

### Rust Specifics
- Test via conditional compilation (\`#[cfg(target_os = "windows")]\`) or path mocking."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/env.md" "### Unit Tests
- \`env_overrides\`: Test safe fetching of variables with fallbacks.

### Rust Specifics
- Test thread-safe environment variable reads (avoiding \`std::env::set_var\` in multi-threaded tests)."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/file.md" "### Unit Tests
- \`path_normalization\`: Test relative, absolute, and escaped path normalization.
- \`file_watching\`: Test debounce logic for rapid file changes.

### Rust Specifics
- Test using \`notify\` crate in an isolated async task using \`tempfile\`."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/git.md" "### Integration Tests
- \`repo_operations\`: Initialize a temp git repo, make a commit, and use the module to read branches and status.

### Rust Specifics
- Use the \`git2\` crate testing patterns on local \`tempfile\` directories."

append_test "/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/modules/remaining-modules.md" "### Unit & Integration Tests (General)
- \`pty\`: Test pseudo-terminal stdout capturing and window resize events (requires OS-specific mock configurations).
- \`sync\`: Test deadlocks and race conditions using concurrent async tasks.
- \`snapshot\`: Test serialization of state to JSON and restoration.
- \`shell\`: Test parsing and execution of simple shell commands and exit code mapping.

### Rust Specifics
- For \`pty\`, test with \`portable-pty\`.
- For \`sync\`, use \`tokio::sync\` primitives and test with \`tokio::spawn\` multi-threading assertions."

