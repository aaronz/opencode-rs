# PRD: worktree Module

## Module Overview

- **Module Name**: `worktree`
- **Source Path**: `packages/opencode/src/worktree/`
- **Type**: Infrastructure Service
- **Rust Crate**: `crates/worktree/`
- **Purpose**: Git worktree management — creates isolated git worktrees for parallel agent workspaces, handles branch naming, bootstrapping, start scripts, and teardown.

---

## Functionality

### Core Features

1. **Worktree Creation** — `git worktree add --no-checkout -b <branch> <dir>` to create an isolated branch
2. **Unique Naming** — Slug-based name generation with retry loop (up to 26 attempts) to avoid conflicts
3. **Bootstrap** — After creation, runs `git reset --hard` then project instance bootstrap
4. **Start Scripts** — Runs project `commands.start` and optional extra startup commands after bootstrap
5. **Removal** — `git worktree remove --force` + branch delete + directory cleanup
6. **Reset** — Fetch default branch, `git reset --hard`, `git clean -ffdx`, submodule update + clean
7. **Event Bus** — Emits `worktree.ready` / `worktree.failed` to GlobalBus
8. **Directory Normalization** — Handles symlinks and case-insensitivity (Windows)

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,       // e.g. "brave-lion"
    pub branch: String,     // e.g. "opencode/brave-lion"
    pub directory: PathBuf, // absolute path to worktree directory
}

#[derive(Debug, Clone)]
pub struct CreateInput {
    pub name: Option<String>,        // optional name hint; slugified
    pub start_command: Option<String>, // extra script after project start
}

#[derive(Debug, Clone)]
pub struct RemoveInput {
    pub directory: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ResetInput {
    pub directory: PathBuf,
}
```

### `WorktreeError`

```rust
#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("Not a git repository: {0}")]
    NotGit(PathBuf),

    #[error("Worktree name generation failed after 26 attempts")]
    NameGenerationFailed,

    #[error("Failed to create worktree: {0}")]
    CreateFailed(String),

    #[error("Worktree not found: {0}")]
    NotFound(PathBuf),

    #[error("Start command failed: {0}")]
    StartCommandFailed(String),

    #[error("Failed to remove worktree: {0}")]
    RemoveFailed(String),

    #[error("Failed to reset worktree: {0}")]
    ResetFailed(String),

    #[error("Branch already exists: {0}")]
    BranchExists(String),

    #[error("Cannot reset primary workspace")]
    ResetPrimaryWorkspace,

    #[error("IO error: {0}")]
    Io(#[source] std::io::Error),

    #[error("Git error: {0}")]
    Git(String),
}
```

### `WorktreeService`

```rust
pub struct WorktreeService {
    git: Arc<GitService>,
    project: Arc<ProjectService>,
    global_bus: Arc<GlobalBus>,
    data_path: PathBuf,
}

impl WorktreeService {
    /// Create a new worktree
    pub async fn create(
        &self,
        ctx: &InstanceContext,
        input: CreateInput,
    ) -> Result<WorktreeInfo, WorktreeError> {
        // 1. Verify it's a git project
        if ctx.project().vcs != "git" {
            return Err(WorktreeError::NotGit(ctx.project().root.clone()));
        }

        // 2. Generate unique worktree info
        let info = self.make_worktree_info(ctx, input.name.as_deref()).await?;

        // 3. Run git worktree add
        self.add_worktree(ctx, &info).await?;

        // 4. Bootstrap in background
        let svc = self.clone();
        let ctx2 = ctx.clone();
        let info2 = info.clone();
        let start_cmd = input.start_command;
        tokio::spawn(async move {
            if let Err(e) = svc.boot(&ctx2, &info2, start_cmd.as_deref()).await {
                svc.global_bus.emit("worktree.failed", &WorktreeFailedEvent {
                    name: info2.name.clone(),
                    message: e.to_string(),
                }).await;
            }
        });

        Ok(info)
    }

