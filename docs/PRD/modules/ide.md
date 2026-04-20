# PRD: ide Module

## Module Overview

- **Module Name**: `ide`
- **Source Path**: `packages/opencode/src/ide/index.ts`
- **Type**: Integration Utility
- **Rust Crate**: `crates/ide/` or `crates/util/src/ide.rs`
- **Purpose**: IDE detection and VS Code extension installation. Detects which IDE (VS Code, Cursor, Windsurf, etc.) the user is running and installs the opencode VS Code extension via the IDE's CLI.

---

## Functionality

### Core Features

1. **IDE Detection** — Reads `TERM_PROGRAM` and `GIT_ASKPASS` environment variables to identify the active IDE
2. **Already Installed Check** — Checks `OPENCODE_CALLER` env var to detect if running inside VS Code/Insiders
3. **Extension Installation** — Runs `<cmd> --install-extension sst-dev.opencode` for the detected IDE
4. **Event Publishing** — Emits `ide.installed` event on the bus after successful installation

---

## Supported IDEs

| Name | CLI Command | `GIT_ASKPASS` fragment |
|------|-------------|------------------------|
| Windsurf | `windsurf` | `Windsurf` |
| Visual Studio Code - Insiders | `code-insiders` | `Code - Insiders` |
| Visual Studio Code | `code` | `Code` |
| Cursor | `cursor` | `Cursor` |
| VSCodium | `codium` | `VSCodium` |

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportedIde {
    Windsurf,
    CodeInsiders,
    Code,
    Cursor,
    Vscodium,
    Unknown,
}

impl SupportedIde {
    pub fn cli_name(&self) -> &'static str {
        match self {
            SupportedIde::Windsurf => "windsurf",
            SupportedIde::CodeInsiders => "code-insiders",
            SupportedIde::Code => "code",
            SupportedIde::Cursor => "cursor",
            SupportedIde::Vscodium => "codium",
            SupportedIde::Unknown => return "",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SupportedIde::Windsurf => "Windsurf",
            SupportedIde::CodeInsiders => "Visual Studio Code - Insiders",
            SupportedIde::Code => "Visual Studio Code",
            SupportedIde::Cursor => "Cursor",
            SupportedIde::Vscodium => "VSCodium",
            SupportedIde::Unknown => "unknown",
        }
    }
}
```

### Standalone Functions

```rust
/// Detect the current IDE from environment variables
pub fn detect_ide() -> SupportedIde {
    if std::env::var("TERM_PROGRAM").as_deref() == Ok("vscode") {
        if let Ok(askpass) = std::env::var("GIT_ASKPASS") {
            for ide in [
                (SupportedIde::Windsurf, "Windsurf"),
                (SupportedIde::CodeInsiders, "Code - Insiders"),
                (SupportedIde::Code, "Code"),
                (SupportedIde::Cursor, "Cursor"),
                (SupportedIde::Vscodium, "VSCodium"),
            ] {
                if askpass.contains(ide.1) {
                    return ide.0;
                }
            }
        }
    }
    SupportedIde::Unknown
}

/// Check if opencode is already running inside an IDE
pub fn already_installed() -> bool {
    matches!(
        std::env::var("OPENCODE_CALLER").as_deref(),
        Ok("vscode") | Ok("vscode-insiders")
    )
}
```

### Installation

```rust
#[derive(Debug, Error)]
pub enum IdeError {
    #[error("Unknown IDE: {0}")]
    Unknown(String),

    #[error("Extension already installed")]
    AlreadyInstalled,

    #[error("Installation failed: {stderr}")]
    InstallFailed { stderr: String },

    #[error("IDE not supported for installation: {0}")]
    NotSupported(SupportedIde),

