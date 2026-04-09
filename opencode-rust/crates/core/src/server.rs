use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub api_prefix: String,
}

pub struct Server {
    config: ServerConfig,
    routes: HashMap<String, String>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: String, handler: String) {
        self.routes.insert(path, handler);
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub fn routes(&self) -> &HashMap<String, String> {
        &self.routes
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new(ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            api_prefix: "/api".to_string(),
        })
    }
}
