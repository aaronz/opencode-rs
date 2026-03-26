use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub url: String,
    pub tools: Vec<McpTool>,
}

pub struct McpManager {
    servers: HashMap<String, McpServer>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub fn register_server(&mut self, server: McpServer) {
        self.servers.insert(server.name.clone(), server);
    }

    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }

    pub fn list_servers(&self) -> Vec<&McpServer> {
        self.servers.values().collect()
    }

    pub fn list_tools(&self) -> Vec<&McpTool> {
        self.servers.values().flat_map(|s| s.tools.iter()).collect()
    }

    pub fn find_tool(&self, name: &str) -> Option<&McpTool> {
        self.servers
            .values()
            .flat_map(|s| s.tools.iter())
            .find(|t| t.name == name)
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}
