use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            colors: ThemeColors {
                background: "#1e1e2e".to_string(),
                foreground: "#cdd6f4".to_string(),
                primary: "#89b4fa".to_string(),
                secondary: "#cba6f7".to_string(),
                accent: "#f38ba8".to_string(),
                error: "#f38ba8".to_string(),
                warning: "#fab387".to_string(),
                success: "#a6e3a1".to_string(),
                muted: "#6c7086".to_string(),
                border: "#313244".to_string(),
            },
        }
    }
}

impl Theme {
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read theme file '{}': {}", path, e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse theme file '{}': {}", path, e))
    }

    pub fn load_from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Failed to parse theme JSON: {}", e))
    }

    pub fn primary_color(&self) -> Color {
        parse_hex_color(&self.colors.primary).unwrap_or(Color::Blue)
    }

    pub fn secondary_color(&self) -> Color {
        parse_hex_color(&self.colors.secondary).unwrap_or(Color::Magenta)
    }

    pub fn error_color(&self) -> Color {
        parse_hex_color(&self.colors.error).unwrap_or(Color::Red)
    }

    pub fn success_color(&self) -> Color {
        parse_hex_color(&self.colors.success).unwrap_or(Color::Green)
    }

    pub fn warning_color(&self) -> Color {
        parse_hex_color(&self.colors.warning).unwrap_or(Color::Yellow)
    }

    pub fn muted_color(&self) -> Color {
        parse_hex_color(&self.colors.muted).unwrap_or(Color::DarkGray)
    }

    pub fn border_color(&self) -> Color {
        parse_hex_color(&self.colors.border).unwrap_or(Color::DarkGray)
    }

    pub fn foreground_color(&self) -> Color {
        parse_hex_color(&self.colors.foreground).unwrap_or(Color::White)
    }

    pub fn accent_color(&self) -> Color {
        parse_hex_color(&self.colors.accent).unwrap_or(Color::Yellow)
    }

    pub fn catppuccin() -> Self {
        Self {
            name: "catppuccin".to_string(),
            colors: ThemeColors {
                background: "#1e1e2e".to_string(),
                foreground: "#cdd6f4".to_string(),
                primary: "#89b4fa".to_string(),
                secondary: "#cba6f7".to_string(),
                accent: "#f38ba8".to_string(),
                error: "#f38ba8".to_string(),
                warning: "#fab387".to_string(),
                success: "#a6e3a1".to_string(),
                muted: "#6c7086".to_string(),
                border: "#313244".to_string(),
            },
        }
    }

    pub fn tokyonight() -> Self {
        Self {
            name: "tokyonight".to_string(),
            colors: ThemeColors {
                background: "#1a1b26".to_string(),
                foreground: "#c0caf5".to_string(),
                primary: "#7aa2f7".to_string(),
                secondary: "#bb9af7".to_string(),
                accent: "#f7768e".to_string(),
                error: "#f7768e".to_string(),
                warning: "#e0af68".to_string(),
                success: "#9ece6a".to_string(),
                muted: "#565f89".to_string(),
                border: "#292e42".to_string(),
            },
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            colors: ThemeColors {
                background: "#2e3440".to_string(),
                foreground: "#eceff4".to_string(),
                primary: "#88c0d0".to_string(),
                secondary: "#b48ead".to_string(),
                accent: "#bf616a".to_string(),
                error: "#bf616a".to_string(),
                warning: "#ebcb8b".to_string(),
                success: "#a3be8c".to_string(),
                muted: "#4c566a".to_string(),
                border: "#3b4252".to_string(),
            },
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: "gruvbox".to_string(),
            colors: ThemeColors {
                background: "#282828".to_string(),
                foreground: "#ebdbb2".to_string(),
                primary: "#83a598".to_string(),
                secondary: "#d3869b".to_string(),
                accent: "#fe8019".to_string(),
                error: "#fb4934".to_string(),
                warning: "#fabd2f".to_string(),
                success: "#b8bb26".to_string(),
                muted: "#928374".to_string(),
                border: "#504945".to_string(),
            },
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark".to_string(),
            colors: ThemeColors {
                background: "#002b36".to_string(),
                foreground: "#839496".to_string(),
                primary: "#268bd2".to_string(),
                secondary: "#6c71c4".to_string(),
                accent: "#cb4b16".to_string(),
                error: "#dc322f".to_string(),
                warning: "#b58900".to_string(),
                success: "#859900".to_string(),
                muted: "#586e75".to_string(),
                border: "#073642".to_string(),
            },
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            name: "solarized-light".to_string(),
            colors: ThemeColors {
                background: "#fdf6e3".to_string(),
                foreground: "#657b83".to_string(),
                primary: "#268bd2".to_string(),
                secondary: "#6c71c4".to_string(),
                accent: "#cb4b16".to_string(),
                error: "#dc322f".to_string(),
                warning: "#b58900".to_string(),
                success: "#859900".to_string(),
                muted: "#93a1a1".to_string(),
                border: "#eee8d5".to_string(),
            },
        }
    }

    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            colors: ThemeColors {
                background: "#ffffff".to_string(),
                foreground: "#24292e".to_string(),
                primary: "#0366d6".to_string(),
                secondary: "#6f42c1".to_string(),
                accent: "#d73a49".to_string(),
                error: "#d73a49".to_string(),
                warning: "#e36209".to_string(),
                success: "#22863a".to_string(),
                muted: "#6a737d".to_string(),
                border: "#e1e4e8".to_string(),
            },
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    if supports_truecolor() {
        Some(Color::Rgb(r, g, b))
    } else {
        Some(Color::Indexed(rgb_to_256_index(r, g, b)))
    }
}

