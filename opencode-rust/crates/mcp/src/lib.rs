pub mod auth;
pub mod client;
pub mod integration;
pub mod pool;
pub mod protocol;
pub mod registry;
pub mod server;
pub mod tool_bridge;

pub use client::{
    ConnectionState, JsonRpcMessage, McpClient, McpError, McpResource, McpTool, McpToolResult,
    McpTransport, StdioProcess,
};
pub use integration::register_mcp_tools;
pub use pool::{EndpointPoolStats, McpConnectionPool, PoolConfig, PoolStats, PooledClient};
pub use protocol::*;
pub use registry::{McpManager, McpPermission, McpRegistry};
pub use server::McpServer;
pub use tool_bridge::McpToolAdapter;
