# PRD: skill Module

## Module Overview

- **Module Name**: `skill`
- **Source Path**: `packages/opencode/src/skill/`
- **Type**: Integration Service
- **Rust Crate**: `crates/skill/`
- **Purpose**: Discovers, loads, and manages agent skills — markdown files (`SKILL.md`) that inject additional instructions into the agent's context. Supports local filesystem discovery, URL-based remote skill packs, and permission-based filtering.

---

## Functionality

### Core Features

1. **Skill Discovery** — Scans multiple directories for `SKILL.md` files using glob patterns
2. **Multi-Source Loading** — Global `~/.claude/skills/`, `~/.agents/skills/`, project-level dirs, config-defined paths, URL-based remote packs
3. **Frontmatter Parsing** — Extracts `name` and `description` from YAML frontmatter; `content` is the markdown body
4. **Remote Skill Packs** — Downloads skill packs from URLs via `index.json` manifest
5. **Deduplication Warning** — Warns on duplicate skill names (last write wins)
6. **Permission Filtering** — `available(agent?)` filters skills by agent permission rules
7. **Skill Formatting** — Formats skill list for injection into agent prompts (verbose XML or compact markdown)
8. **Feature Flag** — `OPENCODE_DISABLE_EXTERNAL_SKILLS` disables global/project external skill scanning

---

## SKILL.md Format

```markdown
---
name: my-skill
description: A brief description of what this skill does.
---

# Skill Content

Markdown instructions injected into the agent context...
```

### Frontmatter Schema

