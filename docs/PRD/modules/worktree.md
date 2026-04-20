# PRD: worktree Module

## Module Overview

- **Module Name**: worktree
- **Source Path**: `packages/opencode/src/worktree/`
- **Type**: Infrastructure Service
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

### Error Types

```typescript
WorktreeNotGitError          // project is not git
WorktreeNameGenerationFailedError
WorktreeCreateFailedError
WorktreeStartCommandFailedError
WorktreeRemoveFailedError
WorktreeResetFailedError
```

---

## API Surface

### Types

```typescript
interface Info {
  name: string      // e.g. "brave-lion"
  branch: string    // e.g. "opencode/brave-lion"
  directory: string // absolute path
}

interface CreateInput {
  name?: string           // optional name hint; slugified
  startCommand?: string   // extra script after project start
}

interface RemoveInput { directory: string }
interface ResetInput  { directory: string }
```

### Service Interface

```typescript
interface Interface {
  makeWorktreeInfo: (name?: string) => Effect<Info>
  createFromInfo: (info: Info, startCommand?: string) => Effect<void>
  create: (input?: CreateInput) => Effect<Info>
  remove: (input: RemoveInput) => Effect<boolean>
  reset: (input: ResetInput) => Effect<boolean>
}
```

### Events (GlobalBus)

```typescript
Event.Ready   // { name: string; branch: string }
Event.Failed  // { message: string }
```

---

## Data Structures

### Storage Layout

```
$DATA/worktree/<projectId>/
  ├── brave-lion/     # worktree directory
  └── silent-hawk/
```

### Branch Convention

```
opencode/<name>    # e.g. opencode/brave-lion
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `git` module | `defaultBranch()` for reset target |
| `project` module | `addSandbox()`, project config (`commands.start`) |
| `global` module | `Global.Path.data` for worktree storage |
| `effect/process` | Running `git` subprocesses |
| `effect/filesystem` | Directory creation/removal |
| `bus/global` | Emitting `worktree.ready` / `worktree.failed` |
| `@opencode-ai/shared/util/slug` | Slug generation |

---

## Acceptance Criteria

- [ ] `create()` generates a unique branch and directory name
- [ ] Worktree is initialized with `git worktree add --no-checkout -b`
- [ ] After creation, `git reset --hard` and project bootstrap run in the background
- [ ] Start scripts (project + extra) run after bootstrap
- [ ] `worktree.ready` is published when bootstrap succeeds
- [ ] `worktree.failed` is published when bootstrap fails
- [ ] `remove()` removes the worktree, branch, and directory
- [ ] `reset()` fetches default branch, hard-resets, cleans, and updates submodules
- [ ] `reset()` refuses to reset the primary workspace
- [ ] Works only for git projects

---

## Rust Implementation Guidance

### Crate: `crates/worktree/`

### Key Crates

```toml
tokio = { features = ["process", "fs"] }
slug = "0.1"                  # Slug generation
rand = "0.8"                  # Random name component
```

### Architecture

```rust
pub struct WorktreeService {
    git: Arc<GitService>,
    project: Arc<ProjectService>,
    global_bus: Arc<GlobalBus>,
    data_path: PathBuf,
}

impl WorktreeService {
    pub async fn create(&self, ctx: &InstanceContext, input: CreateInput) -> Result<WorktreeInfo> {
        let info = self.make_worktree_info(ctx, input.name.as_deref()).await?;
        self.setup(ctx, &info).await?;
        // Boot in background
        let svc = self.clone();
        let ctx2 = ctx.clone();
        let info2 = info.clone();
        tokio::spawn(async move {
            if let Err(e) = svc.boot(&ctx2, &info2, input.start_command.as_deref()).await {
                svc.global_bus.emit("event", WorktreeFailedEvent { message: e.to_string() });
            }
        });
        Ok(info)
    }

    async fn make_worktree_info(&self, ctx: &InstanceContext, name: Option<&str>) -> Result<WorktreeInfo> {
        if ctx.project().vcs != "git" {
            return Err(WorktreeError::NotGit);
        }
        let root = self.data_path.join(&ctx.project().id);
        tokio::fs::create_dir_all(&root).await?;
        let base = name.map(slugify).filter(|s| !s.is_empty());
        self.find_candidate(&root, ctx, base.as_deref()).await
    }
}
```

### Name Generation

```rust
fn slugify(input: &str) -> String {
    input.trim().to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

async fn find_candidate(&self, root: &Path, ctx: &InstanceContext, base: Option<&str>) -> Result<WorktreeInfo> {
    for attempt in 0..26 {
        let name = match base {
            Some(b) if attempt == 0 => b.to_string(),
            Some(b) => format!("{}-{}", b, random_slug()),
            None => random_slug(),
        };
        let dir = root.join(&name);
        let branch = format!("opencode/{}", name);
        if !dir.exists() && !branch_exists(ctx, &branch).await? {
            return Ok(WorktreeInfo { name, branch, directory: dir.to_str().unwrap().to_string() });
        }
    }
    Err(WorktreeError::NameGenerationFailed)
}
```

---

## Test Design

### Unit Tests

```rust
#[test]
fn test_slugify() {
    assert_eq!(slugify("My Feature Branch"), "my-feature-branch");
    assert_eq!(slugify("  --hello--  "), "hello");
}

#[tokio::test]
async fn test_make_worktree_info_generates_unique_name() {
    let svc = WorktreeService::new_test().await;
    let info = svc.make_worktree_info(&ctx(), None).await.unwrap();
    assert!(!info.name.is_empty());
    assert!(info.branch.starts_with("opencode/"));
    assert!(!info.directory.is_empty());
}

#[tokio::test]
async fn test_create_fails_for_non_git_project() {
    let svc = WorktreeService::new_test().await;
    let ctx = InstanceContext::with_vcs("none");
    let result = svc.create(&ctx, CreateInput::default()).await;
    assert!(matches!(result, Err(WorktreeError::NotGit)));
}

#[tokio::test]
async fn test_reset_refuses_primary_workspace() {
    let svc = WorktreeService::new_test().await;
    let ctx = InstanceContext::new_git();
    let result = svc.reset(&ctx, ResetInput { directory: ctx.worktree().to_string() }).await;
    assert!(matches!(result, Err(WorktreeError::ResetFailed { .. })));
}
```

### Integration Tests (no TS tests exist; derive from source)

- Create → verify directory exists, branch exists in git
- Remove → directory deleted, branch deleted
- Reset → files match default branch HEAD
