# Unified Path Resolution for opencode-rs

**Date:** 2026-04-26
**Status:** Approved for Implementation

## Overview

opencode-rs currently has scattered, inconsistent path handling that conflicts with the original opencode project. This design establishes a single canonical path resolution module that ensures all config, data, cache, log, state, and temp paths are:
- opencode-rs specific (no conflicts with opencode)
- deterministic and testable
- overridable via environment variables
- platform-appropriate

## Current Problems

| Location | Current Behavior | Issue |
|----------|-----------------|-------|
| Log file path | `~/.config/opencode/logs/opencode.log` | Uses `opencode`, not `opencode-rs` |
| Config path | `~/.config/opencode/config.json` | Uses `opencode`, not `opencode-rs` |
| Project local | `.opencode/` | Conflicts with opencode project dir |
| Schema cache | `~/.config/opencode/schemas` | Uses `opencode`, not `opencode-rs` |
| Secrets | `~/.local/share/opencode/` | Uses `opencode`, not `opencode-rs` |
| Crash dumps | `~/.config/opencode-rs/crashes` | Inconsistent naming |
| TUI config | `~/.config/opencode-rs/` | Uses `opencode-rs` (correct) |
| OAuth sessions | `~/.config/opencode-rs` | Uses `opencode-rs` (correct) |
| Credentials | `~/.config/opencode-rs` | Uses `opencode-rs` (correct) |

## Design

### New Module: `crates/core/src/paths.rs`

A single module provides all path resolution for the entire codebase.

```rust
use std::path::PathBuf;
use std::sync::OnceLock;

static TEST_OVERRIDE: OnceLock<Option<PathOverride>> = OnceLock::new();

pub struct PathOverride {
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
}

pub fn override_paths(override_: PathOverride) {
    TEST_OVERRIDE.set(Some(override_)).ok();
}

pub fn clear_path_override() {
    TEST_OVERRIDE.set(None).ok();
}

fn get_override() -> Option<&'static PathOverride> {
    TEST_OVERRIDE.get().and_then(|o| o.as_ref())
}

pub struct Paths;

impl Paths {
    fn project_dirs() -> Option<directories::ProjectDirs> {
        directories::ProjectDirs::from("ai", "opencode", "opencode-rs")
    }

    fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME").ok().map(PathBuf::from)
    }

    pub fn config_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.config_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_CONFIG_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::project_dirs()
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| Self::home_dir().unwrap_or_default().join(".config/opencode-rs"))
    }

    pub fn data_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.data_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_DATA_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::project_dirs()
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| {
                Self::home_dir()
                    .unwrap_or_default()
                    .join(".local/share/opencode-rs")
            })
    }

    pub fn cache_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.cache_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_CACHE_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::project_dirs()
            .map(|d| d.cache_dir().to_path_buf())
            .unwrap_or_else(|| {
                Self::home_dir().unwrap_or_default().join(".cache/opencode-rs")
            })
    }

    pub fn log_dir() -> PathBuf {
        if let Some(override_) = get_override() {
            if let Some(ref dir) = override_.log_dir {
                return dir.clone();
            }
        }
        if let Ok(env_dir) = std::env::var("OPENCODE_RS_LOG_DIR") {
            return PathBuf::from(env_dir);
        }
        Self::config_dir().join("logs")
    }

    pub fn log_file() -> PathBuf {
        Self::log_dir().join("opencode-rs.log")
    }

    pub fn schema_cache_dir() -> PathBuf {
        Self::config_dir().join("schemas")
    }

    pub fn secrets_path() -> PathBuf {
        Self::data_dir().join("secrets.json")
    }

    pub fn crash_dump_dir() -> PathBuf {
        Self::config_dir().join("crashes")
    }

    pub fn credentials_path() -> PathBuf {
        Self::data_dir().join("credentials.enc.json")
    }

    pub fn credentials_key_path() -> PathBuf {
        Self::data_dir().join("credentials.key")
    }

    pub fn oauth_sessions_path() -> PathBuf {
        Self::data_dir().join("oauth_sessions.json")
    }

    pub fn project_local_dir() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        for ancestor in cwd.ancestors() {
            let opencode_rs_dir = ancestor.join(".opencode-rs");
            if opencode_rs_dir.is_dir() {
                return Some(opencode_rs_dir);
            }
        }
        None
    }

    pub fn find_project_local_dir() -> Option<PathBuf> {
        Self::project_local_dir()
    }

    pub fn ensure_project_local_dir() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        let opencode_rs_dir = cwd.join(".opencode-rs");
        if std::fs::create_dir_all(&opencode_rs_dir).is_ok() {
            Some(opencode_rs_dir)
        } else {
            None
        }
    }

    pub fn project_tools_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("tools"))
    }

    pub fn project_skills_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("skills"))
    }

    pub fn project_workflows_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("workflows"))
    }

    pub fn project_plugins_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("plugins"))
    }

    pub fn project_commands_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("commands"))
    }

    pub fn project_agents_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("agents"))
    }

    pub fn project_themes_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("themes"))
    }

    pub fn project_modes_dir() -> Option<PathBuf> {
        Self::project_local_dir().map(|p| p.join("modes"))
    }
}
```

### Environment Variables

| Variable | Priority | Description |
|----------|----------|-------------|
| `OPENCODE_RS_CONFIG_DIR` | 1 | Override all config paths |
| `OPENCODE_RS_DATA_DIR` | 1 | Override all data paths |
| `OPENCODE_RS_CACHE_DIR` | 1 | Override all cache paths |
| `OPENCODE_RS_LOG_DIR` | 1 | Override log directory |
| `OPENCODE_CONFIG_DIR` | 2 | Legacy, shows deprecation warning |
| `OPENCODE_DATA_DIR` | 2 | Legacy, shows deprecation warning |