fn supports_truecolor() -> bool {
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        return colorterm.contains("truecolor") || colorterm.contains("24bit");
    }
    if let Ok(term) = std::env::var("TERM") {
        let truecolor_terms = [
            "xterm-truecolor",
            "screen-truecolor",
            "tmux-truecolor",
            "foot",
        ];
        for t in truecolor_terms {
            if term.contains(t) {
                return true;
            }
        }
    }
    false
}

fn rgb_to_256_index(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        if r < 8 {
            return 0;
        }
        if r > 248 {
            return 231;
        }
        return 232 + (r - 8) / 10;
    }

    let r_idx = if r < 95 { 0 } else { 1 + (r - 95) / 40 };
    let g_idx = if g < 95 { 0 } else { 1 + (g - 95) / 40 };
    let b_idx = if b < 95 { 0 } else { 1 + (b - 95) / 40 };

    16 + r_idx * 36 + g_idx * 6 + b_idx
}

pub struct ThemeManager {
    current: Theme,
    presets: HashMap<String, Theme>,
    custom_themes: HashMap<String, Theme>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut presets = HashMap::new();
        presets.insert("default".to_string(), Theme::default());
        presets.insert("light".to_string(), Theme::light());
        presets.insert("dark".to_string(), Theme::default());
        presets.insert("catppuccin".to_string(), Theme::catppuccin());
        presets.insert("tokyonight".to_string(), Theme::tokyonight());
        presets.insert("nord".to_string(), Theme::nord());
        presets.insert("gruvbox".to_string(), Theme::gruvbox());
        presets.insert("solarized-dark".to_string(), Theme::solarized_dark());
        presets.insert("solarized-light".to_string(), Theme::solarized_light());

