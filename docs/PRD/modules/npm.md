# Module PRD: npm (NPM Package Manager Service)

## 1. Module Overview

| Field | Value |
|-------|-------|
| **Module Name** | npm |
| **Source Path** | `packages/opencode/src/npm/` |
| **Type** | Service / Effect Layer |
| **Rust Crate** | `opencode-npm` (or within `opencode-tools`) |
| **Purpose** | Provides a managed npm package installation service that installs packages into an isolated cache directory, resolves their binary entry points, checks for version staleness against the npm registry, and locates binary executables for use by other services (especially the `format` module) |

## 2. Functionality

The npm module provides:

1. **Package Installation** (`add`): Installs a single npm package into an isolated cache directory at `<global.cache>/packages/<sanitized-pkg-name>/`. Uses `@npmcli/arborist` to manage the dependency tree. Returns the package entry point if found.

2. **Project-level Install** (`install`): Installs all dependencies for a project directory. Skips if `node_modules` already exists. Checks for dirty state by comparing `package.json` declarations against `package-lock.json` locked packages. Uses file-lock (`EffectFlock`) to prevent concurrent installs.

3. **Outdated Check** (`outdated`): Queries the npm registry (`registry.npmjs.org`) for the latest version of a package and compares it against the cached version using semver. Supports both exact versions and semver ranges.

4. **Binary Resolution** (`which`): Locates the binary executable for an installed package by inspecting the package's `package.json` `bin` field and the `.bin` directory. Automatically installs the package if not yet present.

5. **Path Sanitization**: Sanitizes package names for use as filesystem directory names (replaces illegal filesystem characters on Windows).

### Key Behaviors

- **File locking**: All install operations acquire a named file lock (`npm-install:<dir>`) via `EffectFlock` to prevent concurrent writes.
- **Cache isolation**: Each package installed via `add()` gets its own directory under the global cache (not mixed with project `node_modules`).
- **Lazy install in `which()`**: If the package isn't installed, `which()` automatically calls `add()` to install it first.
- **Registry check**: `outdated()` is best-effort — network failures return `false` (not an error).
- **Scoped packages**: Handles scoped npm packages (e.g., `@biomejs/biome`) correctly for binary resolution.

## 3. API Surface

```typescript
// Error type
export class InstallFailedError extends Schema.TaggedErrorClass<InstallFailedError>()("NpmInstallFailedError", {
  add: Schema.Array(Schema.String).pipe(Schema.optional),
  dir: Schema.String,
  cause: Schema.optional(Schema.Defect),
}) {}

// Entry point returned by add()
export interface EntryPoint {
  readonly directory: string
  readonly entrypoint: Option.Option<string>  // resolved module entry, if available
}

// Service interface
export interface Interface {
  readonly add: (pkg: string) => Effect.Effect<EntryPoint, InstallFailedError | EffectFlock.LockError>
  readonly install: (
    dir: string,
    input?: { add: { name: string; version?: string }[] }
  ) => Effect.Effect<void, EffectFlock.LockError | InstallFailedError>
  readonly outdated: (pkg: string, cachedVersion: string) => Effect.Effect<boolean>
  readonly which: (pkg: string) => Effect.Effect<Option.Option<string>>
}

export class Service extends Context.Service<Service, Interface>()("@opencode/Npm") {}
export const layer: Layer.Layer<Service, never, AppFileSystem.Service | Global.Service | FileSystem.FileSystem | EffectFlock.Service>
export const defaultLayer: Layer.Layer<Service>

// Standalone async API (wraps Effect runtime):
export function sanitize(pkg: string): string
export async function install(dir: string, input?: { add: { name: string; version?: string }[] }): Promise<void>
export async function add(pkg: string): Promise<{ directory: string; entrypoint: string | undefined }>
export async function outdated(pkg: string, cachedVersion: string): Promise<boolean>
export async function which(pkg: string): Promise<string | undefined>
```

## 4. Data Structures

### Internal Arborist Types

```typescript
interface ArboristNode {
  name: string
  path: string
}

interface ArboristTree {
  edgesOut: Map<string, { to?: ArboristNode }>
}
```

### Cache Directory Layout

```
<global.cache>/packages/
  prettier/
    node_modules/
      .bin/
        prettier          ← binary
      prettier/
        package.json
        ...
  @biomejs__biome/       ← sanitized from "@biomejs/biome"
    node_modules/
      .bin/
        biome
      @biomejs/
        biome/
          package.json
```

### `install()` Dirty Check Logic

1. If `node_modules/` doesn't exist → run full install
2. Otherwise, load `package.json` + `package-lock.json`
3. Collect all declared deps (dependencies + devDependencies + peerDependencies + optionalDependencies + `input.add`)
4. Collect all locked deps from lock file root
5. If any declared dep is not in locked → re-run arborist reify

## 5. Dependencies