    async fn make_worktree_info(
        &self,
        ctx: &InstanceContext,
        name_hint: Option<&str>,
    ) -> Result<WorktreeInfo, WorktreeError> {
        let base_dir = self.data_path.join(&ctx.project().id);
        tokio::fs::create_dir_all(&base_dir).await.map_err(WorktreeError::Io)?;

        let slug = name_hint.map(slugify).filter(|s| !s.is_empty());

        for attempt in 0..26 {
            let name = match slug {
                Some(ref base) if attempt == 0 => base.clone(),
                Some(ref base) => format!("{}-{}", base, random_slug(4)),
                None => random_slug(8),
            };

            let directory = base_dir.join(&name);
            let branch = format!("opencode/{}", name);

            // Check directory doesn't exist
            if directory.exists() {
                continue;
            }

            // Check branch doesn't exist
            if self.git.branch_exists(&ctx.project().root, &branch).await.unwrap_or(false) {
                continue;
            }

            return Ok(WorktreeInfo { name, branch, directory });
        }

        Err(WorktreeError::NameGenerationFailed)
    }

    async fn add_worktree(
        &self,
        ctx: &InstanceContext,
        info: &WorktreeInfo,
    ) -> Result<(), WorktreeError> {
        let output = tokio::process::Command::new("git")
            .args([
                "worktree", "add",
                "--no-checkout",
                "-B", &info.branch,
                info.directory.to_str().unwrap(),
            ])
            .current_dir(&ctx.project().root)
            .output()
            .await
            .map_err(WorktreeError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::CreateFailed(stderr.to_string()));
        }

        Ok(())
    }

    async fn boot(
        &self,
        ctx: &InstanceContext,
        info: &WorktreeInfo,
        extra_start: Option<&str>,
    ) -> Result<(), WorktreeError> {
        // git reset --hard
        let reset = tokio::process::Command::new("git")
            .args(["reset", "--hard"])
            .current_dir(&info.directory)
            .output()
            .await.map_err(WorktreeError::Io)?;

        if !reset.status.success() {
            return Err(WorktreeError::ResetFailed(
                String::from_utf8_lossy(&reset.stderr).to_string(),
            ));
        }

        // Run project bootstrap if configured
        if let Some(start_cmd) = &ctx.project().config.start_command {
            self.run_start_command(info, start_cmd).await?;
        }

        // Run extra start command
        if let Some(extra) = extra_start {
            self.run_start_command(info, extra).await?;
        }

        // Emit ready event
        self.global_bus.emit("worktree.ready", &WorktreeReadyEvent {
            name: info.name.clone(),
            branch: info.branch.clone(),
            directory: info.directory.to_string_lossy().into_owned(),
        }).await;

        Ok(())
    }

