# PRD: shell Module

## Module Overview

- **Module Name**: shell
- **Source Path**: `packages/opencode/src/shell/shell.ts`
- **Type**: Utility
- **Purpose**: Shell detection, selection, and process tree management. Determines the preferred shell for the current platform and user environment, and provides utilities for graceful process tree termination.

---

## Functionality

### Core Features

1. **Shell Detection** — Reads `SHELL` env var, validates against allowlist, falls back to platform default
2. **Shell Blacklist** — `fish` and `nu` are blacklisted as "unacceptable" due to non-POSIX syntax
3. **Login Shell Flags** — Detects shells that need `-l` (login) flag: `bash, dash, fish, ksh, sh, zsh`
4. **POSIX Detection** — Identifies POSIX-compatible shells: `bash, dash, ksh, sh, zsh`
5. **Windows Support** — Falls back to `pwsh.exe`, `powershell.exe`, git-bash, or `cmd.exe`
6. **GitBash Detection** — Locates `git` on PATH and derives the git-bash path on Windows
7. **Process Tree Kill** — Graceful SIGTERM → 200ms wait → SIGKILL on POSIX; `taskkill /f /t` on Windows

### Shell Selection Priority

```
1. $SHELL env var (if not blacklisted)
2. pwsh.exe / powershell.exe (Windows only)
3. Platform fallback:
   - Windows: git-bash → $COMSPEC → cmd.exe
   - macOS: /bin/zsh
   - Linux: bash → /bin/sh
```

---

## API Surface

```typescript
// Exported functions
function name(file: string): string          // basename without extension, lowercase
function login(file: string): boolean        // needs -l flag?
function posix(file: string): boolean        // POSIX compatible?
function gitbash(): string | undefined       // Windows git-bash path
function killTree(proc: ChildProcess, opts?: { exited?: () => boolean }): Promise<void>

// Lazy-evaluated singletons (computed once on first access)
const preferred: string   // $SHELL → validated → fallback
const acceptable: string  // $SHELL → validated (blacklist-filtered) → fallback
```

---

## Data Structures

```typescript
// Constant sets
const BLACKLIST = new Set(["fish", "nu"])
const LOGIN    = new Set(["bash", "dash", "fish", "ksh", "sh", "zsh"])
const POSIX    = new Set(["bash", "dash", "ksh", "sh", "zsh"])

const SIGKILL_TIMEOUT_MS = 200
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `flag` module | `OPENCODE_GIT_BASH_PATH` override |
| `util` module | `Filesystem.stat`, `Filesystem.windowsPath` |
| `util/which` | Locate binaries on PATH |
| `node:child_process` | `spawn` for `taskkill` on Windows |
| `node:timers/promises` | `setTimeout` for SIGKILL delay |

---

## Acceptance Criteria

- [ ] `preferred` returns `$SHELL` when it is not blacklisted
- [ ] `acceptable` returns `$SHELL` only when not in BLACKLIST; otherwise falls back
- [ ] `login()` returns true for bash, zsh, sh, etc.
- [ ] `posix()` returns true for bash, zsh, sh, ksh, dash
- [ ] On macOS, fallback is `/bin/zsh`
- [ ] On Linux, fallback prefers `bash` then `/bin/sh`
- [ ] On Windows, fallback is `pwsh.exe` → `powershell.exe` → git-bash → `cmd.exe`
- [ ] `killTree` uses process group SIGTERM → SIGKILL on POSIX
- [ ] `killTree` uses `taskkill /f /t` on Windows
- [ ] `gitbash()` returns correct path or undefined on non-Windows

---

## Rust Implementation Guidance

### Crate: `crates/shell/` or `crates/util/shell.rs`

### Key Crates

```toml
which = "6"             # Locate binaries on PATH
tokio = { features = ["process"] }
```

### Core Implementation

```rust
pub const BLACKLIST: &[&str] = &["fish", "nu"];
pub const LOGIN_SHELLS: &[&str] = &["bash", "dash", "fish", "ksh", "sh", "zsh"];
pub const POSIX_SHELLS: &[&str] = &["bash", "dash", "ksh", "sh", "zsh"];

pub fn shell_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase()
}

pub fn is_login_shell(path: &str) -> bool {
    LOGIN_SHELLS.contains(&shell_name(path).as_str())
}

pub fn is_posix(path: &str) -> bool {
    POSIX_SHELLS.contains(&shell_name(path).as_str())
}

pub fn preferred_shell() -> String {
    select_shell(std::env::var("SHELL").ok().as_deref(), false)
}

pub fn acceptable_shell() -> String {
    select_shell(std::env::var("SHELL").ok().as_deref(), true) // filters blacklist
}

fn select_shell(env_shell: Option<&str>, blacklist_filter: bool) -> String {
    if let Some(shell) = env_shell {
        let name = shell_name(shell);
        if !blacklist_filter || !BLACKLIST.contains(&name.as_str()) {
            return shell.to_string();
        }
    }
    platform_fallback()
}

#[cfg(target_os = "macos")]
fn platform_fallback() -> String { "/bin/zsh".to_string() }

#[cfg(target_os = "linux")]
fn platform_fallback() -> String {
    which::which("bash").map(|p| p.to_str().unwrap().to_string())
        .unwrap_or_else(|_| "/bin/sh".to_string())
}

#[cfg(target_os = "windows")]
fn platform_fallback() -> String { ... }

pub async fn kill_tree(pid: u32) {
    #[cfg(unix)]
    {
        let _ = nix::sys::signal::killpg(
            nix::unistd::Pid::from_raw(-(pid as i32)),
            nix::sys::signal::Signal::SIGTERM,
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
        let _ = nix::sys::signal::killpg(
            nix::unistd::Pid::from_raw(-(pid as i32)),
            nix::sys::signal::Signal::SIGKILL,
        );
    }
}
```

---

## Test Design

### Unit Tests

```rust
#[test]
fn test_shell_name_extracts_basename() {
    assert_eq!(shell_name("/bin/zsh"), "zsh");
    assert_eq!(shell_name("/usr/bin/bash"), "bash");
    assert_eq!(shell_name("C:\\Program Files\\Git\\bin\\bash.exe"), "bash");
}

#[test]
fn test_login_shell_detection() {
    assert!(is_login_shell("/bin/bash"));
    assert!(is_login_shell("/bin/zsh"));
    assert!(!is_login_shell("/usr/bin/python"));
}

#[test]
fn test_blacklist_filtering() {
    // fish is blacklisted from "acceptable"
    std::env::set_var("SHELL", "/usr/bin/fish");
    let shell = acceptable_shell();
    assert_ne!(shell_name(&shell), "fish");
    // but not from "preferred"
    let preferred = preferred_shell();
    assert_eq!(shell_name(&preferred), "fish");
}

#[test]
fn test_posix_detection() {
    assert!(is_posix("/bin/bash"));
    assert!(!is_posix("/usr/bin/fish"));
}

#[tokio::test]
async fn test_kill_tree_does_not_error_on_dead_process() {
    let child = tokio::process::Command::new("true").spawn().unwrap();
    let pid = child.id().unwrap();
    // Process exits naturally; kill_tree should not panic
    kill_tree(pid).await;
}
```

### Integration Test (from TS patterns)

- `shell.test.ts`: Verify shell selection via env vars, blacklist enforcement, and platform fallbacks
