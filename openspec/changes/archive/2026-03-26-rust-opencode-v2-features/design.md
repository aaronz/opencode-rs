## Context

The Rust OpenCode project has successfully implemented core functionality from the original TypeScript version. However, to achieve full feature parity and pass the original test suite, we need to implement approximately 30 additional modules covering advanced features like effect systems, event handling, CLI commands, format systems, and various infrastructure components.

## Goals / Non-Goals

**Goals:**
- Implement all missing modules from the original OpenCode project
- Achieve feature parity with the TypeScript version
- Pass the original test suite
- Maintain clean architecture and performance

**Non-Goals:**
- Real-time collaboration features (future)
- Desktop GUI applications
- Browser-based interfaces
- Cloud/SaaS infrastructure (separate services)

## Decisions

### 1. Effect System: Custom Implementation
**Decision**: Implement a custom effect system rather than using external libraries.

**Rationale**:
- Simpler dependency graph
- Better performance for Rust
- Easier to maintain and debug

**Alternative Considered**: Using effect-ts or similar.
*Rejected*: Adds complexity and learning curve.

### 2. Event Bus: Tokio Channels
**Decision**: Use tokio channels for the event bus system.

**Rationale**:
- Native async support
- High performance
- Simple API

**Alternative Considered**: Custom event system.
*Rejected*: Would be more complex and less performant.

### 3. Command System: Plugin Architecture
**Decision**: Implement a plugin-based command system.

**Rationale**:
- Easy to extend
- Clean separation of concerns
- Supports third-party integrations

**Alternative Considered**: Monolithic command system.
*Rejected*: Would be harder to maintain and extend.

### 4. Format System: External Tools
**Decision**: Use external formatting tools (rustfmt, prettier, etc.)

**Rationale**:
- Leverage existing tools
- Better formatting quality
- Language-specific support

**Alternative Considered**: Custom formatting logic.
*Rejected*: Would be complex and less accurate.

### 5. IDE Integration: LSP Protocol
**Decision**: Use LSP protocol for IDE integration.

**Rationale**:
- Standard protocol
- Wide IDE support
- Well-documented

**Alternative Considered**: Custom IDE protocols.
*Rejected*: Would limit IDE support.

### 6. Storage System: SQLite + JSON
**Decision**: Use SQLite for structured data and JSON for configuration.

**Rationale**:
- SQLite is lightweight and reliable
- JSON is flexible for configuration
- Both are widely supported

**Alternative Considered**: Custom storage system.
*Rejected*: Would be complex and less reliable.

### 7. Permission System: RBAC
**Decision**: Implement Role-Based Access Control.

**Rationale**:
- Flexible permission model
- Easy to extend
- Standard approach

**Alternative Considered**: Simple permission flags.
*Rejected*: Would be too rigid.

## Risks / Trade-offs

### [Risk] Complexity
**Mitigation**:
- Implement incrementally
- Start with core modules first
- Use existing patterns from original codebase

### [Risk] Performance
**Mitigation**:
- Use async execution
- Optimize hot paths
- Profile regularly

### [Risk] Compatibility
**Mitigation**:
- Follow original API design
- Test with original test suite
- Document differences

### [Risk] Maintainability
**Mitigation**:
- Follow Rust best practices
- Use clear module structure
- Add comprehensive tests

### [Risk] Dependencies
**Mitigation**:
- Prefer standard library
- Minimize external dependencies
- Use well-maintained crates

## Migration Plan

### Phase 1: Core Infrastructure (Week 1-2)
1. Effect system implementation
2. Event bus system
3. Command system foundation
4. Global state management

### Phase 2: CLI & Format (Week 3-4)
1. CLI command system
2. Format system
3. IDE integration
4. Installation management

### Phase 3: Advanced Features (Week 5-6)
1. Snapshot/versioning
2. Sync system
3. Worktree management
4. Account management

### Phase 4: Communication (Week 7-8)
1. ACP protocol
2. Auth module
3. Control plane
4. File system enhancements

### Phase 5: Integration (Week 9-10)
1. Flag system
2. ID generation
3. Patch system
4. Permission system

### Phase 6: Polish (Week 11-12)
1. Plugin system
2. Project detection
3. PTY support
4. Question system

### Phase 7: Server & Share (Week 13-14)
1. Server system
2. Share system
3. Shell integration
4. Skill system

### Phase 8: Utilities (Week 15-16)
1. Util system
2. Storage system
3. Testing & integration
4. Documentation

## Open Questions

1. **Effect System**: Should we implement a full effect system or just basic patterns?

2. **Event Bus**: Should we use broadcast or multicast channels?

3. **CLI Commands**: How to handle command plugins?

4. **Format System**: Should we support multiple formatters per language?

5. **IDE Integration**: Which IDEs should we support first?

6. **Storage**: Should we use SQLite or a custom storage solution?

7. **Permission**: How granular should permissions be?

8. **Plugin**: How to handle plugin dependencies?

9. **Server**: What API format should we use?

10. **Testing**: How to test with the original test suite?