```rust
#[derive(Debug, Deserialize, Serialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
}
```

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    /// Absolute file path to the SKILL.md
    pub location: PathBuf,
    /// Markdown body (without frontmatter)
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkillManifest {
    pub skills: Vec<RemoteSkillEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkillEntry {
    pub name: String,
    pub files: Vec<String>,
}
```

### Service Interface

```rust
pub struct SkillService {
    skills: Arc<RwLock<HashMap<String, SkillInfo>>>,
    dirs: Arc<RwLock<HashSet<PathBuf>>>,
    config: Arc<ConfigService>,
    permission: Arc<PermissionService>,
    cache_dir: PathBuf,
    http: reqwest::Client,
}

impl SkillService {
    /// Discover and load all skills from all configured sources
    pub async fn discover_and_load(&self, ctx: &InstanceContext) -> Result<(), SkillError> {
        let paths = self.scan_all_sources(ctx).await?;
        for path in paths {
            if let Err(e) = self.load_skill(&path).await {
                tracing::warn!(path = %path.display(), error = %e, "Failed to load skill");
                // Per AC: publish error on bus, don't fail the whole load
                let _ = self.bus.publish("session.error", json!({
                    "type": "skill_parse_error",
                    "path": path.to_string_lossy(),
                    "error": e.to_string(),
                })).await;
            }
        }
        Ok(())
    }

    /// Get a skill by name
    pub async fn get(&self, name: &str) -> Option<SkillInfo> {
        self.skills.read().await.get(name).cloned()
    }

    /// Get all skills (optionally filtered by agent permissions)
    pub async fn all(&self, agent: Option<&AgentInfo>) -> Vec<SkillInfo> {
        let skills: Vec<_> = self.skills.read().await.values().cloned().collect();
        if let Some(agent_info) = agent {
            skills.into_iter()
                .filter(|s| self.permission.is_allowed(agent_info, "skill", &s.name))
                .collect()
        } else {
            skills
        }
    }

    /// Get all directories containing skills
    pub async fn dirs(&self) -> Vec<PathBuf> {
        self.dirs.read().await.iter().cloned().collect()
    }

    /// Format skills for injection into agent context
    pub fn fmt(&self, skills: &[SkillInfo], verbose: bool) -> String {
        if verbose {
            skills.iter().map(|s| {
                format!(
                    "<skill>\n  <name>{}</name>\n  <description>{}</description>\n  <location>{}</location>\n</skill>",
                    s.name, s.description, s.location.display()
                )
            }).collect::<Vec<_>>().join("\n")
        } else {
            skills.iter().map(|s| {
                format!("- **{}**: {}", s.name, s.description)
            }).collect::<Vec<_>>().join("\n")
        }
    }

    /// Pull skills from a remote URL
    pub async fn pull(&self, url: &str) -> Result<Vec<PathBuf>, SkillError> {
        // 1. Fetch <url>/index.json
        let manifest: RemoteSkillManifest = self.http
            .get(format!("{}/index.json", url))
            .send().await?
            .json().await?;

        let cache_dir = self.cache_dir.join(url_to_dir_name(url));
        tokio::fs::create_dir_all(&cache_dir).await?;

        // 2. Download each file
        for entry in &manifest.skills {
            for file in &entry.files {
                let content = self.http
                    .get(format!("{}/{}", url, file))
                    .send().await?
                    .text().await?;
                let dest = cache_dir.join(&entry.name).join(file);
                tokio::fs::create_dir_all(dest.parent().unwrap()).await?;
                tokio::fs::write(&dest, content).await?;
            }
        }

        // 3. Return paths containing SKILL.md
        let paths = glob(&cache_dir, "**/SKILL.md")?;
        for path in &paths {
            let _ = self.load_skill(path).await;
        }
        Ok(paths)
    }
}
```

### `SkillError`

```rust
#[derive(Debug, Error)]
pub enum SkillError {
    #[error("Skill file not found: {0}")]
    NotFound(String),

    #[error("Failed to read skill file: {0}")]
    ReadError(#[source] std::io::Error),

    #[error("Failed to parse frontmatter: {0}")]
    ParseError(String),

    #[error("Missing frontmatter field '{field}' in {path}")]
    MissingField { field: String, path: PathBuf },

    #[error("Duplicate skill name: {0}")]
    DuplicateName(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Remote fetch failed: {0}")]
    RemoteError(String),
}
```

---

## Discovery Algorithm

```rust
impl SkillService {
    async fn scan_all_sources(&self, ctx: &InstanceContext) -> Result<Vec<PathBuf>, SkillError> {
        let mut matches = Vec::new();
        let mut dirs = HashSet::new();

        // Feature flag check
        if std::env::var("OPENCODE_DISABLE_EXTERNAL_SKILLS").is_ok() {
            // Only load project-local skills
            let project_skills = self.scan_dir(&ctx.project_dir().join(".claude/skills")).await?;
            matches.extend(project_skills);
        } else {
            // Global sources
            let home = dirs::home_dir().unwrap_or_default();
            for glob_pattern in [
                home.join(".claude/skills/**/SKILL.md"),
                home.join(".agents/skills/**/SKILL.md"),
            ] {
                matches.extend(glob_path(&glob_pattern).await?);
                if let Some(parent) = glob_pattern.parent() {
                    dirs.insert(parent.to_path_buf());
                }
            }

            // Project-local sources (walk up from project dir)
            let project_root = &ctx.project_dir();
            for subdir in [".claude/skills", ".agents/skills"] {
                let path = project_root.join(subdir);
                if path.exists() {
                    matches.extend(glob_path(&path.join("**/SKILL.md")).await?);
                    dirs.insert(path);
                }
            }

            // Config-defined custom paths
            for custom_path in self.config.skills().paths() {
                matches.extend(glob_path(&custom_path.join("**/SKILL.md")).await?);
                dirs.insert(custom_path);
            }
        }

        // Remote URLs
        for url in self.config.skills().urls() {
            if let Err(e) = self.pull(url).await {
                tracing::warn!(url, error = %e, "Failed to pull remote skills");
            }
        }

        // Update dirs cache
        *self.dirs.write().await = dirs;

        Ok(matches)
    }

    async fn load_skill(&self, path: &Path) -> Result<SkillInfo, SkillError> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(SkillError::ReadError)?;

        let (frontmatter, body) = split_frontmatter(&content)?;
        let meta: SkillFrontmatter = serde_yaml::from_str(&frontmatter)
            .map_err(|e| SkillError::ParseError(e.to_string()))?;

        let info = SkillInfo {
            name: meta.name,
            description: meta.description,
            location: path.to_path_buf(),
            content: body.to_string(),
        };

        // Check for duplicate
        let mut skills = self.skills.write().await;
        if skills.contains_key(&info.name) {
            tracing::warn!(name = %info.name, "Duplicate skill name (overwriting)");
        }
        skills.insert(info.name.clone(), info.clone());

        Ok(info)
    }
}

fn split_frontmatter(content: &str) -> Result<(String, &str), SkillError> {
    if let Some(rest) = content.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            return Ok((rest[..end].to_string(), &rest[end + 5..]));
        }
    }
    Err(SkillError::ParseError("Missing --- frontmatter delimiters".into()))
}
```

---

## Crate Layout

```
crates/skill/
├── Cargo.toml       # glob = "0.3", reqwest = { version = "0.11", features = ["json"] }, serde_yaml = "0.9"
├── src/
│   ├── lib.rs       # SkillService, SkillInfo, SkillError
│   ├── discovery.rs # Scan and load from filesystem
│   ├── remote.rs    # Remote skill pack pulling
│   ├── format.rs    # Formatting for agent context
│   └── permissions.rs # Permission filtering
└── tests/
    └── skill_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-skill"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["fs", "sync", "rt"] }
glob = "0.3"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
walkdir = "2"
anyhow = "1.0"
dirs = "5"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
wiremock = "0.6"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `config` module | `config.directories()`, `config.skills.paths`, `config.skills.urls` |
| `bus` module | Error event publishing on skill parse failure |
| `permission` module | `Permission.is_allowed("skill", name, agent)` |
| `global` module | `Global.Path.home`, `Global.Path.cache` |
| `reqwest` | Downloading remote skill packs |
| `glob` / `walkdir` | Pattern-based file scanning |
| `serde_yaml` | Frontmatter parsing |
| `dirs` | Home directory detection |

---

## Acceptance Criteria

