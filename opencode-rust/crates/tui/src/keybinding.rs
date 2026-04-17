//! Keybinding system for the TUI
//!
//! This module provides a configurable keybinding system that allows users to
//! customize keyboard shortcuts. It supports default keybindings, custom
//! keybindings, and conflict detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents a keyboard key with optional modifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
    /// Modifier keys (Ctrl, Alt, Shift)
    pub modifiers: KeyModifiers,
    /// The key code
    pub code: KeyCode,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            s.push_str("Ctrl+");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            s.push_str("Alt+");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            s.push_str("Shift+");
        }
        s.push_str(&self.code.to_string());
        write!(f, "{}", s)
    }
}

impl Key {
    /// Parse a key string like "Ctrl+p" or "Alt+Shift+F1"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return None;
        }

        let mut modifiers = KeyModifiers::empty();
        let mut code = None;

        for part in parts {
            let part_upper = part.to_uppercase();
            match part_upper.as_str() {
                "CTRL" => modifiers |= KeyModifiers::CONTROL,
                "ALT" => modifiers |= KeyModifiers::ALT,
                "SHIFT" => modifiers |= KeyModifiers::SHIFT,
                "ESC" => code = Some(KeyCode::Esc),
                "ENTER" => code = Some(KeyCode::Enter),
                "TAB" => code = Some(KeyCode::Tab),
                "BACKSPACE" => code = Some(KeyCode::Backspace),
                "UP" => code = Some(KeyCode::Up),
                "DOWN" => code = Some(KeyCode::Down),
                "LEFT" => code = Some(KeyCode::Left),
                "RIGHT" => code = Some(KeyCode::Right),
                "HOME" => code = Some(KeyCode::Home),
                "END" => code = Some(KeyCode::End),
                "PAGEUP" => code = Some(KeyCode::PageUp),
                "PAGEDOWN" => code = Some(KeyCode::PageDown),
                "SPACE" => code = Some(KeyCode::Space),
                c if c.starts_with('F') && c.len() <= 3 => {
                    if let Ok(n) = c[1..].parse::<u8>() {
                        code = Some(KeyCode::F(n));
                    }
                }
                c if c.len() == 1 => {
                    #[expect(clippy::expect_used)]
                    let mut ch = part
                        .chars()
                        .next()
                        .expect("single-char key binding is guaranteed non-empty");
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        ch = ch.to_ascii_uppercase();
                    }
                    code = Some(KeyCode::Char(ch));
                }
                _ => return None,
            }
        }

        code.map(|code| Key { modifiers, code })
    }
}

/// Key code enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Escape key
    Esc,
    /// Enter key
    Enter,
    /// Tab key
    Tab,
    /// Backspace key
    Backspace,
    /// Space key
    Space,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Home key
    Home,
    /// End key
    End,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Function keys
    F(u8),
    /// Character keys
    Char(char),
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Esc => write!(f, "Esc"),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::Tab => write!(f, "Tab"),
            KeyCode::Backspace => write!(f, "Backspace"),
            KeyCode::Space => write!(f, "Space"),
            KeyCode::Up => write!(f, "Up"),
            KeyCode::Down => write!(f, "Down"),
            KeyCode::Left => write!(f, "Left"),
            KeyCode::Right => write!(f, "Right"),
            KeyCode::Home => write!(f, "Home"),
            KeyCode::End => write!(f, "End"),
            KeyCode::PageUp => write!(f, "PageUp"),
            KeyCode::PageDown => write!(f, "PageDown"),
            KeyCode::F(n) => write!(f, "F{}", n),
            KeyCode::Char(c) => write!(f, "{}", c),
        }
    }
}

/// Modifier keys
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    bits: u8,
}

impl KeyModifiers {
    pub const CONTROL: Self = Self { bits: 1 };
    pub const ALT: Self = Self { bits: 2 };
    pub const SHIFT: Self = Self { bits: 4 };

    pub fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }
}

impl std::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
}

impl std::ops::BitAnd for KeyModifiers {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }
}

impl std::ops::BitXor for KeyModifiers {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        Self {
            bits: self.bits ^ other.bits,
        }
    }
}

impl std::ops::BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, other: Self) {
        self.bits |= other.bits;
    }
}

