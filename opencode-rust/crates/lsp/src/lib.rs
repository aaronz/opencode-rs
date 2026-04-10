pub mod aggregator;
pub mod builtin;
pub mod client;
pub mod custom;
pub mod error;
pub mod language;
pub mod launch;
pub mod manager;
pub mod server;
pub mod types;

pub use aggregator::DiagnosticAggregator;
pub use builtin::{BuiltInRegistry, BuiltInServer, BundledConfig, DetectionResult, PathIndicator};
pub use client::LspClient;
pub use custom::{
    CustomLspServer, CustomRegistry, CustomServerConfig, RegisterError, ServerCapabilities,
};
pub use error::{
    CrashCause, FailureHandlingConfig, LspError, ProtocolViolationType, UnhealthyReason,
};
pub use language::Language;
pub use launch::LaunchConfig;
pub use manager::LspManager;
pub use server::LspServer;
pub use types::{Diagnostic, Location, Severity, Symbol};
