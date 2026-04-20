# PRD: installation Module

## Module Overview

- **Module Name**: `installation`
- **Source Path**: `packages/opencode/src/installation/`
- **Type**: Infrastructure Service
- **Rust Crate**: `crates/installation/`
- **Purpose**: Detects installation method (npm, brew, curl, scoop, choco, etc.), checks for available updates, performs in-place upgrades, and exposes version info and user-agent strings.

---

## Functionality

### Core Features

1. **Method Detection** — Determines how opencode was installed: `curl`, `npm`, `yarn`, `pnpm`, `bun`, `brew`, `scoop`, `choco`, or `unknown`
2. **Version Info** — Returns current version and latest available version from appropriate registry
3. **Latest Version Fetching** — Queries method-appropriate registry (npm, brew, GitHub releases, chocolatey, scoop)
4. **Upgrade** — Executes the upgrade command for the detected method
5. **Release Type** — Classifies update as `patch`, `minor`, or `major`
6. **User Agent** — Provides `opencode/<channel>/<version>/<client>` string for HTTP requests
7. **Preview/Local Detection** — `is_preview()`, `is_local()` based on channel name

---

## Installation Methods

| Method | Detection | Latest API | Upgrade Command |
|--------|-----------|------------|-----------------|
| `curl` | `execPath` contains `.opencode/bin` or `.local/bin` | GitHub releases API | Download install script |
| `npm` | `npm list -g` output | npm registry | `npm install -g opencode-ai@<ver>` |
| `pnpm` | `pnpm list -g` | npm registry | `pnpm install -g opencode-ai@<ver>` |
| `bun` | `bun pm ls -g` | npm registry | `bun install -g opencode-ai@<ver>` |
| `yarn` | `yarn global list` | npm registry | `yarn global add opencode-ai@<ver>` |
| `brew` | `brew list --formula opencode` | brew formulae API | `brew upgrade opencode` |
| `scoop` | `scoop list opencode` | GitHub raw manifest | `scoop install opencode@<ver>` |
| `choco` | `choco list opencode` | chocolatey OData API | `choco upgrade opencode --version=<ver>` |

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallMethod {
    Curl,
    Npm,
    Pnpm,
    Bun,
    Yarn,
    Brew,
    Scoop,
    Choco,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseType {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeResult {
    pub success: bool,
    pub new_version: Option<String>,
    pub error_message: Option<String>,
}
```

### `InstallationService`

```rust
pub struct InstallationService {
    http: reqwest::Client,
    current_version: String,
    channel: String,
    client: Option<String>, // from OPENCODE_CLIENT env var
}

impl InstallationService {
    /// Detect how opencode was installed
    pub async fn detect_method(&self) -> InstallMethod {
        let exec_path = std::env::current_exe().unwrap_or_default();
        let exec_str = exec_path.to_string_lossy();

        // curl detection: common install script paths
        if exec_str.contains(".opencode/bin") || exec_str.contains(".local/bin") {
            return InstallMethod::Curl;
        }

        // Check each package manager
        if self.check_npm_global().await { return InstallMethod::Npm; }
        if self.check_pnpm_global().await { return InstallMethod::Pnpm; }
        if self.check_bun_global().await { return InstallMethod::Bun; }
        if self.check_yarn_global().await { return InstallMethod::Yarn; }
        if self.check_brew_formula().await { return InstallMethod::Brew; }
        if self.check_scoop().await { return InstallMethod::Scoop; }
        if self.check_choco().await { return InstallMethod::Choco; }

        InstallMethod::Unknown
    }

    /// Fetch the latest version from the appropriate registry
    pub async fn latest_version(&self, method: Option<InstallMethod>) -> Result<String, InstallError> {
        let m = method.unwrap_or_else(|| {
            // Use sync method detection
            self.detect_method_blocking()
        });

        match m {
            InstallMethod::Npm | InstallMethod::Pnpm | InstallMethod::Bun | InstallMethod::Yarn => {
                self.fetch_npm_latest("opencode-ai").await
            }
            InstallMethod::Brew => self.fetch_brew_latest().await,
            InstallMethod::Curl | InstallMethod::Scoop | InstallMethod::Choco | InstallMethod::Unknown => {
                self.fetch_github_latest().await
            }
        }
    }

    /// Upgrade to a specific version using the detected or specified method
    pub async fn upgrade(&self, method: InstallMethod, target_version: &str) -> Result<UpgradeResult, InstallError> {
        let cmd: Vec<&str> = match method {
            InstallMethod::Npm => vec!["npm", "install", "-g", &format!("opencode-ai@{}", target_version)],
            InstallMethod::Pnpm => vec!["pnpm", "install", "-g", &format!("opencode-ai@{}", target_version)],
            InstallMethod::Bun => vec!["bun", "install", "-g", &format!("opencode-ai@{}", target_version)],
            InstallMethod::Yarn => vec!["yarn", "global", "add", &format!("opencode-ai@{}", target_version)],
            InstallMethod::Brew => vec!["brew", "upgrade", "opencode"],
            InstallMethod::Scoop => vec!["scoop", "install", &format!("opencode@{}", target_version)],
            InstallMethod::Choco => vec!["choco", "upgrade", "opencode", "--version", target_version],
            InstallMethod::Curl => return self.upgrade_via_script(target_version).await,
            InstallMethod::Unknown => return Ok(UpgradeResult { success: false, new_version: None, error_message: Some("Unknown installation method".into()) }),
        };

        let output = tokio::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .await
            .map_err(InstallError::Process)?;

        if output.status.success() {
            Ok(UpgradeResult { success: true, new_version: Some(target_version.into()), error_message: None })
        } else {
            Ok(UpgradeResult { success: false, new_version: None, error_message: Some(String::from_utf8_lossy(&output.stderr).into()) })
        }
    }

    /// Get version and latest info
    pub async fn info(&self) -> Result<VersionInfo, InstallError> {
        let latest = self.latest_version(None).await?;
        Ok(VersionInfo { current: self.current_version.clone(), latest })
    }

    /// User agent string
    pub fn user_agent(&self) -> String {
        format!(
            "opencode/{}/{}",
            self.channel,
            self.current_version,
        )
    }

    /// Check if running preview channel
    pub fn is_preview(&self) -> bool {
        self.channel != "latest"
    }

    /// Check if running local build
    pub fn is_local(&self) -> bool {
        self.channel == "local"
    }

    /// Determine release type between two versions
    pub fn release_type(&self, current: &str, latest: &str) -> ReleaseType {
        let curr = semver::Version::parse(current).unwrap_or(semver::Version::new(0, 0, 0));
        let next = semver::Version::parse(latest).unwrap_or(semver::Version::new(0, 0, 0));

        if next.major > curr.major { ReleaseType::Major }
        else if next.minor > curr.minor { ReleaseType::Minor }
        else { ReleaseType::Patch }
    }
}
```

### `InstallError`

```rust
#[derive(Debug, Error)]
pub enum InstallError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse version: {0}")]
    VersionParse(String),

    #[error("Failed to parse semver: {0}")]
    Semver(#[source] semver::Error),

    #[error("Process error: {0}")]
    Process(#[source] std::io::Error),

    #[error("Upgrade failed: {0}")]
    UpgradeFailed(String),

    #[error("Installation method not supported: {0}")]
    MethodNotSupported(InstallMethod),
}
```

---

## NPM Registry Fetch

```rust
impl InstallationService {
    async fn fetch_npm_latest(&self, package: &str) -> Result<String, InstallError> {
        let url = format!("https://registry.npmjs.org/{}/latest", package);
        #[derive(Deserialize)]
        struct NpmVersion { version: String }
        let resp: NpmVersion = self.http.get(&url).send().await?.json().await?;
        Ok(resp.version)
    }
}
```

### GitHub Releases Fetch

```rust
async fn fetch_github_latest(&self) -> Result<String, InstallError> {
    #[derive(Deserialize)]
    struct GithubRelease { tag_name: String }
    let resp: GithubRelease = self.http
        .get("https://api.github.com/repos/opencode-ai/opencode/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send().await?
        .json().await?;
    Ok(resp.tag_name.trim_start_matches('v').to_string())
}
```

### Brew Fetch

```rust
async fn fetch_brew_latest(&self) -> Result<String, InstallError> {
    #[derive(Deserialize)]
    struct BrewInfo { versions: BrewVersions }
    #[derive(Deserialize)]
    struct BrewVersions { stable: String }
    let resp: BrewInfo = self.http
        .get("https://formulae.brew.sh/api/formula/opencode.json")
        .send().await?
        .json().await?;
    Ok(resp.versions.stable)
}
```

---

## Build-Time Version Injection

```rust
// In build.rs or main.rs
const VERSION: &str = env!("OPENCODE_VERSION");
const CHANNEL: &str = env!("OPENCODE_CHANNEL");
const CLIENT: Option<&str> = option_env!("OPENCODE_CLIENT");
```

---

## Crate Layout

```
crates/installation/
├── Cargo.toml       # semver = "1", reqwest = { version = "0.11", features = ["json"] }
├── src/
│   ├── lib.rs       # InstallationService, InstallError, types
│   ├── detect.rs    # Method detection algorithms
│   ├── upgrade.rs   # Upgrade command execution
│   └── fetch.rs     # Version fetching from registries
└── tests/
    └── installation_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-installation"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["process", "rt"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
semver = "1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
wiremock = "0.6"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `semver` | Version comparison for `release_type` |
| `reqwest` | Fetching version from registries |
| `tokio::process` | Running upgrade commands |
| `serde_json` | Parsing npm/brew API responses |

---

## Acceptance Criteria

- [x] `detect_method()` returns the correct installation method for the current environment
- [x] `latest()` fetches the correct version for each method
- [x] `upgrade()` runs the correct command and returns success/error
- [x] `release_type()` correctly classifies major, minor, patch
- [x] `is_preview()` returns true for non-"latest" channels
- [x] `is_local()` returns true for "local" channel
- [x] `user_agent()` is correctly formatted

---

## Test Design

```rust
#[test]
fn test_release_type_major() {
    let svc = InstallationService::new_test("1.0.0", "latest");
    assert_eq!(svc.release_type("1.0.0", "2.0.0"), ReleaseType::Major);
}

#[test]
fn test_release_type_minor() {
    let svc = InstallationService::new_test("1.0.0", "latest");
    assert_eq!(svc.release_type("1.2.0", "1.3.0"), ReleaseType::Minor);
}

#[test]
fn test_release_type_patch() {
    let svc = InstallationService::new_test("1.0.0", "latest");
    assert_eq!(svc.release_type("1.2.3", "1.2.4"), ReleaseType::Patch);
}

#[test]
fn test_is_preview_channel() {
    let svc = InstallationService::new_test("1.0.0", "preview");
    assert!(svc.is_preview());
    assert!(!svc.is_local());
}

#[test]
fn test_is_local_channel() {
    let svc = InstallationService::new_test("1.0.0", "local");
    assert!(!svc.is_preview());
    assert!(svc.is_local());
}

#[test]
fn test_user_agent_format() {
    let svc = InstallationService::new_test("1.0.0", "latest");
    let ua = svc.user_agent();
    assert!(ua.starts_with("opencode/"));
    assert!(ua.contains("latest"));
    assert!(ua.contains("1.0.0"));
}

#[tokio::test]
async fn test_fetch_npm_latest() {
    let mock = MockServer::start().await;
    Mock::given(get, "/opencode-ai/latest")
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({ "version": "1.2.3" })))
        .mount(&mock)
        .await;

    let svc = InstallationService::new_test_with_http(&mock.uri());
    let version = svc.fetch_npm_latest("opencode-ai").await.unwrap();
    assert_eq!(version, "1.2.3");
}
```

---

## Source Reference

*Source: `packages/opencode/src/installation/index.ts`*
*No existing Rust equivalent — implement in `crates/installation/`*
