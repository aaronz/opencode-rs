## Context

The current Rust port (/Users/aaronzh/Documents/GitHub/mycode/rust-opencode-port) has:
- Core CLI with basic commands (run, models, providers, list, serve)
- LLM provider implementations with async streaming
- Basic tool system (grep, read, write, bash, etc.)
- Session management (save/load, list, delete)
- Configuration loading from file/env/CLI
- 172 unit tests passing

Missing major components compared to TypeScript target:
- Database/storage layer for persistent storage
- HTTP server with full REST API endpoints
- Permission system with pattern matching
- LSP integration for code intelligence
- Workspace sync and control plane
- Account management system
- Plugin system for extensibility
- Git integration
- Session sharing capabilities

## Goals / Non-Goals

**Goals:**
1. Implement SQLite-based storage for sessions, projects, accounts
2. Add HTTP server with REST API endpoints matching TypeScript target
3. Implement permission evaluation system with arity checks
4. Add LSP client/server for code intelligence features
5. Implement workspace sync and control plane with SSE
6. Add account management system
7. Implement plugin system for dynamic loading
8. Add Git integration for VCS features
9. Implement session sharing functionality
10. Enable running TypeScript e2e tests against Rust port

**Non-Goals:**
1. Exact copy-paste of TypeScript implementation - focus on behavioral equivalence
2. Maintaining identical file structure - different language idioms acceptable
3. Supporting every possible database backend - SQLite as primary target
4. Re-implementing TypeScript's Effect/FP architecture - use Rust idioms
5. Supporting all TypeScript-specific testing harnesses - focus on core functionality

## Decisions

1. **Storage Choice**: Use SQLite via rusqlite crate for simplicity and zero-config deployment
2. **Server Architecture**: Use Actix-web for HTTP server with async handlers
3. **Permission System**: Implement pattern matching with wildcards and regex support
4. **LSP Implementation**: Use tower-lsp for Language Server Protocol compliance
5. **Control Plane**: Use tokio broadcast channels for SSE event distribution
6. **Account System**: Simple username/password with JWT tokens for auth
7. **Plugin System**: Dynamic loading via dlopen or WASM for sandboxed execution
8. **Git Integration**: Use git2-rs crate for libgit2 bindings
9. **Share System**: Generate shareable links with expiration and access controls

## Risks / Trade-offs

- **Risk**: SQLite may not scale for heavy usage → **Mitigation**: Proper indexing and connection pooling
- **Risk**: Actix-web learning curve → **Mitigation**: Well-documented with good performance
- **Risk**: LSP complexity → **Mitigation**: Start with basic features (diagnostics, completion)
- **Risk**: Plugin security → **Mitigation**: Sandbox plugins with capabilities model
- **Risk**: Git integration complexity → **Mitigation**: Focus on core operations first

## Migration Plan

1. Phase 1: Storage layer implementation (sessions, projects, accounts)
2. Phase 2: HTTP server with basic endpoints (config, models, providers)
3. Phase 3: Permission system and account management
4. Phase 4: LSP integration and code intelligence features
5. Phase 5: Control plane, workspace sync, and SSE
6. Phase 6: Plugin system and Git integration
7. Phase 7: Share system and final e2e test compatibility

## Open Questions

- Should we use a connection pool for SQLite or simple per-connection model?
- How granular should permission patterns be (glob, regex, exact match)?
- What LSP features to implement first (diagnostics, completion, hover, definition)?
- Should plugins be WASM-based or native dynamic libraries?
- How to handle plugin updates and versioning?