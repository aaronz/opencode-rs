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
        })
    }

    pub fn merge_tui_config(&mut self, tui: TuiConfig) {
        if let Some(existing) = &mut self.tui {
            existing.scroll_speed = tui.scroll_speed;
            existing.scroll_acceleration = tui.scroll_acceleration;
            existing.theme = tui.theme;
            existing.show_file_tree = tui.show_file_tree;
            existing.show_skills_panel = tui.show_skills_panel;
        } else {
            self.tui = Some(tui);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_config()
    }
}
