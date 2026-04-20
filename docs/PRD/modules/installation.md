# PRD: installation Module

## Module Overview

- **Module Name**: installation
- **Source Path**: `packages/opencode/src/installation/`
- **Type**: Infrastructure Service
- **Purpose**: Detects the installation method (npm, brew, curl, etc.), checks for available updates, and performs in-place upgrades. Also exposes version info and user-agent string.

---

## Functionality

### Core Features

1. **Method Detection** — Determines how opencode was installed: `curl`, `npm`, `yarn`, `pnpm`, `bun`, `brew`, `scoop`, `choco`, or `unknown`
2. **Version Info** — Returns current version and latest available version
3. **Latest Version Fetching** — Queries method-appropriate registry (npm, brew, GitHub releases, chocolatey, scoop)
4. **Upgrade** — Executes the upgrade command for the detected method
5. **Release Type** — Classifies update as `patch`, `minor`, or `major`
6. **User Agent** — Provides `opencode/<channel>/<version>/<client>` string for HTTP requests
7. **Preview/Local Detection** — `isPreview()`, `isLocal()` based on channel name

### Installation Methods

| Method | Detection | Latest API | Upgrade Command |
|--------|-----------|------------|-----------------|
| `curl` | `execPath` contains `.opencode/bin` or `.local/bin` | GitHub releases API | Download install script |
| `npm` | `npm list -g` output | npm registry | `npm install -g opencode-ai@<ver>` |
| `pnpm` | `pnpm list -g` | npm registry | `pnpm install -g opencode-ai@<ver>` |
| `bun` | `bun pm ls -g` | npm registry | `bun install -g opencode-ai@<ver>` |
| `yarn` | `yarn global list` | npm registry | (via yarn) |
| `brew` | `brew list --formula opencode` | brew formulae API | `brew upgrade opencode` |
| `scoop` | `scoop list opencode` | GitHub raw manifest | `scoop install opencode@<ver>` |
| `choco` | `choco list opencode` | chocolatey OData API | `choco upgrade opencode --version=<ver>` |

---

## API Surface

```typescript
type Method = "curl" | "npm" | "yarn" | "pnpm" | "bun" | "brew" | "scoop" | "choco" | "unknown"
type ReleaseType = "patch" | "minor" | "major"

interface Info {
  version: string   // current
  latest: string    // latest available
}

interface Interface {
  info: () => Effect<Info>
  method: () => Effect<Method>
  latest: (method?: Method) => Effect<string>
  upgrade: (method: Method, target: string) => Effect<void, UpgradeFailedError>
}

// Standalone exports
const USER_AGENT: string   // "opencode/<channel>/<version>/<client>"
function isPreview(): boolean
function isLocal(): boolean
function getReleaseType(current: string, latest: string): ReleaseType
```

---

## Data Structures

```typescript
// Version info from build-time injection
InstallationVersion: string   // e.g. "1.2.3"
InstallationChannel: string   // "latest" | "preview" | "local"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `flag` module | `OPENCODE_CLIENT` for user agent |
| `semver` npm | Version comparison for `getReleaseType` |
| `effect/http` | Fetching version from registries |
| `effect/process` | Running upgrade commands |

---

## Acceptance Criteria

- [ ] `method()` returns the correct installation method for the current environment
- [ ] `latest()` fetches the correct version for each method
- [ ] `upgrade()` runs the correct command and succeeds with exit code 0
- [ ] `upgrade()` returns `UpgradeFailedError` on non-zero exit
- [ ] `getReleaseType()` correctly classifies major, minor, patch
- [ ] `isPreview()` returns true for non-"latest" channels
- [ ] `USER_AGENT` is correctly formatted

---

## Rust Implementation Guidance

### Crate: `crates/installation/`

### Key Crates

```toml
semver = "1"
reqwest = { version = "0.11", features = ["json"] }
tokio = { features = ["process"] }
serde_json = "1"
```

### Architecture

```rust
pub struct InstallationService {
    http: reqwest::Client,
    current_version: &'static str,  // injected at build time via env!()
    channel: &'static str,
}

impl InstallationService {
    pub async fn detect_method(&self) -> InstallMethod {
        let exec = std::env::current_exe().unwrap_or_default();
        let exec_str = exec.to_str().unwrap_or("");

        if exec_str.contains(".opencode/bin") || exec_str.contains(".local/bin") {
            return InstallMethod::Curl;
        }
        // Check each package manager
        for method in [InstallMethod::Npm, InstallMethod::Bun, InstallMethod::Brew, ..] {
            if self.check_package_manager(method).await {
                return method;
            }
        }
        InstallMethod::Unknown
    }

    pub async fn latest_version(&self, method: Option<InstallMethod>) -> Result<String> {
        let m = match method { Some(m) => m, None => self.detect_method().await };
        match m {
            InstallMethod::Npm | InstallMethod::Pnpm | InstallMethod::Bun =>
                self.fetch_npm_latest().await,
            InstallMethod::Brew => self.fetch_brew_latest().await,
            _ => self.fetch_github_latest().await,
        }
    }
}

pub fn release_type(current: &str, latest: &str) -> ReleaseType {
    let curr = semver::Version::parse(current).unwrap();
    let next = semver::Version::parse(latest).unwrap();
    if next.major > curr.major { ReleaseType::Major }
    else if next.minor > curr.minor { ReleaseType::Minor }
    else { ReleaseType::Patch }
}
```

### Build-time Version Injection

```rust
// build.rs
fn main() {
    println!("cargo:rustc-env=OPENCODE_VERSION={}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=OPENCODE_CHANNEL=latest");
}

// main.rs
const VERSION: &str = env!("OPENCODE_VERSION");
const CHANNEL: &str = env!("OPENCODE_CHANNEL");
```

---

## Test Design

```rust
#[test]
fn test_release_type_major() {
    assert_eq!(release_type("1.0.0", "2.0.0"), ReleaseType::Major);
}

#[test]
fn test_release_type_minor() {
    assert_eq!(release_type("1.2.0", "1.3.0"), ReleaseType::Minor);
}

#[test]
fn test_release_type_patch() {
    assert_eq!(release_type("1.2.3", "1.2.4"), ReleaseType::Patch);
}

#[test]
fn test_user_agent_format() {
    let ua = user_agent();
    assert!(ua.starts_with("opencode/"));
    assert!(ua.contains("/latest/") || ua.contains("/preview/"));
}

#[tokio::test]
async fn test_detect_method_returns_unknown_in_test_env() {
    let svc = InstallationService::new_test();
    let method = svc.detect_method().await;
    // In CI/test env, won't find any package manager
    assert!(matches!(method, InstallMethod::Unknown | InstallMethod::Curl));
}
```

### Integration Tests (from TS patterns)

- `installation.test.ts`: Version detection, release type classification, method detection via mocked exec paths
