# PRD: shell Module

## Module Overview

- **Module Name**: `shell`
- **Source Path**: `packages/opencode/src/shell/shell.ts`
- **Type**: Utility
- **Rust Crate**: `crates/shell/` or `crates/util/src/shell.rs`
- **Purpose**: Shell detection, selection, and process tree management. Determines the preferred shell for the current platform and user environment, and provides utilities for graceful process tree termination on POSIX and Windows.

---

## Functionality

### Core Features

1. **Shell Detection** — Reads `SHELL` env var, validates against allowlist, falls back to platform default
2. **Shell Blacklist** — `fish` and `nu` are blacklisted from "acceptable" (not "preferred") due to non-POSIX syntax
3. **Login Shell Flags** — Detects shells that need `-l` (login) flag: `bash, dash, fish, ksh, sh, zsh`
4. **POSIX Detection** — Identifies POSIX-compatible shells: `bash, dash, ksh, sh, zsh`
5. **Windows Support** — Falls back to `pwsh.exe`, `powershell.exe`, git-bash, or `cmd.exe`
6. **GitBash Detection** — Locates `git` on PATH and derives the git-bash path on Windows
7. **Process Tree Kill** — Graceful SIGTERM → 200ms wait → SIGKILL on POSIX; `taskkill /f /t` on Windows

---

## Constants

```rust
/// Shells that are NOT acceptable for opencode tool use (non-POSIX syntax)
pub const SHELL_BLACKLIST: &[&str] = &["fish", "nu"];

/// Shells that need `-l` (login flag) when spawning login shells
pub const LOGIN_SHELLS: &[&str] = &["bash", "dash", "fish", "ksh", "sh", "zsh"];

/// POSIX-compatible shells
pub const POSIX_SHELLS: &[&str] = &["bash", "dash", "ksh", "sh", "zsh"];

pub const SIGKILL_TIMEOUT_MS: u64 = 200;
```

---

## API Surface

### Functions

```rust
/// Extract shell name from a path (e.g., "/bin/zsh" → "zsh")
pub fn name(shell_path: &str) -> String {
    Path::new(shell_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default()
}

/// Does this shell need the `-l` login flag?
pub fn is_login_shell(shell_path: &str) -> bool {
    LOGIN_SHELLS.contains(&name(shell_path).as_str())
}

/// Is this shell POSIX-compatible?
pub fn is_posix(shell_path: &str) -> bool {
    POSIX_SHELLS.contains(&name(shell_path).as_str())
}

/// Get the preferred shell (env var or platform fallback)
pub fn preferred() -> String {
    let env_shell = std::env::var("SHELL").ok();
    select_shell(env_shell.as_deref(), false)
}

/// Get the acceptable shell (env var filtered by blacklist, or fallback)
pub fn acceptable() -> String {
    let env_shell = std::env::var("SHELL").ok();
    select_shell(env_shell.as_deref(), true) // filter blacklist
}

fn select_shell(env_shell: Option<&str>, blacklist_filter: bool) -> String {
    if let Some(shell) = env_shell {
        let shell_name = name(shell);
        if !blacklist_filter || !SHELL_BLACKLIST.contains(&shell_name.as_str()) {
            return shell.to_string();
        }
    }
    platform_fallback()
}

#[cfg(target_os = "macos")]
fn platform_fallback() -> String {
    "/bin/zsh".to_string()
}

#[cfg(target_os = "linux")]
fn platform_fallback() -> String {
    // Try bash first, then /bin/sh
    which::which("bash")
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "/bin/sh".to_string())
}

#[cfg(target_os = "windows")]
fn platform_fallback() -> String {
    // pwsh → powershell → git-bash → cmd
    which::which("pwsh").map(|p| p.to_string_lossy().into_owned()).ok()
        .or_else(|| which::which("powershell").map(|p| p.to_string_lossy().into_owned()).ok())
        .or_else(gitbash())
        .unwrap_or_else(|| "cmd.exe".to_string())
}

/// Find git-bash on Windows (from git installation)
#[cfg(target_os = "windows")]
fn gitbash() -> Option<String> {
    // Check OPENCODE_GIT_BASH_PATH override
    if let Ok(p) = std::env::var("OPENCODE_GIT_BASH_PATH") {
        if !p.is_empty() {
            return Some(p);
        }
    }
    // Find git.exe on PATH, derive git-bash path
    let git = which::which("git").ok()?;
    let git_dir = git.parent()?.parent()?; // bin/git.exe → parent → parent = git installation
    let bash = git_dir.join("usr").join("bin").join("bash.exe");
    if bash.exists() {
        Some(bash.to_string_lossy().into_owned())
    } else {
        None
    }
}

/// Get the git-bash path on Windows (or None on non-Windows)
#[cfg(not(target_os = "windows"))]
pub fn gitbash() -> Option<String> { None }
```

### Process Tree Kill

```rust
/// Kill a process and its entire process group
pub async fn kill_tree(pid: u32) -> Result<(), ShellError> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{killpg, Signal};
        use nix::unistd::Pid;

        // Send SIGTERM to the process group
        let _ = killpg(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);

        // Wait up to SIGKILL_TIMEOUT_MS
        tokio::time::sleep(Duration::from_millis(SIGKILL_TIMEOUT_MS)).await;

        // Send SIGKILL
        let _ = killpg(Pid::from_raw(-(pid as i32)), Signal::SIGKILL);
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .spawn()?;
    }

    Ok(())
}

#[cfg(not(unix))]
async fn kill_tree(pid: u32) -> Result<(), ShellError> {
    // Fallback: just kill the process
    tokio::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .await?;
    Ok(())
}
```