    #[error("IO error: {0}")]
    Io(#[source] std::io::Error),
}

pub struct IdeService {
    bus: Arc<BusService>,
}

impl IdeService {
    pub fn new(bus: Arc<BusService>) -> Self
}

impl IdeService {
    /// Install the VS Code extension for the detected or specified IDE
    pub async fn install(&self, ide_name: Option<&str>) -> Result<(), IdeError> {
        let ide = match ide_name {
            Some(name) => self.parse_ide_name(name),
            None => detect_ide(),
        };

        if ide == SupportedIde::Unknown {
            return Err(IdeError::Unknown("Could not detect IDE".into()));
        }

        if ide == SupportedIde::Unknown {
            return Err(IdeError::NotSupported(ide));
        }

        let cli = ide.cli_name();
        if cli.is_empty() {
            return Err(IdeError::NotSupported(ide));
        }

        let output = tokio::process::Command::new(cli)
            .args(["--install-extension", "sst-dev.opencode"])
            .output()
            .await
            .map_err(IdeError::Io)?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("already installed") {
                return Err(IdeError::AlreadyInstalled);
            }
            // Publish bus event
            let _ = self.bus.publish("ide.installed", json!({ "ide": ide.name() })).await;
            Ok(())
        } else {
            Err(IdeError::InstallFailed {
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            })
        }
    }

    fn parse_ide_name(&self, name: &str) -> SupportedIde {
        match name.to_lowercase().as_str() {
            "windsurf" => SupportedIde::Windsurf,
            "code-insiders" | "vscode-insiders" | "visual studio code - insiders" => SupportedIde::CodeInsiders,
            "code" | "vscode" | "visual studio code" => SupportedIde::Code,
            "cursor" => SupportedIde::Cursor,
            "codium" | "vscodium" => SupportedIde::Vscodium,
            _ => SupportedIde::Unknown,
        }
    }
}
```

---

## Bus Events

```rust
// ide.installed event payload
#[derive(Serialize)]
struct IdeInstalledEvent {
    ide: String,
}
```

---

## Crate Layout

```
crates/ide/
├── Cargo.toml
├── src/
│   ├── lib.rs       # SupportedIde, detect_ide(), IdeService, IdeError
│   ├── detect.rs    # IDE detection from env vars
│   └── install.rs  # Extension installation logic
└── tests/
    └── ide_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-ide"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["process", "rt"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `bus` module | `ide.installed` event publishing |
| `tokio::process` | Running IDE CLI for extension install |
| `serde` | Event serialization |

---

## Acceptance Criteria

- [x] `detect_ide()` returns the correct IDE name when `TERM_PROGRAM=vscode` and `GIT_ASKPASS` matches
- [x] `detect_ide()` returns `Unknown` when no IDE is detected
- [x] `already_installed()` returns `true` when `OPENCODE_CALLER=vscode` or `vscode-insiders`
- [x] `install()` runs the correct CLI command for the target IDE
- [x] `install()` returns `AlreadyInstalled` error when stdout contains "already installed"
- [x] `install()` returns `InstallFailed` error on non-zero exit code
- [x] `ide.installed` bus event is published after successful installation

---

## Test Design

```rust
#[test]
fn test_detect_ide_vscode() {
    std::env::set_var("TERM_PROGRAM", "vscode");
    std::env::set_var("GIT_ASKPASS", "/usr/lib/Visual Studio Code/bin/code");
    assert_eq!(detect_ide(), SupportedIde::Code);
}

#[test]
fn test_detect_ide_cursor() {
    std::env::set_var("TERM_PROGRAM", "vscode");
    std::env::set_var("GIT_ASKPASS", "/Applications/Cursor.app/Contents/Resources/app/gitaskpass.sh");
    assert_eq!(detect_ide(), SupportedIde::Cursor);
}

#[test]
fn test_detect_ide_unknown_without_env() {
    std::env::remove_var("TERM_PROGRAM");
    std::env::remove_var("GIT_ASKPASS");
    assert_eq!(detect_ide(), SupportedIde::Unknown);
}

#[test]
fn test_already_installed_vscode() {
    std::env::set_var("OPENCODE_CALLER", "vscode");
    assert!(already_installed());
}

#[test]
fn test_already_installed_vscode_insiders() {
    std::env::set_var("OPENCODE_CALLER", "vscode-insiders");
    assert!(already_installed());
}

#[test]
fn test_already_installed_not_set() {
    std::env::remove_var("OPENCODE_CALLER");
    assert!(!already_installed());
}

#[tokio::test]
async fn test_install_unknown_ide_errors() {
    let bus = BusService::new_for_test();
    let svc = IdeService::new(bus);
    let result = svc.install(Some("unknown-ide")).await;
    assert!(matches!(result, Err(IdeError::NotSupported(_))));
}
```

---

## Source Reference

*Source: `packages/opencode/src/ide/index.ts`*
*No existing Rust equivalent — implement in `crates/ide/`*
