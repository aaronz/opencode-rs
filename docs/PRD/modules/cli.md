# PRD: cli Module

## Module Overview

**Module Name:** `cli`
**Type:** Core
**Source:** `/packages/opencode/src/cli/`

## Purpose

Command-line interface implementation providing entry point and command routing for all CLI operations. Uses yargs for argument parsing.

## Functionality

### CLI Commands (22 commands)

| Command | File | Description |
|---------|------|-------------|
| `run` | `cmd/run.ts` | Run agent in current directory |
| `models` | `cmd/models.ts` | List available AI models |
| `providers` | `cmd/providers.ts` | List AI providers |
| `agent` | `cmd/agent.ts` | Agent management |
| `serve` | `cmd/serve.ts` | Start API server |
| `web` | `cmd/web.ts` | Start web interface |
| `mcp` | `cmd/mcp.ts` | MCP server management |
| `acp` | `cmd/acp.ts` | Agent Communication Protocol |
| `tui attach` | `cmd/tui/attach.ts` | Attach to TUI session |
| `tui thread` | `cmd/tui/thread.ts` | TUI thread operations |
| `session` | `cmd/session.ts` | Session management |
| `db` | `cmd/db.ts` | Database operations |
| `github` | `cmd/github.ts` | GitHub integration |
| `pr` | `cmd/pr.ts` | Pull request operations |
| `export` | `cmd/export.ts` | Export session/data |
| `import` | `cmd/import.ts` | Import session/data |
| `plug` | `cmd/plug.ts` | Plugin management |
| `stats` | `cmd/stats.ts` | Usage statistics |
| `debug` | `cmd/debug/` | Debug utilities |
| `upgrade` | `cmd/upgrade.ts` | Self-upgrade |
| `uninstall` | `cmd/uninstall.ts` | Uninstall |
| `generate` | `cmd/generate.ts` | Code generation |

### Entry Point

```typescript
// index.ts - Main CLI entry using yargs
const cli = yargs(args)
  .scriptName("opencode")
  .version(InstallationVersion)
  .option("print-logs", { describe: "print logs to stderr", type: "boolean" })
  .option("log-level", { describe: "log level", type: "string" })
  .option("pure", { describe: "run without external plugins", type: "boolean" })
  .command(AcpCommand)
  .command(McpCommand)
  .command(RunCommand)
  // ... more commands
```

### Global Options

| Option | Description |
|--------|-------------|
| `--print-logs` | Print logs to stderr |
| `--log-level` | Log level (DEBUG, INFO, WARN, ERROR) |
| `--pure` | Run without external plugins |
| `-h, --help` | Show help |
| `-v, --version` | Show version |

### CLI Structure

```
cli/
├── index.ts           # Entry point, yargs setup
├── cmd.ts             # Command builder helper
├── ui.ts              # UI utilities
├── error.ts           # Error formatting
├── heap.ts            # Heap memory tracking
├── ui.ts              # Logo, colors, output
├── cmd/
│   ├── run.ts         # opencode run [directory]
│   ├── models.ts      # opencode models [provider]
│   ├── providers.ts   # opencode providers
│   ├── agent.ts       # opencode agent
│   ├── serve.ts       # opencode serve
│   ├── web.ts         # opencode web
│   ├── mcp.ts         # opencode mcp
│   ├── acp.ts         # opencode acp
│   ├── session.ts     # opencode session
│   ├── db.ts          # opencode db
│   ├── github.ts      # opencode github
│   ├── pr.ts          # opencode pr
│   ├── export.ts      # opencode export
│   ├── import.ts      # opencode import
│   ├── plug.ts        # opencode plug
│   ├── stats.ts       # opencode stats
│   ├── upgrade.ts     # opencode upgrade
│   ├── uninstall.ts   # opencode uninstall
│   ├── generate.ts    # opencode generate
│   ├── tui/
│   │   ├── attach.ts  # opencode tui attach
│   │   └── thread.ts  # opencode tui thread
│   └── debug/
│       └── ...        # Debug subcommands
```

### Process Lifecycle

1. Parse arguments with yargs
2. Initialize logging
3. Start heap tracking
4. Set process metadata (AGENT, OPENCODE, OPENCODE_PID)
5. Run database migration if needed (one-time)
6. Execute command handler
7. Handle errors and exit

### Database Migration

On first run, performs one-time database migration:
- Shows progress bar with percentage
- Displays label for current migration step
- Handles both TTY and non-TTY output modes

## Dependencies

- `yargs` - CLI argument parsing
- `installation` - Version info
- `global` - Global paths
- `storage` - Database
- `Log` - Logging

## Acceptance Criteria

1. All 22 commands are registered and functional
2. Global options work correctly
3. Help output is clear and complete
4. Error messages are user-friendly
5. Database migration runs on first run

## Rust Implementation Guidance

The Rust equivalent should:
- Use `clap` for CLI argument parsing
- Implement subcommands for each command
- Use `anyhow` for error handling
- Consider using `tracing` for logging
