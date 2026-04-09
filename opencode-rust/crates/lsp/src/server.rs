use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, String>>>,
}

impl Backend {
    fn get_document(&self, uri: &Url) -> Option<String> {
        self.documents.read().get(uri).cloned()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "opencode-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "OpenCode LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let source = params.text_document.text;

        self.documents.write().insert(uri.clone(), source);

        self.client
            .log_message(MessageType::INFO, &format!("Opened document: {}", uri))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        if let Some(content) = params.content_changes.last() {
            self.documents
                .write()
                .insert(uri.clone(), content.text.clone());
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                &format!("Saved: {}", params.text_document.uri),
            )
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.documents.write().remove(&uri);
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = vec![
            CompletionItem::new_simple("fn".to_string(), "Function definition".to_string()),
            CompletionItem::new_simple("let".to_string(), "Variable binding".to_string()),
            CompletionItem::new_simple("mut".to_string(), "Mutable binding".to_string()),
            CompletionItem::new_simple("pub".to_string(), "Public visibility".to_string()),
            CompletionItem::new_simple("use".to_string(), "Import statement".to_string()),
            CompletionItem::new_simple("struct".to_string(), "Structure definition".to_string()),
            CompletionItem::new_simple("enum".to_string(), "Enumeration definition".to_string()),
            CompletionItem::new_simple("impl".to_string(), "Implementation block".to_string()),
            CompletionItem::new_simple("trait".to_string(), "Trait definition".to_string()),
            CompletionItem::new_simple("match".to_string(), "Pattern matching".to_string()),
        ];

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();

        if let Some(source) = self.get_document(&uri) {
            let position = params.text_document_position_params.position;
            let lines: Vec<&str> = source.lines().collect();

            if (position.line as usize) < lines.len() {
                let line = lines[position.line as usize];
                let word = extract_word_at_position(line, position.character as usize);

                if !word.is_empty() {
                    let word_type = classify_word(&source, &word, position.line as usize);
                    let hover_text = format!("{}{}", word, word_type);

                    return Ok(Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(hover_text)),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();

        if let Some(source) = self.get_document(&uri) {
            let position = params.text_document_position_params.position;
            let lines: Vec<&str> = source.lines().collect();

            if (position.line as usize) < lines.len() {
                let line = lines[position.line as usize];
                let word = extract_word_at_position(line, position.character as usize);

                if !word.is_empty() {
                    if let Some(def_line) = find_definition_line(&source, &word) {
                        let range = Range::new(
                            Position::new(def_line as u32, 0),
                            Position::new(def_line as u32, 0),
                        );
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                            uri, range,
                        ))));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();

        if let Some(source) = self.get_document(&uri) {
            let position = params.text_document_position.position;
            let lines: Vec<&str> = source.lines().collect();

            if (position.line as usize) < lines.len() {
                let line = lines[position.line as usize];
                let word = extract_word_at_position(line, position.character as usize);

                let locations = find_all_references(&source, &word);
                let result: Vec<Location> = locations
                    .into_iter()
                    .map(|(line, col)| {
                        Location::new(
                            uri.clone(),
                            Range::new(
                                Position::new(line as u32, col as u32),
                                Position::new(line as u32, (col + word.len()) as u32),
                            ),
                        )
                    })
                    .collect();

                return Ok(Some(result));
            }
        }

        Ok(Some(Vec::new()))
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let _uri = params.text_document.uri.clone();
        let diagnostics = params.context.diagnostics;

        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        for diag in diagnostics {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Ignore this diagnostic".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diag]),
                ..Default::default()
            }));
        }

        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Extract to function".to_string(),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            ..Default::default()
        }));

        Ok(Some(actions))
    }
}

fn extract_word_at_position(line: &str, col: usize) -> String {
    let mut start = col;
    let mut end = col;

    let chars: Vec<char> = line.chars().collect();

    while start > 0 && chars[start - 1].is_alphanumeric() {
        start -= 1;
    }

    while end < chars.len() && chars[end].is_alphanumeric() {
        end += 1;
    }

    chars[start..end].iter().collect()
}

fn classify_word(source: &str, word: &str, _current_line: usize) -> String {
    let keywords = [
        "fn", "let", "mut", "pub", "use", "struct", "enum", "impl", "trait", "match", "mod",
        "crate", "self", "super", "async", "await", "move", "ref", "static", "const", "type",
        "where", "for", "loop", "while", "if", "else", "return", "break", "continue",
    ];

    if keywords.contains(&word) {
        return " (keyword)".to_string();
    }

    let primitive_types = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Arc",
        "Rc",
    ];

    if primitive_types.contains(&word) {
        return " (type)".to_string();
    }

    let lines: Vec<&str> = source.lines().collect();
    for line in &lines {
        if line.contains(&format!("fn {}", word)) || line.contains(&format!("pub fn {}", word)) {
            if line.contains("->") {
                return " (function)".to_string();
            }
        }
        if line.contains(&format!("struct {}", word)) {
            return " (struct)".to_string();
        }
        if line.contains(&format!("enum {}", word)) {
            return " (enum)".to_string();
        }
        if line.contains(&format!("trait {}", word)) {
            return " (trait)".to_string();
        }
        if line.contains(&format!("impl{}", word)) || line.contains(&format!("impl {}", word)) {
            return " (impl block)".to_string();
        }
        if line.starts_with(&format!("const {}", word))
            || line.starts_with(&format!("static {}", word))
        {
            return " (constant)".to_string();
        }
    }

    for line in &lines {
        if line.contains(&format!("let {} ", word)) || line.contains(&format!("let mut {} ", word))
        {
            return " (variable)".to_string();
        }
    }

    String::new()
}

fn find_definition_line(source: &str, _word: &str) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains("fn ") || line.contains("struct ") || line.contains("enum ") {
            return Some(i);
        }
    }

    None
}

fn find_all_references(source: &str, word: &str) -> Vec<(usize, usize)> {
    let mut refs = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let mut col = 0;
        while let Some(pos) = line[col..].find(word) {
            refs.push((line_idx, col + pos));
            col += pos + 1;
        }
    }

    refs
}

pub struct LspServer {
    pub backend: Arc<Backend>,
}

impl LspServer {
    pub fn new(client: Client) -> Self {
        let backend = Arc::new(Backend {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        });

        Self { backend }
    }
}
