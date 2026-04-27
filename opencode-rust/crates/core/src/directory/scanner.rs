use crate::OpenCodeError;
use crate::skill::Skill;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::types::{AgentDefinition, CommandDefinition, ModeDefinition, PluginInfo, ThemeInfo};

pub struct DirectoryScanner {
    root_path: Option<PathBuf>,
    global_path: Option<PathBuf>,
}

impl DirectoryScanner {
    pub fn new() -> Self {
        let global_path = directories::ProjectDirs::from("com", "opencode", "rs")
            .map(|dirs| dirs.config_dir().to_path_buf());

        Self {
            root_path: None,
            global_path,
        }
    }

    pub fn with_root_path(mut self, path: PathBuf) -> Self {
        self.root_path = Some(path);
        self
    }

    fn find_opencode_dir(start_dir: &Path) -> Option<PathBuf> {
        for ancestor in start_dir.ancestors() {
            let opencode_dir = ancestor.join(".opencode-rs");
            if opencode_dir.exists() && opencode_dir.is_dir() {
                return Some(opencode_dir);
            }
        }
        None
    }

    pub fn discover_agents(&self) -> Result<Vec<AgentDefinition>, OpenCodeError> {
        let mut agents = Vec::new();

        if let Some(global) = &self.global_path {
            let agents_dir = global.join("agents");
            if agents_dir.exists() {
                agents.extend(self.load_agents_from_dir(&agents_dir)?);
            }
        }

        if let Some(root) = &self.root_path {
            if let Some(opencode_dir) = Self::find_opencode_dir(root) {
                let project_agents = opencode_dir.join("agents");
                if project_agents.exists() {
                    let mut project_agents_vec = self.load_agents_from_dir(&project_agents)?;
                    project_agents_vec.reverse();
                    for (i, agent) in project_agents_vec.into_iter().enumerate() {
                        if i < agents.len() {
                            agents[i] = agent;
                        } else {
                            agents.push(agent);
                        }
                    }
                }
            }
        }

        Ok(agents)
    }

