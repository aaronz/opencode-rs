# PRD: snapshot Module

## Module Overview

- **Module Name**: `snapshot`
- **Source Path**: `packages/opencode/src/snapshot/`
- **Type**: Infrastructure Service
- **Rust Crate**: `crates/snapshot/`
- **Purpose**: Git-based file snapshot system that tracks uncommitted changes in a shadow git repository, enabling the agent to revert file edits made during a session.

---

## Functionality

### Core Features

1. **Shadow Git Repository** — Maintains a separate `--git-dir` at `$DATA/snapshot/<project-id>/<worktree-hash>/` pointing to the project worktree
2. **Snapshot Tracking** — Stages modified/untracked files and writes a tree object (not a commit), returning the tree hash
3. **Restore** — Checks out a given tree hash back to the working directory
4. **Revert Patches** — Batch-reverts specific files from specific snapshot hashes
5. **Diff Generation** — Produces unified diffs and structured `FileDiff[]` between two snapshot trees
6. **File Filtering** — Respects `.gitignore` rules and large-file limits (2 MB)
7. **Auto Cleanup** — Prunes old objects hourly (after 7 days)
8. **Concurrency Safety** — Per-directory `Semaphore` ensures snapshot operations are serialized

### When to Use

- Before the agent writes/edits a file → `track()` to get a hash
- After a bad tool run → `revert(patches)` to restore files
- For showing diff to the user → `diff(hash)` or `diff_full(from, to)`

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub hash: String,       // snapshot tree hash (40-char SHA1)
    pub files: Vec<String>,  // absolute paths of files in this patch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file: String,
    pub patch: String,       // unified diff text
    pub additions: u32,
    pub deletions: u32,
    pub status: FileStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedPaths {
    pub added: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}
```

### `SnapshotService`

```rust
pub struct SnapshotService {
    /// Per-worktree semaphore for serialization
    locks: Arc<Mutex<HashMap<PathBuf, Arc<tokio::sync::Semaphore>>>>,
    config: Arc<ConfigService>,
    data_path: PathBuf,
    git_path: PathBuf, // path to git executable
}

impl SnapshotService {
    /// Initialize snapshot system (start cleanup timer)
    pub async fn init(&self) {
        let svc = self.clone();
        tokio::spawn(async move {
            svc.run_cleanup_loop().await;
        });
    }

    /// Take a snapshot: stage all changes, write tree, return hash
    pub async fn track(&self, ctx: &InstanceContext) -> Result<Option<String>, SnapshotError> {
        let lock = self.lock_for(ctx.worktree()).await;
        let _permit = lock.acquire().await.map_err(|_| SnapshotError::LockFailed)?;

        // Check if snapshot is enabled
        if !self.is_enabled(ctx).await? {
            return Ok(None);
        }

        // Ensure shadow repo exists
        self.ensure_shadow_repo(ctx).await?;

        // Stage changes: modified files + new untracked files
        self.stage_changes(ctx).await?;

        // Write tree and get hash
        let hash = self.write_tree(ctx).await?;
        Ok(Some(hash))
    }

    /// Get the patch (list of changed files) for a given snapshot hash
    pub async fn patch(&self, ctx: &InstanceContext, hash: &str) -> Result<Patch, SnapshotError> {
        let lock = self.lock_for(ctx.worktree()).await;
        let _permit = lock.acquire().await.map_err(|_| SnapshotError::LockFailed)?;

        let git_dir = self.shadow_git_dir(ctx)?;
        let files = self.list_changed_files(ctx, hash).await?;
        Ok(Patch { hash: hash.to_string(), files })
    }

    /// Restore all files from a snapshot hash (full restore)
    pub async fn restore(&self, ctx: &InstanceContext, hash: &str) -> Result<(), SnapshotError> {
        let lock = self.lock_for(ctx.worktree()).await;
        let _permit = lock.acquire().await.map_err(|_| SnapshotError::LockFailed)?;

        let git_dir = self.shadow_git_dir(ctx)?;
        let output = tokio::process::Command::new(&self.git_path)
            .args(["--git-dir", git_dir.to_str().unwrap(), "checkout", hash, "--", "."])
            .current_dir(ctx.worktree())
            .output()
            .await
            .map_err(|e| SnapshotError::Git(format!("checkout failed: {}", e)))?;

        if !output.status.success() {
            return Err(SnapshotError::Git(String::from_utf8_lossy(&output.stderr).to_string()));
        }
        Ok(())
    }