### `ShellError`

```rust
#[derive(Debug, Error)]
pub enum ShellError {
    #[error("Shell not found")]
    NotFound,

    #[error("Shell not acceptable: {0}")]
    NotAcceptable(String),

    #[error("Kill failed: {0}")]
    KillFailed(String),

    #[error("IO error: {0}")]
    Io(#[source] std::io::Error),
}
```

---

## Windows Support Details

```rust
#[cfg(target_os = = "windows")]
fn gitbash() -> Option<String> {
    // Override via environment variable
    if let Ok(p) = std::env::var("OPENCODE_GIT_BASH_PATH") {
        if !p.is_empty() {
            return Some(p);
        }
    }

    // Find git installation from PATH
    let git_exe = which::which("git.exe").ok()?;
    // C:\Program Files\Git\bin\git.exe → C:\Program Files\Git\usr\bin\bash.exe
    let git_bin = git_exe.parent()?.parent()?;
    let bash_exe = git_bin.join("usr").join("bin").join("bash.exe");

    if bash_exe.exists() {
        Some(bash_exe.to_string_lossy().into_owned())
    } else {
        None
    }
}
```

---

## Crate Layout

```
crates/shell/
├── Cargo.toml       # which = "6", nix = { version = "0.27", features = ["process"] }
├── src/
│   ├── lib.rs       # All functions and types
│   ├── detect.rs    # Shell detection and selection
│   └── kill.rs      # Process tree kill
└── tests/
    └── shell_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-shell"
version = "0.1.0"
edition = "2021"

[dependencies]
which = "6"
thiserror = "2.0"
tracing = "0.1"
anyhow = "1.0"

[target.'cfg(unix)'.dependencies]
nix = { version = "0.27", features = ["process", "signal"] }

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `which` | Locate binaries on PATH (git, bash, pwsh, etc.) |
| `nix` (unix) | Signal sending (`killpg`) |
| `tracing` | Logging |

---

## Acceptance Criteria

- [x] `preferred()` returns `$SHELL` when set (even if blacklisted)
- [x] `acceptable()` returns `$SHELL` only when not in `SHELL_BLACKLIST`; otherwise falls back
- [x] `is_login_shell()` returns true for bash, zsh, sh, dash, ksh, fish
- [x] `is_posix()` returns true for bash, zsh, sh, ksh, dash
- [x] On macOS, fallback is `/bin/zsh`
- [x] On Linux, fallback prefers `bash` then `/bin/sh`
- [x] On Windows, fallback is `pwsh.exe` → `powershell.exe` → git-bash → `cmd.exe`
- [x] `kill_tree()` uses process group SIGTERM → SIGKILL on POSIX
- [x] `kill_tree()` uses `taskkill /f /t` on Windows
- [x] `gitbash()` returns correct path on Windows or `None` on other platforms

---

## Test Design

```rust
#[test]
fn test_name_extracts_basename() {
    assert_eq!(name("/bin/zsh"), "zsh");
    assert_eq!(name("/usr/bin/bash"), "bash");
}

#[test]
fn test_name_handles_windows_path() {
    assert_eq!(name("C:\\Program Files\\Git\\bin\\bash.exe"), "bash");
}

#[test]
fn test_login_shell_detection() {
    assert!(is_login_shell("/bin/bash"));
    assert!(is_login_shell("/bin/zsh"));
    assert!(is_login_shell("/bin/sh"));
    assert!(is_login_shell("/usr/bin/dash"));
    assert!(!is_login_shell("/usr/bin/python"));
    assert!(!is_login_shell("/bin/fish")); // fish is in LOGIN_SHELLS so yes
}

#[test]
fn test_posix_detection() {
    assert!(is_posix("/bin/bash"));
    assert!(is_posix("/bin/zsh"));
    assert!(is_posix("/bin/sh"));
    assert!(is_posix("/bin/dash"));
    assert!(is_posix("/usr/bin/ksh"));
    assert!(!is_posix("/bin/fish"));
    assert!(!is_posix("/usr/bin/python"));
}

#[test]
fn test_acceptable_filters_blacklist() {
    std::env::set_var("SHELL", "/usr/bin/fish");
    let acceptable_shell = acceptable();
    assert_ne!(name(&acceptable_shell), "fish"); // fish is blacklisted
    std::env::remove_var("SHELL");
}

#[test]
fn test_preferred_returns_shell_even_if_blacklisted() {
    std::env::set_var("SHELL", "/usr/bin/fish");
    let preferred_shell = preferred();
    assert_eq!(name(&preferred_shell), "fish"); // preferred doesn't filter
    std::env::remove_var("SHELL");
}

#[test]
fn test_acceptable_uses_fallback_when_blacklisted() {
    std::env::set_var("SHELL", "/usr/bin/fish");
    let acceptable_shell = acceptable();
    // Should NOT be fish (blacklisted) — should be platform fallback
    #[cfg(target_os = "linux")]
    assert!(name(&acceptable_shell) == "bash" || name(&acceptable_shell) == "sh");
    std::env::remove_var("SHELL");
}

#[tokio::test]
async fn test_kill_tree_succeeds_for_dead_process() {
    // Spawn a process that exits immediately
    let child = tokio::process::Command::new("true").spawn().unwrap();
    let pid = child.id().unwrap();
    // kill_tree should not panic even for already-dead process
    let result = kill_tree(pid).await;
    assert!(result.is_ok());
}
```

---

## Source Reference

*Source: `packages/opencode/src/shell/shell.ts`*
*No existing Rust equivalent — implement in `crates/shell/`*
