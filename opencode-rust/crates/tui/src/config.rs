use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILENAME: &str = "mycode.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub tui: Option<TuiConfig>,
    pub user: Option<UserConfig>,
    pub providers: Option<Vec<ProviderConfig>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TuiConfig {
    #[serde(default = "default_scroll_speed")]
    pub scroll_speed: u32,
    #[serde(default = "default_scroll_acceleration")]
    pub scroll_acceleration: f64,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub show_file_tree: bool,
    #[serde(default)]
    pub show_skills_panel: bool,
    #[serde(default = "default_diff_style")]
    pub diff_style: String,
    #[serde(default = "default_typewriter_speed")]
    pub typewriter_speed: u64,
    #[serde(default = "default_max_context_size")]
    pub max_context_size: usize,
    #[serde(default)]
    pub keybinds: Option<KeybindConfig>,
    #[serde(default)]
    pub custom_themes: Vec<CustomTheme>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomTheme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub error: String,
    pub warning: String,
    pub success: String,
    pub muted: String,
    pub border: String,
}

impl CustomTheme {
    pub fn validate(&self) -> Result<(), String> {
        Self::validate_color(&self.name, &self.background)?;
        Self::validate_color(&self.name, &self.foreground)?;
        Self::validate_color(&self.name, &self.primary)?;
        Self::validate_color(&self.name, &self.secondary)?;
        Self::validate_color(&self.name, &self.accent)?;
        Self::validate_color(&self.name, &self.error)?;
        Self::validate_color(&self.name, &self.warning)?;
        Self::validate_color(&self.name, &self.success)?;
        Self::validate_color(&self.name, &self.muted)?;
        Self::validate_color(&self.name, &self.border)?;
        Ok(())
    }

