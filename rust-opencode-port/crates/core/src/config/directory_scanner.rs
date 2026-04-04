use std::path::{Path, PathBuf};

/// Information about a discovered agent
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

/// Information about a discovered command
#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

/// Information about a discovered plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub path: PathBuf,
}

/// Information about a discovered skill
#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

/// Information about a discovered tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

/// Information about a discovered theme
#[derive(Debug, Clone)]
pub struct ThemeInfo {
    pub name: String,
    pub path: PathBuf,
}

/// Result of scanning .opencode directory
#[derive(Debug, Clone, Default)]
pub struct OpencodeDirectoryScan {
    pub agents: Vec<AgentInfo>,
    pub commands: Vec<CommandInfo>,
    pub plugins: Vec<PluginInfo>,
    pub skills: Vec<SkillInfo>,
    pub tools: Vec<ToolInfo>,
    pub themes: Vec<ThemeInfo>,
}

/// Directory scanner for .opencode subdirectories
pub struct DirectoryScanner {}

impl DirectoryScanner {
    pub fn new() -> Self {
        DirectoryScanner {}
    }

    /// Scan agents from .opencode/agents/ directory
    /// Each agent is in a subdirectory with AGENT.md file
    pub fn scan_agents(&self, base_path: &Path) -> Vec<AgentInfo> {
        let agents_dir = base_path.join("agents");
        if !agents_dir.exists() {
            return Vec::new();
        }

        let mut agents = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let agent_md = path.join("AGENT.md");
                    if agent_md.exists() {
                        if let Ok(content) = std::fs::read_to_string(&agent_md) {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unnamed")
                                .to_string();
                            agents.push(AgentInfo {
                                name,
                                path: agent_md,
                                content,
                            });
                        }
                    }
                }
            }
        }
        agents
    }

    /// Scan commands from .opencode/commands/ directory
    /// Each command is a .md file named <name>.md
    pub fn scan_commands(&self, base_path: &Path) -> Vec<CommandInfo> {
        let commands_dir = base_path.join("commands");
        if !commands_dir.exists() {
            return Vec::new();
        }

        let mut commands = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&commands_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let name = path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unnamed")
                            .to_string();
                        commands.push(CommandInfo {
                            name,
                            path,
                            content,
                        });
                    }
                }
            }
        }
        commands
    }

    /// Scan plugins from .opencode/plugins/ directory
    /// Each plugin is a .wasm file named <name>.wasm
    pub fn scan_plugins(&self, base_path: &Path) -> Vec<PluginInfo> {
        let plugins_dir = base_path.join("plugins");
        if !plugins_dir.exists() {
            return Vec::new();
        }

        let mut plugins = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    let name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unnamed")
                        .to_string();
                    plugins.push(PluginInfo { name, path });
                }
            }
        }
        plugins
    }

    /// Scan skills from .opencode/skills/ directory
    /// Each skill is in a subdirectory with SKILL.md file
    pub fn scan_skills(&self, base_path: &Path) -> Vec<SkillInfo> {
        let skills_dir = base_path.join("skills");
        if !skills_dir.exists() {
            return Vec::new();
        }

        let mut skills = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Ok(content) = std::fs::read_to_string(&skill_md) {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unnamed")
                                .to_string();
                            skills.push(SkillInfo {
                                name,
                                path: skill_md,
                                content,
                            });
                        }
                    }
                }
            }
        }
        skills
    }

    /// Scan tools from .opencode/tools/ directory
    /// Each tool is in a subdirectory with TOOL.md file
    pub fn scan_tools(&self, base_path: &Path) -> Vec<ToolInfo> {
        let tools_dir = base_path.join("tools");
        if !tools_dir.exists() {
            return Vec::new();
        }

        let mut tools = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&tools_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let tool_md = path.join("TOOL.md");
                    if tool_md.exists() {
                        if let Ok(content) = std::fs::read_to_string(&tool_md) {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unnamed")
                                .to_string();
                            tools.push(ToolInfo {
                                name,
                                path: tool_md,
                                content,
                            });
                        }
                    }
                }
            }
        }
        tools
    }

    /// Scan themes from .opencode/themes/ directory
    /// Each theme is a .json file named <name>.json
    pub fn scan_themes(&self, base_path: &Path) -> Vec<ThemeInfo> {
        let themes_dir = base_path.join("themes");
        if !themes_dir.exists() {
            return Vec::new();
        }

        let mut themes = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unnamed")
                        .to_string();
                    themes.push(ThemeInfo { name, path });
                }
            }
        }
        themes
    }

    /// Scan all .opencode subdirectories
    pub fn scan_all(&self, base_path: &Path) -> OpencodeDirectoryScan {
        OpencodeDirectoryScan {
            agents: self.scan_agents(base_path),
            commands: self.scan_commands(base_path),
            plugins: self.scan_plugins(base_path),
            skills: self.scan_skills(base_path),
            tools: self.scan_tools(base_path),
            themes: self.scan_themes(base_path),
        }
    }
}

