# PRD: ide Module

## Module Overview

- **Module Name**: ide
- **Source Path**: `packages/opencode/src/ide/index.ts`
- **Type**: Integration Utility
- **Purpose**: IDE detection and extension installation. Detects which IDE the user is running (VS Code, Cursor, Windsurf, etc.) and installs the opencode VS Code extension via the IDE's CLI.

---

## Functionality

### Core Features

1. **IDE Detection** — Reads `TERM_PROGRAM` and `GIT_ASKPASS` environment variables to identify the active IDE
2. **Already Installed Check** — Checks `OPENCODE_CALLER` env var to detect if running inside VS Code/Insiders
3. **Extension Installation** — Runs `<cmd> --install-extension sst-dev.opencode` for the detected IDE
4. **Event Publishing** — Emits `ide.installed` event on the bus after successful installation

### Supported IDEs

| Name | CLI Command |
|------|------------|
| Windsurf | `windsurf` |
| Visual Studio Code - Insiders | `code-insiders` |
| Visual Studio Code | `code` |
| Cursor | `cursor` |
| VSCodium | `codium` |

### Detection Logic

```typescript
if (TERM_PROGRAM === "vscode") {
  GIT_ASKPASS.includes(ide.name) → return ide.name
}
return "unknown"
```

---

## API Surface

```typescript
// Events
Event.Installed = BusEvent.define("ide.installed", z.object({ ide: z.string() }))

// Errors
AlreadyInstalledError   // extension already installed (stdout includes "already installed")
InstallFailedError      // non-zero exit code ({ stderr: string })

// Functions
function ide(): string                     // detect current IDE name or "unknown"
function alreadyInstalled(): boolean       // check OPENCODE_CALLER env var
async function install(ide: SupportedIdeName): Promise<void>
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `bus/bus-event` | Event definition |
| `util/process` | `Process.run()` for CLI invocation |

---

## Acceptance Criteria

- [ ] `ide()` returns the correct IDE name when `TERM_PROGRAM=vscode` and `GIT_ASKPASS` matches
- [ ] `ide()` returns `"unknown"` when no IDE is detected
- [ ] `alreadyInstalled()` returns true when `OPENCODE_CALLER=vscode`
- [ ] `install()` runs the correct CLI command for the target IDE
- [ ] `install()` throws `AlreadyInstalledError` when stdout contains "already installed"
- [ ] `install()` throws `InstallFailedError` on non-zero exit code

---

## Rust Implementation Guidance

### Module: `crates/ide/` or `crates/util/src/ide.rs`

```rust
pub const SUPPORTED_IDES: &[(&str, &str)] = &[
    ("Windsurf", "windsurf"),
    ("Visual Studio Code - Insiders", "code-insiders"),
    ("Visual Studio Code", "code"),
    ("Cursor", "cursor"),
    ("VSCodium", "codium"),
];

pub fn detect_ide() -> &'static str {
    if std::env::var("TERM_PROGRAM").as_deref() == Ok("vscode") {
        if let Ok(askpass) = std::env::var("GIT_ASKPASS") {
            for (name, _) in SUPPORTED_IDES {
                if askpass.contains(name) { return name; }
            }
        }
    }
    "unknown"
}

pub fn already_installed() -> bool {
    matches!(
        std::env::var("OPENCODE_CALLER").as_deref(),
        Ok("vscode") | Ok("vscode-insiders")
    )
}

pub async fn install(ide_name: &str) -> Result<(), IdeError> {
    let cmd = SUPPORTED_IDES.iter()
        .find(|(n, _)| *n == ide_name)
        .map(|(_, c)| *c)
        .ok_or(IdeError::Unknown)?;

    let output = tokio::process::Command::new(cmd)
        .args(["--install-extension", "sst-dev.opencode"])
        .output().await?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("already installed") {
            return Err(IdeError::AlreadyInstalled);
        }
        return Ok(());
    }
    Err(IdeError::InstallFailed {
        stderr: String::from_utf8_lossy(&output.stderr).into()
    })
}
```

---

## Test Design

```rust
#[test]
fn test_detect_ide_returns_vscode() {
    std::env::set_var("TERM_PROGRAM", "vscode");
    std::env::set_var("GIT_ASKPASS", "/usr/lib/Visual Studio Code/bin/code");
    assert_eq!(detect_ide(), "Visual Studio Code");
}

#[test]
fn test_detect_ide_unknown_without_env() {
    std::env::remove_var("TERM_PROGRAM");
    assert_eq!(detect_ide(), "unknown");
}

#[test]
fn test_already_installed_with_caller_env() {
    std::env::set_var("OPENCODE_CALLER", "vscode");
    assert!(already_installed());
}

#[tokio::test]
async fn test_install_returns_already_installed_error() {
    // Mock command that outputs "already installed"
    // Use a wrapper script or mock in tests
}
```

### Integration Tests (from TS patterns)

- `ide.test.ts`: IDE detection via env var mocking, install flow with mock CLI
