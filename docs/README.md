# OpenCode RS Documentation

Welcome to the OpenCode RS documentation. OpenCode RS is a Rust-based AI coding agent providing interactive developer assistance via TUI, HTTP API, and SDK.

## Table of Contents

- [Getting Started](./getting-started.md) - Installation, quick start, configuration
- [SDK Guide](./sdk-guide.md) - Rust SDK for programmatic access
- [Plugin Development](./plugin-dev.md) - WASM plugin system development

## Crate API Reference

The OpenCode RS codebase is organized into the following crates:

| Crate | Description | Documentation |
|-------|-------------|---------------|
| [opencode-core](../opencode-rust/crates/core/src/lib.rs) | Session management, tool registry, error handling | [![docs.rs](https://img.shields.io/docsrs/opencode-core)](https://docs.rs/opencode-core) |
| [opencode-agent](../opencode-rust/crates/agent/src/lib.rs) | Agent orchestration and LLM integration | [![docs.rs](https://img.shields.io/docsrs/opencode-agent)](https://docs.rs/opencode-agent) |
| [opencode-llm](../opencode-rust/crates/llm/src/lib.rs) | Multi-provider LLM adapter (20+ providers) | [![docs.rs](https://img.shields.io/docsrs/opencode-llm)](https://docs.rs/opencode-llm) |
| [opencode-tools](../opencode-rust/crates/tools/src/lib.rs) | File I/O, grep, git, search tools | [![docs.rs](https://img.shields.io/docsrs/opencode-tools)](https://docs.rs/opencode-tools) |
| [opencode-tui](../opencode-rust/crates/tui/src/lib.rs) | Terminal UI built with ratatui | [![docs.rs](https://img.shields.io/docsrs/opencode-tui)](https://docs.rs/opencode-tui) |
| [opencode-server](../opencode-rust/crates/server/src/lib.rs) | HTTP REST API with actix-web | [![docs.rs](https://img.shields.io/docsrs/opencode-server)](https://docs.rs/opencode-server) |
| [opencode-storage](../opencode-rust/crates/storage/src/lib.rs) | SQLite-based session persistence | [![docs.rs](https://img.shields.io/docsrs/opencode-storage)](https://docs.rs/opencode-storage) |
| [opencode-sdk](../opencode-rust/crates/sdk/src/lib.rs) | Public Rust API for external consumers | [![docs.rs](https://img.shields.io/docsrs/opencode-sdk)](https://docs.rs/opencode-sdk) |
| [opencode-auth](../opencode-rust/crates/auth/src/lib.rs) | Authentication (Argon2, JWT, AES-GCM) | [![docs.rs](https://img.shields.io/docsrs/opencode-auth)](https://docs.rs/opencode-auth) |
| [opencode-permission](../opencode-rust/crates/permission/src/lib.rs) | Role-based access control | [![docs.rs](https://img.shields.io/docsrs/opencode-permission)](https://docs.rs/opencode-permission) |
| [opencode-plugin](../opencode-rust/crates/plugin/src/lib.rs) | WASM plugin system | [![docs.rs](https://img.shields.io/docsrs/opencode-plugin)](https://docs.rs/opencode-plugin) |
| [opencode-git](../opencode-rust/crates/git/src/lib.rs) | Git operations | [![docs.rs](https://img.shields.io/docsrs/opencode-git)](https://docs.rs/opencode-git) |
| [opencode-lsp](../opencode-rust/crates/lsp/src/lib.rs) | Language Server Protocol integration | [![docs.rs](https://img.shields.io/docsrs/opencode-lsp)](https://docs.rs/opencode-lsp) |
| [opencode-mcp](../opencode-rust/crates/mcp/src/lib.rs) | Model Context Protocol support | [![docs.rs](https://img.shields.io/docsrs/opencode-mcp)](https://docs.rs/opencode-mcp) |
| [opencode-cli](../opencode-rust/crates/cli/src/lib.rs) | CLI commands and entry points | [![docs.rs](https://img.shields.io/docsrs/opencode-cli)](https://docs.rs/opencode-cli) |
| [opencode-config](../opencode-rust/crates/config/src/lib.rs) | Configuration management | [![docs.rs](https://img.shields.io/docsrs/opencode-config)](https://docs.rs/opencode-config) |
| [opencode-control-plane](../opencode-rust/crates/control-plane/src/lib.rs) | Control plane client | [![docs.rs](https://img.shields.io/docsrs/opencode-control-plane)](https://docs.rs/opencode-control-plane) |

## Quick Links

### Getting Started
- [Installation](./getting-started.md#installation)
- [Quick Start](./getting-started.md#quick-start)
- [Configuration](./getting-started.md#configuration)
- [Agent Modes](./getting-started.md#agent-modes)
- [Available Tools](./getting-started.md#available-tools)

### SDK Guide
- [Installation](./sdk-guide.md#installation)
- [Core Concepts](./sdk-guide.md#core-concepts)
- [Session](./sdk-guide.md#session)
- [Tool Registry](./sdk-guide.md#tool-registry)
- [LLM Providers](./sdk-guide.md#llm-providers)
- [Configuration](./sdk-guide.md#configuration)
- [Session Management](./sdk-guide.md#session-management)
- [Async/Await Pattern](./sdk-guide.md#asyncawait-pattern)

### Plugin Development
- [Architecture](./plugin-dev.md#architecture)
- [Creating a New Plugin](./plugin-dev.md#creating-a-new-plugin)
- [Building Plugins](./plugin-dev.md#building-plugins)
- [Loading Plugins](./plugin-dev.md#loading-plugins)
- [Lifecycle Hooks](./plugin-dev.md#lifecycle-hooks)
- [Plugin Capabilities](./plugin-dev.md#plugin-capabilities)

## Additional Resources

- [Main README](../README.md) - Project overview
- [AGENTS.md](../AGENTS.md) - AI agent instructions
- [ratatui-testing](../ratatui-testing/) - TUI testing framework
- [PRD Documents](./PRD/) - Product requirements and design documents
