//! RuntimeHandle trait - abstract interface for UI ↔ Runtime boundary.
//!
//! This trait decouples the TUI/UI layer from the concrete RuntimeFacade implementation,
//! allowing for easier testing and future replacements without changing client code.

use async_trait::async_trait;
use futures_util::stream::Stream;
use opencode_core::events::DomainEvent;
use opencode_llm::Provider;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::commands::RuntimeFacadeCommand;
use crate::errors::RuntimeFacadeError;
use crate::types::{RuntimeFacadeResponse, RuntimeStatus};

/// Trait abstracting the Runtime's command interface.
///
/// UI components (TUI, CLI, Server) should depend on this trait rather than
/// the concrete RuntimeFacade or RuntimeFacadeHandle types.
#[async_trait]
pub trait RuntimeHandle: Send + Sync {
    /// Execute a command and return the response.
    async fn execute(
        &self,
        command: RuntimeFacadeCommand,
    ) -> Result<RuntimeFacadeResponse, RuntimeFacadeError>;

    /// Get the current runtime status.
    async fn status(&self) -> RuntimeStatus;

    /// Set the LLM provider.
    async fn set_provider(&self, provider: Arc<dyn Provider + Send + Sync>);

    /// Subscribe to runtime domain events.
    fn subscribe(&self) -> broadcast::Receiver<DomainEvent>;
}

/// A type-erased, reference-counted runtime handle suitable for passing across threads.
pub type DynRuntimeHandle = Arc<dyn RuntimeHandle>;

/// Trait for runtime that supports streaming responses.
#[async_trait]
pub trait StreamingRuntimeHandle: RuntimeHandle {
    /// Execute a command with streaming response.
    async fn execute_streaming(
        &self,
        command: RuntimeFacadeCommand,
    ) -> Result<
        Box<dyn Stream<Item = Result<RuntimeFacadeResponse, RuntimeFacadeError>> + Send>,
        RuntimeFacadeError,
    >;
}