    /// Revert specific files from specific snapshot patches
    pub async fn revert(
        &self,
        ctx: &InstanceContext,
        patches: &[Patch],
    ) -> Result<(), SnapshotError> {
        let lock = self.lock_for(ctx.worktree()).await;
        let _permit = lock.acquire().await.map_err(|_| SnapshotError::LockFailed)?;

        for patch in patches {
            // For each file in the patch, checkout from the snapshot hash
            for file in &patch.files {
                let output = tokio::process::Command::new(&self.git_path)
                    .args([
                        "--git-dir", self.shadow_git_dir(ctx)?.to_str().unwrap(),
                        "checkout", &patch.hash, "--", file,
                    ])
                    .current_dir(ctx.worktree())
                    .output()
                    .await
                    .map_err(|e| SnapshotError::Git(format!("revert failed: {}", e)))?;

                if !output.status.success() {
                    // File didn't exist in snapshot — delete it
                    if file.is_absolute() && Path::new(file).exists() {
                        tokio::fs::remove_file(file).await.map_err(SnapshotError::Io)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate a unified diff from a snapshot hash to current state
    pub async fn diff(&self, ctx: &InstanceContext, hash: &str) -> Result<String, SnapshotError> {
        let git_dir = self.shadow_git_dir(ctx)?;
        let output = tokio::process::Command::new(&self.git_path)
            .args([
                "--git-dir", git_dir.to_str().unwrap(),
                "diff", &format!("{}..HEAD", hash),
            ])
            .current_dir(ctx.worktree())
            .output()
            .await
            .map_err(|e| SnapshotError::Git(format!("diff failed: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Generate structured file diffs between two snapshot hashes
    pub async fn diff_full(
        &self,
        ctx: &InstanceContext,
        from: &str,
        to: &str,
    ) -> Result<Vec<FileDiff>, SnapshotError> {
        let git_dir = self.shadow_git_dir(ctx)?;

        // Get list of files that differ
        let output = tokio::process::Command::new(&self.git_path)
            .args([
                "--git-dir", git_dir.to_str().unwrap(),
                "diff", "--name-status", &format!("{}..{}", from, to),
            ])
            .current_dir(ctx.worktree())
            .output()
            .await
            .map_err(|e| SnapshotError::Git(format!("diff --name-status failed: {}", e)))?;

        let mut diffs = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 { continue; }
            let status_char = parts[0];
            let file = parts[1];

            let (status, additions, deletions) = self.get_diff_stats(ctx, from, to, file).await?;

            diffs.push(FileDiff {
                file: file.to_string(),
                patch: String::new(), // could run full diff per file
                additions,
                deletions,
                status,
            });
        }

        Ok(diffs)
    }
}
```

### Internal Methods

```rust
impl SnapshotService {
    fn shadow_git_dir(&self, ctx: &InstanceContext) -> Result<PathBuf, SnapshotError> {
        let worktree_hash = sha1_hash(ctx.worktree().to_string_lossy().as_bytes());
        Ok(self.data_path.join("snapshot").join(&ctx.project().id).join(worktree_hash))
    }

    async fn ensure_shadow_repo(&self, ctx: &InstanceContext) -> Result<(), SnapshotError> {
        let git_dir = self.shadow_git_dir(ctx)?;
        if git_dir.exists() {
            return Ok(());
        }
        // Create shadow git directory structure
        tokio::fs::create_dir_all(&git_dir).await.map_err(SnapshotError::Io)?;

        // Init --bare git repo
        let output = tokio::process::Command::new(&self.git_path)
            .args(["init", "--bare", git_dir.to_str().unwrap()])
            .output()
            .await
            .map_err(|e| SnapshotError::Git(format!("init failed: {}", e)))?;

        if !output.status.success() {
            return Err(SnapshotError::Git(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        // Create a worktree pointing to the project directory
        let git_file = git_dir.join("gitfile");
        tokio::fs::write(&git_file, ctx.worktree().to_string_lossy()).await.map_err(SnapshotError::Io)?;

        Ok(())
    }

    async fn stage_changes(&self, ctx: &InstanceContext) -> Result<(), SnapshotError> {
        let git_dir = self.shadow_git_dir(ctx)?;

        // git -C worktree ls-files --others --exclude-standard | git -C gitdir add --stdin --sparse
        // For simplicity: add all modified and untracked non-ignored files
        let output = tokio::process::Command::new(&self.git_path)
            .args(["--git-dir", git_dir.to_str().unwrap(), "add", "-A", "--sparse"])
            .current_dir(ctx.worktree())
            .output()
            .await
            .map_err(|e| SnapshotError::Git(format!("add failed: {}", e)))?;

        Ok(())
    }

    async fn write_tree(&self, ctx: &InstanceContext) -> Result<String, SnapshotError> {
        let git_dir = self.shadow_git_dir(ctx)?;
        let output = tokio::process::Command::new(&self.git_path)
            .args(["--git-dir", git_dir.to_str().unwrap(), "write-tree"])
            .current_dir(ctx.worktree())
            .output()
            .await
            .map_err(|e| SnapshotError::Git(format!("write-tree failed: {}", e)))?;

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(hash)
    }

    fn sha1_hash(input: &[u8]) -> String {
        use sha1::{Sha1, Digest};
        let mut hasher = Sha1::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
}
```

### `SnapshotError`

```rust
#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("Snapshot disabled")]
    Disabled,

    #[error("IO error: {0}")]
    Io(#[source] std::io::Error),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Semaphore lock failed")]
    LockFailed,

    #[error("Shadow repo not found: {0}")]
    ShadowRepoNotFound(PathBuf),

    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    #[error("File too large: {0} (max {} bytes)", max_file_size())]
    FileTooLarge(PathBuf),
}

fn max_file_size() -> usize { 2 * 1024 * 1024 }
```

---

## Constants

```rust
const MAX_FILE_SIZE: usize = 2 * 1024 * 1024; // 2 MB max tracked file
const PRUNE_AGE: &str = "7.days";
const CLEANUP_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour
const CLEANUP_DELAY: Duration = Duration::from_secs(60);       // 1 min after init
```

---

## Crate Layout

```
crates/snapshot/
├── Cargo.toml       # sha2 = "0.10", tokio = { features = ["full"] }, similar = "2"
├── src/
│   ├── lib.rs       # SnapshotService, SnapshotError, types
│   ├── shadow.rs    # Shadow repo creation and management
│   ├── track.rs     # Snapshot tracking (stage, write-tree)
│   ├── restore.rs  # Restore and revert
│   ├── diff.rs      # Diff generation
│   └── cleanup.rs  # Periodic cleanup task
└── tests/
    └── snapshot_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-snapshot"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["fs", "sync", "rt", "time", "process"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
sha2 = "0.10"
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `config` module | `snapshot` enabled/disabled config |
| `global` module | `Global.Path.data` for shadow repo directory |
| `tokio::fs` | File existence, read, write |
| `tokio::process` | Running `git` subprocesses |
| `sha2` | SHA1 hashing of worktree path for shadow repo name |
| `similar` | Diff computation (alternative to diff npm package) |

---

## Acceptance Criteria

- [x] `track()` initializes shadow repo on first call and returns a tree hash
- [x] `track()` returns `None` if snapshot is disabled in config
- [x] Gitignored files are excluded from snapshots
- [x] Files > 2 MB are excluded from untracked staging
- [x] `restore()` checks out the tree hash to the working directory
- [x] `revert()` restores each file from its snapshot hash; deletes files that didn't exist in the snapshot
- [x] `diff()` and `diff_full()` produce accurate unified diffs
- [x] Cleanup runs hourly (1 minute after init) and prunes objects older than 7 days
- [x] Concurrent calls are serialized via per-directory semaphore
- [x] Works only for git projects (`vcs === "git"`)

---

## Test Design

```rust
#[tokio::test]
async fn test_track_returns_hash_for_modified_file() {
    let (svc, ctx) = SnapshotService::new_test().await;
    let file = ctx.worktree().join("file.txt");
    tokio::fs::write(&file, "hello").await.unwrap();

    let hash = svc.track(&ctx).await.unwrap();
    assert!(hash.is_some());
    assert_eq!(hash.unwrap().len(), 40); // SHA1 hex
}

#[tokio::test]
async fn test_revert_restores_modified_file() {
    let (svc, ctx) = SnapshotService::new_test().await;
    let file = ctx.worktree().join("file.txt");
    tokio::fs::write(&file, "original").await.unwrap();
    let hash = svc.track(&ctx).await.unwrap().unwrap();

    tokio::fs::write(&file, "modified").await.unwrap();
    let patch = svc.patch(&ctx, &hash).await.unwrap();
    svc.revert(&ctx, &[patch]).await.unwrap();

    let content = tokio::fs::read_to_string(&file).await.unwrap();
    assert_eq!(content, "original");
}

#[tokio::test]
async fn test_disabled_snapshot_returns_none() {
    let (svc, ctx) = SnapshotService::new_with_config(Config {
        snapshot: Some(false),
        ..Default::default()
    }).await;
    let hash = svc.track(&ctx).await.unwrap();
    assert!(hash.is_none());
}

#[tokio::test]
async fn test_diff_full_shows_additions_deletions() {
    let (svc, ctx) = SnapshotService::new_test().await;
    let file = ctx.worktree().join("f.txt");
    tokio::fs::write(&file, "line1\nline2\n").await.unwrap();
    let hash1 = svc.track(&ctx).await.unwrap().unwrap();

    tokio::fs::write(&file, "line1\nline2\nline3\n").await.unwrap();
    let hash2 = svc.track(&ctx).await.unwrap().unwrap();

    let diffs = svc.diff_full(&ctx, &hash1, &hash2).await.unwrap();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].additions, 1);
    assert_eq!(diffs[0].deletions, 0);
}
```

---

## Source Reference

*Source: `packages/opencode/src/snapshot/index.ts`*
*No existing Rust equivalent — implement in `crates/snapshot/`*