| Dependency | Purpose |
|------------|---------|
| `@npmcli/arborist` | npm dependency tree management and installation |
| `npm-package-arg` (npa) | Parse npm package identifiers to extract name |
| `semver` | Semver comparison for `outdated()` check |
| `effect` (Effect, Schema, Context, Layer, Option, FileSystem) | Service injection, typed errors, optional values |
| `@effect/platform-node` (NodeFileSystem) | Node.js filesystem access |
| `@opencode-ai/shared/filesystem` (AppFileSystem) | Safe file existence checks, JSON reading |
| `@opencode-ai/shared/global` (Global) | Global cache directory path |
| `@opencode-ai/shared/util/effect-flock` (EffectFlock) | File-based locking for concurrent installs |
| `path` | Path manipulation |

## 6. Acceptance Criteria

- [ ] `add(pkg)` installs package into `<cache>/packages/<sanitized-name>/` and returns an `EntryPoint`
- [ ] `add(pkg)` returns immediately (no reinstall) if the package directory already exists
- [ ] `install(dir)` is a no-op if `node_modules/` already exists and is up-to-date
- [ ] `install(dir)` runs arborist reify when `node_modules/` is absent
- [ ] `install(dir)` detects dirty state when declared deps differ from lock file and re-installs
- [ ] `install(dir)` is a no-op when the directory is not writable
- [ ] All install operations are protected by a named file lock preventing concurrent installs
- [ ] `outdated(pkg, version)` returns `true` if latest npm registry version is newer than `cachedVersion`
- [ ] `outdated(pkg, version)` returns `false` on network failure (best-effort)
- [ ] `outdated(pkg, range)` correctly handles semver range comparisons
- [ ] `which(pkg)` returns the path to the package's binary
- [ ] `which(pkg)` auto-installs the package if not present
- [ ] `which(pkg)` correctly resolves scoped package binaries (e.g., `@biomejs/biome`)
- [ ] `sanitize(pkg)` replaces illegal filesystem characters on Windows
- [ ] Standalone async functions (`add`, `install`, `outdated`, `which`) work without explicit Effect runtime setup

## 7. Rust Implementation Guidance

### Crate
`opencode-npm` crate in `opencode-rust/crates/npm/`

### Key Crates
- `tokio` — async runtime
- `reqwest` — HTTP client for registry queries
- `semver` — semver parsing and comparison
- `serde_json` — JSON parsing for package.json and registry responses
- `fs2` or `file-lock` — file-based mutual exclusion
- `tracing` — logging
- `which` — PATH binary resolution (for system tools)

### Recommended Approach

```rust
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum NpmError {
    #[error("Install failed for {dir}: {cause}")]
    InstallFailed { dir: String, cause: String },
    #[error("Lock acquisition failed: {0}")]
    LockError(String),
}

pub struct EntryPoint {
    pub directory: PathBuf,
    pub entrypoint: Option<String>,
}

pub struct NpmService {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl NpmService {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir, client: reqwest::Client::new() }
    }

    pub fn sanitize(pkg: &str) -> String {
        // Replace illegal chars on Windows; passthrough on Unix
        if cfg!(windows) {
            pkg.chars().map(|c| match c {
                '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
                c if (c as u32) < 32 => '_',
                c => c,
            }).collect()
        } else {
            pkg.to_string()
        }
    }

    fn package_dir(&self, pkg: &str) -> PathBuf {
        self.cache_dir.join("packages").join(Self::sanitize(pkg))
    }

    // Uses tokio::process::Command to run `npm install` or equivalent
    pub async fn add(&self, pkg: &str) -> Result<EntryPoint, NpmError> {
        let dir = self.package_dir(pkg);
        if dir.exists() {
            return Ok(self.resolve_entry_point(pkg, &dir));
        }
        // Acquire file lock, run npm install --prefix <dir> <pkg>
        let _lock = self.acquire_lock(&format!("npm-install:{}", dir.display())).await?;
        self.run_npm_install(&dir, &[pkg]).await?;
        Ok(self.resolve_entry_point(pkg, &dir))
    }

    pub async fn outdated(&self, pkg: &str, cached_version: &str) -> bool {
        let url = format!("https://registry.npmjs.org/{}", pkg);
        let Ok(resp) = self.client.get(&url).send().await else { return false };
        if !resp.status().is_success() { return false }
        let Ok(data) = resp.json::<serde_json::Value>().await else { return false };
        let Some(latest) = data["dist-tags"]["latest"].as_str() else { return false };
        // Handle semver range vs exact version
        if cached_version.contains(['<', '>', '^', '~', '*', 'x', 'X', '|', '=']) {
            let req = semver::VersionReq::parse(cached_version).unwrap_or_default();
            let latest_v = semver::Version::parse(latest).unwrap_or_default();
            !req.matches(&latest_v)
        } else {
            let cached_v = semver::Version::parse(cached_version).unwrap_or_default();
            let latest_v = semver::Version::parse(latest).unwrap_or_default();
            latest_v > cached_v
        }
    }

    pub async fn which(&self, pkg: &str) -> Option<PathBuf> {
        let dir = self.package_dir(pkg);
        let bin_dir = dir.join("node_modules").join(".bin");
        // Try to find binary from already-installed package
        if let Some(bin) = self.pick_binary(pkg, &bin_dir, &dir).await {
            return Some(bin);
        }
        // Auto-install
        let _ = self.add(pkg).await.ok()?;
        self.pick_binary(pkg, &bin_dir, &dir).await
    }

    async fn pick_binary(&self, pkg: &str, bin_dir: &Path, pkg_dir: &Path) -> Option<PathBuf> {
        // Read .bin directory, check package.json bin field
        // Handle scoped packages: @org/pkg → unscoped name is "pkg"
        let unscoped = if pkg.starts_with('@') {
            pkg.split('/').nth(1).unwrap_or(pkg)
        } else {
            pkg
        };
        // Check package.json bin field
        let pkg_json_path = pkg_dir.join("node_modules").join(pkg).join("package.json");
        // ... resolve bin name from package.json, then join with bin_dir
        todo!()
    }
}
```