        Self {
            current: Theme::default(),
            presets,
            custom_themes: HashMap::new(),
        }
    }

    pub fn register_custom_theme(&mut self, theme: Theme) -> Result<(), String> {
        if self.presets.contains_key(&theme.name) {
            return Err(format!(
                "Theme '{}' conflicts with existing preset",
                theme.name
            ));
        }
        self.custom_themes.insert(theme.name.clone(), theme);
        Ok(())
    }

    pub fn unregister_custom_theme(&mut self, name: &str) -> bool {
        self.custom_themes.remove(name).is_some()
    }

    pub fn list_custom_themes(&self) -> Vec<&str> {
        self.custom_themes.keys().map(|s| s.as_str()).collect()
    }

    pub fn is_custom_theme(&self, name: &str) -> bool {
        self.custom_themes.contains_key(name)
    }

    pub fn set_theme_by_name(&mut self, name: &str) -> Result<(), String> {
        if let Some(theme) = self.presets.get(name) {
            self.current = theme.clone();
            Ok(())
        } else if let Some(theme) = self.custom_themes.get(name) {
            self.current = theme.clone();
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }

    pub fn load_custom_themes(&mut self, themes: Vec<crate::config::CustomTheme>) -> Vec<String> {
        let mut errors = Vec::new();
        for ct in themes {
            if let Err(e) = ct.validate() {
                errors.push(e);
                continue;
            }
            let theme = Theme {
                name: ct.name.clone(),
                colors: crate::theme::ThemeColors {
                    background: ct.background,
                    foreground: ct.foreground,
                    primary: ct.primary,
                    secondary: ct.secondary,
                    accent: ct.accent,
                    error: ct.error,
                    warning: ct.warning,
                    success: ct.success,
                    muted: ct.muted,
                    border: ct.border,
                },
            };
            if let Err(e) = self.register_custom_theme(theme) {
                errors.push(e);
            }
        }
        errors
    }

    pub fn export_custom_themes(&self) -> Vec<crate::config::CustomTheme> {
        self.custom_themes
            .values()
            .map(|t| crate::config::CustomTheme {
                name: t.name.clone(),
                background: t.colors.background.clone(),
                foreground: t.colors.foreground.clone(),
                primary: t.colors.primary.clone(),
                secondary: t.colors.secondary.clone(),
                accent: t.colors.accent.clone(),
                error: t.colors.error.clone(),
                warning: t.colors.warning.clone(),
                success: t.colors.success.clone(),
                muted: t.colors.muted.clone(),
                border: t.colors.border.clone(),
            })
            .collect()
    }

    pub fn with_theme(theme: Theme) -> Self {
        let mut mgr = Self::new();
        mgr.current = theme;
        mgr
    }

    pub fn load_theme_file(&mut self, path: &str) -> Result<(), String> {
        self.current = Theme::load_from_file(path)?;
        Ok(())
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.current = theme;
    }

    pub fn current(&self) -> &Theme {
        &self.current
    }

    pub fn list_themes(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.presets.keys().map(|s| s.as_str()).collect();
        names.extend(self.custom_themes.keys().map(|s| s.as_str()));
        names
    }

    pub fn get_preset(&self, name: &str) -> Option<&Theme> {
        self.presets.get(name)
    }

    pub fn get_custom_theme(&self, name: &str) -> Option<&Theme> {
        self.custom_themes.get(name)
    }

    pub fn save_to_config(&self) -> Result<(), String> {
        let config_dir = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.config", home)
        });
        let config_path = format!("{}/opencode-rs/config.toml", config_dir);
        let config_dir_path = format!("{}/opencode-rs", config_dir);

        if !std::path::Path::new(&config_dir_path).exists() {
            std::fs::create_dir_all(&config_dir_path)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let config_content = format!("theme = \"{}\"\n", self.current.name);
        std::fs::write(&config_path, config_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    pub fn load_from_config(&mut self) -> Result<(), String> {
        let config_dir = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.config", home)
        });
        let config_path = format!("{}/opencode-rs/config.toml", config_dir);

        if !std::path::Path::new(&config_path).exists() {
            return Ok(());
        }

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in config_content.lines() {
            if line.starts_with("theme") {
                if let Some(theme_name) = line.split('=').nth(1) {
                    let theme_name = theme_name.trim().trim_matches('"');
                    if let Some(theme) = self.presets.get(theme_name) {
                        self.current = theme.clone();
                    } else if let Some(theme) = self.custom_themes.get(theme_name) {
                        self.current = theme.clone();
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
    }

    #[test]
    fn test_parse_hex_color() {
        let color = parse_hex_color("#89b4fa");
        assert!(color.is_some());
        assert_eq!(color.unwrap(), Color::Rgb(0x89, 0xb4, 0xfa));
    }

    #[test]
    fn test_load_from_json() {
        let json = "{
            \"name\": \"test\",
            \"colors\": {
                \"background\": \"#000000\",
                \"foreground\": \"#ffffff\",
                \"primary\": \"#0000ff\",
                \"secondary\": \"#ff00ff\",
                \"accent\": \"#ff0000\",
                \"error\": \"#ff0000\",
                \"warning\": \"#ffff00\",
                \"success\": \"#00ff00\",
                \"muted\": \"#888888\",
                \"border\": \"#444444\"
            }
        }";
        let theme = Theme::load_from_json(json).unwrap();
        assert_eq!(theme.name, "test");
    }

    #[test]
    fn test_theme_manager() {
        let mut mgr = ThemeManager::new();
        assert_eq!(mgr.current().name, "default");
        let new_theme = Theme::load_from_json(
            "{
            \"name\": \"dark\",
            \"colors\": {
                \"background\": \"#000000\", \"foreground\": \"#ffffff\",
                \"primary\": \"#0000ff\", \"secondary\": \"#ff00ff\",
                \"accent\": \"#ff0000\", \"error\": \"#ff0000\",
                \"warning\": \"#ffff00\", \"success\": \"#00ff00\",
                \"muted\": \"#888888\", \"border\": \"#444444\"
            }
        }",
        )
        .unwrap();
        mgr.set_theme(new_theme);
        assert_eq!(mgr.current().name, "dark");
    }
}
