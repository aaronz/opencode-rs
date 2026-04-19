# PRD-25: CLI Parity Fixes - Implementation Guide

**Date**: 2026-04-18

---

## 问题 Summary

| # | 问题 | 当前行为 | 期望行为 |
|---|------|---------|---------|
| 1 | 错误返回 exit 2 | exit 1 |
| 2 | `--verbose` 不识别 | exit 0 |
| 3 | session start 不识别 | 执行session start |
| 4 | workspace init 不识别 | 执行workspace init |
| 5 | permissions grant 不识别 | 执行permissions grant |

---

## Files to Modify

```
opencode-rs/opencode-rust/crates/cli/src/main.rs  (主要)
opencode-rs/opencode-rust/crates/cli/src/cmd/     (命令模块)
```

---

## Current CLI Structure (main.rs)

```rust
#[derive(Parser)]
#[command(name = "opencode-rs")]
struct Cli {
    // Global flags
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),
    Serve(ServeArgs),
    Session(SessionArgs),      // ← 当前: 不支持 start/stop/list
    List(ListArgs),
    // ... 其他
}
```

问题: `SessionArgs` 是单一结构, 不支持子命令 (start/stop/list)

---

## Fix 1: 错误返回 exit code 2 → 1

**当前代码** (可能在哪里):
```rust
// 搜索 process::exit(2)
std::process::exit(2);
```

**修改**: 找所有 error 情况用的 `exit(2)` 改为 `exit(1)`

**验证**:
```bash
opencode-rs --invalid-option
echo $?  # 应返回 1
```

---

## Fix 2: 添加 `--verbose` flag

**当前代码** (main.rs 第66行):
```rust
#[arg(short = 'v', long = "version")]
pub version: bool,
```

**修改**: 添加 verbose 作为全局flag
```rust
#[derive(Parser)]
#[command(name = "opencode-rs")]
struct Cli {
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    // 添加这个
    #[arg(short = 'V', long = "verbose", global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}
```

**验证**:
```bash
opencode-rs --verbose --help
echo $?  # 应返回 0
```

---

## Fix 3: session 子命令

**当前代码** (main.rs 第143行):
```rust
#[command(about = "Manage sessions")]
Session(SessionArgs),
```

**修改1**: main.rs 添加 Session 子命令enum
```rust
#[derive(Subcommand, Debug)]
enum SessionCommands {
    Start(session::SessionStart),
    Stop(session::SessionStop),
    List(session::SessionList),
    Status(session::SessionStatus),
}

#[derive(Subcommand, Debug)]
enum Commands {
    // ... existing
    Session(SessionCommands),  // 改为 Subcommand
}
```

**修改2**: cmd/session.rs 添加子命令结构
```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "session")]
pub struct SessionStart {
    // options like --continue, --fork, etc
}

#[derive(Parser, Debug)]
#[command(name = "stop")]
pub struct SessionStop {}

#[derive(Parser, Debug)]
#[command(name = "list")]
pub struct SessionList {}

#[derive(Parser, Debug)]
#[command(name = "status")]
pub struct SessionStatus {}
```

**Verification**:
```bash
opencode-rs session start
# 不应再报 "unexpected argument 'start'"
```

---

## Fix 4: workspace 子命令

**修改1**: main.rs
```rust
#[derive(Subcommand, Debug)]
enum WorkspaceCommands {
    Init(workspace::WorkspaceInit),
    Status(workspace::WorkspaceStatus),
    List(workspace::WorkspaceList),
    Cleanup(workspace::WorkspaceCleanup),
    Config(workspace::WorkspaceConfig),
}

#[derive(Subcommand, Debug)]
enum Commands {
    // ...
    Workspace(WorkspaceCommands),
}
```

**修改2**: cmd/workspace.rs
```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "init")]
pub struct WorkspaceInit {}

#[derive(Parser, Debug)]
#[command(name = "status")]
pub struct WorkspaceStatus {}

#[derive(Parser, Debug)]
#[command(name = "list")]
pub struct WorkspaceList {}

#[derive(Parser, Debug)]
#[command(name = "cleanup")]
pub struct WorkspaceCleanup {}

#[derive(Parser, Debug)]
#[command(name = "config")]
pub struct WorkspaceConfig {}
```

---

## Fix 5: permissions 子命令

**修改1**: main.rs
```rust
#[derive(Subcommand, Debug)]
enum PermissionsCommands {
    Grant(permissions::PermissionsGrant),
    Revoke(permissions::PermissionsRevoke),
}

#[derive(Subcommand, Debug)]
enum FileCommands {
    Read(files::FileRead),
}

#[derive(Subcommand, Debug)]
enum Commands {
    // ...
    Permissions(Permissions::PermissionsCommands),
    File(FileCommands),
}
```

**修改2**: cmd/permissions.rs
```rust
#[derive(Parser, Debug)]
pub struct PermissionsGrant {
    pub path: String,
}

#[derive(Parser, Debug)]
pub struct PermissionsRevoke {
    pub path: String,
}
```

**修改3**: cmd/files.rs
```rust
#[derive(Parser, Debug)]
pub struct FileRead {
    pub path: String,
}
```

---

## Build & Verify

```bash
# Build
cd ~/Documents/GitHub/opencode-rs
./build.sh

# Copy to PATH
cp target/release/opencode-rs ~/.opencode/bin/opencode-rs

# Test individual commands
opencode-rs session start
opencode-rs workspace init
opencode-rs permissions grant /path

# Run harness tests
cd ~/Documents/GitHub/opencode-harness
cargo run -- run --task harness/tasks/cli
```

**Expected Results**: Exit code mismatches resolved

---

## Verification Commands

| Test | Command | Expected Exit |
|------|---------|---------------|
| invalid option | `opencode-rs --invalid` | 1 |
| verbose help | `opencode-rs --verbose --help` | 0 |
| session start | `opencode-rs session start` | 1 (no active session) |
| workspace init | `opencode-rs workspace init` | 1 or 0 |
| permissions | `opencode-rs permissions grant /tmp` | 1 or 0 |