    fn validate_color(theme_name: &str, color: &str) -> Result<(), String> {
        let color = color.trim_start_matches('#');
        if color.len() != 6 {
            return Err(format!(
                "Theme '{}': Invalid color '{}' - must be 6 hex digits",
                theme_name, color
            ));
        }
        if u8::from_str_radix(&color[0..2], 16).is_err()
            || u8::from_str_radix(&color[2..4], 16).is_err()
            || u8::from_str_radix(&color[4..6], 16).is_err()
        {
            return Err(format!(
                "Theme '{}': Invalid hex color '{}'",
                theme_name, color
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeybindConfig {
    #[serde(default)]
    pub commands: Option<String>,
    #[serde(default)]
    pub timeline: Option<String>,
    #[serde(default)]
    pub new_session: Option<String>,
    #[serde(default)]
    pub toggle_files: Option<String>,
    #[serde(default)]
    pub settings: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
}

impl KeybindConfig {
    pub fn detect_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        let mut seen = std::collections::HashMap::new();

        if let Some(ref cmds) = self.commands {
            if !cmds.is_empty() {
                seen.insert(cmds.clone(), "commands");
            }
        }
        if let Some(ref tl) = self.timeline {
            if !tl.is_empty() {
                if let Some(prev) = seen.get(tl) {
                    conflicts.push(format!("timeline conflicts with {}", prev));
                } else {
                    seen.insert(tl.clone(), "timeline");
                }
            }
        }
        if let Some(ref ns) = self.new_session {
            if !ns.is_empty() {
                if let Some(prev) = seen.get(ns) {
                    conflicts.push(format!("new_session conflicts with {}", prev));
                } else {
                    seen.insert(ns.clone(), "new_session");
                }
            }
        }
        if let Some(ref tf) = self.toggle_files {
            if !tf.is_empty() {
                if let Some(prev) = seen.get(tf) {
                    conflicts.push(format!("toggle_files conflicts with {}", prev));
                } else {
                    seen.insert(tf.clone(), "toggle_files");
                }
            }
        }
        if let Some(ref st) = self.settings {
            if !st.is_empty() {
                if let Some(prev) = seen.get(st) {
                    conflicts.push(format!("settings conflicts with {}", prev));
                } else {
                    seen.insert(st.clone(), "settings");
                }
            }
        }
        if let Some(ref sr) = self.search {
            if !sr.is_empty() {
                if let Some(prev) = seen.get(sr) {
                    conflicts.push(format!("search conflicts with {}", prev));
                } else {
                    seen.insert(sr.clone(), "search");
                }
            }
        }

        conflicts
    }
}

fn default_diff_style() -> String {
    "auto".to_string()
}

fn default_typewriter_speed() -> u64 {
    20
}

fn default_max_context_size() -> usize {
    5000
}

fn default_scroll_speed() -> u32 {
    10
}
fn default_scroll_acceleration() -> f64 {
    0.5
}
fn default_theme() -> String {
    "dark".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConfig {
    pub username: Option<String>,
    #[serde(default)]
    pub remember_username: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn default_config() -> Self {
        Self {
            tui: Some(TuiConfig {
                scroll_speed: 10,
                scroll_acceleration: 0.5,
                theme: "dark".to_string(),
                show_file_tree: true,
                show_skills_panel: false,
                diff_style: "auto".to_string(),
                typewriter_speed: 20,
                max_context_size: 5000,
                keybinds: None,
                custom_themes: Vec::new(),
            }),
            user: None,
            providers: None,
        }
    }

    pub fn load_from_default_path() -> Result<Self, String> {
        let config_path = Self::default_config_path();
        if config_path.exists() {
            Self::load(&config_path)
        } else {
            Ok(Self::default_config())
        }
    }

    pub fn default_config_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| dirs::config_dir())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencode-rs");

        config_dir.join(CONFIG_FILENAME)
    }

    pub fn tui_config(&self) -> TuiConfig {
        self.tui.clone().unwrap_or_else(|| TuiConfig {
            scroll_speed: 10,
            scroll_acceleration: 0.5,
            theme: "dark".to_string(),
            show_file_tree: true,
            show_skills_panel: false,
            diff_style: "auto".to_string(),
            typewriter_speed: 20,
            max_context_size: 5000,
            keybinds: None,
            custom_themes: Vec::new(),
        })
    }

    pub fn merge_tui_config(&mut self, tui: TuiConfig) {
        if let Some(existing) = &mut self.tui {
            existing.scroll_speed = tui.scroll_speed;
            existing.scroll_acceleration = tui.scroll_acceleration;
            existing.theme = tui.theme;
            existing.show_file_tree = tui.show_file_tree;
            existing.show_skills_panel = tui.show_skills_panel;
            existing.diff_style = tui.diff_style;
            existing.typewriter_speed = tui.typewriter_speed;
            existing.max_context_size = tui.max_context_size;
            existing.keybinds = tui.keybinds;
            existing.custom_themes = tui.custom_themes;
        } else {
            self.tui = Some(tui);
        }
    }

    pub fn diff_style(&self) -> DiffStyle {
        let style = self.tui_config().diff_style;
        match style.as_str() {
            "side-by-side" => DiffStyle::SideBySide,
            "unified" => DiffStyle::Unified,
            _ => DiffStyle::Auto,
        }
    }

    pub fn typewriter_speed(&self) -> u64 {
        self.tui_config().typewriter_speed
    }

    pub fn max_context_size(&self) -> usize {
        self.tui_config().max_context_size
    }

    pub fn keybinds(&self) -> Option<KeybindConfig> {
        self.tui_config().keybinds.clone()
    }

    pub fn validate_keybinds(&self) -> Vec<String> {
        if let Some(ref keybinds) = self.keybinds() {
            keybinds.detect_conflicts()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffStyle {
    Auto,
    SideBySide,
    Unified,
}

impl Default for Config {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default_config();
        assert!(config.tui.is_some());
        assert!(config.user.is_none());
        assert!(config.providers.is_none());
    }

    #[test]
    fn test_config_tui_config() {
        let config = Config::default_config();
        let tui_config = config.tui_config();
        assert_eq!(tui_config.scroll_speed, 5);
        assert_eq!(tui_config.scroll_acceleration, 0.5);
        assert_eq!(tui_config.theme, "default");
    }

    #[test]
    fn test_config_diff_style_auto() {
        let config = Config::default_config();
        assert!(matches!(config.diff_style(), DiffStyle::Auto));
    }

    #[test]
    fn test_config_diff_style_side_by_side() {
        let mut config = Config::default_config();
        config.tui_config_mut().diff_style = "side-by-side".to_string();
        assert!(matches!(config.diff_style(), DiffStyle::SideBySide));
    }

    #[test]
    fn test_config_diff_style_unified() {
        let mut config = Config::default_config();
        config.tui_config_mut().diff_style = "unified".to_string();
        assert!(matches!(config.diff_style(), DiffStyle::Unified));
    }

    #[test]
    fn test_config_typewriter_speed() {
        let config = Config::default_config();
        assert_eq!(config.typewriter_speed(), 50);
    }

    #[test]
    fn test_config_max_context_size() {
        let config = Config::default_config();
        assert_eq!(config.max_context_size(), 100000);
    }

    #[test]
    fn test_config_keybinds_none() {
        let config = Config::default_config();
        assert!(config.keybinds().is_none());
    }

    #[test]
    fn test_config_validate_keybinds_empty() {
        let config = Config::default_config();
        assert!(config.validate_keybinds().is_empty());
    }

    #[test]
    fn test_custom_theme_validate_valid() {
        let theme = CustomTheme {
            name: "test".to_string(),
            background: "#000000".to_string(),
            foreground: "#ffffff".to_string(),
            primary: "#ff0000".to_string(),
            secondary: "#00ff00".to_string(),
            accent: "#0000ff".to_string(),
            error: "#ff0000".to_string(),
            warning: "#ffff00".to_string(),
            success: "#00ff00".to_string(),
            muted: "#888888".to_string(),
            border: "#cccccc".to_string(),
        };
        assert!(theme.validate().is_ok());
    }

    #[test]
    fn test_custom_theme_validate_invalid_color_length() {
        let theme = CustomTheme {
            name: "test".to_string(),
            background: "#000".to_string(),
            foreground: "#ffffff".to_string(),
            primary: "#ff0000".to_string(),
            secondary: "#00ff00".to_string(),
            accent: "#0000ff".to_string(),
            error: "#ff0000".to_string(),
            warning: "#ffff00".to_string(),
            success: "#00ff00".to_string(),
            muted: "#888888".to_string(),
            border: "#cccccc".to_string(),
        };
        let result = theme.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be 6 hex digits"));
    }

    #[test]
    fn test_custom_theme_validate_invalid_hex() {
        let theme = CustomTheme {
            name: "test".to_string(),
            background: "#ggffff".to_string(),
            foreground: "#ffffff".to_string(),
            primary: "#ff0000".to_string(),
            secondary: "#00ff00".to_string(),
            accent: "#0000ff".to_string(),
            error: "#ff0000".to_string(),
            warning: "#ffff00".to_string(),
            success: "#00ff00".to_string(),
            muted: "#888888".to_string(),
            border: "#cccccc".to_string(),
        };
        let result = theme.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex color"));
    }

    #[test]
    fn test_diff_style_default() {
        assert!(matches!(DiffStyle::default(), DiffStyle::Auto));
    }

    #[test]
    fn test_diff_style_equality() {
        assert_eq!(DiffStyle::Auto, DiffStyle::Auto);
        assert_eq!(DiffStyle::SideBySide, DiffStyle::SideBySide);
        assert_eq!(DiffStyle::Unified, DiffStyle::Unified);
        assert_ne!(DiffStyle::Auto, DiffStyle::SideBySide);
    }

    #[test]
    fn test_config_with_tui() {
        let mut config = Config::default_config();
        let tui = TuiConfig {
            scroll_speed: 10,
            scroll_acceleration: 1.0,
            theme: "dark".to_string(),
            show_file_tree: true,
            show_skills_panel: true,
            diff_style: "side-by-side".to_string(),
            typewriter_speed: 100,
            max_context_size: 50000,
            keybinds: None,
            custom_themes: Vec::new(),
        };
        config.merge_tui_config(tui);
        assert!(config.tui.is_some());
        assert_eq!(config.tui_config().scroll_speed, 10);
    }

    #[test]
    fn test_tui_config_creation() {
        let tui = TuiConfig {
            scroll_speed: 10,
            scroll_acceleration: 1.0,
            theme: "dark".to_string(),
            show_file_tree: true,
            show_skills_panel: true,
            diff_style: "side-by-side".to_string(),
            typewriter_speed: 100,
            max_context_size: 50000,
            keybinds: None,
            custom_themes: Vec::new(),
        };
        assert_eq!(tui.scroll_speed, 10);
        assert_eq!(tui.scroll_acceleration, 1.0);
        assert_eq!(tui.theme, "dark");
        assert!(tui.show_file_tree);
        assert!(tui.show_skills_panel);
    }
}