impl std::ops::BitAndAssign for KeyModifiers {
    fn bitand_assign(&mut self, other: Self) {
        self.bits &= other.bits;
    }
}

impl std::ops::BitXorAssign for KeyModifiers {
    fn bitxor_assign(&mut self, other: Self) {
        self.bits ^= other.bits;
    }
}

impl fmt::Display for KeyModifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.contains(Self::CONTROL) {
            parts.push("Ctrl");
        }
        if self.contains(Self::ALT) {
            parts.push("Alt");
        }
        if self.contains(Self::SHIFT) {
            parts.push("Shift");
        }
        write!(f, "{}", parts.join("+"))
    }
}

impl KeyModifiers {
    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

/// Actions that can be triggered by keybindings
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeybindingAction {
    /// Open command palette
    CommandPalette,
    /// Toggle timeline view
    Timeline,
    /// Create new session
    NewSession,
    /// Toggle file tree
    ToggleFiles,
    /// Open settings
    Settings,
    /// Search
    Search,
    /// Navigate up
    NavigateUp,
    /// Navigate down
    NavigateDown,
    /// Submit input
    Submit,
    /// Cancel current action
    Cancel,
    /// Toggle skills panel
    ToggleSkills,
    /// Interrupt current operation
    Interrupt,
    /// Toggle full screen
    ToggleFullscreen,
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Toggle terminal
    ToggleTerminal,
    /// Toggle sidebar
    ToggleSidebar,
    /// Toggle active sidebar section
    ToggleSidebarSection,
    /// Navigate to previous sidebar section
    NavigateSidebarPrev,
    /// Navigate to next sidebar section
    NavigateSidebarNext,
    /// Custom action (stored as string)
    Custom(String),
}

impl fmt::Display for KeybindingAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeybindingAction::CommandPalette => write!(f, "command_palette"),
            KeybindingAction::Timeline => write!(f, "timeline"),
            KeybindingAction::NewSession => write!(f, "new_session"),
            KeybindingAction::ToggleFiles => write!(f, "toggle_files"),
            KeybindingAction::Settings => write!(f, "settings"),
            KeybindingAction::Search => write!(f, "search"),
            KeybindingAction::NavigateUp => write!(f, "navigate_up"),
            KeybindingAction::NavigateDown => write!(f, "navigate_down"),
            KeybindingAction::Submit => write!(f, "submit"),
            KeybindingAction::Cancel => write!(f, "cancel"),
            KeybindingAction::ToggleSkills => write!(f, "toggle_skills"),
            KeybindingAction::Interrupt => write!(f, "interrupt"),
            KeybindingAction::ToggleFullscreen => write!(f, "toggle_fullscreen"),
            KeybindingAction::ScrollUp => write!(f, "scroll_up"),
            KeybindingAction::ScrollDown => write!(f, "scroll_down"),
            KeybindingAction::PageUp => write!(f, "page_up"),
            KeybindingAction::PageDown => write!(f, "page_down"),
            KeybindingAction::ToggleTerminal => write!(f, "toggle_terminal"),
            KeybindingAction::ToggleSidebar => write!(f, "toggle_sidebar"),
            KeybindingAction::ToggleSidebarSection => write!(f, "toggle_sidebar_section"),
            KeybindingAction::NavigateSidebarPrev => write!(f, "navigate_sidebar_prev"),
            KeybindingAction::NavigateSidebarNext => write!(f, "navigate_sidebar_next"),
            KeybindingAction::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Default keybindings
#[derive(Debug, Clone)]
pub struct DefaultKeybindings;

impl DefaultKeybindings {
    /// Get the default keybindings map
    pub fn get() -> HashMap<KeybindingAction, Key> {
        let mut map = HashMap::new();
        map.insert(
            KeybindingAction::CommandPalette,
            Key {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('p'),
            },
        );
        map.insert(
            KeybindingAction::Timeline,
            Key {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('t'),
            },
        );
        map.insert(
            KeybindingAction::NewSession,
            Key {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('n'),
            },
        );
        map.insert(
            KeybindingAction::ToggleFiles,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                code: KeyCode::Char('f'),
            },
        );
        map.insert(
            KeybindingAction::Settings,
            Key {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char(','),
            },
        );
        map.insert(
            KeybindingAction::Search,
            Key {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('/'),
            },
        );
        map.insert(
            KeybindingAction::NavigateUp,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::Up,
            },
        );
        map.insert(
            KeybindingAction::NavigateDown,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::Down,
            },
        );
        map.insert(
            KeybindingAction::Submit,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::Enter,
            },
        );
        map.insert(
            KeybindingAction::Cancel,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::Esc,
            },
        );
        map.insert(
            KeybindingAction::ToggleSkills,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                code: KeyCode::Char('s'),
            },
        );
        map.insert(
            KeybindingAction::Interrupt,
            Key {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('c'),
            },
        );
        map.insert(
            KeybindingAction::ScrollUp,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::PageUp,
            },
        );
        map.insert(
            KeybindingAction::ScrollDown,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::PageDown,
            },
        );
        map.insert(
            KeybindingAction::PageUp,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::PageUp,
            },
        );
        map.insert(
            KeybindingAction::PageDown,
            Key {
                modifiers: KeyModifiers::empty(),
                code: KeyCode::PageDown,
            },
        );
        map.insert(
            KeybindingAction::ToggleTerminal,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                code: KeyCode::Char('t'),
            },
        );
        map.insert(
            KeybindingAction::ToggleSidebar,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                code: KeyCode::Char('b'),
            },
        );
        map.insert(
            KeybindingAction::ToggleSidebarSection,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                code: KeyCode::Char('h'),
            },
        );
        map.insert(
            KeybindingAction::NavigateSidebarPrev,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT,
                code: KeyCode::Left,
            },
        );
        map.insert(
            KeybindingAction::NavigateSidebarNext,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT,
                code: KeyCode::Right,
            },
        );
        map
    }
}

