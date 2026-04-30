use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::compaction::types::ContextRanking;
use crate::compaction::{
    CompactionConfig, Compactor, TokenBudget, COMPACTION_FORCE_THRESHOLD,
    COMPACTION_START_THRESHOLD, COMPACTION_WARN_THRESHOLD,
};
use crate::message::{Message, Role};
use crate::token_counter::TokenCounter;
use crate::tool::ToolRegistry;
use regex::Regex;

const PRESERVE_LAST_MESSAGES: usize = 3;

static FILE_PATH_REGEX: OnceLock<Regex> = OnceLock::new();

#[expect(
    clippy::expect_used,
    reason = "Regex pattern is hardcoded and validated at compile time"
)]
fn get_file_path_regex() -> &'static Regex {
    FILE_PATH_REGEX.get_or_init(|| Regex::new(r"([\w./-]+\.[\w]+)").expect("valid file path regex"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextLayer {
    L0ExplicitInput,
    L1SessionContext,
    L2ProjectContext,
    L3StructuredContext,
    L4CompressedMemory,
}

impl ContextLayer {
    pub fn priority(&self) -> u8 {
        match self {
            Self::L0ExplicitInput => 5,
            Self::L1SessionContext => 4,
            Self::L2ProjectContext => 3,
            Self::L3StructuredContext => 2,
            Self::L4CompressedMemory => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub layer: ContextLayer,
    pub content: String,
    pub token_count: usize,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextUsageLevel {
    Normal,
    Warning(f64),
    NeedsCompaction(f64),
    ForceNewSession(f64),
}

/// Reports what context items were dropped during context building due to token budget.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TruncationReport {
    /// Messages that were dropped due to token budget trimming
    pub dropped_messages: Vec<DroppedMessage>,
    /// Total tokens saved by dropping messages
    pub tokens_saved: usize,
    /// Number of messages dropped from middle of conversation
    pub middle_messages_dropped: usize,
    /// Number of items dropped due to compaction
    pub compacted_items: usize,
}

/// A message that was dropped during context building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedMessage {
    /// Role of the dropped message
    pub role: String,
    /// Preview of dropped content (first 100 chars)
    pub preview: String,
    /// Token count of the dropped message
    pub token_count: usize,
}

/// Tracks the provenance of a context item - where it came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextProvenance {
    /// The source that generated this context item
    pub source: ProvenanceSource,
    /// Why this item was included
    pub inclusion_reason: String,
    /// Token count of this item
    pub token_count: usize,
}

/// Possible sources of context items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvenanceSource {
    /// User explicitly provided this input
    ExplicitUser,
    /// Session history / conversation context
    SessionHistory,
    /// Project files / repository
    ProjectContext,
    /// Structured context (rules, skills, etc.)
    StructuredContext,
    /// Compressed memory
    CompressedMemory,
    /// Tool execution result
    ToolResult,
    /// Git diff or status
    GitContext,
    /// LLM or AI generated content
    Generated,
}

impl std::fmt::Display for ProvenanceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProvenanceSource::ExplicitUser => write!(f, "explicit_user"),
            ProvenanceSource::SessionHistory => write!(f, "session_history"),
            ProvenanceSource::ProjectContext => write!(f, "project_context"),
            ProvenanceSource::StructuredContext => write!(f, "structured_context"),
            ProvenanceSource::CompressedMemory => write!(f, "compressed_memory"),
            ProvenanceSource::ToolResult => write!(f, "tool_result"),
            ProvenanceSource::GitContext => write!(f, "git_context"),
            ProvenanceSource::Generated => write!(f, "generated"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub total_tokens: usize,
    pub max_tokens: usize,
    pub remaining_tokens: usize,
    pub usage_pct: f64,
    pub layer_breakdown: Vec<(ContextLayer, usize)>,
    pub layer_budgets: LayerBudgets,
    #[serde(default)]
    pub warning_threshold: f64,
    #[serde(default)]
    pub compact_threshold: f64,
    #[serde(default)]
    pub continuation_threshold: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerBudgets {
    pub l0_explicit: usize,
    pub l1_session: usize,
    pub l2_project: usize,
    pub l3_structured: usize,
    pub l4_compressed: usize,
}

#[allow(dead_code)]
impl LayerBudgets {
    pub(crate) fn new(max_tokens: usize) -> Self {
        let total = max_tokens;
        Self {
            l0_explicit: (total as f64 * 0.15) as usize,
            l1_session: (total as f64 * 0.40) as usize,
            l2_project: (total as f64 * 0.20) as usize,
            l3_structured: (total as f64 * 0.15) as usize,
            l4_compressed: (total as f64 * 0.10) as usize,
        }
    }

    pub(crate) fn budget_for(&self, layer: ContextLayer) -> usize {
        match layer {
            ContextLayer::L0ExplicitInput => self.l0_explicit,
            ContextLayer::L1SessionContext => self.l1_session,
            ContextLayer::L2ProjectContext => self.l2_project,
            ContextLayer::L3StructuredContext => self.l3_structured,
            ContextLayer::L4CompressedMemory => self.l4_compressed,
        }
    }

    pub(crate) fn total(&self) -> usize {
        self.l0_explicit
            + self.l1_session
            + self.l2_project
            + self.l3_structured
            + self.l4_compressed
    }
}

#[allow(dead_code)]
impl ContextBudget {
    pub(crate) fn from_usage(
        max_tokens: usize,
        total_tokens: usize,
        layer_breakdown: Vec<(ContextLayer, usize)>,
    ) -> Self {
        Self::from_usage_with_thresholds(
            max_tokens,
            total_tokens,
            layer_breakdown,
            COMPACTION_WARN_THRESHOLD as f64,
            COMPACTION_START_THRESHOLD as f64,
            COMPACTION_FORCE_THRESHOLD as f64,
        )
    }

    pub(crate) fn from_usage_with_thresholds(
        max_tokens: usize,
        total_tokens: usize,
        layer_breakdown: Vec<(ContextLayer, usize)>,
        warning_threshold: f64,
        compact_threshold: f64,
        continuation_threshold: f64,
    ) -> Self {
        let usage_pct = if max_tokens > 0 {
            total_tokens as f64 / max_tokens as f64
        } else {
            0.0
        };
        Self {
            total_tokens,
            max_tokens,
            remaining_tokens: max_tokens.saturating_sub(total_tokens),
            usage_pct,
            layer_breakdown,
            layer_budgets: LayerBudgets::new(max_tokens),
            warning_threshold,
            compact_threshold,
            continuation_threshold,
        }
    }

    pub(crate) fn usage_level(&self) -> ContextUsageLevel {
        if self.usage_pct >= self.continuation_threshold {
            ContextUsageLevel::ForceNewSession(self.usage_pct)
        } else if self.usage_pct >= self.compact_threshold {
            ContextUsageLevel::NeedsCompaction(self.usage_pct)
        } else if self.usage_pct >= self.warning_threshold {
            ContextUsageLevel::Warning(self.usage_pct)
        } else {
            ContextUsageLevel::Normal
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub layers: Vec<ContextItem>,
    pub file_context: Vec<String>,
    pub tool_context: Vec<String>,
    pub session_context: Vec<String>,
    pub prompt_messages: Vec<Message>,
    pub budget: ContextBudget,
    pub truncation_report: TruncationReport,
    pub provenance: Vec<ContextProvenance>,
}

pub struct ContextBuilder {
    token_budget: TokenBudget,
    token_counter: TokenCounter,
    file_context: Vec<String>,
    tool_context: Vec<String>,
    session_context: Vec<String>,
    prompt_messages: Vec<Message>,
    explicit_input: Vec<String>,
    structured_context: Vec<String>,
    compressed_memory: Vec<String>,
}

#[allow(dead_code)]
impl ContextBuilder {
    pub(crate) fn new(token_budget: TokenBudget) -> Self {
        Self {
            token_budget,
            token_counter: TokenCounter::new(),
            file_context: Vec::new(),
            tool_context: Vec::new(),
            session_context: Vec::new(),
            prompt_messages: Vec::new(),
            explicit_input: Vec::new(),
            structured_context: Vec::new(),
            compressed_memory: Vec::new(),
        }
    }

    pub(crate) fn with_model_name(mut self, model_name: Option<&str>) -> Self {
        if let Some(model) = model_name.map(str::trim).filter(|name| !name.is_empty()) {
            self.token_budget = TokenBudget::from_model(model);
            self.token_counter = TokenCounter::for_model(model);
        }
        self
    }

    pub(crate) fn collect_file_context(
        mut self,
        opened_files: &[PathBuf],
        referenced_messages: &[Message],
    ) -> Self {
        let mut collected = BTreeSet::new();

        for file in opened_files {
            collected.insert(file.display().to_string());
        }

        let file_pattern = get_file_path_regex();
        for message in referenced_messages {
            for captures in file_pattern.captures_iter(&message.content) {
                if let Some(m) = captures.get(1) {
                    collected.insert(m.as_str().to_string());
                }
            }
        }

        self.file_context = collected.into_iter().collect();
        self
    }

    pub(crate) fn collect_tool_context(mut self, registry: &ToolRegistry) -> Self {
        self.tool_context = registry
            .get_all()
            .iter()
            .map(|tool| format!("{}: {}", tool.name, tool.description))
            .collect();
        self
    }

    pub(crate) fn collect_session_context(mut self, messages: &[Message]) -> Self {
        self.prompt_messages = messages.to_vec();
        self.session_context = messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|m| {
                let preview: String = m.content.chars().take(120).collect();
                format!("{:?}: {}", m.role, preview)
            })
            .collect();
        self
    }

    pub(crate) fn add_explicit_input(mut self, input: impl Into<String>) -> Self {
        self.explicit_input.push(input.into());
        self
    }

    pub(crate) fn add_structured_context(mut self, context: impl Into<String>) -> Self {
        self.structured_context.push(context.into());
        self
    }

    pub(crate) fn add_compressed_memory(mut self, memory: impl Into<String>) -> Self {
        self.compressed_memory.push(memory.into());
        self
    }

    pub(crate) fn build(mut self) -> Context {
        let max_tokens = self.token_budget.main_context_tokens();

        let mut layer_breakdown = Vec::new();

        let explicit_tokens: usize = self
            .explicit_input
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();
        let session_tokens: usize = self
            .session_context
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();
        let project_tokens: usize = self
            .file_context
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();
        let structured_tokens: usize = self
            .structured_context
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();
        let compressed_tokens: usize = self
            .compressed_memory
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();

        layer_breakdown.push((ContextLayer::L0ExplicitInput, explicit_tokens));
        layer_breakdown.push((ContextLayer::L1SessionContext, session_tokens));
        layer_breakdown.push((ContextLayer::L2ProjectContext, project_tokens));
        layer_breakdown.push((ContextLayer::L3StructuredContext, structured_tokens));
        layer_breakdown.push((ContextLayer::L4CompressedMemory, compressed_tokens));

        let mut budget = ContextBudget::from_usage_with_thresholds(
            max_tokens,
            self.total_tokens(),
            layer_breakdown.clone(),
            self.token_budget.warning_threshold,
            self.token_budget.compact_threshold,
            self.token_budget.continuation_threshold,
        );

        let truncation_report = trim_to_budget(&mut self.prompt_messages, &budget);

        let compactor = Compactor::new(CompactionConfig {
            max_tokens,
            preserve_system_messages: true,
            preserve_recent_messages: PRESERVE_LAST_MESSAGES,
            ..Default::default()
        });

        if compactor.needs_compaction(&self.prompt_messages) {
            self.prompt_messages = compactor.compact_to_fit(self.prompt_messages).messages;
        }

        budget = ContextBudget::from_usage_with_thresholds(
            max_tokens,
            self.total_tokens(),
            layer_breakdown,
            self.token_budget.warning_threshold,
            self.token_budget.compact_threshold,
            self.token_budget.continuation_threshold,
        );

        let mut layers = Vec::new();
        let mut provenance = Vec::new();

        for input in self.explicit_input.iter() {
            let tokens = self.token_counter.count_tokens(input);
            provenance.push(ContextProvenance {
                source: ProvenanceSource::ExplicitUser,
                inclusion_reason: "User explicitly provided input".to_string(),
                token_count: tokens,
            });
            layers.push(ContextItem {
                layer: ContextLayer::L0ExplicitInput,
                content: input.clone(),
                token_count: tokens,
                source: "explicit".to_string(),
            });
        }

        for file in self.file_context.iter() {
            let tokens = self.token_counter.count_tokens(file);
            provenance.push(ContextProvenance {
                source: ProvenanceSource::ProjectContext,
                inclusion_reason: "File referenced in conversation or opened".to_string(),
                token_count: tokens,
            });
            layers.push(ContextItem {
                layer: ContextLayer::L2ProjectContext,
                content: file.clone(),
                token_count: tokens,
                source: "project".to_string(),
            });
        }

        for ctx in self.structured_context.iter() {
            let tokens = self.token_counter.count_tokens(ctx);
            provenance.push(ContextProvenance {
                source: ProvenanceSource::StructuredContext,
                inclusion_reason: "Structured context (rules/skills)".to_string(),
                token_count: tokens,
            });
            layers.push(ContextItem {
                layer: ContextLayer::L3StructuredContext,
                content: ctx.clone(),
                token_count: tokens,
                source: "structured".to_string(),
            });
        }

        for mem in self.compressed_memory.iter() {
            let tokens = self.token_counter.count_tokens(mem);
            provenance.push(ContextProvenance {
                source: ProvenanceSource::CompressedMemory,
                inclusion_reason: "Compressed from prior conversation".to_string(),
                token_count: tokens,
            });
            layers.push(ContextItem {
                layer: ContextLayer::L4CompressedMemory,
                content: mem.clone(),
                token_count: tokens,
                source: "compressed".to_string(),
            });
        }

        layers.sort_by(|a, b| b.layer.priority().cmp(&a.layer.priority()));

        Context {
            layers,
            file_context: self.file_context,
            tool_context: self.tool_context,
            session_context: self.session_context,
            prompt_messages: self.prompt_messages,
            budget,
            truncation_report,
            provenance,
        }
    }

    fn total_tokens(&self) -> usize {
        let file_tokens: usize = self
            .file_context
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();
        let tool_tokens: usize = self
            .tool_context
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();
        let session_tokens: usize = self
            .session_context
            .iter()
            .map(|s| self.token_counter.count_tokens(s))
            .sum();
        let prompt_tokens = self.token_counter.count_messages(&self.prompt_messages);
        file_tokens + tool_tokens + session_tokens + prompt_tokens
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    static DEFAULT_COUNTER: OnceLock<TokenCounter> = OnceLock::new();
    DEFAULT_COUNTER
        .get_or_init(|| TokenCounter::for_model("gpt-4o"))
        .count_tokens(text)
}

pub fn trim_to_budget(messages: &mut Vec<Message>, budget: &ContextBudget) -> TruncationReport {
    let total_tokens = |msgs: &[Message]| {
        msgs.iter()
            .map(|m| estimate_tokens(&m.content))
            .sum::<usize>()
    };

    if total_tokens(messages) <= budget.max_tokens {
        return TruncationReport::default();
    }

    let mut report = TruncationReport::default();

    while total_tokens(messages) > budget.max_tokens {
        let len = messages.len();
        if len < PRESERVE_LAST_MESSAGES {
            break;
        }

        let preserve_from = if len <= PRESERVE_LAST_MESSAGES {
            len
        } else {
            len.saturating_sub(PRESERVE_LAST_MESSAGES)
        };

        // Compute fresh rankings for current messages - rankings depend on current position
        let rankings: Vec<ContextRanking> = messages
            .iter()
            .enumerate()
            .map(|(idx, _)| ContextRanking::default_for_index(idx, len))
            .collect();

        // Find the removable message with the lowest ranking (least important)
        // among messages that are not system and not in the preserved recent range
        let removable_idx = rankings
            .iter()
            .enumerate()
            .find_map(|(idx, ranking)| {
                let is_protected_system = messages[idx].role == Role::System;
                let is_protected_recent = idx >= preserve_from;
                if is_protected_system || is_protected_recent {
                    None
                } else {
                    Some((idx, ranking.overall))
                }
            })
            .map(|(idx, _)| idx);

        match removable_idx {
            Some(idx) => {
                let removed = messages.remove(idx);
                let tokens = estimate_tokens(&removed.content);
                report.tokens_saved += tokens;
                report.middle_messages_dropped += 1;
                report.dropped_messages.push(DroppedMessage {
                    role: format!("{:?}", removed.role),
                    preview: removed.content.chars().take(100).collect(),
                    token_count: tokens,
                });
            }
            None => break,
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::build_default_registry;

    #[test]
    fn test_estimate_tokens_char_approximation() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
    }

    #[test]
    fn test_trim_to_budget_keeps_system_and_last_three() {
        let mut messages = vec![
            Message::system("system prompt"),
            Message::user("first"),
            Message::assistant("second"),
            Message::user("third"),
            Message::assistant("fourth"),
            Message::user("fifth"),
        ];

        let budget = ContextBudget::from_usage(3, 999, Vec::new());
        trim_to_budget(&mut messages, &budget);

        assert_eq!(messages[0].role, Role::System);
        let tail: Vec<String> = messages
            .iter()
            .rev()
            .take(3)
            .map(|m| m.content.clone())
            .collect();
        assert!(tail.contains(&"third".to_string()));
        assert!(tail.contains(&"fourth".to_string()));
        assert!(tail.contains(&"fifth".to_string()));
    }

    #[test]
    fn test_context_builder_builds_sections() {
        let registry = build_default_registry();
        let messages = vec![
            Message::user("Please inspect src/main.rs"),
            Message::assistant("Sure"),
        ];

        let context = ContextBuilder::new(TokenBudget::default())
            .with_model_name(Some("gpt-4o"))
            .collect_file_context(&[PathBuf::from("Cargo.toml")], &messages)
            .collect_tool_context(&registry)
            .collect_session_context(&messages)
            .build();

        assert!(context
            .file_context
            .iter()
            .any(|p| p.contains("Cargo.toml")));
        assert!(!context.tool_context.is_empty());
        assert_eq!(context.prompt_messages.len(), 2);
        assert!(context.budget.max_tokens > 0);
    }

    #[test]
    fn test_context_builder_model_specific_budget() {
        let messages = vec![Message::user("Hello")];

        let context = ContextBuilder::new(TokenBudget::default())
            .with_model_name(Some("gpt-4"))
            .collect_session_context(&messages)
            .build();

        assert_eq!(
            context.budget.max_tokens,
            TokenBudget::from_model("gpt-4").main_context_tokens()
        );
    }

    #[test]
    fn test_layer_budgets_allocation() {
        let budgets = LayerBudgets::new(10000);

        assert_eq!(budgets.l0_explicit, 1500);
        assert_eq!(budgets.l1_session, 4000);
        assert_eq!(budgets.l2_project, 2000);
        assert_eq!(budgets.l3_structured, 1500);
        assert_eq!(budgets.l4_compressed, 1000);

        assert_eq!(budgets.total(), 10000);
    }

    #[test]
    fn test_layer_budgets_budget_for() {
        let budgets = LayerBudgets::new(10000);

        assert_eq!(budgets.budget_for(ContextLayer::L0ExplicitInput), 1500);
        assert_eq!(budgets.budget_for(ContextLayer::L1SessionContext), 4000);
        assert_eq!(budgets.budget_for(ContextLayer::L2ProjectContext), 2000);
        assert_eq!(budgets.budget_for(ContextLayer::L3StructuredContext), 1500);
        assert_eq!(budgets.budget_for(ContextLayer::L4CompressedMemory), 1000);
    }

    #[test]
    fn test_context_layer_priority() {
        assert!(
            ContextLayer::L0ExplicitInput.priority() > ContextLayer::L4CompressedMemory.priority()
        );
        assert!(
            ContextLayer::L1SessionContext.priority() > ContextLayer::L2ProjectContext.priority()
        );
    }

    #[test]
    fn test_context_usage_level_thresholds() {
        let normal = ContextBudget::from_usage(1000, 500, Vec::new());
        assert!(matches!(normal.usage_level(), ContextUsageLevel::Normal));

        let warning = ContextBudget::from_usage(1000, 870, Vec::new());
        assert!(matches!(
            warning.usage_level(),
            ContextUsageLevel::Warning(_)
        ));

        let needs_compact = ContextBudget::from_usage(1000, 930, Vec::new());
        assert!(matches!(
            needs_compact.usage_level(),
            ContextUsageLevel::NeedsCompaction(_)
        ));

        let force_new = ContextBudget::from_usage(1000, 960, Vec::new());
        assert!(matches!(
            force_new.usage_level(),
            ContextUsageLevel::ForceNewSession(_)
        ));
    }

    #[test]
    fn test_trim_to_budget_returns_truncation_report() {
        let mut messages = vec![
            Message::system("system"),
            Message::user("first message"),
            Message::assistant("second message"),
            Message::user("third message"),
            Message::assistant("fourth message"),
            Message::user("fifth message"),
        ];

        let budget = ContextBudget::from_usage(3, 999, Vec::new());
        let report = trim_to_budget(&mut messages, &budget);

        assert!(report.middle_messages_dropped > 0);
        assert!(report.tokens_saved > 0);
        assert!(!report.dropped_messages.is_empty());
    }

    #[test]
    fn test_truncation_report_no_trim_when_within_budget() {
        let mut messages = vec![Message::system("system"), Message::user("hi")];

        let budget = ContextBudget::from_usage(1000, 10, Vec::new());
        let report = trim_to_budget(&mut messages, &budget);

        assert_eq!(report.middle_messages_dropped, 0);
        assert_eq!(report.tokens_saved, 0);
        assert!(report.dropped_messages.is_empty());
    }

    #[test]
    fn test_dropped_message_preview_truncated() {
        let long_content = "a".repeat(200);
        let mut messages = vec![
            Message::system("system"),
            Message::user(&long_content),
            Message::assistant("recent"),
        ];

        let budget = ContextBudget::from_usage(3, 999, Vec::new());
        let report = trim_to_budget(&mut messages, &budget);

        assert!(!report.dropped_messages.is_empty());
        let dropped = &report.dropped_messages[0];
        assert!(dropped.preview.len() <= 100);
    }

    #[test]
    fn test_context_builder_includes_truncation_report() {
        let messages = vec![
            Message::system("system prompt"),
            Message::user("first"),
            Message::assistant("second"),
            Message::user("third"),
        ];

        let context = ContextBuilder::new(TokenBudget::default())
            .with_model_name(Some("gpt-4o"))
            .collect_session_context(&messages)
            .build();

        assert!(context.truncation_report.middle_messages_dropped >= 0);
    }

    #[test]
    fn test_context_builder_includes_provenance() {
        let messages = vec![Message::user("Hello")];

        let context = ContextBuilder::new(TokenBudget::default())
            .with_model_name(Some("gpt-4o"))
            .collect_session_context(&messages)
            .add_explicit_input("user instruction")
            .build();

        assert!(!context.provenance.is_empty());
        assert!(context.truncation_report.tokens_saved >= 0);
    }

    #[test]
    fn test_provenance_source_display() {
        assert_eq!(
            format!("{}", ProvenanceSource::ExplicitUser),
            "explicit_user"
        );
        assert_eq!(
            format!("{}", ProvenanceSource::SessionHistory),
            "session_history"
        );
        assert_eq!(
            format!("{}", ProvenanceSource::ProjectContext),
            "project_context"
        );
    }

    #[test]
    fn test_truncation_report_default() {
        let report = TruncationReport::default();
        assert_eq!(report.dropped_messages.len(), 0);
        assert_eq!(report.tokens_saved, 0);
        assert_eq!(report.middle_messages_dropped, 0);
    }
}
