# PRD-25: CLI Parity Fixes

**Updated**: 2026-04-20
**Test Results**: 19/19 tests failing

---

## Test Results Summary

```
CLI         : 6 fails (ExitCode: 3, OutputText: 3)
Session     : 4 fails (ExitCode: 3, OutputText: 1)
Workspace   : 5 fails (ExitCode: 5, OutputText: 0)
Permissions: 4 fails (ExitCode: 4, OutputText: 0)
─────────────────────────────────────────
Total       : 19 fails
```

---

## Issue #1: Exit Code Wrong (Most Critical)

**Pattern**: `legacy=Some(1), rust=Some(2)`

**Root Cause**: clap 默认error exit code是2, 需要改成1

**Fix**: 在main.rs设置error处理

```rust
// main.rs 开头添加
use std::process::ExitCode;

fn main() -> ExitCode {
    // 错误处理逻辑
    // 所有error情况用 process::exit(1)
}
```

---

## Issue #2: Verbose Flag Not Recognized

**Test**: SMOKE-CLI-006 (`--verbose --help`)  
**Result**: `legacy=0, rust=2`

```
error: unexpected argument '--verbose' found
```

**Fix**: main.rs 添加verbose flag

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long = "verbose", global = true)]
    verbose: bool,
    // ...
}
```

---

## Issue #3: Session/workspace/permissions Commands Not Parsed

**Pattern**: "unexpected argument 'start' found"

**Test**: 
- `session start` → exit 2
- `workspace init` → exit 2
- `permissions grant /path` → exit 2

**Root Cause**: `session`、`workspace`、`permissions` 没有作为subcommand正确实现

**Fix**: main.rs 修改命令结构

```rust
// 当前错误的写法
Session(SessionArgs),

// 正确的写法 - 用enum
#[derive(Subcommand)]
enum SessionCommands {
    Start(SessionStart),
    Stop(SessionStop),
    List(SessionList),
    Status(SessionStatus),
}

#[derive(Subcommand)]
enum Commands {
    Session(SessionCommands),  // 嵌套
    Workspace(WorkspaceCommands),
    Permissions(PermissionsCommands),
}
```

---

## File Changes

| File | Change |
|------|--------|
| `crates/cli/src/main.rs` | 1. error exit code 2→1 |
| `crates/cli/src/main.rs` | 2. 添加 `--verbose` |
| `crates/cli/src/main.rs` | 3. Session/workspace/permissions 改用嵌套Subcommand |

---

## Implementation Steps

### Step 1: Fix Error Exit Code

```rust
// 搜索 exit(2) 改为 exit(1)
```

### Step 2: Add Verbose Flag

```rust
// main.rs Cli struct添加
#[arg(long = "verbose", global = true)]
pub verbose: bool,
```

### Step 3: Fix Session Command

```rust
// main.rs 添加
#[derive(Subcommand, Debug)]
enum SessionCommands {
    Start,
    Stop,
    List,
    Status,
}

// 修改enum
#[derive(Subcommand, Debug)]
enum Commands {
    Session(SessionCommands),
    // 不要写作 Session(SessionArgs)
}
```

### Step 4: Fix Workspace Command (same pattern)

```rust
#[derive(Subcommand, Debug)]
enum WorkspaceCommands {
    Init,
    Status,
    List,
    Cleanup,
    Config,
}
```

### Step 5: Fix Permissions Command

```rust
#[derive(Subcommand, Debug)]
enum PermissionsCommands {
    Grant { path: String },
    Revoke { path: String },
}

#[derive(Subcommand, Debug)]
enum FileCommands {
    Read { path: String },
}
```

---

## Build & Test

```bash
# Build
cd ~/Documents/GitHub/opencode-rs
./build.sh
cp target/release/opencode-rs ~/.opencode/bin/opencode-rs

# Test individual fixes
~/.opencode/bin/opencode-rs --invalid  # 应返回1
~/.opencode/bin/opencode-rs --verbose --help  # 应返回0
~/.opencode/bin/opencode-rs session start  # 应识别,不是"unexpected argument"

# Run full test
cd ~/Documents/GitHub/opencode-harness
cargo run -- run --task harness/tasks/cli
```

---

## Expected Results After Fix

| Test | Current | After Fix |
|------|---------|----------|
| CLI-001 --help | Output diff | Output diff (acceptable) |
| CLI-002 --version | Output diff | Output diff (acceptable) |
| CLI-003 workspace | exit 2 | exit 1 |
| CLI-004 invalid | exit 2 | exit 1 ✅ |
| CLI-005 config | exit 1 | exit 0 ✅ ALREADY WORKS |
| CLI-006 verbose | exit 2 | exit 0 |
| Session-001 start | exit 2 | exit 1 |
| Session-002 stop | exit 2 | exit 1 |
| Session-003 list | output | output (acceptable) |
| Session-004 status | exit 2 | exit 1 |
| Workspace-001 | exit 2 | exit 1 |
| Workspace-002 | exit 2 | exit 1 |
| Workspace-003 | exit 0 | exit 0 ✅ |
| Workspace-004 | exit 2 | exit 1 |
| Workspace-005 | exit 2 | exit 1 |
| Perm-001 | exit 2 | exit 1 |
| Perm-002 | exit 2 | exit 1 |
| Perm-003 | exit 2 | exit 1 |
| Perm-004 | exit 2 | exit 1 |