/// Configuration for keybindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    /// Command palette keybinding
    pub command_palette: Option<String>,
    /// Timeline keybinding
    pub timeline: Option<String>,
    /// New session keybinding
    pub new_session: Option<String>,
    /// Toggle files keybinding
    pub toggle_files: Option<String>,
    /// Settings keybinding
    pub settings: Option<String>,
    /// Search keybinding
    pub search: Option<String>,
    /// Navigate up keybinding
    pub navigate_up: Option<String>,
    /// Navigate down keybinding
    pub navigate_down: Option<String>,
    /// Submit keybinding
    pub submit: Option<String>,
    /// Cancel keybinding
    pub cancel: Option<String>,
    /// Toggle skills keybinding
    pub toggle_skills: Option<String>,
    /// Interrupt keybinding
    pub interrupt: Option<String>,
    /// Scroll up keybinding
    pub scroll_up: Option<String>,
    /// Scroll down keybinding
    pub scroll_down: Option<String>,
    /// Page up keybinding
    pub page_up: Option<String>,
    /// Page down keybinding
    pub page_down: Option<String>,
    /// Toggle terminal keybinding
    pub toggle_terminal: Option<String>,
    /// Toggle sidebar keybinding
    pub toggle_sidebar: Option<String>,
    /// Toggle sidebar section keybinding
    pub toggle_sidebar_section: Option<String>,
    /// Navigate sidebar previous keybinding
    pub navigate_sidebar_prev: Option<String>,
    /// Navigate sidebar next keybinding
    pub navigate_sidebar_next: Option<String>,
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            command_palette: None,
            timeline: None,
            new_session: None,
            toggle_files: None,
            settings: None,
            search: None,
            navigate_up: None,
            navigate_down: None,
            submit: None,
            cancel: None,
            toggle_skills: None,
            interrupt: None,
            scroll_up: None,
            scroll_down: None,
            page_up: None,
            page_down: None,
            toggle_terminal: None,
            toggle_sidebar: None,
            toggle_sidebar_section: None,
            navigate_sidebar_prev: None,
            navigate_sidebar_next: None,
        }
    }
}

impl KeybindingConfig {
    /// Create a new keybinding config with all defaults
    pub fn default_config() -> Self {
        Self::default()
    }

    /// Create a new empty keybinding config (no custom keybindings)
    pub fn empty() -> Self {
        Self::default()
    }

    /// Detect conflicts in the keybinding configuration
    pub fn detect_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        let mut seen: HashMap<String, String> = HashMap::new();