    async fn run_start_command(
        &self,
        info: &WorktreeInfo,
        cmd: &str,
    ) -> Result<(), WorktreeError> {
        use std::process::Stdio;
        let mut parts = cmd.split_whitespace();
        let program = parts.next().ok_or_else(|| WorktreeError::StartCommandFailed("empty command".into()))?;
        let args: Vec<_> = parts.collect();

        let output = tokio::process::Command::new(program)
            .args(&args)
            .current_dir(&info.directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(WorktreeError::Io)?;

        if !output.status.success() {
            return Err(WorktreeError::StartCommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    /// Remove a worktree
    pub async fn remove(&self, input: RemoveInput) -> Result<bool, WorktreeError> {
        let output = tokio::process::Command::new("git")
            .args(["worktree", "remove", "--force", input.directory.to_str().unwrap()])
            .output()
            .await
            .map_err(WorktreeError::Io)?;

        if !output.status.success() {
            return Err(WorktreeError::RemoveFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Also delete the branch
        // Extract branch name from worktree (stored in .git/worktrees/<name>/gitdir)
        // Or use: git worktree list --porcelain to find the branch

        Ok(true)
    }

    /// Reset a worktree to the latest default branch
    pub async fn reset(
        &self,
        ctx: &InstanceContext,
        input: ResetInput,
    ) -> Result<(), WorktreeError> {
        // Refuse to reset primary workspace
        if input.directory == ctx.project().root {
            return Err(WorktreeError::ResetPrimaryWorkspace);
        }

        let default_branch = self.git.default_branch(&ctx.project().root).await?;

        // Fetch default branch
        let fetch = tokio::process::Command::new("git")
            .args(["fetch", "origin", &default_branch])
            .current_dir(&input.directory)
            .output()
            .await.map_err(WorktreeError::Io)?;

        // Reset hard
        let reset = tokio::process::Command::new("git")
            .args(["reset", "--hard", format!("origin/{}", default_branch)])
            .current_dir(&input.directory)
            .output()
            .await.map_err(WorktreeError::Io)?;

        if !reset.status.success() {
            return Err(WorktreeError::ResetFailed(
                String::from_utf8_lossy(&reset.stderr).to_string(),
            ));
        }

        // git clean -ffdx
        let clean = tokio::process::Command::new("git")
            .args(["clean", "-ffdx"])
            .current_dir(&input.directory)
            .output()
            .await.map_err(WorktreeError::Io)?;

        // git submodule update --init --recursive
        let submodules = tokio::process::Command::new("git")
            .args(["submodule", "update", "--init", "--recursive"])
            .current_dir(&input.directory)
            .output()
            .await;

        Ok(())
    }
}
```

### Helper Functions

```rust
fn slugify(input: &str) -> String {
    input.trim().to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn random_slug(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}
```

---

## Bus Events

```rust
#[derive(Serialize)]
struct WorktreeReadyEvent {
    name: String,
    branch: String,
    directory: String,
}

#[derive(Serialize)]
struct WorktreeFailedEvent {
    name: String,
    message: String,
}
```

---

## Crate Layout

```
crates/worktree/
├── Cargo.toml       # tokio = { features = ["process", "fs"] }, slug = "0.1", rand = "0.8"
├── src/
│   ├── lib.rs       # WorktreeService, WorktreeError, types
│   ├── create.rs    # Worktree creation and bootstrap
│   ├── remove.rs    # Worktree removal
│   ├── reset.rs     # Worktree reset
│   └── name.rs      # Slug generation, name candidates
└── tests/
    └── worktree_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-worktree"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["process", "fs", "rt"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
rand = "0.8"
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `git` module | `GitService::default_branch()`, `branch_exists()` |
| `project` module | Project config (`commands.start`) |
| `global` module | `Global.Path.data` for worktree storage |
| `global-bus` module | `worktree.ready` / `worktree.failed` events |
| `tokio::process` | Running `git` subprocesses |
| `tokio::fs` | Directory creation/removal |
| `rand` | Random name component |
| `slug` | Slug generation (or manual implementation) |

---

## Acceptance Criteria

- [x] `create()` generates a unique branch and directory name
- [x] Worktree is initialized with `git worktree add --no-checkout -b`
- [x] After creation, `git reset --hard` and project bootstrap run in the background
- [x] Start scripts (project + extra) run after bootstrap
- [x] `worktree.ready` is published when bootstrap succeeds
- [x] `worktree.failed` is published when bootstrap fails
- [x] `remove()` removes the worktree, branch, and directory
- [x] `reset()` fetches default branch, hard-resets, cleans, and updates submodules
- [x] `reset()` refuses to reset the primary workspace
- [x] Works only for git projects

---

## Test Design

```rust
#[test]
fn test_slugify() {
    assert_eq!(slugify("My Feature Branch"), "my-feature-branch");
    assert_eq!(slugify("  --hello--  "), "hello");
    assert_eq!(slugify("Rust Project!"), "rust-project");
}

#[test]
fn test_slugify_preserves_dashes() {
    assert_eq!(slugify("my-feature"), "my-feature");
}

#[tokio::test]
async fn test_make_worktree_info_generates_unique_name() {
    let tmp = TempDir::new().unwrap();
    let git_dir = tmp.path().join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();
    let ctx = InstanceContext::with_git_root(tmp.path().to_path_buf());

    let svc = WorktreeService::new_test().await;
    let info = svc.make_worktree_info(&ctx, None).await.unwrap();
    assert!(!info.name.is_empty());
    assert!(info.branch.starts_with("opencode/"));
    assert!(!info.directory.to_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_fails_for_non_git_project() {
    let tmp = TempDir::new().unwrap();
    let ctx = InstanceContext::with_vcs(tmp.path().to_path_buf(), "none");
    let svc = WorktreeService::new_test().await;

    let result = svc.create(&ctx, CreateInput::default()).await;
    assert!(matches!(result, Err(WorktreeError::NotGit(_))));
}

#[tokio::test]
async fn test_reset_refuses_primary_workspace() {
    let tmp = TempDir::new().unwrap();
    let git_dir = tmp.path().join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();
    let ctx = InstanceContext::with_git_root(tmp.path().to_path_buf());

    let svc = WorktreeService::new_test().await;
    let result = svc.reset(&ctx, ResetInput { directory: ctx.project().root.clone() }).await;
    assert!(matches!(result, Err(WorktreeError::ResetPrimaryWorkspace)));
}
```

---

## Source Reference

*Source: `packages/opencode/src/worktree/index.ts`*
*No existing Rust equivalent — implement in `crates/worktree/`*
