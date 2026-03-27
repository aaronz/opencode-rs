use crate::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub location: PathBuf,
    pub content: String,
}

pub struct SkillManager {
    skills: Vec<Skill>,
}

impl SkillManager {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn discover(&mut self) -> Result<(), OpenCodeError> {
        self.skills.clear();

        let config_dir = dirs::config_dir()
            .ok_or_else(|| OpenCodeError::Config("Cannot find config directory".to_string()))?
            .join("opencode-rs")
            .join("skills");

        if !config_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(&config_dir).max_depth(2) {
            let entry = entry.map_err(|e: walkdir::Error| OpenCodeError::Config(e.to_string()))?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "md" && entry.file_name().to_string_lossy().contains("SKILL") {
                        let content =
                            std::fs::read_to_string(entry.path()).map_err(OpenCodeError::Io)?;

                        let name = entry
                            .path()
                            .file_stem()
                            .map(|s: &std::ffi::OsStr| {
                                s.to_string_lossy()
                                    .replace("_SKILL", "")
                                    .replace("SKILL", "")
                            })
                            .unwrap_or_default();

                        self.skills.push(Skill {
                            name,
                            description: "Custom skill".to_string(),
                            location: entry.path().to_path_buf(),
                            content,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub fn list(&self) -> &[Skill] {
        &self.skills
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name == name)
    }

    pub fn all(&self) -> Vec<Skill> {
        self.skills.clone()
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_skill_manager_new() {
        let sm = SkillManager::new();
        assert!(sm.list().is_empty());
    }

    #[test]
    fn test_skill_manager_list() {
        let sm = SkillManager::new();
        assert_eq!(sm.list().len(), 0);
    }

    #[test]
    fn test_skill_manager_get_not_found() {
        let sm = SkillManager::new();
        assert!(sm.get("nonexistent").is_none());
    }

    #[test]
    fn test_skill_manager_all() {
        let sm = SkillManager::new();
        assert!(sm.all().is_empty());
    }

    #[test]
    fn test_skill_struct() {
        let skill = Skill {
            name: "test".to_string(),
            description: "A test skill".to_string(),
            location: PathBuf::from("/path/to/skill"),
            content: "skill content".to_string(),
        };
        assert_eq!(skill.name, "test");
        assert_eq!(skill.description, "A test skill");
    }
}