        let bindings: Vec<(&str, &Option<String>)> = vec![
            ("command_palette", &self.command_palette),
            ("timeline", &self.timeline),
            ("new_session", &self.new_session),
            ("toggle_files", &self.toggle_files),
            ("settings", &self.settings),
            ("search", &self.search),
            ("navigate_up", &self.navigate_up),
            ("navigate_down", &self.navigate_down),
            ("submit", &self.submit),
            ("cancel", &self.cancel),
            ("toggle_skills", &self.toggle_skills),
            ("interrupt", &self.interrupt),
            ("scroll_up", &self.scroll_up),
            ("scroll_down", &self.scroll_down),
            ("page_up", &self.page_up),
            ("page_down", &self.page_down),
            ("toggle_terminal", &self.toggle_terminal),
        ];

        for (name, keybind) in bindings {
            if let Some(key) = keybind {
                if key.is_empty() {
                    continue;
                }
                if let Some(prev) = seen.get(key) {
                    conflicts.push(format!(
                        "Keybinding conflict: '{}' is assigned to both '{}' and '{}'",
                        key, prev, name
                    ));
                } else {
                    seen.insert(key.clone(), name.to_string());
                }
            }
        }

        conflicts
    }

    /// Check if any keybindings are configured
    pub fn has_custom_keybindings(&self) -> bool {
        let bindings: Vec<&Option<String>> = vec![
            &self.command_palette,
            &self.timeline,
            &self.new_session,
            &self.toggle_files,
            &self.settings,
            &self.search,
            &self.navigate_up,
            &self.navigate_down,
            &self.submit,
            &self.cancel,
            &self.toggle_skills,
            &self.interrupt,
            &self.scroll_up,
            &self.scroll_down,
            &self.page_up,
            &self.page_down,
            &self.toggle_terminal,
        ];

        bindings
            .iter()
            .any(|b| b.as_ref().map_or(false, |s| !s.is_empty()))
    }
}

/// A single keybinding entry
#[derive(Debug, Clone)]
pub struct Keybinding {
    pub action: KeybindingAction,
    pub key: Key,
    pub description: String,
}

impl Keybinding {
    pub fn new(action: KeybindingAction, key: Key, description: &str) -> Self {
        Self {
            action,
            key,
            description: description.to_string(),
        }
    }
}

/// Keybinding registry that manages all keybindings
#[derive(Debug, Clone)]
pub struct KeybindingRegistry {
    bindings: HashMap<KeybindingAction, Key>,
    custom_config: KeybindingConfig,
}

impl Default for KeybindingRegistry {
    fn default() -> Self {
        Self::new(KeybindingConfig::default())
    }
}

impl KeybindingRegistry {
    /// Create a new keybinding registry with the given custom config
    pub fn new(custom_config: KeybindingConfig) -> Self {
        Self {
            bindings: DefaultKeybindings::get(),
            custom_config,
        }
    }

    /// Get the key for a specific action, considering custom overrides
    pub fn get_key(&self, action: &KeybindingAction) -> Option<Key> {
        // Check if there's a custom override for this action
        if let Some(key) = self.get_custom_key(action) {
            return Some(key.clone());
        }

        // Fall back to default
        self.bindings.get(action).cloned()
    }

    /// Get custom key for an action
    fn get_custom_key(&self, action: &KeybindingAction) -> Option<Key> {
        let key_str = match action {
            KeybindingAction::CommandPalette => self.custom_config.command_palette.as_ref(),
            KeybindingAction::Timeline => self.custom_config.timeline.as_ref(),
            KeybindingAction::NewSession => self.custom_config.new_session.as_ref(),
            KeybindingAction::ToggleFiles => self.custom_config.toggle_files.as_ref(),
            KeybindingAction::Settings => self.custom_config.settings.as_ref(),
            KeybindingAction::Search => self.custom_config.search.as_ref(),
            KeybindingAction::NavigateUp => self.custom_config.navigate_up.as_ref(),
            KeybindingAction::NavigateDown => self.custom_config.navigate_down.as_ref(),
            KeybindingAction::Submit => self.custom_config.submit.as_ref(),
            KeybindingAction::Cancel => self.custom_config.cancel.as_ref(),
            KeybindingAction::ToggleSkills => self.custom_config.toggle_skills.as_ref(),
            KeybindingAction::Interrupt => self.custom_config.interrupt.as_ref(),
            KeybindingAction::ScrollUp => self.custom_config.scroll_up.as_ref(),
            KeybindingAction::ScrollDown => self.custom_config.scroll_down.as_ref(),
            KeybindingAction::PageUp => self.custom_config.page_up.as_ref(),
            KeybindingAction::PageDown => self.custom_config.page_down.as_ref(),
            KeybindingAction::ToggleTerminal => self.custom_config.toggle_terminal.as_ref(),
            KeybindingAction::ToggleFullscreen => None,
            KeybindingAction::ToggleSidebar => self.custom_config.toggle_sidebar.as_ref(),
            KeybindingAction::ToggleSidebarSection => {
                self.custom_config.toggle_sidebar_section.as_ref()
            }
            KeybindingAction::NavigateSidebarPrev => {
                self.custom_config.navigate_sidebar_prev.as_ref()
            }
            KeybindingAction::NavigateSidebarNext => {
                self.custom_config.navigate_sidebar_next.as_ref()
            }
            KeybindingAction::Custom(_) => None,
        };

        key_str
            .and_then(|s| if s.is_empty() { None } else { Some(s) })
            .and_then(|s| Key::parse(s))
    }

