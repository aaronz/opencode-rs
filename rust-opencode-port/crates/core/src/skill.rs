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

        Self {
            skills: RwLock::new(Vec::new()),
            global_skills_path: global_path,
            project_skills_path: None,
            discovered: RwLock::new(false),
        }
    }

    pub fn with_project_path(mut self, project_path: PathBuf) -> Self {
        self.project_skills_path = Some(project_path.join(".opencode").join("skills"));
        self
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

        if let Some(ref global_path) = self.global_skills_path {
            skills.extend(self.discover_in_dir(global_path)?);
        }

        if let Some(ref project_path) = self.project_skills_path {
            skills.extend(self.discover_in_dir(project_path)?);
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
}