    fn load_agents_from_dir(&self, dir: &Path) -> Result<Vec<AgentDefinition>, OpenCodeError> {
        let mut agents = Vec::new();
        if !dir.exists() {
            return Ok(agents);
        }

        for entry in WalkDir::new(dir).max_depth(2) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" || ext == "yaml" || ext == "yml" || ext == "json" {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if let Some(agent) = self.parse_agent_file(path, &content) {
                                agents.push(agent);
                            }
                        }
                    }
                }
            }
        }

        Ok(agents)
    }

    fn parse_agent_file(&self, path: &Path, content: &str) -> Option<AgentDefinition> {
        if let Some(after_first_dash) = content.strip_prefix("---") {
            if let Some(end) = after_first_dash.find("---") {
                let yaml_part = &after_first_dash[..end];
                let body = &after_first_dash[end + 3..];

                let mut name = path.file_stem()?.to_str()?.to_string();
                let mut description = None;
                let mut model = None;
                let mut tools = None;
                let options = None;

                for line in yaml_part.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim();
                        match key {
                            "name" => name = value.to_string(),
                            "description" => description = Some(value.to_string()),
                            "model" => model = Some(value.to_string()),
                            "tools" => {
                                tools =
                                    Some(value.split(',').map(|s| s.trim().to_string()).collect());
                            }
                            _ => {}
                        }
                    }
                }

                return Some(AgentDefinition {
                    name,
                    description,
                    prompt: body.trim().to_string(),
                    model,
                    tools,
                    options,
                });
            }
        }

        if let Ok(agent) = serde_json::from_str::<AgentDefinition>(content) {
            return Some(agent);
        }

        None
    }

    pub fn discover_commands(&self) -> Result<Vec<CommandDefinition>, OpenCodeError> {
        let mut commands = Vec::new();

        if let Some(global) = &self.global_path {
            let commands_dir = global.join("commands");
            if commands_dir.exists() {
                commands.extend(self.load_commands_from_dir(&commands_dir)?);
            }
        }

        if let Some(root) = &self.root_path {
            if let Some(opencode_dir) = Self::find_opencode_dir(root) {
                let project_commands = opencode_dir.join("commands");
                if project_commands.exists() {
                    let mut project_commands_vec =
                        self.load_commands_from_dir(&project_commands)?;
                    project_commands_vec.reverse();
                    for (i, cmd) in project_commands_vec.into_iter().enumerate() {
                        if i < commands.len() {
                            commands[i] = cmd;
                        } else {
                            commands.push(cmd);
                        }
                    }
                }
            }
        }

        Ok(commands)
    }

    fn load_commands_from_dir(&self, dir: &Path) -> Result<Vec<CommandDefinition>, OpenCodeError> {
        let mut commands = Vec::new();
        if !dir.exists() {
            return Ok(commands);
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "md" || ext == "json" {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Some(cmd) = self.parse_command_file(&path, &content) {
                                    commands.push(cmd);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(commands)
    }

    fn parse_command_file(&self, _path: &Path, content: &str) -> Option<CommandDefinition> {
        if let Some(after_first_dash) = content.strip_prefix("---") {
            if let Some(end) = after_first_dash.find("---") {
                let yaml_part = &after_first_dash[..end];
                let body = &after_first_dash[end + 3..];

                let mut description = None;
                let mut agent = None;
                let mut model = None;
                let template = body.trim().to_string();

                for line in yaml_part.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim();
                        match key {
                            "description" => description = Some(value.to_string()),
                            "agent" => agent = Some(value.to_string()),
                            "model" => model = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }

                return Some(CommandDefinition {
                    description,
                    agent,
                    model,
                    template,
                });
            }
        }

        if let Ok(cmd) = serde_json::from_str::<CommandDefinition>(content) {
            return Some(cmd);
        }

        None
    }

    pub fn discover_modes(&self) -> Result<Vec<ModeDefinition>, OpenCodeError> {
        let mut modes = Vec::new();

        if let Some(global) = &self.global_path {
            let modes_dir = global.join("modes");
            if modes_dir.exists() {
                modes.extend(self.load_modes_from_dir(&modes_dir)?);
            }
        }

        if let Some(root) = &self.root_path {
            if let Some(opencode_dir) = Self::find_opencode_dir(root) {
                let project_modes = opencode_dir.join("modes");
                if project_modes.exists() {
                    let mut project_modes_vec = self.load_modes_from_dir(&project_modes)?;
                    project_modes_vec.reverse();
                    for (i, mode) in project_modes_vec.into_iter().enumerate() {
                        if i < modes.len() {
                            modes[i] = mode;
                        } else {
                            modes.push(mode);
                        }
                    }
                }
            }
        }

        Ok(modes)
    }

    fn load_modes_from_dir(&self, dir: &Path) -> Result<Vec<ModeDefinition>, OpenCodeError> {
        let mut modes = Vec::new();
        if !dir.exists() {
            return Ok(modes);
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "md" || ext == "json" {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Some(mode) = self.parse_mode_file(&path, &content) {
                                    modes.push(mode);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(modes)
    }

    fn parse_mode_file(&self, _path: &Path, content: &str) -> Option<ModeDefinition> {
        if let Some(after_first_dash) = content.strip_prefix("---") {
            if let Some(end) = after_first_dash.find("---") {
                let yaml_part = &after_first_dash[..end];
                let body = &after_first_dash[end + 3..];

                let mut description = None;
                let mut agent = None;
                let mut prompt = None;

                for line in yaml_part.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim();
                        match key {
                            "description" => description = Some(value.to_string()),
                            "agent" => agent = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }

                if !body.trim().is_empty() {
                    prompt = Some(body.trim().to_string());
                }

                return Some(ModeDefinition {
                    description,
                    agent,
                    prompt,
                });
            }
        }

        if let Ok(mode) = serde_json::from_str::<ModeDefinition>(content) {
            return Some(mode);
        }

        None
    }

    pub fn discover_plugins(&self) -> Result<Vec<PluginInfo>, OpenCodeError> {
        let mut plugins = Vec::new();

        if let Some(global) = &self.global_path {
            let plugins_dir = global.join("plugins");
            if plugins_dir.exists() {
                plugins.extend(self.find_plugins_in_dir(&plugins_dir)?);
            }
        }

        if let Some(root) = &self.root_path {
            if let Some(opencode_dir) = Self::find_opencode_dir(root) {
                let project_plugins = opencode_dir.join("plugins");
                if project_plugins.exists() {
                    plugins.extend(self.find_plugins_in_dir(&project_plugins)?);
                }
            }
        }

        Ok(plugins)
    }

    fn find_plugins_in_dir(&self, dir: &Path) -> Result<Vec<PluginInfo>, OpenCodeError> {
        let mut plugins = Vec::new();
        if !dir.exists() {
            return Ok(plugins);
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "wasm" || ext == "json" {
                            let name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            let capabilities = if ext == "json" {
                                std::fs::read_to_string(&path)
                                    .ok()
                                    .and_then(|c| {
                                        serde_json::from_str::<serde_json::Value>(&c).ok()
                                    })
                                    .and_then(|v| v.get("capabilities").cloned())
                                    .and_then(|v| serde_json::from_value(v).ok())
                            } else {
                                None
                            };

                            plugins.push(PluginInfo {
                                name,
                                path,
                                capabilities,
                            });
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    pub fn discover_themes(&self) -> Result<Vec<ThemeInfo>, OpenCodeError> {
        let mut themes = Vec::new();

        if let Some(global) = &self.global_path {
            let themes_dir = global.join("themes");
            if themes_dir.exists() {
                themes.extend(self.find_themes_in_dir(&themes_dir)?);
            }
        }

        if let Some(root) = &self.root_path {
            if let Some(opencode_dir) = Self::find_opencode_dir(root) {
                let project_themes = opencode_dir.join("themes");
                if project_themes.exists() {
                    themes.extend(self.find_themes_in_dir(&project_themes)?);
                }
            }
        }

        Ok(themes)
    }

    fn find_themes_in_dir(&self, dir: &Path) -> Result<Vec<ThemeInfo>, OpenCodeError> {
        let mut themes = Vec::new();
        if !dir.exists() {
            return Ok(themes);
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(theme_json) = serde_json::from_str::<serde_json::Value>(&content)
                        {
                            let name = theme_json
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or_else(|| {
                                    path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("unknown")
                                })
                                .to_string();

                            let description = theme_json
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            let author = theme_json
                                .get("author")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            themes.push(ThemeInfo {
                                name,
                                path,
                                description,
                                author,
                            });
                        }
                    }
                }
            }
        }

        Ok(themes)
    }

    pub fn discover_skills(&self) -> Result<Vec<Skill>, OpenCodeError> {
        let mut skills = Vec::new();

        if let Some(global) = &self.global_path {
            let skills_dir = global.join("skills");
            if skills_dir.exists() {
                if let Ok(global_skills) =
                    crate::skill::SkillManager::new().discover_in_dir(&skills_dir)
                {
                    skills.extend(global_skills);
                }
            }
        }

        if let Some(root) = &self.root_path {
            if let Some(opencode_dir) = Self::find_opencode_dir(root) {
                let project_skills_dir = opencode_dir.join("skills");
                if project_skills_dir.exists() {
                    if let Ok(project_skills) =
                        crate::skill::SkillManager::new().discover_in_dir(&project_skills_dir)
                    {
                        let project_names: std::collections::HashSet<String> = project_skills
                            .iter()
                            .map(|s: &Skill| s.name.clone())
                            .collect();

                        skills.retain(|s: &Skill| !project_names.contains(&s.name));
                        skills.extend(project_skills);
                    }
                }
            }
        }

        Ok(skills)
    }
}

impl Default for DirectoryScanner {
    fn default() -> Self {
        Self::new()
    }
}