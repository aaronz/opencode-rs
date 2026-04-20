pub mod aggregator;
pub mod builtin;
pub mod client;
pub mod completion;
pub mod custom;
pub mod diagnostics;
pub mod error;
pub mod experimental;
pub mod language;
pub mod launch;
pub mod manager;
pub mod mock;
mod server;
pub mod references;
pub mod types;

pub use aggregator::DiagnosticAggregator;
pub use builtin::{BuiltInRegistry, BuiltInServer, BundledConfig, DetectionResult, PathIndicator};
pub use client::LspClient;
pub use completion::{
    build_keyword_completion_items, build_method_completion_items, create_completion_item,
    filter_completions_by_context, filter_completions_by_prefix, get_completions,
    get_completion_trigger_character, handle_completion_trigger, CompletionParams,
    CompletionResult, CompletionTriggerContext, CompletionTriggerKind,
};
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
pub use references::{
    filter_references_by_file, find_references_in_document, find_references_workspace,
    get_declaration_location, ReferencesContext, ReferencesParams, ReferencesResult,
};
pub use types::{Diagnostic, Location, Severity, Symbol};
