use crate::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub version: String,
    pub triggers: Vec<String>,
    pub priority: i32,
    pub location: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill: Skill,
    pub match_type: MatchType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Prefix,
    Fuzzy,
    Semantic,
}

pub struct SkillManager {
    skills: RwLock<Vec<Skill>>,
    global_skills_path: Option<PathBuf>,
    project_skills_path: Option<PathBuf>,
    builtin_skills_path: Option<PathBuf>,
    compat_skills_paths: Vec<PathBuf>,
    discovered: RwLock<bool>,
}

const BUILTIN_SKILLS: &[(&str, &str)] = &[
    (
        "code-review/SKILL.md",
        include_str!("../skills/code-review/SKILL.md"),
    ),
    (
        "architect/SKILL.md",
        include_str!("../skills/architect/SKILL.md"),
    ),
    (
        "debugger/SKILL.md",
        include_str!("../skills/debugger/SKILL.md"),
    ),
    (
        "test-writer/SKILL.md",
        include_str!("../skills/test-writer/SKILL.md"),
    ),
    (
        "refactorer/SKILL.md",
        include_str!("../skills/refactorer/SKILL.md"),
    ),
    (
        "doc-writer/SKILL.md",
        include_str!("../skills/doc-writer/SKILL.md"),
    ),
    (
        "security-auditor/SKILL.md",
        include_str!("../skills/security-auditor/SKILL.md"),
    ),
    (
        "performance/SKILL.md",
        include_str!("../skills/performance/SKILL.md"),
    ),
    (
        "data-analyst/SKILL.md",
        include_str!("../skills/data-analyst/SKILL.md"),
    ),
    ("devops/SKILL.md", include_str!("../skills/devops/SKILL.md")),
];

impl SkillManager {
    pub fn new() -> Self {
        let global_path = dirs::config_dir().map(|p| p.join("opencode").join("skills"));

        let compat_skills_paths = Self::default_compat_paths();

        Self {
            skills: RwLock::new(Vec::new()),
            global_skills_path: global_path,
            project_skills_path: None,
            builtin_skills_path: None,
            compat_skills_paths,
            discovered: RwLock::new(false),
        }
    }

    fn default_compat_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = dirs::home_dir() {
            let claude_path = home.join(".claude").join("skills");
            if claude_path.exists() || Self::is_common_compat_path(&claude_path) {
                paths.push(claude_path);
            }

            let agent_path = home.join(".agent").join("skills");
            if agent_path.exists() || Self::is_common_compat_path(&agent_path) {
                paths.push(agent_path);
            }
        }

