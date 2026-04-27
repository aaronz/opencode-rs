//! Directory scanner for loading OpenCode configurations.
//!
//! This module provides [`DirectoryScanner`] for discovering agents, commands,
//! modes, plugins, and themes from the filesystem.
//!
//! # Definitions Loaded
//!
//! - [`AgentDefinition`] - Agent configurations from `agents/` directory
//! - [`CommandDefinition`] - Command templates from `commands/` directory
//! - [`ModeDefinition`] - Mode configurations from `modes/` directory
//! - [`PluginInfo`] - Plugin metadata from `plugins/` directory
//! - [`ThemeInfo`] - Theme configurations from `themes/` directory
//!
//! # Directory Structure
//!
//! ```text
//! ~/.config/opencode-rs/
//!   agents/      - Agent definition files
//!   commands/    - Command templates
//!   modes/       - Mode configurations
//!   plugins/     - Plugin metadata
//!   themes/      - UI themes
//! ```
//!
//! # Use Case
//!
//! Used at startup to discover available agents, commands, and plugins.

pub mod scanner;
pub mod types;

pub use scanner::DirectoryScanner;
pub use types::{AgentDefinition, CommandDefinition, ModeDefinition, PluginInfo, ThemeInfo};