- [x] Skills are discovered from all configured source directories
- [x] Frontmatter `name` and `description` are extracted correctly
- [x] Duplicate skill names log a warning (last wins)
- [x] `get(name)` returns the correct skill info
- [x] `available(agent)` filters skills denied by agent permission rules
- [x] Remote skill packs are downloaded and cached
- [x] `OPENCODE_DISABLE_EXTERNAL_SKILLS` prevents global/project discovery
- [x] `fmt()` produces correct verbose XML and compact markdown output
- [x] Parse errors publish `session.error` on the bus

---

## Test Design

### Unit Tests

```rust
#[test]
fn test_split_frontmatter_extracts_correctly() {
    let content = "---\nname: my-skill\ndescription: Does something.\n---\n\n# Content\nHello";
    let (fm, body) = split_frontmatter(content).unwrap();
    let meta: SkillFrontmatter = serde_yaml::from_str(&fm).unwrap();
    assert_eq!(meta.name, "my-skill");
    assert_eq!(meta.description, "Does something.");
    assert_eq!(body.trim(), "# Content\nHello");
}

#[test]
fn test_split_frontmatter_missing_delimiter_errors() {
    let content = "no frontmatter\n---\nname: x\n---\n";
    assert!(split_frontmatter(content).is_err());
}

#[test]
fn test_split_frontmatter_empty_frontmatter() {
    let content = "---\n---\nbody content";
    let (fm, body) = split_frontmatter(content).unwrap();
    assert!(fm.is_empty());
    assert_eq!(body.trim(), "body content");
}

#[tokio::test]
async fn test_load_skill_extracts_frontmatter() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("SKILL.md");
    tokio::fs::write(&path, "---\nname: test\ndescription: desc\n---\n\n# Test").await.unwrap();

    let svc = SkillService::new_test().await;
    let info = svc.load_skill(&path).await.unwrap();
    assert_eq!(info.name, "test");
    assert_eq!(info.description, "desc");
}

#[tokio::test]
async fn test_get_returns_skill_by_name() {
    let svc = SkillService::new_test().await;
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("SKILL.md");
    tokio::fs::write(&path, "---\nname: my-skill\ndescription: desc\n---\n\n# Body").await.unwrap();
    svc.load_skill(&path).await.unwrap();

    let info = svc.get("my-skill").await;
    assert!(info.is_some());
    assert_eq!(info.unwrap().description, "desc");
}

#[test]
fn test_fmt_verbose_produces_xml() {
    let svc = SkillService::new_test().await;
    let skills = vec![SkillInfo {
        name: "s1".into(),
        description: "d1".into(),
        location: PathBuf::from("/path/SKILL.md"),
        content: "# s1".into(),
    }];
    let out = svc.fmt(&skills, true);
    assert!(out.contains("<skill>"));
    assert!(out.contains("<name>s1</name>"));
    assert!(out.contains("<description>d1</description>"));
}

#[test]
fn test_fmt_compact_produces_markdown() {
    let svc = SkillService::new_test().await;
    let skills = vec![SkillInfo {
        name: "s1".into(),
        description: "d1".into(),
        location: PathBuf::from("/path/SKILL.md"),
        content: "# s1".into(),
    }];
    let out = svc.fmt(&skills, false);
    assert!(out.contains("- **s1**: d1"));
    assert!(!out.contains("<skill>"));
}

#[tokio::test]
async fn test_duplicate_name_warns() {
    let svc = SkillService::new_test().await;
    let tmp = TempDir::new().unwrap();

    // Two files with same name
    tokio::fs::write(tmp.path().join("a/SKILL.md"), "---\nname: dup\ndescription: first\n---\n# A").await.unwrap();
    tokio::fs::write(tmp.path().join("b/SKILL.md"), "---\nname: dup\ndescription: second\n---\n# B").await.unwrap();

    svc.load_skill(&tmp.path().join("a/SKILL.md")).await.unwrap();
    // Second load should warn but not panic
    svc.load_skill(&tmp.path().join("b/SKILL.md")).await.unwrap();

    // Last one wins
    let info = svc.get("dup").await.unwrap();
    assert_eq!(info.description, "second");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_pull_remote_skill_pack() {
    // Mock HTTP server serving index.json + SKILL.md files
    Mock::given(get, "/index.json")
        .respond_with(json_response(200, &RemoteSkillManifest {
            skills: vec![RemoteSkillEntry {
                name: "remote-skill".into(),
                files: vec!["SKILL.md".into()],
            }],
        }))
        .and(
            Mock::given(get, "/SKILL.md")
                .respond_with(body("---\nname: remote-skill\ndescription: From remote\n---\n\n# Remote"))
        )
        .mount(&mock_server)
        .await;

    let svc = SkillService::new_test().await;
    svc.pull(&mock_server.uri()).await.unwrap();

    let info = svc.get("remote-skill").await;
    assert!(info.is_some());
}
```

---

## Source Reference

*Source: `packages/opencode/src/skill/index.ts`*
*No existing Rust equivalent — implement in `crates/skill/`*
