## Context

The Rust port of OpenCode currently implements basic functionality from the original TypeScript version. To achieve full feature parity and pass the original test suite, we need to implement 16+ missing tools, enhance the provider system with auth/models/error handling, add session management features (compaction, processor, etc.), and complete the LSP implementation.

## Goals / Non-Goals

**Goals:**
- Implement all missing tools from the original OpenCode
- Enhance provider system with authentication, model registry, error handling, and message transformation
- Add session management features: compaction, processor, prompt management, status tracking, summary generation, revert capability
- Complete LSP system with server, language detection, launch configuration
- Add MCP (Model Context Protocol) support
- Add missing modules: config, storage, git, permission, plugin, project, format

**Non-Goals:**
- Desktop/Web UI implementation (separate project)
- Enterprise features (control-plane, identity, etc.)
- Infrastructure/DevOps tools
- Real-time collaboration features

## Decisions

### 1. Tool Architecture: Unified Tool Trait
**Decision**: Create a unified Tool trait with execute method that returns ToolResult.

**Rationale**: 
- Consistent interface across all tools
- Easy to add new tools without changing core architecture
- Supports async execution

**Alternative Considered**: Separate tool types with different interfaces.
*Rejected*: Would make tool registry complex and harder to maintain.

### 2. Provider System: Plugin Architecture
**Decision**: Provider as trait with auth, models, error, transform as separate plugins.

**Rationale**:
- Flexible for adding new providers
- Auth/model/error/transform can be customized per provider
- Easy to test and maintain

**Alternative Considered**: Monolithic provider with all features included.
*Rejected*: Would be harder to extend and maintain.

### 3. Session Management: Event-Driven Architecture
**Decision**: Use event-driven architecture for session management.

**Rationale**:
- Easy to add hooks for compaction, processor, etc.
- Supports extensibility without changing core session logic
- Better performance for long sessions

**Alternative Considered**: Direct method calls for session management.
*Rejected*: Would make it harder to add new features and hooks.

### 4. LSP System: Server Implementation
**Decision**: Implement full LSP server with language detection and launch config.

**Rationale**:
- Better integration with IDEs
- Supports all LSP features
- Better performance than client-only approach

**Alternative Considered**: Client-only LSP with external server.
*Rejected*: Would limit features and performance.

### 5. MCP Support: Protocol Implementation
**Decision**: Implement MCP protocol for tool/toolset integration.

**Rationale**:
- Supports tool/toolset discovery and execution
- Better integration with external tools
- Standard protocol for AI tooling

**Alternative Considered**: Custom tool discovery protocol.
*Rejected*: MCP is becoming industry standard.

## Risks / Trade-offs

### [Risk] Complexity
**Mitigation**: 
- Implement incrementally
- Start with core tools first
- Use existing patterns from original codebase

### [Risk] Performance
**Mitigation**:
- Use async execution for tools
- Use caching where appropriate
- Profile and optimize hot paths

### [Risk] Compatibility
**Mitigation**:
- Follow original API design
- Test with original test suite
- Document any differences

### [Risk] Maintainability
**Mitigation**:
- Follow Rust best practices
- Use clear module structure
- Add comprehensive tests

## Migration Plan

### Phase 1: Core Tools (Week 1)
1. Implement bash, apply_patch, edit, batch tools
2. Implement codesearch, ls, lsp tools
3. Implement multiedit, question, read tools

### Phase 2: Provider Enhancement (Week 2)
1. Add auth system with API key validation
2. Add model registry with capabilities
3. Add error handling with retries
4. Add message transformation pipeline

### Phase 3: Session Enhancement (Week 3)
1. Add compaction for long conversations
2. Add session processor with hooks
3. Add prompt management system
4. Add session status tracking

### Phase 4: LSP Completion (Week 4)
1. Implement full LSP server
2. Add language detection
3. Add launch configuration
4. Add all LSP tools

### Phase 5: Missing Systems (Week 5)
1. Add MCP support
2. Add config management
3. Add storage system
4. Add git integration
5. Add permission control

### Phase 6: Testing & Integration (Week 6)
1. Run original test suite
2. Fix compatibility issues
3. Optimize performance
4. Add documentation

## Open Questions

1. **MCP Implementation**: Should we implement full MCP protocol or just basic tool discovery?

2. **Session Storage**: Should we use SQLite for session storage or JSON files?

3. **LSP Server**: Should we implement full LSP server or just client with external server?

4. **Error Handling**: How granular should error handling be for each tool?

5. **Plugin System**: Should we support third-party plugins or just internal modules?
