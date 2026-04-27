use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: Option<String>,
    pub prompt: String,
    pub model: Option<String>,
    pub tools: Option<Vec<String>>,
    pub options: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub description: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeDefinition {
    pub description: Option<String>,
    pub agent: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub path: PathBuf,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub author: Option<String>,
}
