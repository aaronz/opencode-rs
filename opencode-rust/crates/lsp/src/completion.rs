use serde::{Deserialize, Serialize};

use crate::types::{CompletionItem as CompletionItemType, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionTriggerKind {
    Invoked,
    TriggerCharacter,
    Retrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTriggerContext {
    pub trigger_kind: CompletionTriggerKind,
    pub trigger_character: Option<char>,
}

impl Default for CompletionTriggerContext {
    fn default() -> Self {
        Self {
            trigger_kind: CompletionTriggerKind::Invoked,
            trigger_character: None,
        }
    }
}

impl CompletionTriggerContext {
    pub fn new(trigger_kind: CompletionTriggerKind, trigger_character: Option<char>) -> Self {
        Self {
            trigger_kind,
            trigger_character,
        }
    }

    pub fn is_triggered_by_dot(&self) -> bool {
        self.trigger_character == Some('.')
    }

    pub fn is_triggered_by_double_colon(&self) -> bool {
        self.trigger_character == Some(':')
    }

    pub fn is_method_completion(&self) -> bool {
        self.is_triggered_by_dot() || self.is_triggered_by_double_colon()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    pub uri: String,
    pub position: Position,
    pub context: Option<CompletionTriggerContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionResult {
    pub items: Vec<CompletionItemType>,
    pub is_incomplete: bool,
}

pub fn create_completion_item(
    label: impl Into<String>,
    kind: Option<u32>,
    detail: Option<String>,
    insert_text: Option<String>,
) -> CompletionItemType {
    CompletionItemType {
        label: label.into(),
        kind,
        detail,
        insert_text,
    }
}

pub fn filter_completions_by_prefix(
    items: &[CompletionItemType],
    prefix: &str,
) -> Vec<CompletionItemType> {
    if prefix.is_empty() {
        return items.to_vec();
    }

    items
        .iter()
        .filter(|item| item.label.starts_with(prefix))
        .cloned()
        .collect()
}

pub fn filter_completions_by_context(
    items: &[CompletionItemType],
    context: &CompletionTriggerContext,
) -> Vec<CompletionItemType> {
    if context.is_method_completion() {
        items
            .iter()
            .filter(|item| {
                item.label.starts_with(|c: char| c.is_lowercase())
                    || item.label.starts_with("self")
                    || item.label.starts_with("Self")
            })
            .cloned()
            .collect()
    } else {
        items.to_vec()
    }
}

pub fn get_completion_trigger_character(context: Option<&CompletionTriggerContext>) -> Option<char> {
    context.and_then(|c| c.trigger_character)
}

pub fn handle_completion_trigger(context: Option<CompletionTriggerContext>) -> CompletionTriggerKind {
    context
        .map(|c| c.trigger_kind)
        .unwrap_or(CompletionTriggerKind::Invoked)
}

pub fn build_method_completion_items() -> Vec<CompletionItemType> {
    vec![
        create_completion_item(
            "clone",
            Some(1),
            Some("Returns a clone of the value".to_string()),
            Some("clone()".to_string()),
        ),
        create_completion_item(
            "unwrap",
            Some(1),
            Some("Unwraps the value or panics".to_string()),
            Some("unwrap()".to_string()),
        ),
        create_completion_item(
            "expect",
            Some(1),
            Some("Unwraps the value with a message".to_string()),
            Some("expect(msg)".to_string()),
        ),
        create_completion_item(
            "map",
            Some(1),
            Some("Maps the value using a closure".to_string()),
            Some("map(|x| x)".to_string()),
        ),
        create_completion_item(
            "and_then",
            Some(1),
            Some("Chains another operation".to_string()),
            Some("and_then(|x| x)".to_string()),
        ),
        create_completion_item(
            "is_some",
            Some(2),
            Some("Returns true if the value is Some".to_string()),
            None,
        ),
        create_completion_item(
            "is_none",
            Some(2),
            Some("Returns true if the value is None".to_string()),
            None,
        ),
        create_completion_item(
            "is_err",
            Some(2),
            Some("Returns true if the value is Err".to_string()),
            None,
        ),
        create_completion_item(
            "is_ok",
            Some(2),
            Some("Returns true if the value is Ok".to_string()),
            None,
        ),
    ]
}

pub fn build_keyword_completion_items() -> Vec<CompletionItemType> {
    vec![
        create_completion_item(
            "fn",
            Some(3),
            Some("Function definition".to_string()),
            Some("fn ".to_string()),
        ),
        create_completion_item(
            "let",
            Some(3),
            Some("Variable binding".to_string()),
            Some("let ".to_string()),
        ),
        create_completion_item(
            "mut",
            Some(3),
            Some("Mutable binding".to_string()),
            Some("mut ".to_string()),
        ),
        create_completion_item(
            "pub",
            Some(3),
            Some("Public visibility".to_string()),
            Some("pub ".to_string()),
        ),
        create_completion_item(
            "use",
            Some(3),
            Some("Import statement".to_string()),
            Some("use ".to_string()),
        ),
        create_completion_item(
            "struct",
            Some(3),
            Some("Structure definition".to_string()),
            Some("struct ".to_string()),
        ),
        create_completion_item(
            "enum",
            Some(3),
            Some("Enumeration definition".to_string()),
            Some("enum ".to_string()),
        ),
        create_completion_item(
            "impl",
            Some(3),
            Some("Implementation block".to_string()),
            Some("impl ".to_string()),
        ),
        create_completion_item(
            "trait",
            Some(3),
            Some("Trait definition".to_string()),
            Some("trait ".to_string()),
        ),
        create_completion_item(
            "match",
            Some(3),
            Some("Pattern matching".to_string()),
            Some("match ".to_string()),
        ),
        create_completion_item(
            "if",
            Some(3),
            Some("Conditional expression".to_string()),
            Some("if ".to_string()),
        ),
        create_completion_item(
            "else",
            Some(3),
            Some("Else branch".to_string()),
            Some("else ".to_string()),
        ),
        create_completion_item(
            "while",
            Some(3),
            Some("While loop".to_string()),
            Some("while ".to_string()),
        ),
        create_completion_item(
            "for",
            Some(3),
            Some("For loop".to_string()),
            Some("for ".to_string()),
        ),
        create_completion_item(
            "loop",
            Some(3),
            Some("Infinite loop".to_string()),
            Some("loop ".to_string()),
        ),
        create_completion_item(
            "return",
            Some(3),
            Some("Return statement".to_string()),
            Some("return ".to_string()),
        ),
        create_completion_item(
            "async",
            Some(3),
            Some("Async function".to_string()),
            Some("async ".to_string()),
        ),
        create_completion_item(
            "await",
            Some(3),
            Some("Await expression".to_string()),
            Some("await ".to_string()),
        ),
    ]
}

pub fn get_completions(params: &CompletionParams) -> CompletionResult {
    let context = params.context.as_ref();
    let trigger_kind = handle_completion_trigger(params.context.clone());

    let items = match trigger_kind {
        CompletionTriggerKind::TriggerCharacter => {
            if context.map(|c| c.is_method_completion()).unwrap_or(false) {
                build_method_completion_items()
            } else {
                build_keyword_completion_items()
            }
        }
        _ => build_keyword_completion_items(),
    };

    CompletionResult {
        items,
        is_incomplete: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_item_struct_has_required_fields() {
        let item = create_completion_item(
            "test_function",
            Some(1),
            Some("A test function".to_string()),
            Some("test_function()".to_string()),
        );

        assert_eq!(item.label, "test_function");
        assert_eq!(item.kind, Some(1));
        assert_eq!(item.detail, Some("A test function".to_string()));
        assert_eq!(item.insert_text, Some("test_function()".to_string()));
    }

    #[test]
    fn test_completion_item_with_only_label() {
        let item = create_completion_item("fn", None, None, None);

        assert_eq!(item.label, "fn");
        assert_eq!(item.kind, None);
        assert_eq!(item.detail, None);
        assert_eq!(item.insert_text, None);
    }

    #[test]
    fn test_completion_trigger_context_dot() {
        let context = CompletionTriggerContext::new(
            CompletionTriggerKind::TriggerCharacter,
            Some('.'),
        );

        assert!(context.is_triggered_by_dot());
        assert!(!context.is_triggered_by_double_colon());
        assert!(context.is_method_completion());
    }

    #[test]
    fn test_completion_trigger_context_double_colon() {
        let context = CompletionTriggerContext::new(
            CompletionTriggerKind::TriggerCharacter,
            Some(':'),
        );

        assert!(!context.is_triggered_by_dot());
        assert!(context.is_triggered_by_double_colon());
        assert!(context.is_method_completion());
    }

    #[test]
    fn test_completion_trigger_context_default() {
        let context = CompletionTriggerContext::default();

        assert_eq!(context.trigger_kind, CompletionTriggerKind::Invoked);
        assert!(context.trigger_character.is_none());
        assert!(!context.is_method_completion());
    }

    #[test]
    fn test_filter_completions_by_prefix() {
        let items = vec![
            create_completion_item("fn", Some(1), None, None),
            create_completion_item("for", Some(1), None, None),
            create_completion_item("format", Some(1), None, None),
            create_completion_item("if", Some(1), None, None),
        ];

        let filtered = filter_completions_by_prefix(&items, "f");

        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|i| i.label.starts_with('f')));
    }

    #[test]
    fn test_filter_completions_by_prefix_empty() {
        let items = vec![
            create_completion_item("fn", Some(1), None, None),
            create_completion_item("for", Some(1), None, None),
        ];

        let filtered = filter_completions_by_prefix(&items, "");

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_completions_by_prefix_no_match() {
        let items = vec![
            create_completion_item("fn", Some(1), None, None),
            create_completion_item("for", Some(1), None, None),
        ];

        let filtered = filter_completions_by_prefix(&items, "xyz");

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_completions_by_context_method() {
        let context = CompletionTriggerContext::new(
            CompletionTriggerKind::TriggerCharacter,
            Some('.'),
        );

        let items = vec![
            create_completion_item("clone", Some(1), None, None),
            create_completion_item("CLONE", Some(1), None, None),
            create_completion_item("self", Some(1), None, None),
            create_completion_item("Self", Some(1), None, None),
            create_completion_item("let", Some(1), None, None),
        ];

        let filtered = filter_completions_by_context(&items, &context);

        assert_eq!(filtered.len(), 4);
        assert!(filtered.iter().all(|i| {
            i.label.starts_with(|c: char| c.is_lowercase()) || i.label.starts_with("self") || i.label.starts_with("Self")
        }));
    }

    #[test]
    fn test_filter_completions_by_context_non_method() {
        let context = CompletionTriggerContext::new(
            CompletionTriggerKind::Invoked,
            None,
        );

        let items = vec![
            create_completion_item("clone", Some(1), None, None),
            create_completion_item("let", Some(1), None, None),
        ];

        let filtered = filter_completions_by_context(&items, &context);

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_get_completion_trigger_character() {
        let context = CompletionTriggerContext::new(
            CompletionTriggerKind::TriggerCharacter,
            Some('.'),
        );

        let char = get_completion_trigger_character(Some(&context));
        assert_eq!(char, Some('.'));

        let char = get_completion_trigger_character(None);
        assert_eq!(char, None);
    }

    #[test]
    fn test_handle_completion_trigger() {
        let context = CompletionTriggerContext::new(
            CompletionTriggerKind::TriggerCharacter,
            Some('.'),
        );

        let trigger = handle_completion_trigger(Some(context));
        assert_eq!(trigger, CompletionTriggerKind::TriggerCharacter);

        let trigger = handle_completion_trigger(None);
        assert_eq!(trigger, CompletionTriggerKind::Invoked);
    }

    #[test]
    fn test_build_method_completion_items() {
        let items = build_method_completion_items();

        assert!(!items.is_empty());
        assert!(items.iter().all(|i| {
            i.label.starts_with(|c: char| c.is_lowercase())
                || i.label.starts_with("self")
                || i.label.starts_with("Self")
        }));
    }

    #[test]
    fn test_build_keyword_completion_items() {
        let items = build_keyword_completion_items();

        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.label == "fn"));
        assert!(items.iter().any(|i| i.label == "let"));
        assert!(items.iter().any(|i| i.label == "mut"));
        assert!(items.iter().any(|i| i.label == "pub"));
    }

    #[test]
    fn test_get_completions_with_trigger() {
        let params = CompletionParams {
            uri: "file:///test.rs".to_string(),
            position: Position { line: 0, character: 5 },
            context: Some(CompletionTriggerContext::new(
                CompletionTriggerKind::TriggerCharacter,
                Some('.'),
            )),
        };

        let result = get_completions(&params);

        assert!(!result.items.is_empty());
        assert!(result.items.iter().all(|i| {
            i.label.starts_with(|c: char| c.is_lowercase())
                || i.label.starts_with("self")
                || i.label.starts_with("Self")
        }));
    }

    #[test]
    fn test_get_completions_without_trigger() {
        let params = CompletionParams {
            uri: "file:///test.rs".to_string(),
            position: Position { line: 0, character: 5 },
            context: None,
        };

        let result = get_completions(&params);

        assert!(!result.items.is_empty());
        assert!(result.items.iter().any(|i| i.label == "fn"));
    }

    #[test]
    fn test_completion_result_default() {
        let result = CompletionResult::default();

        assert!(result.items.is_empty());
        assert!(!result.is_incomplete);
    }

    #[test]
    fn test_create_completion_item_all_fields() {
        let item = CompletionItemType {
            label: "test".to_string(),
            kind: Some(14),
            detail: Some("Test detail".to_string()),
            insert_text: Some("test()".to_string()),
        };

        assert_eq!(item.label, "test");
        assert_eq!(item.kind, Some(14));
        assert_eq!(item.detail, Some("Test detail".to_string()));
        assert_eq!(item.insert_text, Some("test()".to_string()));
    }
}