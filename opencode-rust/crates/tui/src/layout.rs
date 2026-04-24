use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPreset {
    Default,
    Wide,
    Narrow,
    Focus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnMode {
    ThreeColumn,
    TwoColumn,
    SingleColumn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutProportions {
    pub sidebar_width: u16,
    pub main_width: u16,
    pub right_panel_width: u16,
    pub show_right_panel: bool,
    pub column_mode: ColumnMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutManager {
    current_layout: LayoutPreset,
    responsive: bool,
}

impl LayoutPreset {
    pub fn proportions(self) -> LayoutProportions {
        match self {
            LayoutPreset::Default => LayoutProportions {
                sidebar_width: 25,
                main_width: 50,
                right_panel_width: 25,
                show_right_panel: true,
                column_mode: ColumnMode::ThreeColumn,
            },
            LayoutPreset::Wide => LayoutProportions {
                sidebar_width: 18,
                main_width: 62,
                right_panel_width: 20,
                show_right_panel: true,
                column_mode: ColumnMode::ThreeColumn,
            },
            LayoutPreset::Narrow => LayoutProportions {
                sidebar_width: 30,
                main_width: 46,
                right_panel_width: 24,
                show_right_panel: true,
                column_mode: ColumnMode::ThreeColumn,
            },
            LayoutPreset::Focus => LayoutProportions {
                sidebar_width: 20,
                main_width: 80,
                right_panel_width: 0,
                show_right_panel: false,
                column_mode: ColumnMode::SingleColumn,
            },
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            LayoutPreset::Default => "Default",
            LayoutPreset::Wide => "Wide",
            LayoutPreset::Narrow => "Narrow",
            LayoutPreset::Focus => "Focus",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value.trim() {
            "Default" => Some(LayoutPreset::Default),
            "Wide" => Some(LayoutPreset::Wide),
            "Narrow" => Some(LayoutPreset::Narrow),
            "Focus" => Some(LayoutPreset::Focus),
            _ => None,
        }
    }
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            current_layout: LayoutPreset::Default,
            responsive: true,
        }
    }

    pub fn switch_to(&mut self, preset: LayoutPreset) {
        self.current_layout = preset;
    }

    pub fn get_layout(&self) -> LayoutPreset {
        self.current_layout
    }

    pub fn get_proportions(&self) -> LayoutProportions {
        self.current_layout.proportions()
    }

    pub fn get_proportions_for_width(&self, terminal_width: u16) -> LayoutProportions {
        if !self.responsive {
            return self.current_layout.proportions();
        }

        if terminal_width >= 160 {
            let mut p = self.current_layout.proportions();
            p.column_mode = ColumnMode::ThreeColumn;
            p.show_right_panel = true;
            p
        } else if terminal_width >= 100 {
            let mut p = self.current_layout.proportions();
            p.column_mode = ColumnMode::TwoColumn;
            p.show_right_panel = false;
            p.right_panel_width = 0;
            p.main_width += p.right_panel_width;
            p
        } else {
            let mut p = self.current_layout.proportions();
            p.column_mode = ColumnMode::SingleColumn;
            p.show_right_panel = false;
            p.right_panel_width = 0;
            p.sidebar_width = 0;
            p.main_width = terminal_width;
            p
        }
    }

    pub fn set_responsive(&mut self, enabled: bool) {
        self.responsive = enabled;
    }

    pub fn is_responsive(&self) -> bool {
        self.responsive
    }

    pub fn cycle_next(&mut self) {
        let next = match self.current_layout {
            LayoutPreset::Default => LayoutPreset::Wide,
            LayoutPreset::Wide => LayoutPreset::Narrow,
            LayoutPreset::Narrow => LayoutPreset::Focus,
            LayoutPreset::Focus => LayoutPreset::Default,
        };
        self.current_layout = next;
    }

    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.current_layout.as_str())
    }

    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let value = std::fs::read_to_string(path)?;
        let preset = LayoutPreset::from_str(value.trim()).unwrap_or(LayoutPreset::Default);
        Ok(Self {
            current_layout: preset,
            responsive: true,
        })
    }

    pub fn check_minimum_size(width: u16, height: u16) -> Result<(), String> {
        const MIN_WIDTH: u16 = 80;
        const MIN_HEIGHT: u16 = 24;

        if width < MIN_WIDTH || height < MIN_HEIGHT {
            Err(format!(
                "Terminal too small: {}x{} (minimum: {}x{})\nPlease resize your terminal and try again.",
                width, height, MIN_WIDTH, MIN_HEIGHT
            ))
        } else {
            Ok(())
        }
    }

    pub fn recommended_size_for(preset: LayoutPreset) -> (u16, u16) {
        match preset {
            LayoutPreset::Default => (120, 40),
            LayoutPreset::Wide => (160, 50),
            LayoutPreset::Narrow => (100, 35),
            LayoutPreset::Focus => (100, 30),
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_next_wraps_around() {
        let mut manager = LayoutManager::new();
        assert_eq!(manager.get_layout(), LayoutPreset::Default);
        manager.cycle_next();
        assert_eq!(manager.get_layout(), LayoutPreset::Wide);
        manager.cycle_next();
        assert_eq!(manager.get_layout(), LayoutPreset::Narrow);
        manager.cycle_next();
        assert_eq!(manager.get_layout(), LayoutPreset::Focus);
        manager.cycle_next();
        assert_eq!(manager.get_layout(), LayoutPreset::Default);
    }

    #[test]
    fn focus_hides_right_panel() {
        let mut manager = LayoutManager::new();
        manager.switch_to(LayoutPreset::Focus);
        let p = manager.get_proportions();
        assert!(!p.show_right_panel);
        assert_eq!(p.right_panel_width, 0);
    }

    #[test]
    fn responsive_three_column_at_160_plus() {
        let manager = LayoutManager::new();
        let p = manager.get_proportions_for_width(160);
        assert_eq!(p.column_mode, ColumnMode::ThreeColumn);
        assert!(p.show_right_panel);
    }

    #[test]
    fn responsive_two_column_at_100_to_159() {
        let manager = LayoutManager::new();
        let p = manager.get_proportions_for_width(120);
        assert_eq!(p.column_mode, ColumnMode::TwoColumn);
        assert!(!p.show_right_panel);
    }

    #[test]
    fn responsive_single_column_below_100() {
        let manager = LayoutManager::new();
        let p = manager.get_proportions_for_width(80);
        assert_eq!(p.column_mode, ColumnMode::SingleColumn);
        assert!(!p.show_right_panel);
        assert_eq!(p.sidebar_width, 0);
    }

    #[test]
    fn responsive_can_be_disabled() {
        let mut manager = LayoutManager::new();
        manager.set_responsive(false);
        let p = manager.get_proportions_for_width(80);
        assert_eq!(p.column_mode, ColumnMode::ThreeColumn);
        assert!(!manager.is_responsive());
    }
}
