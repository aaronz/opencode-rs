# PRD: skill Module

## Module Overview

- **Module Name**: skill
- **Source Path**: `packages/opencode/src/skill/`
- **Type**: Integration Service
- **Purpose**: Discovers, loads, and manages agent skills — markdown files (`SKILL.md`) that inject additional instructions into the agent's context. Supports local filesystem discovery, URL-based remote skill packs, and permission-based filtering.

---

## Functionality

### Core Features

1. **Skill Discovery** — Scans multiple directories for `SKILL.md` files using glob patterns
2. **Multi-Source Loading** — Global `~/.claude/`, `~/.agents/`, project-level directories, config-defined paths, URL-based remote packs
3. **Frontmatter Parsing** — Extracts `name` and `description` from YAML frontmatter; `content` is the markdown body
4. **Remote Skill Packs** — Downloads skill packs from URLs via `index.json` manifest (via `Discovery` sub-service)
5. **Deduplication Warning** — Warns on duplicate skill names (last write wins)
6. **Permission Filtering** — `available(agent?)` filters skills by agent permission rules
7. **Skill Formatting** — Formats skill list for injection into agent prompts (verbose XML or compact markdown)
8. **Feature Flag** — `OPENCODE_DISABLE_EXTERNAL_SKILLS` disables global/project external skill scanning

### Discovery Patterns

```
Global:   ~/.claude/skills/**/SKILL.md
          ~/.agents/skills/**/SKILL.md
Project:  (walk up from project dir) .claude/skills/**/SKILL.md
                                     .agents/skills/**/SKILL.md
Config:   config.directories() → {skill,skills}/**/SKILL.md
Custom:   config.skills.paths[] → **/SKILL.md
Remote:   config.skills.urls[]  → Discovery.pull(url) → **/SKILL.md
```

---

## API Surface

### Skill Info

```typescript
interface Info {
  name: string
  description: string
  location: string   // absolute file path
  content: string    // markdown body (without frontmatter)
}
```

### Service Interface

```typescript
interface Interface {
  get: (name: string) => Effect<Info | undefined>
  all: () => Effect<Info[]>
  dirs: () => Effect<string[]>           // all directories containing skills
  available: (agent?: Agent.Info) => Effect<Info[]>  // permission-filtered
}
```

### Formatting

```typescript
function fmt(list: Info[], opts: { verbose: boolean }): string
// verbose=false → markdown list: "- **name**: description"
// verbose=true  → XML blocks with name, description, location
```

### Discovery Sub-Service (`discovery.ts`)

```typescript
interface Discovery.Interface {
  pull: (url: string) => Effect<string[]>  // returns local skill directories
}
```

Remote protocol:
1. Fetch `<url>/index.json` → `{ skills: [{ name, files }] }`
2. Download each file to `$CACHE/skills/<name>/<file>`
3. Return directories containing `SKILL.md`

---

## Data Structures

```typescript
// Internal state
type State = {
  skills: Record<string, Info>  // name → Info
  dirs: Set<string>             // all directories with skills
}

type ScanState = {
  matches: Set<string>  // absolute SKILL.md paths
  dirs: Set<string>
}
```

### SKILL.md Format

