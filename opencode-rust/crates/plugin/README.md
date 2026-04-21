# OpenCode Plugin System

WASM-based plugin system for extending OpenCode RS functionality.

## Features

- **WASM Runtime**: Secure sandboxed plugin execution using wasmi
- **Tool Registration**: Plugins can register custom tools
- **Lifecycle Hooks**: Hook into session start/end, tool calls, and messages
- **Permission System**: Configurable filesystem and network access

## Documentation

- [Plugin Development Guide](../../docs/plugin-dev.md) - Detailed plugin development instructions

## Building Plugins

```bash
cd plugins/hello_world
./build.sh
```

## Loading Plugins

Plugins are discovered from:
- **Global**: `~/.config/opencode/plugins/`
- **Project**: `./.opencode/plugins/` (project root)

## License

MIT