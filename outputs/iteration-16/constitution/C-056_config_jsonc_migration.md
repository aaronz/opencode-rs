# Constitution C-056: Configuration Format Migration (TOML → JSONC)

**Version**: 1.0  
**Date**: 2026-04-07  
**Iteration**: v16  
**Status**: Adopted

---

## Preamble

This Constitution documents the design decisions for migrating OpenCode-RS configuration from TOML format to JSONC (JSON with Comments), including backward compatibility, migration tooling, and deprecation handling.

## Article 1: Rationale

### Section 1.1: Why JSONC?

| TOML Advantage | JSONC Advantage |
|----------------|----------------|
| Native Rust support | IDE/editor syntax highlighting |
| Human-readable | Comments for documentation |
| Strict spec | Common web format |
| Table-based hierarchy | Ubiquitous tooling |

**Decision**: JSONC provides better developer experience through IDE support and inline documentation while maintaining JSON compatibility.

### Section 1.2: Migration Strategy

**Phased Approach**:
1. Support both TOML and JSONC during transition
2. Warn users on TOML file usage
3. Provide CLI migration tool
4. Deprecate TOML in future major version

## Article 2: Format Specification

### Section 2.1: JSONC Format

```jsonc
{
  // Global settings
  "model": "claude-sonnet-4",
  
  // Provider configuration
  "providers": {
    "openai": {
      "api_key": "${OPENAI_API_KEY}"  // Environment variable substitution
    }
  },
  
  /* Multi-line
     comment example */
  "agents": [
    { "name": "coder", "model": "claude-sonnet-4" }
  ]
}
```

### Section 2.2: Supported Features

| Feature | TOML | JSONC | Notes |
|---------|------|-------|-------|
| Comments (`//`, `/* */`) | ❌ | ✅ | JSONC only |
| Key-value pairs | ✅ | ✅ | Identical |
| Nested objects | ✅ | ✅ | JSONC uses nested JSON |
| Arrays | ✅ | ✅ | Identical |
| Environment variables | `${VAR}` | `${VAR}` | Both supported |

## Article 3: File Discovery

### Section 3.1: Config Search Order

Files are searched in order of precedence (highest first):

```
1. .opencode/config.jsonc        (project-local)
2. .opencode/config.json          (project-local, fallback)
3. .opencode/config.toml          (project-local, deprecated)
4. ~/.config/opencode/config.jsonc (user-global)
5. ~/.config/opencode/config.json  (user-global, fallback)
6. ~/.config/opencode/config.toml  (user-global, deprecated)
```

### Section 3.2: Migration Trigger

| Condition | Action |
|----------|--------|
| TOML exists, no JSONC | Auto-migrate on first run |
| Both exist | Use JSONC, ignore TOML |
| Only TOML | Use TOML with deprecation warning |

## Article 4: Migration Tooling

### Section 4.1: CLI Commands

```bash
# Check config format
opencode-rs config --info

# Migrate TOML to JSONC
opencode-rs config --migrate

# Remove deprecated TOML after migration
opencode-rs config --remove-toml
```

### Section 4.2: API

```rust
impl Config {
    /// Migrate TOML config to JSONC format
    pub fn migrate_toml_to_jsonc(&self) -> Result<String, ConfigError>;
    
    /// Check if migration is needed
    pub fn needs_migration() -> bool;
}
```

## Article 5: Deprecation Timeline

| Version | TOML Support |
|---------|--------------|
| v1.x | Warn but functional |
| v2.0 | Removed |

### Section 5.1: Deprecation Warning

When TOML config is detected, log:

```
[WARN] TOML configuration is deprecated.
       Please migrate to JSONC: opencode-rs config --migrate
       TOML support will be removed in v2.0
```

## Article 6: Testing Requirements

### Section 6.1: Required Tests

1. **TOML Parsing Test**: Verify existing TOML configs still parse
2. **JSONC Parsing Test**: Verify JSONC with comments parses correctly
3. **Migration Test**: Verify TOML→JSONC conversion preserves data
4. **Merge Test**: Verify multi-file config merge works with both formats
5. **Deprecation Warning Test**: Verify warning is logged for TOML usage

## Article 7: Adoption

This Constitution is effective immediately upon merge to main branch.

---

**Ratified**: 2026-04-07  
**Expires**: Never  
**Amendments**: Requires RFC process
