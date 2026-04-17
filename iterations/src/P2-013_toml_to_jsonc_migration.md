# TOML to JSONC Configuration Migration Guide

**Date:** 2026-04-17
**Status:** Complete (TOML support removed - see P2-014)
**Related Tasks:** P2-010, P2-011, P2-012, P2-013, **P2-014**

---

## Overview

> **IMPORTANT:** TOML configuration support has been completely removed as of P2-014. JSONC is now the only supported configuration format. If you still have a `config.toml` file, you must manually convert it to JSONC format.

OpenCode RS has migrated from TOML configuration format to JSONC (JSON with Comments). This guide explains the migration process, automatic conversion behavior, and manual migration steps.

## Why JSONC?

JSONC offers several advantages over TOML:

1. **Comments support** - JSONC allows `//` and `/* */` style comments, improving configuration documentation
2. **Consistency** - Aligns with the broader JavaScript/TypeScript ecosystem
3. **Single format** - Reduces maintenance burden of supporting multiple config formats
4. **Better tooling** - Native JSON support in most editors and tools

## Automatic Migration

### How It Works

When OpenCode RS detects a TOML configuration file, it automatically:

1. **Parses the TOML** configuration
2. **Shows a deprecation warning** in logs
3. **Converts to JSONC** format alongside the original TOML
4. **Continues operation** using the TOML file (for backward compatibility)

### Example Deprecation Warning

```
WARN  opencode_config: TOML configuration format is deprecated and will be removed in a future release. Auto-converting config.toml to JSONC format.
```

### Generated Files

After automatic conversion, you will see:

```
~/.config/opencode/
├── config.toml        # Original TOML (can be deleted after migration)
└── config.jsonc       # New JSONC format (recommended)
```

## Manual Migration

If you prefer manual migration or need to understand the changes:

### Step 1: Export Current Configuration

```bash
# Find your current config location
opencode config path
```

### Step 2: Convert Format

#### TOML to JSONC Mapping

| TOML Feature | JSONC Equivalent | Example |
|--------------|------------------|---------|
| Section headers `[section]` | Nested objects | `[server]` → `"server": {}` |
| Arrays `[]` | Arrays `[]` | Same syntax |
| Strings `""` | Strings `""` | Same syntax |
| Multiline strings `"""` | Use `\n` or template | Single line strings |
| Inline tables `{a = 1}` | Objects `{"a": 1}` | Spread to separate lines |
| Comments `# comment` | `// comment` or `/* */` | Comments preserved |

### Step 3: Configuration Schema

#### Server Configuration

**TOML (deprecated):**
```toml
[server]
port = 3000
hostname = "127.0.0.1"

[server.desktop]
enabled = true
auto_open_browser = true

[server.acp]
enabled = true
server_id = "local"
version = "1.0"
```

**JSONC:**
```jsonc
{
  "server": {
    "port": 3000,
    "hostname": "127.0.0.1",
    "desktop": {
      "enabled": true,
      "auto_open_browser": true
    },
    "acp": {
      "enabled": true,
      "server_id": "local",
      "version": "1.0"
    }
  }
}
```

#### Agent Configuration

**TOML (deprecated):**
```toml
[[agent]]
name = "default"
mode = "build"

[[agent]]
name = "review"
mode = "review"
```

**JSONC:**
```jsonc
{
  "agent": [
    {
      "name": "default",
      "permission": "build"
    },
    {
      "name": "review",
      "permission": "review"
    }
  ]
}
```

#### LLM Provider Configuration

**TOML (deprecated):**
```toml
[llm]
provider = "openai"
model = "gpt-4"

[llm.openai]
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"
```

**JSONC:**
```jsonc
{
  "llm": {
    "provider": "openai",
    "model": "gpt-4",
    "openai": {
      "api_key": "${OPENAI_API_KEY}",
      "base_url": "https://api.openai.com/v1"
    }
  }
}
```

## Deprecated Fields

The following fields are deprecated and will be removed in v4.0:

| Old Field | New Field | Migration |
|-----------|-----------|-----------|
| `agent[].mode` | `agent[].permission` | Use permission levels: `build`, `plan`, `general`, `expert`, `review`, `debug` |
| `tools` | `permission` | Use new permission structure |
| `theme` | `tui.json` | Theme moved to separate TUI config |
| `keybinds` | `tui.json` | Keybinds moved to separate TUI config |

### Deprecated Field Warning Example

```
WARN  opencode_config: Deprecated config field 'agent.default.mode' detected: Use 'permission' field instead. Will be removed in v4.0. See https://docs.opencode.ai/config/migration for migration guide.
```

## Variable Substitution

Both TOML and JSONC support variable substitution using `${VAR_NAME}` syntax:

```jsonc
{
  "llm": {
    "openai": {
      "api_key": "${OPENAI_API_KEY}",
      "base_url": "${OPENAI_BASE_URL:-https://api.openai.com/v1}"
    }
  }
}
```

Default values can be specified with `:-` syntax:
- `${VAR:-default}` - Use "default" if VAR is unset

## File Locations

Configuration is searched in this order:

1. `$OPENCODE_CONFIG_DIR/config.json`
2. `$OPENCODE_CONFIG_DIR/config.jsonc`
3. `$OPENCODE_CONFIG_DIR/config.toml` (deprecated)
4. `~/.config/opencode/config.json`
5. `~/.config/opencode/config.jsonc`
6. `~/.config/opencode/config.toml` (deprecated)

## Migration Checklist

- [ ] Identify current config file location
- [ ] Review deprecation warnings in logs
- [ ] Create backup of existing TOML config
- [ ] Manually convert or let auto-conversion create JSONC
- [ ] Update deprecated field usage
- [ ] Move theme to `tui.json` if used
- [ ] Move keybinds to `tui.json` if used
- [ ] Verify config loads without errors: `opencode config validate`
- [ ] Delete old TOML file after successful migration

## Troubleshooting

### Config Not Loading

```bash
# Check config path
opencode config path

# Validate config syntax
opencode config validate --path ~/.config/opencode/config.jsonc
```

### Conversion Failures

If automatic conversion fails:

1. Check TOML syntax: [TOML lint](https://www.toml-lint.io/)
2. Verify file permissions
3. Check disk space

### Migration Issues

Common issues and solutions:

| Issue | Solution |
|-------|----------|
| Missing `permission` field | Add permission level explicitly |
| Array of tables syntax | Convert `[[agent]]` to JSON array format |
| Inline tables | Expand to separate objects |

## Future Removal

> **UPDATE (P2-014):** TOML support has been completely removed. The following was the planned removal that has now been completed:

1. ~~JSONC will be the only supported format~~
2. ~~Auto-conversion will no longer occur~~
3. ~~TOML files will be ignored~~ - TOML files are now rejected with an error

If you have a `config.toml` file, you must manually convert it to `config.jsonc` format.

## References

- [Config Module Documentation](./opencode-rust/crates/config/src/lib.rs)
- [JSONC Parser](./opencode-rust/crates/config/src/jsonc.rs)
- [Spec v29](./iteration-29/spec_v29.md) - FR-027
- [Gap Analysis](./iteration-29/gap-analysis.md)