impl Default for DirectoryScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Load .opencode directory from both project and global locations
/// Project .opencode/ takes precedence over global .opencode/
pub fn load_opencode_directory() -> OpencodeDirectoryScan {
    let scanner = DirectoryScanner::new();
    let mut result = OpencodeDirectoryScan::default();

    // First, scan global .opencode/ (lower priority)
    if let Some(dirs) = directories::ProjectDirs::from("com", "opencode", "rs") {
        let global_opencode = dirs.config_dir().join(".opencode");
        if global_opencode.exists() {
            let global_scan = scanner.scan_all(&global_opencode);
            // Prepend global items (project items will take precedence when we merge)
            result.agents.extend(global_scan.agents);
            result.commands.extend(global_scan.commands);
            result.plugins.extend(global_scan.plugins);
            result.skills.extend(global_scan.skills);
            result.tools.extend(global_scan.tools);
            result.themes.extend(global_scan.themes);
        }
    }

    // Then, scan project .opencode/ (higher priority - will override)
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let project_opencode = ancestor.join(".opencode");
            if project_opencode.exists() {
                let project_scan = scanner.scan_all(&project_opencode);

                // Merge project items (they override global ones with same name)
                merge_scan_results(&mut result, project_scan);
                break; // Only use the closest .opencode directory
            }
        }
    }

    result
}

/// Merge scan results, with newer items overriding older ones with the same name
fn merge_scan_results(base: &mut OpencodeDirectoryScan, override_scan: OpencodeDirectoryScan) {
    // Agents: override by name
    for override_agent in override_scan.agents {
        if let Some(existing) = base
            .agents
            .iter_mut()
            .find(|a| a.name == override_agent.name)
        {
            *existing = override_agent;
        } else {
            base.agents.push(override_agent);
        }
    }

    // Commands: override by name
    for override_cmd in override_scan.commands {
        if let Some(existing) = base
            .commands
            .iter_mut()
            .find(|c| c.name == override_cmd.name)
        {
            *existing = override_cmd;
        } else {
            base.commands.push(override_cmd);
        }
    }

    // Plugins: override by name
    for override_plugin in override_scan.plugins {
        if let Some(existing) = base
            .plugins
            .iter_mut()
            .find(|p| p.name == override_plugin.name)
        {
            *existing = override_plugin;
        } else {
            base.plugins.push(override_plugin);
        }
    }

    // Skills: override by name
    for override_skill in override_scan.skills {
        if let Some(existing) = base
            .skills
            .iter_mut()
            .find(|s| s.name == override_skill.name)
        {
            *existing = override_skill;
        } else {
            base.skills.push(override_skill);
        }
    }

    // Tools: override by name
    for override_tool in override_scan.tools {
        if let Some(existing) = base.tools.iter_mut().find(|t| t.name == override_tool.name) {
            *existing = override_tool;
        } else {
            base.tools.push(override_tool);
        }
    }

    // Themes: override by name
    for override_theme in override_scan.themes {
        if let Some(existing) = base
            .themes
            .iter_mut()
            .find(|t| t.name == override_theme.name)
        {
            *existing = override_theme;
        } else {
            base.themes.push(override_theme);
        }
    }
}

