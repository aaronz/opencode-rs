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
    #[serde(default)]
    pub keybinds: Option<KeybindConfig>,
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
                keybinds: None,
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
            keybinds: None,
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
            existing.keybinds = tui.keybinds;
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
