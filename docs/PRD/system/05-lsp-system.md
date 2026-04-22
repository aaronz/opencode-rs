# PRD: LSP (Language Server Protocol) Integration

## Overview

OpenCode integrates with Language Server Protocol servers to provide code intelligence for LLM interactions.

---

## Built-in LSP Servers

| LSP Server | Extensions | Requirement |
|------------|-----------|-------------|
| astro | .astro | Auto-installed for Astro projects |
| bash | .sh, .bash, .zsh, .ksh | auto-installed bash-language-server |
| clangd | .c, .cpp, .cc, .cxx, .h, .hpp, .hh, .hxx | Auto-installed for C/C++ projects |
| csharp | .cs | .NET SDK required |
| clojure-lsp | .clj, .cljs, .cljc, .edn | `clojure-lsp` command |
| dart | .dart | `dart` command |
| deno | .ts, .tsx, .js, .jsx, .mjs | `deno` with deno.json |
| elixir-ls | .ex, .exs | `elixir` command |
| eslint | .ts, .tsx, .js, .jsx, .mjs, .cjs, .mts, .cts, .vue | ESLint in project |
| fsharp | .fs, .fsi, .fsx, .fsscript | .NET SDK required |
| gleam | .gleam | `gleam` command |
| gopls | .go | `go` command |
| hls | .hs, .lhs | `haskell-language-server-wrapper` |
| jdtls | .java | Java SDK 21+ |
| julials | .jl | Julia + LanguageServer.jl |
| kotlin-ls | .kt, .kts | Auto-installed for Kotlin projects |
| lua-ls | .lua | Auto-installed |
| nixd | .nix | `nixd` command |
| ocaml-lsp | .ml, .mli | `ocamllsp` command |
| oxlint | .ts, .tsx, .js, .jsx, .mjs, .cjs, .mts, .cts, .vue, .astro, .svelte | oxlint in project |
| php | .php | Auto-installed intelephense |
| prisma | .prisma | `prisma` command |
| pyright | .py, .pyi | pyright dependency |
| ruby-lsp | .rb, .rake, .gemspec, .ru | ruby + gem commands |
| rust | .rs | `rust-analyzer` command |
| sourcekit-lsp | .swift, .objc, .objcpp | Xcode (macOS) or swift |
| svelte | .svelte | Auto-installed |
| terraform | .tf, .tfvars | Auto-installed from GitHub |
| tinymist | .typ, .typc | Auto-installed |
| typescript | .ts, .tsx, .js, .jsx, .mjs, .cjs, .mts, .cts | typescript in project |
| vue | .vue | Auto-installed |
| yaml-ls | .yaml, .yml | Auto-installed |
| zls | .zig, .zon | `zig` command |

---

## LSP Configuration

### Global Enable/Disable

```json
{
  "lsp": false  // disable all LSP servers
}
```

### Per-Server Configuration

```json
{
  "lsp": {
    "typescript": {
      "disabled": true
    },
    "rust": {
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

### Custom LSP Server

```json
{
  "lsp": {
    "custom-lsp": {
      "command": ["custom-lsp-server", "--stdio"],
      "extensions": [".custom"],
      "env": {
        "CUSTOM_VAR": "value"
      },
      "initialization": {
        "preferences": {
          "importModuleSpecifierPreference": "relative"
        }
      }
    }
  }
}
```

### Configuration Options

| Option | Type | Description |
|--------|------|-------------|
| `disabled` | boolean | Disable this LSP server |
| `command` | string[] | Startup command and arguments |
| `extensions` | string[] | File extensions to handle |
| `env` | object | Environment variables |
| `initialization` | object | LSP initialization options |

For full LSP config schema in `opencode.json`, see [Configuration System](./06-configuration-system.md).

---

## Auto-Download

LSP servers are auto-downloaded when:
1. Project contains relevant file extensions
2. Requirements are met (e.g., .NET SDK for C#)

Disable auto-download:
```bash
OPENCODE_DISABLE_LSP_DOWNLOAD=true opencode
```

---

## Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | Config ownership, `lsp` key schema |
| [07-server-api.md](./07-server-api.md) | LSP diagnostics API endpoints |

---

## LSP Tool (Experimental)

When enabled (`OPENCODE_EXPERIMENTAL_LSP_TOOL=true`), LLM can directly interact with LSP:

```json
{
  "permission": {
    "lsp": "allow"
  }
}
```

### Supported Operations

- `goToDefinition`
- `findReferences`
- `hover`
- `documentSymbol`
- `workspaceSymbol`
- `goToImplementation`
- `prepareCallHierarchy`
- `incomingCalls`
- `outgoingCalls`

---

## Diagnostics

LSP diagnostics are used to provide feedback to LLM:
- Errors and warnings from LSP servers appear in context
- Helps LLM write code that passes type checking

---

## PHP Intelephense License

For advanced Intelephense features, place license file at:
- macOS/Linux: `$HOME/intelephense/license.txt`
- Windows: `%USERPROFILE%/intelephense/license.txt`

File should contain only the license key.