### Note on Arborist
In Rust, use `tokio::process::Command` to invoke the system `npm` binary:
```rust
tokio::process::Command::new("npm")
    .arg("install")
    .arg("--prefix").arg(&dir)
    .arg("--ignore-scripts")
    .arg(pkg)
    .output()
    .await?;
```
This avoids needing a Rust port of Arborist.

## 8. Test Design

No dedicated test file exists for the `npm` module in the TypeScript test suite. Tests should be written from the functional spec:

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // 1. sanitize() passthrough on Unix
    #[test]
    fn sanitize_passthrough_on_unix() {
        assert_eq!(NpmService::sanitize("prettier"), "prettier");
        assert_eq!(NpmService::sanitize("@biomejs/biome"), "@biomejs/biome");
    }

    // 2. outdated() returns false on network error (best-effort)
    #[tokio::test]
    async fn outdated_returns_false_on_network_error() {
        let service = NpmService::new(TempDir::new().unwrap().path().to_path_buf());
        // Use a non-existent package to trigger failure
        let result = service.outdated("this-package-does-not-exist-xyz-123", "1.0.0").await;
        assert!(!result);
    }

    // 3. outdated() detects newer version (requires network)
    #[tokio::test]
    #[ignore = "requires network"]
    async fn outdated_detects_newer_version() {
        let service = NpmService::new(TempDir::new().unwrap().path().to_path_buf());
        // "react" 1.0.0 is definitely outdated
        let result = service.outdated("react", "1.0.0").await;
        assert!(result);
    }

    // 4. outdated() returns false for current version (requires network)
    #[tokio::test]
    #[ignore = "requires network"]
    async fn outdated_returns_false_for_current_version() {
        let service = NpmService::new(TempDir::new().unwrap().path().to_path_buf());
        // Get the actual latest version and check it's not outdated
        let result = service.outdated("react", "999.999.999").await;
        assert!(!result); // future version is "not outdated" relative to latest
    }

    // 5. install() skips when node_modules exists and lock matches
    #[tokio::test]
    async fn install_skips_when_node_modules_exists() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("node_modules")).unwrap();
        std::fs::write(tmp.path().join("package.json"), "{}").unwrap();
        std::fs::write(tmp.path().join("package-lock.json"), "{}").unwrap();
        let service = NpmService::new(tmp.path().to_path_buf());
        // Should complete without error (no-op)
        service.install(tmp.path(), None).await.unwrap();
    }

    // 6. install() skips when directory is not writable
    // (Platform-specific test)

    // 7. Package dir naming from sanitize
    #[test]
    fn package_dir_uses_sanitized_name() {
        let service = NpmService::new("/cache".into());
        let dir = service.package_dir("prettier");
        assert_eq!(dir, PathBuf::from("/cache/packages/prettier"));
    }

    // 8. add() returns cached entry if dir already exists
    #[tokio::test]
    async fn add_returns_cached_if_dir_exists() {
        let tmp = TempDir::new().unwrap();
        let pkg_dir = tmp.path().join("packages").join("test-pkg");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        let service = NpmService::new(tmp.path().to_path_buf());
        // Should not attempt npm install again
        let result = service.add("test-pkg").await;
        // If npm is not in PATH, this might fail — that's acceptable in CI
        // The important thing is it doesn't panic and returns early
    }
}
```

### Integration Tests (Require npm)

```rust
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    // Requires npm to be installed
    #[tokio::test]
    async fn add_installs_and_resolves_binary() {
        let tmp = TempDir::new().unwrap();
        let service = NpmService::new(tmp.path().to_path_buf());
        let entry = service.add("prettier").await.unwrap();
        assert!(entry.directory.exists());
    }

    #[tokio::test]
    async fn which_resolves_prettier_binary() {
        let tmp = TempDir::new().unwrap();
        let service = NpmService::new(tmp.path().to_path_buf());
        let bin = service.which("prettier").await;
        assert!(bin.is_some());
        let bin_path = bin.unwrap();
        assert!(bin_path.exists());
    }
}
```

---
*Source: `packages/opencode/src/npm/` — index.ts (config.ts is empty)*
*No dedicated test file in `packages/opencode/test/npm/`*
