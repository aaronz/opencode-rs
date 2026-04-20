pub mod aggregator;
pub mod builtin;
pub mod client;
pub mod custom;
pub mod diagnostics;
pub mod error;
pub mod experimental;
pub mod language;
pub mod launch;
pub mod manager;
pub mod mock;
mod server;
pub mod types;

pub use aggregator::DiagnosticAggregator;
pub use builtin::{BuiltInRegistry, BuiltInServer, BundledConfig, DetectionResult, PathIndicator};
pub use client::LspClient;
pub use custom::{
    CustomLspServer, CustomRegistry, CustomServerConfig, RegisterError, ServerCapabilities,
};
pub use diagnostics::{
    filter_diagnostics_by_file, filter_diagnostics_by_severity, handle_publish_diagnostics,
   DiagnosticSeverity, PublishDiagnosticsParams,
};
pub use error::{
    CrashCause, FailureHandlingConfig, LspError, ProtocolViolationType, UnhealthyReason,
};
pub use experimental::{ExperimentalLspTool, ExperimentalLspToolArgs};
pub use language::Language;
pub use launch::LaunchConfig;
pub use manager::LspManager;
pub use mock::MockLspServer;
pub use types::{Diagnostic, Location, Severity, Symbol};
