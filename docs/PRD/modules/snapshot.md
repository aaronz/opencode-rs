# PRD: snapshot Module

## Module Overview

- **Module Name**: snapshot
- **Source Path**: `packages/opencode/src/snapshot/`
- **Type**: Infrastructure Service
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
- For showing diff to the user → `diff(hash)` or `diffFull(from, to)`

---

## API Surface

### Types

```typescript
interface Patch {
  hash: string      // snapshot tree hash
  files: string[]   // absolute paths of files included in this patch
}

interface FileDiff {
  file: string
  patch: string       // unified diff text
  additions: number
  deletions: number
  status?: "added" | "deleted" | "modified"
}
```

### Service Interface

```typescript
interface Interface {
  init: () => Effect<void>
  cleanup: () => Effect<void>
  track: () => Effect<string | undefined>           // returns tree hash
  patch: (hash: string) => Effect<Patch>            // list files changed since hash
  restore: (snapshot: string) => Effect<void>       // full restore to hash
  revert: (patches: Patch[]) => Effect<void>        // selective file revert
  diff: (hash: string) => Effect<string>            // unified diff text
  diffFull: (from: string, to: string) => Effect<FileDiff[]>  // structured diffs
}
```

---

## Data Structures

### Shadow Repo Layout

```
$DATA/snapshot/<projectId>/<hash(worktree)>/
  ├── HEAD
  ├── objects/
  ├── refs/
  └── info/
      └── exclude    # gitignore rules from project .git + large-file blocks
```

### Git Operations Used

| Operation | Purpose |
|---|---|
| `git --git-dir=... write-tree` | Create snapshot tree hash |
| `git --git-dir=... add --all --sparse` | Stage changed files |
| `git --git-dir=... diff-files` | List modified tracked files |
| `git --git-dir=... ls-files --others` | List untracked files |
| `git --git-dir=... check-ignore` | Identify .gitignored files |
| `git --git-dir=... checkout <hash> -- <file>` | Revert individual files |
| `git --git-dir=... cat-file --batch` | Batch read file contents for diffs |
| `git --git-dir=... gc --prune=7.days` | Cleanup old objects |

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `config` module | `snapshot` enabled/disabled config |
| `global` module | `Global.Path.data` for shadow repo directory |
| `effect/filesystem` | File existence, read, write |
| `effect/process` | Running `git` subprocesses |
| `diff` npm package | `structuredPatch` / `formatPatch` for JS-side diffs |

---

## Acceptance Criteria

- [ ] `track()` initializes shadow repo on first call and returns a tree hash
- [ ] `track()` returns `undefined` if snapshot is disabled in config
- [ ] Gitignored files are excluded from snapshots
- [ ] Files > 2 MB are excluded from untracked staging
- [ ] `restore()` checks out the tree hash to the working directory
- [ ] `revert()` restores each file from its snapshot hash; deletes files that didn't exist in the snapshot
- [ ] `diff()` and `diffFull()` produce accurate unified diffs
- [ ] Cleanup runs hourly (1 minute after init) and prunes objects older than 7 days
- [ ] Concurrent calls are serialized via per-directory semaphore
- [ ] Works only for git projects (`vcs === "git"`)

---

## Rust Implementation Guidance

### Crate: `crates/snapshot/`

### Key Crates

```toml
tokio = { features = ["full", "process"] }
similar = "2"            # Diff computation (alternative to diff npm)
sha2 = "0.10"           # Hash worktree path for shadow repo name
tokio::sync::Semaphore  # Per-directory serialization
```

### Architecture

```rust
pub struct SnapshotService {
    /// Per-worktree semaphore for serialization
    locks: Mutex<HashMap<PathBuf, Arc<Semaphore>>>,
    config: Arc<ConfigService>,
    data_path: PathBuf,
}

impl SnapshotService {
    pub async fn track(&self, ctx: &InstanceContext) -> Result<Option<String>> {
        let lock = self.lock_for(ctx.worktree()).await;
        let _permit = lock.acquire().await.unwrap();

        if !self.is_enabled(ctx).await? { return Ok(None); }
        self.ensure_shadow_repo(ctx).await?;
        self.stage_changes(ctx).await?;
        let hash = self.write_tree(ctx).await?;
        Ok(Some(hash))
    }

    async fn git(&self, args: &[&str], cwd: &Path) -> Result<GitResult> {
        let output = tokio::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .await?;
        Ok(GitResult {
            code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        })
    }
}
```

### Key Constants

```rust
const BUFFER_LIMIT: usize = 2 * 1024 * 1024;  // max file size for tracking
const PRUNE_AGE: &str = "7.days";
const CLEANUP_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour
const CLEANUP_DELAY: Duration = Duration::from_secs(60);      // 1 min after init
```

---

## Test Design

### Unit Tests

```rust
#[tokio::test]
async fn test_track_returns_hash_for_modified_file() {
    let (svc, ctx) = SnapshotService::new_test().await;
    std::fs::write(ctx.worktree().join("file.txt"), "hello").unwrap();
    let hash = svc.track(&ctx).await.unwrap();
    assert!(hash.is_some());
    assert_eq!(hash.unwrap().len(), 40); // SHA1 hex
}

#[tokio::test]
async fn test_revert_restores_modified_file() {
    let (svc, ctx) = SnapshotService::new_test().await;
    std::fs::write(ctx.worktree().join("file.txt"), "original").unwrap();
    let hash = svc.track(&ctx).await.unwrap().unwrap();
    std::fs::write(ctx.worktree().join("file.txt"), "modified").unwrap();
    let patch = svc.patch(&ctx, &hash).await.unwrap();
    svc.revert(&ctx, &[patch]).await.unwrap();
    let content = std::fs::read_to_string(ctx.worktree().join("file.txt")).unwrap();
    assert_eq!(content, "original");
}

#[tokio::test]
async fn test_disabled_snapshot_returns_none() {
    let (svc, ctx) = SnapshotService::new_with_config(Config { snapshot: Some(false), ..Default::default() }).await;
    let hash = svc.track(&ctx).await.unwrap();
    assert!(hash.is_none());
}

#[tokio::test]
async fn test_diff_full_shows_additions_deletions() {
    let (svc, ctx) = SnapshotService::new_test().await;
    std::fs::write(ctx.worktree().join("f.txt"), "line1\nline2\n").unwrap();
    let hash1 = svc.track(&ctx).await.unwrap().unwrap();
    std::fs::write(ctx.worktree().join("f.txt"), "line1\nline2\nline3\n").unwrap();
    let hash2 = svc.track(&ctx).await.unwrap().unwrap();
    let diffs = svc.diff_full(&ctx, &hash1, &hash2).await.unwrap();
    assert_eq!(diffs[0].additions, 1);
    assert_eq!(diffs[0].deletions, 0);
}
```

### Integration Tests (from TS patterns)

- `snapshot.test.ts`: Track → modify file → patch → revert → verify file restored; cleanup prunes correctly