### Default Paths

| Path Type | Default Location |
|-----------|-----------------|
| Config | `~/.config/opencode-rs/` |
| Data | `~/.local/share/opencode-rs/` |
| Cache | `~/.cache/opencode-rs/` |
| Logs | `~/.config/opencode-rs/logs/` |
| Schema cache | `~/.config/opencode-rs/schemas/` |
| Secrets | `~/.local/share/opencode-rs/secrets.json` |
| Credentials | `~/.local/share/opencode-rs/credentials.enc.json` |
| OAuth sessions | `~/.local/share/opencode-rs/oauth_sessions.json` |
| Crash dumps | `~/.config/opencode-rs/crashes/` |
| Project local | `./.opencode-rs/` |

### Legacy Compatibility

1. **No automatic migration** - opencode-rs never moves or modifies opencode files
2. **Deprecation warnings** - Using `OPENCODE_CONFIG_DIR` or `OPENCODE_DATA_DIR` logs a warning
3. **Project directory detection** - Still detects `.opencode/` as project indicator (for compatibility with existing opencode projects) but creates `.opencode-rs/` for opencode-rs-specific project files

### Backward Compatibility with opencode

If a user has an existing `.opencode/` project directory, opencode-rs:
1. Uses `.opencode/` as the project root indicator (existing behavior)
2. Creates NEW opencode-rs-specific subdirectories as `.opencode-rs/` (not inside `.opencode/`)
3. Does not read or write opencode-specific files

## Refactoring Map

### Phase 1: Create paths.rs module
- Create `crates/core/src/paths.rs` with the `Paths` struct and all path resolution functions
- Add `pub mod paths;` to `crates/core/src/lib.rs`
- Add tests in `crates/core/src/paths.rs`

### Phase 2: Update logging
- `crates/util/src/logging.rs`: Replace `log_file_path()` to use `paths::log_file()`

### Phase 3: Update config
- `crates/config/src/lib.rs`: Replace `Config::config_path()` to use `paths::config_dir()`
- `crates/config/src/schema.rs`: Replace `schema_cache_dir()` to use `paths::schema_cache_dir()`
- `crates/config/src/secret_storage.rs`: Replace `default_secrets_path()` to use `paths::secrets_path()`
- `crates/config/src/directory_scanner.rs`: Update `load_opencode_directory()` to prefer `.opencode-rs/` over `.opencode/`

### Phase 4: Update auth
- `crates/auth/src/oauth.rs`: Replace `OAuthSessionStore::default_path()` to use `paths::oauth_sessions_path()`
- `crates/auth/src/credential_store.rs`: Replace credential store paths to use `paths::credentials_path()` and `paths::credentials_key_path()`

### Phase 5: Update core
- `crates/core/src/crash_recovery.rs`: Replace crash dump dir to use `paths::crash_dump_dir()`
- `crates/core/src/project.rs`: Update project root detection to find `.opencode-rs/` directories
- `crates/core/src/skill.rs`: Replace `.opencode/skills` with `.opencode-rs/skills`

### Phase 6: Update CLI commands
- `crates/cli/src/cmd/workspace.rs`: Replace `.opencode` with `.opencode-rs`
- `crates/cli/src/cmd/plugin.rs`: Replace `~/.config/opencode/plugins` and `.opencode/plugins` with opencode-rs paths
- `crates/cli/src/cmd/github.rs`: Replace `.opencode/workflows` with `.opencode-rs/workflows`
- `crates/cli/src/cmd/debug.rs`: Update debug output to show opencode-rs paths
- `crates/cli/src/cmd/db.rs`: Use `paths::data_dir()` for database path
- `crates/cli/src/cmd/shortcuts.rs`: Use `paths::data_dir()`
- `crates/cli/src/cmd/permissions.rs`: Use `paths::data_dir()`
- `crates/cli/src/cmd/acp.rs`: Replace `OPENCODE_CONFIG_DIR` check with `paths::config_dir()`

### Phase 7: Update TUI
- `crates/tui/src/app.rs`: Replace `dirs::config_dir().join("opencode")` with `paths::config_dir()`
- `crates/tui/src/config.rs`: Replace `default_config_path()` to use `paths::config_dir()`

### Phase 8: Update tools
- `crates/tools/src/discovery.rs`: Replace `PROJECT_TOOLS_DIR` constant from `.opencode/tools` to `.opencode-rs/tools`

## Test Plan

### Unit Tests
1. `test_paths_config_dir_default()` - Verify default config dir
2. `test_paths_config_dir_override()` - Verify `OPENCODE_RS_CONFIG_DIR` override
3. `test_paths_log_file_override()` - Verify `OPENCODE_RS_LOG_DIR` override
4. `test_paths_project_local_dir()` - Verify `.opencode-rs/` detection
5. `test_paths_legacy_env_warnings()` - Verify deprecation warnings for legacy env vars
6. `test_paths_no_opencode_conflict()` - Verify paths don't contain "opencode" (the original)

### Integration Tests
1. Config discovery from opencode-rs specific locations
2. Log file creation in opencode-rs specific directory
3. Project local `.opencode-rs/` creation
4. No `.opencode/` directory creation when using opencode-rs

### Isolation
All tests use temporary directories and path overrides to avoid touching real user directories.

## Documentation Updates

1. **README.md**: Update default paths section
2. **AGENTS.md**: Document new path locations
3. **CONTRIBUTING.md**: Update plugin path documentation
4. **CLI help text**: Add `--show-paths` flag to `opencode debug`

## Non-Goals

- No automatic migration of existing opencode files
- No reading of opencode config files
- No creation of `.opencode/` directories by default
- No backward compatibility mode by default (opt-in only)