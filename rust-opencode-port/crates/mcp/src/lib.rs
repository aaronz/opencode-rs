pub mod auth;
pub mod protocol;
pub mod server;
pub mod client;
pub mod tool_bridge;
pub mod integration;
pub mod registry;
pub mod pool;

pub use protocol::*;
pub use server::McpServer;
pub use client::{
    ConnectionState, JsonRpcMessage, McpClient, McpError, McpResource, McpTool, McpToolResult,
    McpTransport, StdioProcess,
};
pub use tool_bridge::McpToolAdapter;
pub use integration::register_mcp_tools;
pub use registry::{McpManager, McpPermission, McpRegistry};
pub use pool::{McpConnectionPool, PoolConfig, PooledClient, PoolStats, EndpointPoolStats};
