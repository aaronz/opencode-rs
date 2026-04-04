use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPreset {
    Default,
    Wide,
    Narrow,
    Focus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutProportions {
    pub sidebar_width: u16,
    pub main_width: u16,
    pub right_panel_width: u16,
    pub show_right_panel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutManager {
    current_layout: LayoutPreset,
}

impl LayoutPreset {
    pub fn proportions(self) -> LayoutProportions {
        match self {
            LayoutPreset::Default => LayoutProportions {
                sidebar_width: 25,
                main_width: 50,
                right_panel_width: 25,
                show_right_panel: true,
            },
            LayoutPreset::Wide => LayoutProportions {
                sidebar_width: 18,
                main_width: 62,
                right_panel_width: 20,
                show_right_panel: true,
            },
            LayoutPreset::Narrow => LayoutProportions {
                sidebar_width: 30,
                main_width: 46,
                right_panel_width: 24,
                show_right_panel: true,
            },
            LayoutPreset::Focus => LayoutProportions {
                sidebar_width: 20,
                main_width: 80,
                right_panel_width: 0,
                show_right_panel: false,
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
        })
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
}
