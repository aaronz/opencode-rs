# OpenCode Rust Monorepo

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)

A Rust implementation of the OpenCode AI coding agent with a comprehensive TUI testing framework.

## Projects

### opencode-rust
Rust implementation of OpenCode AI coding agent featuring:
- **Core**: Session management, tool registry, and error handling
- **Agent**: Agent implementations with LLM integration
- **LLM**: Provider integrations for various language models
- **Tools**: File operations, grep, git, and other developer tools
- **TUI**: Terminal user interface built with ratatui
- **MCP**: Model Context Protocol support
- **Server**: HTTP server with actix-web
- **Storage**: SQLite-based session persistence

### ratatui-testing
A comprehensive testing framework for Rust TUI applications:
- Snapshot testing for TUI output
- PTY simulation for interactive testing
- CLI testing utilities
- State testing helpers

## Building

```bash
# Build from project root (recommended)
./build.sh

# Build release
cargo build --release

# Build debug
cargo build

# Run all tests
cargo test

# Build with all features
cargo build --all-features
```

### Build Script Options

```bash
./build.sh           # Release build (default)
./build.sh --debug   # Debug build
./build.sh --test    # Build + run tests
```

## Structure

```
opencode-rust/
├── crates/
│   ├── core/       # Core functionality
│   ├── cli/        # CLI commands
│   ├── llm/        # LLM provider integrations
│   ├── tools/      # Tool implementations
│   ├── tui/        # Terminal UI
│   ├── agent/      # Agent implementations
│   └── ...         # Other modules
└── tests/          # Integration tests

ratatui-testing/
├── src/            # Framework source
└── tests/          # Integration tests
```

## Scripts

### iterate-prd.sh - PRD迭代开发
主迭代脚本，执行完整的PRD差距分析 → Spec更新 → Plan生成 → 任务实现流程。

```bash
# 基本用法（使用默认PRD.md）
./iterate-prd.sh

# 指定PRD文件
./iterate-prd.sh --prd path/to/prd.md

# 指定PRD文件夹（自动合并多个md文件）
./iterate-prd.sh --prd path/to/prd-folder/

# 指定模型
./iterate-prd.sh --model anthropic/claude-3-5-sonnet

# 指定最大实现轮次
./iterate-prd.sh --rounds 5

# 恢复中断的迭代
./iterate-prd.sh --resume 3

# 启用详细日志
./iterate-prd.sh --verbose

# 启用会话共享
./iterate-prd.sh --share
```

**参数说明：**
| 参数 | 说明 |
|------|------|
| `--prd\|-p` | PRD文件或文件夹路径 |
| `--model\|-m` | 使用的LLM模型 |
| `--rounds\|-r` | 最大外循环轮次（默认10） |
| `--resume` | 从指定迭代号恢复 |
| `--verbose\|-v` | 启用详细日志 |
| `--share\|-S` | 共享会话模式 |
| `--log` | 指定日志文件路径 |

**迭代工作流程：**
1. Gap Analysis - 分析PRD与实现的差距
2. Update Spec - 更新规格文档
3. Update Plan/Tasks - 生成任务清单
4. Per-Task Implementation - 逐个实现任务

### run-opencode-loop.sh
OpenCode循环执行脚本，自动化LLM调用和代码实现循环。

```bash
./run-opencode-loop.sh
./run-opencode-loop.sh --verbose
```

### run-tests.sh
自动化测试脚本，运行cargo test并处理测试结果。

```bash
./run-tests.sh
./run-tests.sh --verbose
```

### append_tests.sh
自动追加测试用例到现有测试文件。

```bash
./append_tests.sh <test_file>
```

### build-plugins.sh
构建WASM插件。

```bash
./scripts/build-plugins.sh
```

### iterate-monitor.sh
监控迭代进度和状态的辅助脚本。

```bash
./scripts/iterate-monitor.sh
```

## Documentation

- [AGENTS.md](./AGENTS.md) - AI agent instructions
- [PRD.md](./docs/PRD.md) - Product requirements
- [docs/](docs/) - Design and analysis documents
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Plugin development guide