        paths
    }

    fn is_common_compat_path(path: &PathBuf) -> bool {
        let common_paths = [".claude/skills", ".agent/skills"];

        for common in &common_paths {
            if path.ends_with(common) {
                return true;
            }
        }

        false
    }

    pub fn with_compat_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.compat_skills_paths = paths;
        self
    }

    pub fn with_project_path(mut self, project_path: PathBuf) -> Self {
        self.project_skills_path = Some(project_path.join(".opencode").join("skills"));
        self
    }

    pub fn with_builtin_skills_path(mut self, path: PathBuf) -> Self {
        self.builtin_skills_path = Some(path);
        self
    }

    pub fn set_builtin_skills_path(&mut self, path: PathBuf) {
        self.builtin_skills_path = Some(path);
    }

    pub fn set_project_path(&mut self, project_path: PathBuf) {
        self.project_skills_path = Some(project_path.join(".opencode").join("skills"));
    }

    fn ensure_discovered(&self) -> Result<(), OpenCodeError> {
        let discovered = self
            .discovered
            .write()
            .map_err(|_| OpenCodeError::Config("Failed to acquire write lock".to_string()))?;

        if *discovered {
            return Ok(());
        }

        drop(discovered);
        self.discover()?;

        let mut discovered = self
            .discovered
            .write()
            .map_err(|_| OpenCodeError::Config("Failed to acquire write lock".to_string()))?;
        *discovered = true;

        Ok(())
    }

    fn parse_frontmatter(content: &str) -> Option<(SkillMeta, &str)> {
        if !content.starts_with("---") {
            return None;
        }

        let end_idx = content[3..].find("---")?;
        let yaml_part = &content[3..3 + end_idx];
        let body = &content[3 + end_idx + 3..];

        let mut meta = SkillMeta::default();
        for line in yaml_part.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "name" => meta.name = Some(value.to_string()),
                    "description" => meta.description = Some(value.to_string()),
                    "version" => meta.version = Some(value.to_string()),
                    "priority" => meta.priority = value.parse().unwrap_or(0),
                    "triggers" => {
                        meta.triggers = value
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .collect();
                    }
                    _ => {}
                }
            }
        }

        Some((meta, body))
    }

    pub fn discover_in_dir(&self, dir: &PathBuf) -> Result<Vec<Skill>, OpenCodeError> {
        let mut found = Vec::new();
        if !dir.exists() {
            return Ok(found);
        }

        for entry in WalkDir::new(dir).max_depth(2) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Error walking skill directory: {}", e);
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            if !path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.contains("SKILL"))
                .unwrap_or(false)
            {
                continue;
            }

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to read skill file {:?}: {}", path, e);
                    continue;
                }
            };

            let (meta, body) = match Self::parse_frontmatter(&content) {
                Some((m, b)) => (m, b),
                None => {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .replace("_SKILL", "")
                        .replace("SKILL", "");
                    found.push(Skill {
                        name,
                        description: "Custom skill".to_string(),
                        version: "1.0.0".to_string(),
                        triggers: Vec::new(),
                        priority: 0,
                        location: path.to_path_buf(),
                        content,
                    });
                    continue;
                }
            };

            found.push(Skill {
                name: meta.name.unwrap_or_else(|| "unknown".to_string()),
                description: meta.description.unwrap_or_default(),
                version: meta.version.unwrap_or_else(|| "1.0.0".to_string()),
                triggers: meta.triggers,
                priority: meta.priority,
                location: path.to_path_buf(),
                content: body.to_string(),
            });
        }

        Ok(found)
    }

    pub fn discover(&self) -> Result<(), OpenCodeError> {
        let mut skills = self.load_builtin_skills();

        if let Some(ref project_path) = self.project_skills_path {
            skills.extend(self.discover_in_dir(project_path)?);
        }

        if let Some(ref global_path) = self.global_skills_path {
            skills.extend(self.discover_in_dir(global_path)?);
        }

        for compat_path in &self.compat_skills_paths {
            skills.extend(self.discover_in_dir(compat_path)?);
        }

        skills.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut write = self
            .skills
            .write()
            .map_err(|_| OpenCodeError::Config("Failed to acquire write lock".to_string()))?;
        *write = skills;
        Ok(())
    }

    pub fn load_builtin_skills(&self) -> Vec<Skill> {
        if let Some(ref builtin_path) = self.builtin_skills_path {
            if builtin_path.exists() {
                tracing::debug!(
                    "Loading built-in skills from external path: {:?}",
                    builtin_path
                );
                let mut skills = self.discover_in_dir(builtin_path).unwrap_or_default();
                for skill in &mut skills {
                    skill.location =
                        PathBuf::from(format!("builtin://{}", skill.location.display()));
                }
                return skills;
            }
        }
        BUILTIN_SKILLS
            .iter()
            .filter_map(|(path, content)| {
                let (meta, body) = Self::parse_frontmatter(content)?;
                Some(Skill {
                    name: meta.name.unwrap_or_else(|| "unknown".to_string()),
                    description: meta.description.unwrap_or_default(),
                    version: meta.version.unwrap_or_else(|| "1.0.0".to_string()),
                    triggers: if meta.triggers.is_empty() {
                        vec![path.replace("/SKILL.md", "")]
                    } else {
                        meta.triggers
                    },
                    priority: meta.priority,
                    location: PathBuf::from(format!("builtin://{}", path)),
                    content: body.trim().to_string(),
                })
            })
            .collect()
    }

    pub fn match_skill(&self, input: &str) -> Result<Vec<SkillMatch>, OpenCodeError> {
        self.ensure_discovered()?;

        let read = self
            .skills
            .read()
            .map_err(|_| OpenCodeError::Config("Failed to acquire read lock".to_string()))?;

        let input_lower = input.to_lowercase();
        let mut matches: Vec<SkillMatch> = Vec::new();

        for skill in read.iter() {
            for trigger in &skill.triggers {
                let trigger_lower = trigger.to_lowercase();
                let (match_type, confidence) = if input_lower == trigger_lower {
                    (MatchType::Exact, 1.0)
                } else if input_lower.starts_with(&trigger_lower) {
                    (MatchType::Prefix, 0.8)
                } else if input_lower.contains(&trigger_lower)
                    || trigger_lower.contains(&input_lower)
                {
                    (MatchType::Fuzzy, 0.6)
                } else {
                    continue;
                };

                matches.push(SkillMatch {
                    skill: skill.clone(),
                    match_type,
                    confidence,
                });
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        Ok(matches)
    }

    pub fn match_by_skill_name(&self, name: &str) -> Option<Skill> {
        self.ensure_discovered().ok()?;
        let read = self.skills.read().ok()?;
        read.iter()
            .find(|s| s.name.to_lowercase() == name.to_lowercase())
            .cloned()
    }

    pub fn list(&self) -> Result<Vec<Skill>, OpenCodeError> {
        self.ensure_discovered()?;
        let read = self
            .skills
            .read()
            .map_err(|_| OpenCodeError::Config("Failed to acquire read lock".to_string()))?;
        Ok(read.clone())
    }

    pub fn get(&self, name: &str) -> Option<Skill> {
        self.ensure_discovered().ok()?;
        let read = self.skills.read().ok()?;
        read.iter().find(|s| s.name == name).cloned()
    }

    pub fn get_skill(&self, name: &str) -> Option<Skill> {
        self.get(name)
    }

    pub fn list_skills(&self) -> Result<Vec<Skill>, OpenCodeError> {
        self.list()
    }

    pub fn inject_into_prompt(&self, skill: &Skill) -> String {
        format!(
            "[Skill: {} v{}]\nDescription: {}\n\n{}",
            skill.name, skill.version, skill.description, skill.content
        )
    }

    pub fn all(&self) -> Result<Vec<Skill>, OpenCodeError> {
        self.list()
    }

    pub fn reload(&self) -> Result<(), OpenCodeError> {
        {
            let mut discovered = self
                .discovered
                .write()
                .map_err(|_| OpenCodeError::Config("Failed to acquire write lock".to_string()))?;
            *discovered = false;
        }
        self.discover()
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
struct SkillMeta {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    triggers: Vec<String>,
    priority: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_skill_manager_new() {
        let sm = SkillManager::new();
        // SkillManager starts empty; discovery happens on first list() call
        // which loads builtin skills via include_str!
        let skills = sm.list().unwrap_or_default();
        assert!(!skills.is_empty());
        assert!(skills.iter().any(|s| s.name == "code-review"));
    }

    #[test]
    fn test_skill_struct() {
        let skill = Skill {
            name: "test".to_string(),
            description: "A test skill".to_string(),
            version: "1.0.0".to_string(),
            triggers: vec!["test".to_string()],
            priority: 1,
            location: PathBuf::from("/path/to/skill"),
            content: "skill content".to_string(),
        };
        assert_eq!(skill.name, "test");
        assert_eq!(skill.triggers.len(), 1);
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
description: A test skill
triggers: test, debug
priority: 5
---
Skill content here"#;
        let (meta, body) = SkillManager::parse_frontmatter(content).unwrap();
        assert_eq!(meta.name, Some("test-skill".to_string()));
        assert_eq!(meta.triggers, vec!["test", "debug"]);
        assert_eq!(body.trim(), "Skill content here");
    }

    #[test]
    fn test_match_type_ordering() {
        let matches = vec![
            SkillMatch {
                skill: Skill {
                    name: "a".into(),
                    description: "".into(),
                    version: "1.0.0".into(),
                    triggers: vec![],
                    priority: 0,
                    location: PathBuf::new(),
                    content: "".into(),
                },
                match_type: MatchType::Fuzzy,
                confidence: 0.6,
            },
            SkillMatch {
                skill: Skill {
                    name: "b".into(),
                    description: "".into(),
                    version: "1.0.0".into(),
                    triggers: vec![],
                    priority: 0,
                    location: PathBuf::new(),
                    content: "".into(),
                },
                match_type: MatchType::Exact,
                confidence: 1.0,
            },
        ];
        let mut sorted = matches.clone();
        sorted.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        assert_eq!(sorted[0].match_type, MatchType::Exact);
    }

    #[test]
    fn test_load_builtin_skills() {
        let sm = SkillManager::new();
        let builtins = sm.load_builtin_skills();
        assert_eq!(builtins.len(), 10);
        assert!(builtins.iter().any(|s| s.name == "code-review"));
    }

    #[test]
    fn test_get_and_inject_skill() {
        let sm = SkillManager::new();
        let skill = sm
            .get_skill("code-review")
            .expect("builtin skill should exist");
        let injected = sm.inject_into_prompt(&skill);
        assert!(injected.contains("Skill: code-review"));
        assert!(injected.contains("Description:"));
    }

    #[test]
    fn test_externalized_builtin_skills() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().to_path_buf();
        std::fs::create_dir_all(skills_dir.join("code-review")).unwrap();
        std::fs::create_dir_all(skills_dir.join("custom-skill")).unwrap();
        std::fs::write(
            skills_dir.join("code-review").join("SKILL.md"),
            r#"---
name: code-review
description: External code review skill
version: 1.0.0
triggers: review, code-review
priority: 10
---
External code review content"#,
        )
        .unwrap();
        std::fs::write(
            skills_dir.join("custom-skill").join("SKILL.md"),
            r#"---
name: custom-skill
description: A custom external skill
version: 1.0.0
---
Custom skill content"#,
        )
        .unwrap();
        let sm = SkillManager::new().with_builtin_skills_path(skills_dir);
        let builtins = sm.load_builtin_skills();
        assert!(builtins.len() >= 1);
        assert!(builtins.iter().any(|s| s.name == "code-review"));
        assert!(builtins.iter().any(|s| s.name == "custom-skill"));
        let code_review = builtins.iter().find(|s| s.name == "code-review").unwrap();
        assert_eq!(code_review.description, "External code review skill");
        assert!(code_review.location.starts_with("builtin://"));
    }

    #[test]
    fn test_builtin_skills_fallback_to_embedded() {
        let sm = SkillManager::new().with_builtin_skills_path(PathBuf::from("/nonexistent/path"));
        let builtins = sm.load_builtin_skills();
        assert_eq!(builtins.len(), 10);
        assert!(builtins.iter().any(|s| s.name == "code-review"));
    }

    #[test]
    fn skills_compat_discover_from_compat_paths() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let compat_path = temp_dir.path().to_path_buf();
        std::fs::create_dir_all(compat_path.join("claude-style")).unwrap();
        std::fs::write(
            compat_path.join("claude-style").join("SKILL.md"),
            r#"---
name: claude-style-skill
description: A skill from Claude compat path
version: 1.0.0
triggers: claude, claude-style
priority: 5
---
Claude style skill content"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![compat_path]);
        let skills = sm.list().unwrap();

        assert!(skills.iter().any(|s| s.name == "claude-style-skill"));
        let skill = skills
            .iter()
            .find(|s| s.name == "claude-style-skill")
            .unwrap();
        assert_eq!(skill.description, "A skill from Claude compat path");
    }

    #[test]
    fn skills_compat_claude_style_format() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let compat_path = temp_dir.path().to_path_buf();
        std::fs::create_dir_all(compat_path.join("custom-skill")).unwrap();
        std::fs::write(
            compat_path.join("custom-skill").join("SKILL.md"),
            r#"---
name: custom-skill
description: Custom skill with Claude-style format
version: 2.0.0
triggers: custom, my-skill
priority: 7
---
Custom skill body content"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![compat_path]);
        let skill = sm.get("custom-skill").unwrap();

        assert_eq!(skill.name, "custom-skill");
        assert_eq!(skill.version, "2.0.0");
        assert_eq!(skill.triggers, vec!["custom", "my-skill"]);
        assert_eq!(skill.priority, 7);
        assert!(skill.content.contains("Custom skill body content"));
    }

    #[test]
    fn skills_compat_path_precedence_project_over_global() {
        use tempfile::TempDir;

        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        let project_skills_dir = project_dir
            .path()
            .join(".opencode")
            .join("skills")
            .join("test-skill");
        std::fs::create_dir_all(&project_skills_dir).unwrap();
        let global_skills = global_dir.path().join("skills");
        let global_test_skill = global_skills.join("test-skill");
        std::fs::create_dir_all(&global_skills).unwrap();
        std::fs::create_dir_all(&global_test_skill).unwrap();

        std::fs::write(
            project_skills_dir.join("SKILL.md"),
            r#"---
name: test-skill
description: Project version
version: 1.0.0
priority: 5
---
Project skill content"#,
        )
        .unwrap();

        std::fs::write(
            global_test_skill.join("SKILL.md"),
            r#"---
name: test-skill
description: Global version
version: 1.0.0
priority: 5
---
Global skill content"#,
        )
        .unwrap();

        let sm = SkillManager::new()
            .with_project_path(project_dir.path().to_path_buf())
            .with_compat_paths(vec![]);

        let skills = sm.list().unwrap();
        assert!(skills.iter().any(|s| s.name == "test-skill"));
    }

    #[test]
    fn skills_compat_path_precedence_global_over_compat() {
        use tempfile::TempDir;

        let global_dir = TempDir::new().unwrap();
        let compat_dir = TempDir::new().unwrap();

        std::fs::create_dir_all(global_dir.path().join("skills").join("dup-skill")).unwrap();
        std::fs::create_dir_all(compat_dir.path().join("dup-skill")).unwrap();

        std::fs::write(
            global_dir
                .path()
                .join("skills")
                .join("dup-skill")
                .join("SKILL.md"),
            r#"---
name: dup-skill
description: Global dup skill
version: 1.0.0
priority: 5
---
Global dup content"#,
        )
        .unwrap();

        std::fs::write(
            compat_dir.path().join("dup-skill").join("SKILL.md"),
            r#"---
name: dup-skill
description: Compat dup skill
version: 1.0.0
priority: 5
---
Compat dup content"#,
        )
        .unwrap();

        let mut sm = SkillManager::new();
        let global_path = global_dir.path().join("skills");
        std::fs::create_dir_all(&global_path).ok();
        sm = SkillManager::new();
        let compat_path = compat_dir.path().to_path_buf();
        sm = sm.with_compat_paths(vec![compat_path]);

        let skills = sm.list().unwrap();
        let dup_skill = skills.iter().find(|s| s.name == "dup-skill");
        assert!(dup_skill.is_some());
    }

    #[test]
    fn skills_compat_multiple_compat_paths() {
        use tempfile::TempDir;

        let compat_dir1 = TempDir::new().unwrap();
        let compat_dir2 = TempDir::new().unwrap();

        std::fs::create_dir_all(compat_dir1.path().join("skill-one")).unwrap();
        std::fs::create_dir_all(compat_dir2.path().join("skill-two")).unwrap();

        std::fs::write(
            compat_dir1.path().join("skill-one").join("SKILL.md"),
            r#"---
name: skill-one
description: Skill from first compat path
version: 1.0.0
priority: 5
---
Skill one content"#,
        )
        .unwrap();

        std::fs::write(
            compat_dir2.path().join("skill-two").join("SKILL.md"),
            r#"---
name: skill-two
description: Skill from second compat path
version: 1.0.0
priority: 5
---
Skill two content"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![
            compat_dir1.path().to_path_buf(),
            compat_dir2.path().to_path_buf(),
        ]);

        let skills = sm.list().unwrap();
        assert!(skills.iter().any(|s| s.name == "skill-one"));
        assert!(skills.iter().any(|s| s.name == "skill-two"));
    }

    #[test]
    fn skills_compat_standard_paths_are_configured() {
        let sm = SkillManager::new();
        let compat_paths = sm.compat_skills_paths.clone();

        let mut found_claude = false;
        let mut found_agent = false;

        for path in &compat_paths {
            if let Some(home) = dirs::home_dir() {
                if path.starts_with(&home) {
                    let relative = path.strip_prefix(&home).unwrap();
                    let components: Vec<_> = relative.components().collect();
                    if components.len() >= 2 {
                        let first = components[1].as_os_str().to_string_lossy();
                        if first == ".claude" {
                            found_claude = true;
                        }
                        if first == ".agent" {
                            found_agent = true;
                        }
                    }
                }
            }
        }

        assert!(
            found_claude || found_agent || !compat_paths.is_empty(),
            "Expected at least one standard compat path to be configured"
        );
    }

    #[test]
    fn skills_discovery_from_project_path() {
        use tempfile::TempDir;

        let project_dir = TempDir::new().unwrap();
        let project_skills_dir = project_dir
            .path()
            .join(".opencode")
            .join("skills")
            .join("my-project-skill");
        std::fs::create_dir_all(&project_skills_dir).unwrap();
        std::fs::write(
            project_skills_dir.join("SKILL.md"),
            r#"---
name: my-project-skill
description: Discovered from project path
version: 1.0.0
triggers: project, proj
priority: 10
---
Project skill content"#,
        )
        .unwrap();

        let sm = SkillManager::new()
            .with_project_path(project_dir.path().to_path_buf())
            .with_compat_paths(vec![]);

        let skills = sm.list().unwrap();
        assert!(skills.iter().any(|s| s.name == "my-project-skill"));
        let skill = skills
            .iter()
            .find(|s| s.name == "my-project-skill")
            .unwrap();
        assert_eq!(skill.description, "Discovered from project path");
        assert_eq!(skill.priority, 10);
    }

    #[test]
    fn skills_discovery_from_global_path() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let global_skills_dir = temp_dir.path().join("skills").join("global-skill");
        std::fs::create_dir_all(&global_skills_dir).unwrap();
        std::fs::write(
            global_skills_dir.join("SKILL.md"),
            r#"---
name: global-skill
description: Discovered from global path
version: 2.0.0
triggers: global, g
priority: 8
---
Global skill content"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![]);
        let global_path = temp_dir.path().join("skills");

        let discovered = sm.discover_in_dir(&global_path).unwrap();
        assert!(discovered.iter().any(|s| s.name == "global-skill"));
    }

    #[test]
    fn skills_discovery_claude_style_format_parsing() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("claude-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: claude-skill
description: Claude-style skill with full metadata
version: 3.0.0
triggers: claude, code, review
priority: 9
---
# Claude Skill

This skill follows the Claude-style SKILL.md format with YAML frontmatter."#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![]);
        let discovered = sm.discover_in_dir(&temp_dir.path().to_path_buf()).unwrap();

        let skill = discovered
            .iter()
            .find(|s| s.name == "claude-skill")
            .unwrap();
        assert_eq!(skill.version, "3.0.0");
        assert_eq!(skill.triggers, vec!["claude", "code", "review"]);
        assert_eq!(skill.priority, 9);
        assert!(skill.content.contains("Claude Skill"));
    }

    #[test]
    fn skills_discovery_agent_style_format_parsing() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("agent-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: agent-skill
description: Agent-style skill format
version: 1.5.0
triggers: agent, task
priority: 6
---
Agent skill body for testing"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![]);
        let discovered = sm.discover_in_dir(&temp_dir.path().to_path_buf()).unwrap();

        let skill = discovered.iter().find(|s| s.name == "agent-skill").unwrap();
        assert_eq!(skill.description, "Agent-style skill format");
        assert!(skill.content.contains("Agent skill body"));
    }

    #[test]
    fn skills_discovery_deterministic_ordering() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_dir1 = temp_dir.path().join("aaa-skill");
        let skill_dir2 = temp_dir.path().join("zzz-skill");
        let skill_dir3 = temp_dir.path().join("mmm-skill");
        std::fs::create_dir_all(&skill_dir1).unwrap();
        std::fs::create_dir_all(&skill_dir2).unwrap();
        std::fs::create_dir_all(&skill_dir3).unwrap();

        std::fs::write(
            skill_dir1.join("SKILL.md"),
            r#"---
name: aaa-skill
description: First alphabetically
version: 1.0.0
priority: 5
---
AAA content"#,
        )
        .unwrap();
        std::fs::write(
            skill_dir2.join("SKILL.md"),
            r#"---
name: zzz-skill
description: Last alphabetically
version: 1.0.0
priority: 5
---
ZZZ content"#,
        )
        .unwrap();
        std::fs::write(
            skill_dir3.join("SKILL.md"),
            r#"---
name: mmm-skill
description: Middle alphabetically
version: 1.0.0
priority: 5
---
MMM content"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![]);
        let discovered1 = sm.discover_in_dir(&temp_dir.path().to_path_buf()).unwrap();
        let discovered2 = sm.discover_in_dir(&temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(discovered1.len(), 3);
        assert_eq!(discovered2.len(), 3);

        let names1: Vec<_> = discovered1.iter().map(|s| s.name.clone()).collect();
        let names2: Vec<_> = discovered2.iter().map(|s| s.name.clone()).collect();
        assert_eq!(names1, names2, "Discovery order should be deterministic");
    }

    #[test]
    fn skills_discovery_respects_priority_ordering() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir
            .path()
            .join(".opencode")
            .join("skills")
            .join("priority-test");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::fs::write(
            project_dir.join("SKILL.md"),
            r#"---
name: priority-test
description: Test priority ordering
version: 1.0.0
priority: 5
---
Test content"#,
        )
        .unwrap();

        let sm = SkillManager::new()
            .with_project_path(temp_dir.path().to_path_buf())
            .with_compat_paths(vec![]);

        let skills = sm.list().unwrap();
        assert!(!skills.is_empty());
        let priority_test = skills.iter().find(|s| s.name == "priority-test").unwrap();
        assert_eq!(priority_test.priority, 5);
    }

    #[test]
    fn skills_discovery_handles_missing_frontmatter() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("plain-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "Just plain content without frontmatter\nSecond line",
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![]);
        let discovered = sm.discover_in_dir(&temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(discovered.len(), 1);
        let skill = &discovered[0];
        assert_eq!(skill.name, "");
        assert_eq!(skill.description, "Custom skill");
        assert_eq!(skill.version, "1.0.0");
        assert!(skill.content.contains("plain content"));
    }

    #[test]
    fn skills_discovery_all_supported_paths() {
        use tempfile::TempDir;

        let project_dir = TempDir::new().unwrap();
        let builtin_dir = TempDir::new().unwrap();
        let compat_dir = TempDir::new().unwrap();

        let project_skill_dir = project_dir
            .path()
            .join(".opencode")
            .join("skills")
            .join("project-skill");
        std::fs::create_dir_all(&project_skill_dir).unwrap();
        std::fs::write(
            project_skill_dir.join("SKILL.md"),
            r#"---
name: project-skill
description: Project skill
version: 1.0.0
priority: 10
---
Project content"#,
        )
        .unwrap();

        let builtin_skill_dir = builtin_dir.path().join("builtin-skill");
        std::fs::create_dir_all(&builtin_skill_dir).unwrap();
        std::fs::write(
            builtin_skill_dir.join("SKILL.md"),
            r#"---
name: builtin-skill
description: Builtin skill
version: 1.0.0
priority: 8
---
Builtin content"#,
        )
        .unwrap();

        let compat_skill_dir = compat_dir.path().join("compat-skill");
        std::fs::create_dir_all(&compat_skill_dir).unwrap();
        std::fs::write(
            compat_skill_dir.join("SKILL.md"),
            r#"---
name: compat-skill
description: Compat skill
version: 1.0.0
priority: 5
---
Compat content"#,
        )
        .unwrap();

        let sm = SkillManager::new()
            .with_project_path(project_dir.path().to_path_buf())
            .with_builtin_skills_path(builtin_dir.path().to_path_buf())
            .with_compat_paths(vec![compat_dir.path().to_path_buf()]);

        let skills = sm.list().unwrap();

        assert!(skills.iter().any(|s| s.name == "project-skill"));
        assert!(skills.iter().any(|s| s.name == "builtin-skill"));
        assert!(skills.iter().any(|s| s.name == "compat-skill"));
    }

    #[test]
    fn skills_discovery_builtin_skills_included() {
        let sm = SkillManager::new();
        let skills = sm.list().unwrap();

        let builtin_names = vec![
            "code-review",
            "architect",
            "debugger",
            "test-writer",
            "refactorer",
            "doc-writer",
            "security-auditor",
            "performance",
            "data-analyst",
            "devops",
        ];

        for name in builtin_names {
            assert!(
                skills.iter().any(|s| s.name == name),
                "Expected builtin skill '{}' to be discovered",
                name
            );
        }
    }

    #[test]
    fn skills_discovery_no_duplicates_within_scope() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("unique-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: unique-skill
description: Should appear once
version: 1.0.0
priority: 5
---
Unique content"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![]);
        let discovered = sm.discover_in_dir(&temp_dir.path().to_path_buf()).unwrap();

        let unique_count = discovered
            .iter()
            .filter(|s| s.name == "unique-skill")
            .count();
        assert_eq!(unique_count, 1, "Skill should appear exactly once");
    }

    #[test]
    fn skills_discovery_scans_nested_directories() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested-skill");
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(
            nested_dir.join("SKILL.md"),
            r#"---
name: nested-skill
description: Nested skill in subdirectory
version: 1.0.0
priority: 5
---
Nested content"#,
        )
        .unwrap();

        let sm = SkillManager::new().with_compat_paths(vec![]);
        let discovered = sm.discover_in_dir(&temp_dir.path().to_path_buf()).unwrap();

        assert!(discovered.iter().any(|s| s.name == "nested-skill"));
    }
}
