mod types;

use std::collections::HashMap;

use crate::mcp::types::{McpPrompt, McpResource, McpServer, McpTool};

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

    pub fn get_server_mut(&mut self, name: &str) -> Option<&mut McpServer> {
        self.servers.get_mut(name)
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

    pub fn list_resources(&self) -> Vec<&McpResource> {
        self.servers
            .values()
            .flat_map(|s| s.resources.iter())
            .collect()
    }

    pub fn find_resource(&self, uri: &str) -> Option<&McpResource> {
        self.servers
            .values()
            .flat_map(|s| s.resources.iter())
            .find(|r| r.uri == uri)
    }

    pub fn list_prompts(&self) -> Vec<&McpPrompt> {
        self.servers
            .values()
            .flat_map(|s| s.prompts.iter())
            .collect()
    }

    pub fn validate_tool_args(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Result<(), String> {
        let tool = self
            .find_tool(tool_name)
            .ok_or_else(|| format!("Tool '{}' not found in any MCP server", tool_name))?;
        tool.validate_args(args)
    }

    pub fn update_server_discovery(
        &mut self,
        name: &str,
        tools: Vec<McpTool>,
        resources: Vec<McpResource>,
        prompts: Vec<McpPrompt>,
    ) -> bool {
        if let Some(server) = self.servers.get_mut(name) {
            server.capabilities.tools = !tools.is_empty();
            server.capabilities.resources = !resources.is_empty();
            server.capabilities.prompts = !prompts.is_empty();
            server.tools = tools;
            server.resources = resources;
            server.prompts = prompts;
            true
        } else {
            false
        }
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::{McpCapabilities, McpPrompt, McpPromptArgument, McpResource, McpServer, McpTool};
    use serde_json::json;

    #[test]
    fn test_mcp_tool_validate_args_ok() {
        let tool = McpTool {
            name: "test".to_string(),
            description: "test".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                }
            }),
        };
        assert!(tool.validate_args(&json!({"path": "/foo"})).is_ok());
    }

    #[test]
    fn test_mcp_tool_validate_args_missing_required() {
        let tool = McpTool {
            name: "test".to_string(),
            description: "test".to_string(),
            parameters: json!({
                "required": ["path"],
                "properties": { "path": { "type": "string" } }
            }),
        };
        let result = tool.validate_args(&json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path"));
    }

    #[test]
    fn test_mcp_tool_validate_args_wrong_type() {
        let tool = McpTool {
            name: "test".to_string(),
            description: "test".to_string(),
            parameters: json!({
                "properties": { "count": { "type": "number" } }
            }),
        };
        let result = tool.validate_args(&json!({"count": "not-a-number"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_manager_discovery_update() {
        let mut mgr = McpManager::new();
        mgr.register_server(McpServer::new("s1", "http://localhost"));
        let tools = vec![McpTool {
            name: "t1".to_string(),
            description: "d".to_string(),
            parameters: json!({}),
        }];
        let updated = mgr.update_server_discovery("s1", tools, vec![], vec![]);
        assert!(updated);
        assert!(mgr.get_server("s1").unwrap().capabilities.tools);
        assert_eq!(mgr.list_tools().len(), 1);
    }

    #[test]
    fn test_mcp_server_new() {
        let server = McpServer::new("test_server", "http://localhost:8080");
        assert_eq!(server.name, "test_server");
        assert_eq!(server.url, "http://localhost:8080");
        assert!(server.tools.is_empty());
        assert!(server.resources.is_empty());
        assert!(!server.capabilities.tools);
    }

    #[test]
    fn test_mcp_server_find_resource() {
        let server = McpServer::new("s1", "http://localhost");
        let result = server.find_resource("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_mcp_server_find_prompt() {
        let server = McpServer::new("s1", "http://localhost");
        let result = server.find_prompt("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_mcp_manager_list_servers() {
        let mut mgr = McpManager::new();
        mgr.register_server(McpServer::new("s1", "http://localhost:8080"));
        mgr.register_server(McpServer::new("s2", "http://localhost:8081"));
        let servers = mgr.list_servers();
        assert_eq!(servers.len(), 2);
    }

    #[test]
    fn test_mcp_manager_list_resources() {
        let mut mgr = McpManager::new();
        let mut server = McpServer::new("s1", "http://localhost:8080");
        server.resources.push(McpResource {
            uri: "file://test".to_string(),
            name: "test".to_string(),
            description: None,
            mime_type: None,
            schema: None,
        });
        mgr.register_server(server);
        let resources = mgr.list_resources();
        assert_eq!(resources.len(), 1);
    }

    #[test]
    fn test_mcp_manager_list_prompts() {
        let mut mgr = McpManager::new();
        let mut server = McpServer::new("s1", "http://localhost:8080");
        server.prompts.push(McpPrompt {
            name: "test_prompt".to_string(),
            description: None,
            arguments: vec![],
        });
        mgr.register_server(server);
        let prompts = mgr.list_prompts();
        assert_eq!(prompts.len(), 1);
    }

    #[test]
    fn test_mcp_manager_find_resource() {
        let mut mgr = McpManager::new();
        let mut server = McpServer::new("s1", "http://localhost:8080");
        server.resources.push(McpResource {
            uri: "file://test".to_string(),
            name: "test".to_string(),
            description: None,
            mime_type: None,
            schema: None,
        });
        mgr.register_server(server);
        let result = mgr.find_resource("file://test");
        assert!(result.is_some());
        let result = mgr.find_resource("file://nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_mcp_manager_get_server_mut() {
        let mut mgr = McpManager::new();
        mgr.register_server(McpServer::new("s1", "http://localhost:8080"));
        let server = mgr.get_server_mut("s1");
        assert!(server.is_some());
    }
}