    /// Get the action associated with a key
    pub fn get_action(&self, key: &Key) -> Option<KeybindingAction> {
        // Check custom bindings first
        for (action, _) in self.custom_config_iter() {
            if let Some(custom_key) = self.get_custom_key(action) {
                if custom_key == *key {
                    return Some(action.clone());
                }
            }
        }

        // Check default bindings
        for (action, default_key) in &self.bindings {
            // Skip actions that have custom overrides
            if self.get_custom_key(action).is_some() {
                continue;
            }
            if default_key == key {
                return Some(action.clone());
            }
        }

        None
    }

    /// Get all keybinding conflicts
    pub fn get_conflicts(&self) -> Vec<String> {
        self.custom_config.detect_conflicts()
    }

    /// Check if there are any conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.get_conflicts().is_empty()
    }

    /// Iterator over custom keybindings
    fn custom_config_iter(&self) -> impl Iterator<Item = (&KeybindingAction, &Option<String>)> {
        vec![
            (
                &KeybindingAction::CommandPalette,
                &self.custom_config.command_palette,
            ),
            (&KeybindingAction::Timeline, &self.custom_config.timeline),
            (
                &KeybindingAction::NewSession,
                &self.custom_config.new_session,
            ),
            (
                &KeybindingAction::ToggleFiles,
                &self.custom_config.toggle_files,
            ),
            (&KeybindingAction::Settings, &self.custom_config.settings),
            (&KeybindingAction::Search, &self.custom_config.search),
            (
                &KeybindingAction::NavigateUp,
                &self.custom_config.navigate_up,
            ),
            (
                &KeybindingAction::NavigateDown,
                &self.custom_config.navigate_down,
            ),
            (&KeybindingAction::Submit, &self.custom_config.submit),
            (&KeybindingAction::Cancel, &self.custom_config.cancel),
            (
                &KeybindingAction::ToggleSkills,
                &self.custom_config.toggle_skills,
            ),
            (&KeybindingAction::Interrupt, &self.custom_config.interrupt),
            (&KeybindingAction::ScrollUp, &self.custom_config.scroll_up),
            (
                &KeybindingAction::ScrollDown,
                &self.custom_config.scroll_down,
            ),
            (&KeybindingAction::PageUp, &self.custom_config.page_up),
            (&KeybindingAction::PageDown, &self.custom_config.page_down),
            (
                &KeybindingAction::ToggleTerminal,
                &self.custom_config.toggle_terminal,
            ),
            (
                &KeybindingAction::ToggleSidebar,
                &self.custom_config.toggle_sidebar,
            ),
            (
                &KeybindingAction::ToggleSidebarSection,
                &self.custom_config.toggle_sidebar_section,
            ),
            (
                &KeybindingAction::NavigateSidebarPrev,
                &self.custom_config.navigate_sidebar_prev,
            ),
            (
                &KeybindingAction::NavigateSidebarNext,
                &self.custom_config.navigate_sidebar_next,
            ),
        ]
        .into_iter()
    }

    /// Get all bindings as a list (for UI display)
    pub fn get_all_bindings(&self) -> Vec<Keybinding> {
        let mut result = Vec::new();

        for (action, default_key) in &self.bindings {
            let custom_key = self.get_custom_key(action);
            let key = custom_key.unwrap_or_else(|| default_key.clone());

            let description = match action {
                KeybindingAction::CommandPalette => "Open command palette",
                KeybindingAction::Timeline => "Toggle timeline view",
                KeybindingAction::NewSession => "Create new session",
                KeybindingAction::ToggleFiles => "Toggle file tree",
                KeybindingAction::Settings => "Open settings",
                KeybindingAction::Search => "Search",
                KeybindingAction::NavigateUp => "Navigate up",
                KeybindingAction::NavigateDown => "Navigate down",
                KeybindingAction::Submit => "Submit input",
                KeybindingAction::Cancel => "Cancel",
                KeybindingAction::ToggleSkills => "Toggle skills panel",
                KeybindingAction::Interrupt => "Interrupt operation",
                KeybindingAction::ToggleFullscreen => "Toggle fullscreen",
                KeybindingAction::ScrollUp => "Scroll up",
                KeybindingAction::ScrollDown => "Scroll down",
                KeybindingAction::PageUp => "Page up",
                KeybindingAction::PageDown => "Page down",
                KeybindingAction::ToggleTerminal => "Toggle terminal",
                KeybindingAction::ToggleSidebar => "Toggle sidebar",
                KeybindingAction::ToggleSidebarSection => "Toggle sidebar section",
                KeybindingAction::NavigateSidebarPrev => "Navigate sidebar previous",
                KeybindingAction::NavigateSidebarNext => "Navigate sidebar next",
                KeybindingAction::Custom(s) => s.as_str(),
            };

            result.push(Keybinding::new(action.clone(), key, description));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parsing_simple() {
        let key = Key::parse("Ctrl+p").unwrap();
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert_eq!(key.code, KeyCode::Char('p'));
    }

    #[test]
    fn test_key_parsing_with_shift() {
        let key = Key::parse("Shift+Ctrl+p").unwrap();
        assert!(key.modifiers.contains(KeyModifiers::SHIFT));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert_eq!(key.code, KeyCode::Char('P'));
    }

    #[test]
    fn test_key_parsing_escape() {
        let key = Key::parse("Esc").unwrap();
        assert!(key.modifiers.is_empty());
        assert_eq!(key.code, KeyCode::Esc);
    }

    #[test]
    fn test_key_parsing_enter() {
        let key = Key::parse("Enter").unwrap();
        assert_eq!(key.code, KeyCode::Enter);
    }

    #[test]
    fn test_key_parsing_function_key() {
        let key = Key::parse("F1").unwrap();
        assert_eq!(key.code, KeyCode::F(1));

        let key = Key::parse("F12").unwrap();
        assert_eq!(key.code, KeyCode::F(12));
    }

    #[test]
    fn test_key_parsing_arrow_keys() {
        assert_eq!(Key::parse("Up").unwrap().code, KeyCode::Up);
        assert_eq!(Key::parse("Down").unwrap().code, KeyCode::Down);
        assert_eq!(Key::parse("Left").unwrap().code, KeyCode::Left);
        assert_eq!(Key::parse("Right").unwrap().code, KeyCode::Right);
    }

    #[test]
    fn test_key_parsing_invalid() {
        assert!(Key::parse("").is_none());
        assert!(Key::parse("InvalidKey").is_none());
    }

    #[test]
    fn test_key_display() {
        let key = Key {
            modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT,
            code: KeyCode::Char('c'),
        };
        assert_eq!(key.to_string(), "Ctrl+Alt+c");
    }

    #[test]
    fn test_key_equality() {
        let key1 = Key::parse("Ctrl+p").unwrap();
        let key2 = Key::parse("Ctrl+p").unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_key_modifiers_display() {
        assert_eq!(KeyModifiers::CONTROL.to_string(), "Ctrl");
        assert_eq!(
            (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT).to_string(),
            "Ctrl+Alt+Shift"
        );
    }

    #[test]
    fn test_key_modifiers_contains() {
        let modifiers = KeyModifiers::CONTROL | KeyModifiers::ALT;
        assert!(modifiers.contains(KeyModifiers::CONTROL));
        assert!(modifiers.contains(KeyModifiers::ALT));
        assert!(!modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_keybinding_action_display() {
        assert_eq!(
            KeybindingAction::CommandPalette.to_string(),
            "command_palette"
        );
        assert_eq!(
            KeybindingAction::Custom("test".to_string()).to_string(),
            "custom:test"
        );
    }

    #[test]
    fn test_default_keybindings_exist() {
        let defaults = DefaultKeybindings::get();
        assert!(!defaults.is_empty());
        assert!(defaults.contains_key(&KeybindingAction::CommandPalette));
        assert!(defaults.contains_key(&KeybindingAction::Timeline));
        assert!(defaults.contains_key(&KeybindingAction::NewSession));
    }

    #[test]
    fn test_default_keybindings_values() {
        let defaults = DefaultKeybindings::get();

        // Ctrl+p for command palette
        let key = defaults.get(&KeybindingAction::CommandPalette).unwrap();
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert_eq!(key.code, KeyCode::Char('p'));

        // Ctrl+t for timeline
        let key = defaults.get(&KeybindingAction::Timeline).unwrap();
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert_eq!(key.code, KeyCode::Char('t'));

        // Escape for cancel
        let key = defaults.get(&KeybindingAction::Cancel).unwrap();
        assert!(key.modifiers.is_empty());
        assert_eq!(key.code, KeyCode::Esc);
    }

    #[test]
    fn test_keybinding_config_empty_has_no_conflicts() {
        let config = KeybindingConfig::empty();
        assert!(config.detect_conflicts().is_empty());
        assert!(!config.has_custom_keybindings());
    }

    #[test]
    fn test_keybinding_config_detect_no_conflicts() {
        let mut config = KeybindingConfig::default();
        config.command_palette = Some("Ctrl+p".to_string());
        config.timeline = Some("Ctrl+t".to_string());

        let conflicts = config.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_keybinding_config_detect_conflicts() {
        let mut config = KeybindingConfig::default();
        config.command_palette = Some("Ctrl+p".to_string());
        config.timeline = Some("Ctrl+p".to_string()); // Same key!

        let conflicts = config.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("conflict"));
        assert!(conflicts[0].contains("Ctrl+p"));
    }

    #[test]
    fn test_keybinding_config_multiple_conflicts() {
        let mut config = KeybindingConfig::default();
        config.command_palette = Some("Ctrl+x".to_string());
        config.timeline = Some("Ctrl+y".to_string());
        config.new_session = Some("Ctrl+x".to_string()); // Conflicts with command_palette

        let conflicts = config.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("command_palette"));
        assert!(conflicts[0].contains("new_session"));
    }

    #[test]
    fn test_keybinding_config_empty_string_no_conflict() {
        let mut config = KeybindingConfig::default();
        config.command_palette = Some("".to_string()); // Empty means use default
        config.timeline = Some("Ctrl+p".to_string());

        let conflicts = config.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_keybinding_registry_get_key_default() {
        let config = KeybindingConfig::empty();
        let registry = KeybindingRegistry::new(config);

        // Should get default key
        let key = registry.get_key(&KeybindingAction::CommandPalette);
        assert!(key.is_some());
        assert_eq!(key.unwrap().code, KeyCode::Char('p'));
    }

    #[test]
    fn test_keybinding_registry_get_key_custom() {
        let mut config = KeybindingConfig::empty();
        config.command_palette = Some("Ctrl+Shift+p".to_string());
        let registry = KeybindingRegistry::new(config);

        // Should get custom key
        let key = registry.get_key(&KeybindingAction::CommandPalette);
        assert!(key.is_some());
        let key = key.unwrap();
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert!(key.modifiers.contains(KeyModifiers::SHIFT));
        assert_eq!(key.code, KeyCode::Char('P'));
    }

    #[test]
    fn test_keybinding_registry_get_action() {
        let config = KeybindingConfig::empty();
        let registry = KeybindingRegistry::new(config);

        // Test default key
        let key = Key {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('p'),
        };
        let action = registry.get_action(&key);
        assert_eq!(action, Some(KeybindingAction::CommandPalette));
    }

    #[test]
    fn test_keybinding_registry_get_action_custom() {
        let mut config = KeybindingConfig::empty();
        config.command_palette = Some("Ctrl+Shift+x".to_string());
        let registry = KeybindingRegistry::new(config);

        let key = Key {
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            code: KeyCode::Char('X'),
        };
        let action = registry.get_action(&key);
        assert_eq!(action, Some(KeybindingAction::CommandPalette));
    }

    #[test]
    fn test_keybinding_registry_no_conflicts_default() {
        let config = KeybindingConfig::empty();
        let registry = KeybindingRegistry::new(config);
        assert!(!registry.has_conflicts());
    }

    #[test]
    fn test_keybinding_registry_has_conflicts() {
        let mut config = KeybindingConfig::empty();
        config.command_palette = Some("Ctrl+p".to_string());
        config.timeline = Some("Ctrl+p".to_string());
        let registry = KeybindingRegistry::new(config);

        assert!(registry.has_conflicts());
        let conflicts = registry.get_conflicts();
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_keybinding_registry_get_all_bindings() {
        let config = KeybindingConfig::empty();
        let registry = KeybindingRegistry::new(config);

        let bindings = registry.get_all_bindings();
        assert!(!bindings.is_empty());

        // Find command palette binding
        let cmd_binding = bindings
            .iter()
            .find(|b| matches!(b.action, KeybindingAction::CommandPalette));
        assert!(cmd_binding.is_some());
        assert_eq!(cmd_binding.unwrap().key.code, KeyCode::Char('p'));
    }

    #[test]
    fn test_keybinding_registry_get_all_bindings_with_custom() {
        let mut config = KeybindingConfig::empty();
        config.command_palette = Some("Ctrl+Shift+m".to_string());
        let registry = KeybindingRegistry::new(config);

        let bindings = registry.get_all_bindings();

        // Command palette should have custom key
        let cmd_binding = bindings
            .iter()
            .find(|b| matches!(b.action, KeybindingAction::CommandPalette));
        assert!(cmd_binding.is_some());
        assert_eq!(
            cmd_binding.unwrap().key,
            Key {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                code: KeyCode::Char('M'),
            }
        );

        // Timeline should still have default
        let timeline_binding = bindings
            .iter()
            .find(|b| matches!(b.action, KeybindingAction::Timeline));
        assert!(timeline_binding.is_some());
        assert_eq!(timeline_binding.unwrap().key.code, KeyCode::Char('t'));
    }

    #[test]
    fn test_keybinding_descriptions() {
        let config = KeybindingConfig::empty();
        let registry = KeybindingRegistry::new(config);

        let bindings = registry.get_all_bindings();
        for binding in bindings {
            assert!(!binding.description.is_empty());
        }
    }

    #[test]
    fn test_key_parsing_space() {
        let key = Key::parse("Space").unwrap();
        assert_eq!(key.code, KeyCode::Space);
    }

    #[test]
    fn test_key_parsing_backspace() {
        let key = Key::parse("Backspace").unwrap();
        assert_eq!(key.code, KeyCode::Backspace);
    }

    #[test]
    fn test_key_parsing_home_end() {
        assert_eq!(Key::parse("Home").unwrap().code, KeyCode::Home);
        assert_eq!(Key::parse("End").unwrap().code, KeyCode::End);
    }

    #[test]
    fn test_key_parsing_pageup_pagedown() {
        assert_eq!(Key::parse("PageUp").unwrap().code, KeyCode::PageUp);
        assert_eq!(Key::parse("PageDown").unwrap().code, KeyCode::PageDown);
    }

    #[test]
    fn test_key_parsing_case_insensitive() {
        assert!(Key::parse("ctrl+p").is_some());
        assert!(Key::parse("CTRL+P").is_some());
        assert!(Key::parse("CtRl+P").is_some());
    }

    #[test]
    fn test_has_custom_keybindings() {
        let mut config = KeybindingConfig::empty();
        assert!(!config.has_custom_keybindings());

        config.command_palette = Some("Ctrl+b".to_string());
        assert!(config.has_custom_keybindings());
    }

    #[test]
    fn test_custom_key_empty_string_uses_default() {
        let mut config = KeybindingConfig::empty();
        config.command_palette = Some("".to_string()); // Empty string = use default
        let registry = KeybindingRegistry::new(config);

        // Empty custom should fall back to default
        let key = registry.get_key(&KeybindingAction::CommandPalette);
        assert!(key.is_some());
        assert_eq!(key.unwrap().code, KeyCode::Char('p'));
    }
}
