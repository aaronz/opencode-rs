use opencode_core::context::{Context, ContextLayer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeContextSummary {
    pub total_tokens: usize,
    pub max_tokens: usize,
    pub remaining_tokens: usize,
    pub usage_pct: f64,
    pub layer_count: usize,
    pub file_count: usize,
    pub tool_count: usize,
    pub session_count: usize,
    pub prompt_message_count: usize,
    pub layer_breakdown: Vec<(ContextLayer, usize)>,
}

impl RuntimeContextSummary {
    pub fn from_context(context: &Context) -> Self {
        Self {
            total_tokens: context.budget.total_tokens,
            max_tokens: context.budget.max_tokens,
            remaining_tokens: context.budget.remaining_tokens,
            usage_pct: context.budget.usage_pct,
            layer_count: context.layers.len(),
            file_count: context.file_context.len(),
            tool_count: context.tool_context.len(),
            session_count: context.session_context.len(),
            prompt_message_count: context.prompt_messages.len(),
            layer_breakdown: context.budget.layer_breakdown.clone(),
        }
    }
}