```markdown
---
name: my-skill
description: A brief description of what this skill does.
---

# Skill Content

Markdown instructions that are injected into the agent context...
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `config` module | `config.directories()`, `config.skills.paths`, `config.skills.urls` |
| `bus` module | Error event publishing on skill parse failure |
| `permission` module | `Permission.evaluate("skill", name, agent.permission)` |
| `flag` module | `OPENCODE_DISABLE_EXTERNAL_SKILLS` |
| `global` module | `Global.Path.home`, `Global.Path.cache` |
| `effect/http` | Downloading remote skill packs |
| `effect/filesystem` | File/directory existence checks |
| `@opencode-ai/shared/util/glob` | Pattern-based file scanning |

---

## Acceptance Criteria

- [ ] Skills are discovered from all configured source directories
- [ ] Frontmatter `name` and `description` are extracted correctly
- [ ] Duplicate skill names log a warning
- [ ] `get(name)` returns the correct skill info
- [ ] `available(agent)` filters skills denied by agent permission rules
- [ ] Remote skill packs are downloaded and cached
- [ ] `OPENCODE_DISABLE_EXTERNAL_SKILLS` prevents global/project discovery
- [ ] `fmt()` produces correct verbose XML and compact markdown output
- [ ] Parse errors publish `Session.Event.Error` on the bus

---

## Rust Implementation Guidance

### Crate: `crates/skill/`

### Key Crates

```toml
glob = "0.3"
reqwest = { version = "0.11", features = ["json"] }
serde_yaml = "0.9"       # Frontmatter parsing
serde_json = "1"
tokio = { features = ["full"] }
```

### Architecture

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub location: PathBuf,
    pub content: String,
}

pub struct SkillService {
    skills: Arc<RwLock<HashMap<String, SkillInfo>>>,
    dirs: Arc<RwLock<HashSet<PathBuf>>>,
    config: Arc<ConfigService>,
    permission: Arc<PermissionService>,
}

impl SkillService {
    pub async fn discover_and_load(&self, ctx: &InstanceContext) -> Result<()> {
        let matches = self.scan_all_sources(ctx).await?;
        for path in matches {
            if let Ok(info) = self.parse_skill_md(&path).await {
                self.skills.write().await.insert(info.name.clone(), info);
            }
        }
        Ok(())
    }

    async fn parse_skill_md(&self, path: &Path) -> Result<SkillInfo> {
        let content = tokio::fs::read_to_string(path).await?;
        let (frontmatter, body) = split_frontmatter(&content)?;
        let meta: SkillMeta = serde_yaml::from_str(&frontmatter)?;
        Ok(SkillInfo {
            name: meta.name,
            description: meta.description,
            location: path.to_owned(),
            content: body.to_string(),
        })
    }
}

fn split_frontmatter(content: &str) -> Result<(String, &str)> {
    if content.starts_with("---\n") {
        let end = content[4..].find("\n---\n").map(|i| i + 4);
        if let Some(end_pos) = end {
            return Ok((content[4..end_pos].to_string(), &content[end_pos+5..]));
        }
    }
    Ok((String::new(), content))
}
```

---

## Test Design

### Unit Tests

```rust
#[test]
fn test_parse_skill_md_extracts_name_and_description() {
    let content = "---\nname: my-skill\ndescription: Does something.\n---\n\n# Content\nHello";
    let (fm, body) = split_frontmatter(content).unwrap();
    let meta: SkillMeta = serde_yaml::from_str(&fm).unwrap();
    assert_eq!(meta.name, "my-skill");
    assert_eq!(body.trim(), "# Content\nHello");
}

#[tokio::test]
async fn test_get_returns_skill_by_name() {
    let svc = SkillService::new_test().await;
    let skill_path = write_test_skill("test-skill", "Test description");
    svc.load_from_path(&skill_path).await.unwrap();
    let info = svc.get("test-skill").await.unwrap();
    assert!(info.is_some());
    assert_eq!(info.unwrap().description, "Test description");
}

#[tokio::test]
async fn test_duplicate_skill_name_warns() {
    // Two files with same name — second should overwrite, warning logged
    let svc = SkillService::new_test().await;
    // verify no panic, just a warning
}

#[test]
fn test_fmt_verbose_produces_xml() {
    let info = vec![SkillInfo { name: "s1".into(), description: "d1".into(), .. }];
    let out = fmt(&info, true);
    assert!(out.contains("<skill>"));
    assert!(out.contains("<name>s1</name>"));
}

#[test]
fn test_fmt_compact_produces_markdown() {
    let info = vec![SkillInfo { name: "s1".into(), description: "d1".into(), .. }];
    let out = fmt(&info, false);
    assert!(out.contains("- **s1**: d1"));
}
```

### Integration Tests (from TS patterns)

- `skill.test.ts`: Discovery from filesystem, permission filtering, format output
- `discovery.test.ts`: Remote URL pull — mock HTTP server serving `index.json`, verify skills downloaded and loaded
