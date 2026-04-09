# PRD: Formatters and Code Formatting

## Overview

OpenCode automatically formats code after writes/edits using language-specific formatters.

---

## Built-in Formatters

| Formatter | Extensions | Requirement |
|-----------|-----------|-------------|
| air | .R | `air` command |
| biome | .js, .jsx, .ts, .tsx, .html, .css, .md, .json, .yaml | biome.json config |
| cargofmt | .rs | `cargo fmt` |
| clang-format | .c, .cpp, .h, .hpp, .ino | .clang-format config |
| cljfmt | .clj, .cljs, .cljc, .edn | `cljfmt` |
| dart | .dart | `dart` command |
| dfmt | .d | `dfmt` command |
| gleam | .gleam | `gleam` command |
| gofmt | .go | `gofmt` command |
| htmlbeautifier | .erb, .html.erb | `htmlbeautifier` |
| ktlint | .kt, .kts | `ktlint` |
| mix | .ex, .exs, .eex, .heex, .leex, .neex, .sface | `mix` |
| nixfmt | .nix | `nixfmt` |
| ocamlformat | .ml, .mli | .ocamlformat config |
| ormolu | .hs | `ormolu` |
| oxfmt | .js, .jsx, .ts, .tsx | package.json dep + env |
| pint | .php | composer.json dep |
| prettier | .js, .jsx, .ts, .tsx, .html, .css, .md, .json, .yaml | package.json dep |
| rubocop | .rb, .rake, .gemspec, .ru | `rubocop` |
| ruff | .py, .pyi | ruff config |
| rustfmt | .rs | `rustfmt` |
| shfmt | .sh, .bash | `shfmt` |
| standardrb | .rb, .rake, .gemspec, .ru | `standardrb` |
| terraform | .tf, .tfvars | `terraform` |
| uv | .py, .pyi | `uv` command |
| zig | .zig, .zon | `zig` command |

---

## Auto-detection

Formatters auto-detect when:
1. Project has corresponding config file (e.g., `.prettierrc`, `biome.json`)
2. Formatter command is available in PATH
3. Project has formatter as dependency

---

## Configuration

### Disable All

```json
{
  "formatter": false
}
```

### Disable Specific

```json
{
  "formatter": {
    "prettier": {
      "disabled": true
    }
  }
}
```

### Custom Formatter

```json
{
  "formatter": {
    "custom-formatter": {
      "command": ["deno", "fmt", "$FILE"],
      "extensions": [".md"],
      "environment": {
        "NODE_ENV": "development"
      }
    }
  }
}
```

### Configuration Options

| Option | Type | Description |
|--------|------|-------------|
| `disabled` | boolean | Disable this formatter |
| `command` | string[] | Custom command with `$FILE` placeholder |
| `environment` | object | Environment variables |
| `extensions` | string[] | Override handled extensions |

For full formatter config schema, see [Configuration System](./06-configuration-system.md).

---

## Format Trigger

Formats run automatically after:
- `write` tool creates/overwrites file
- `edit` tool modifies file
- `patch` tool applies changes

---

## Format Flow

1. Detect file extension
2. Find matching formatter(s)
3. Execute formatter command with file path
4. Apply formatted content back to file

---

## Manual Format

No direct command, but triggering a file write/edit will auto-format.

---

## Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | Config ownership, formatter config schema |
