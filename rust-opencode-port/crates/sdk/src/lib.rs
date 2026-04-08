//! # OpenCode SDK
//!
//! Rust SDK for OpenCode RS - Programmatic access to OpenCode capabilities.
//!
//! ## Features
//!
//! - **Session Management**: Create, load, save, fork, and abort sessions
//! - **Tool Execution**: Execute and list available tools
//! - **Auth Integration**: API key authentication
//! - **Error Handling**: Structured error types with error codes (1xxx-9xxx)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use opencode_sdk::OpenCodeClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OpenCodeClient::builder()
//!         .api_key("your-api-key")
//!         .build()?;
//!
//!     let session = client.create_session(Some("Hello, world!")).await?;
//!     println!("Created session: {}", session.session_id);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Codes
//!
//! | Range | Category |
//! |-------|----------|
//! | 1xxx  | Authentication |
//! | 2xxx  | Authorization |
//! | 3xxx  | Provider |
//! | 4xxx  | Tool |
//! | 5xxx  | Session |
//! | 6xxx  | Config |
//! | 7xxx  | Validation |
//! | 9xxx  | Internal |

pub mod client;
pub mod error;
pub mod session;
pub mod tools;
pub mod auth;

pub use client::{ClientConfig, OpenCodeClient, ClientBuilder};
pub use error::{SdkError, SdkResult};
pub use session::{SdkSession, SessionInfo};
pub use tools::{ToolDefinition, ToolResult, ToolExecutor};
pub use auth::ApiKeyAuth;

/// Re-export core types for convenience
pub use opencode_core::Message;
