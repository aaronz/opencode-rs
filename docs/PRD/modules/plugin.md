# PRD: plugin Module

## Module Overview

**Module Name:** `plugin`
**Type:** Integration
**Source:** `/packages/opencode/src/plugin/`

## Purpose

Plugin system for extensibility. Allows external plugins to add providers, tools, and hooks.

## Functionality

### Core Features

1. **Plugin Loading**
   - Load plugins from `opencode.plugins` config
   - Plugin discovery
   - Plugin initialization

2. **Plugin Hooks**

   | Hook | Timing | Purpose |
   |------|--------|---------|
   | `config` | Before config loaded | Modify configuration |
   | `provider` | Provider init | Add custom providers |
   | `models` | Model loading | Add/modify models |
   | `auth` | Auth loading | Custom auth loader |
   | `tool` | Tool init | Add custom tools |

3. **Plugin Interface**

   ```typescript
   interface Plugin {
     name: string
     version: string
     provider?: {
       id: string
       models?: (provider: ProviderInfo, context: Context) => Promise<Record<string, Model>>
       auth?: {
         provider: string
         loader: (getAuth: () => Promise<AuthInfo>, provider: ProviderInfo) => Promise<any>
       }
     }
   }
   ```

### Configuration

```json
{
  "plugins": [
    {
      "name": "my-plugin",
      "path": "./plugins/my-plugin"
    }
  ]
}
```

## Dependencies

- Config module for plugin discovery

## Acceptance Criteria

1. Plugins are discovered and loaded
2. Hooks are called at correct times
3. Plugin errors don't crash main app
4. Plugins can add providers and tools

## Rust Implementation Guidance

The Rust equivalent should:
- Use dynamic library loading (`libloading`)
- Define plugin trait for hooks
- Use `serde` for config
- Implement proper error isolation

## Test Design

### Unit Tests
- `hook_execution`: Mock plugins and verify that `config`, `provider`, and `tool` hooks are called in the correct order.
- `error_isolation`: Ensure that a panicking or erroring plugin does not crash the core application.

### Integration Tests
- `dylib_loading`: Compile a dummy plugin to a `.so`/`.dylib`, load it dynamically, and verify hook execution.

### Rust Specifics
- Test FFI boundaries and  safety carefully.
- Alternatively, if plugins are WASM, test WASM host execution via `wasmtime`.
