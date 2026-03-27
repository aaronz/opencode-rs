## Why

The previous gap fill completed core CLI functionality but the Rust port still lacks many features from the TypeScript target. Key missing areas include: database/storage layer, LSP integration, permission system, server routes, and advanced session management. The goal is full feature parity to run target e2e tests.

## What Changes

1. **Implement Storage Layer**: Add SQLite database support for sessions, projects, accounts
2. **Add Server Routes**: Implement HTTP endpoints for session, provider, project, config, MCP
3. **Implement Permission System**: Advanced permission evaluation with arity checks
4. **Add LSP Integration**: Language server protocol client and server
5. **Implement Control Plane**: Workspace sync, SSE for real-time updates
6. **Add Account System**: User account management and authentication
7. **Implement Plugin System**: Dynamic plugin loading
8. **Add Git Integration**: Git operations via libgit2 or similar
9. **Implement Share System**: Session sharing functionality

## Capabilities

### New Capabilities
- `storage-database`: SQLite-based persistent storage for sessions, projects, accounts
- `server-routes`: HTTP API endpoints matching target's REST API
- `permission-system`: Advanced permission evaluation with pattern matching
- `lsp-integration`: Language server protocol for code intelligence
- `control-plane`: Workspace synchronization and real-time updates via SSE
- `account-management`: User accounts, authentication, billing
- `plugin-system`: Dynamic plugin loading and management
- `git-integration`: Git operations for VCS features
- `share-system`: Session sharing with share links

## Impact

- `rust-opencode-port/crates/storage/` - New crate for SQLite operations
- `rust-opencode-port/crates/server/` - New crate for HTTP server
- `rust-opencode-port/crates/permission/` - Permission evaluation
- `rust-opencode-port/crates/lsp/` - LSP client/server
- `rust-opencode-port/crates/control-plane/` - Workspace sync
- Existing `crates/core/` - Extended with account, plugin, git modules