#[allow(dead_code)]
pub fn resolve_theme_path(name: &str, scan_dirs: &[String]) -> Option<PathBuf> {
    for dir in scan_dirs {
        let themes_dir = Path::new(dir);
        let theme_path = themes_dir.join(format!("{}.json", name));
        if theme_path.exists() {
            return Some(theme_path);
        }
        let alt_path = themes_dir.join(name);
        if alt_path.exists() {
            return Some(alt_path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_agents() {
        let temp = TempDir::new().unwrap();
        let opencode_dir = temp
            .path()
            .join(".opencode")
            .join("agents")
            .join("test-agent");
        fs::create_dir_all(&opencode_dir).unwrap();
        fs::write(opencode_dir.join("AGENT.md"), "# Test Agent").unwrap();

        let scanner = DirectoryScanner::new();
        let agents = scanner.scan_agents(&temp.path().join(".opencode"));

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "test-agent");
        assert_eq!(agents[0].content, "# Test Agent");
    }

    #[test]
    fn test_scan_commands() {
        let temp = TempDir::new().unwrap();
        let commands_dir = temp.path().join(".opencode").join("commands");
        fs::create_dir_all(&commands_dir).unwrap();
        fs::write(commands_dir.join("test-cmd.md"), "# Test Command").unwrap();

        let scanner = DirectoryScanner::new();
        let commands = scanner.scan_commands(&temp.path().join(".opencode"));

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "test-cmd");
        assert_eq!(commands[0].content, "# Test Command");
    }

    #[test]
    fn test_scan_plugins() {
        let temp = TempDir::new().unwrap();
        let plugins_dir = temp.path().join(".opencode").join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(plugins_dir.join("test-plugin.wasm"), b"wasm-content").unwrap();

        let scanner = DirectoryScanner::new();
        let plugins = scanner.scan_plugins(&temp.path().join(".opencode"));

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "test-plugin");
    }

    #[test]
    fn test_scan_themes() {
        let temp = TempDir::new().unwrap();
        let themes_dir = temp.path().join(".opencode").join("themes");
        fs::create_dir_all(&themes_dir).unwrap();
        fs::write(themes_dir.join("test-theme.json"), r#"{"name": "test"}"#).unwrap();

        let scanner = DirectoryScanner::new();
        let themes = scanner.scan_themes(&temp.path().join(".opencode"));

        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "test-theme");
    }

    #[test]
    fn test_scan_all() {
        let temp = TempDir::new().unwrap();
        let opencode_dir = temp.path().join(".opencode");

        // Create agents
        let agent_dir = opencode_dir.join("agents").join("my-agent");
        fs::create_dir_all(&agent_dir).unwrap();
        fs::write(agent_dir.join("AGENT.md"), "Agent content").unwrap();

        // Create commands
        let commands_dir = opencode_dir.join("commands");
        fs::create_dir_all(&commands_dir).unwrap();
        fs::write(commands_dir.join("my-cmd.md"), "Command content").unwrap();

        // Create themes
        let themes_dir = opencode_dir.join("themes");
        fs::create_dir_all(&themes_dir).unwrap();
        fs::write(themes_dir.join("my-theme.json"), r#"{"name": "mytheme"}"#).unwrap();

        let scanner = DirectoryScanner::new();
        let scan = scanner.scan_all(&opencode_dir);

        assert_eq!(scan.agents.len(), 1);
        assert_eq!(scan.commands.len(), 1);
        assert_eq!(scan.themes.len(), 1);
    }
}
