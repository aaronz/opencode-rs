use ratatui::style::Color;
use serde::{Deserialize, Serialize};

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
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

pub struct ThemeManager {
    current: Theme,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current: Theme::default(),
        }
    }

    pub fn with_theme(theme: Theme) -> Self {
        Self { current: theme }
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
