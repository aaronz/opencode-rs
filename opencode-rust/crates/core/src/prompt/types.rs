use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub content: String,
    pub variables: Vec<String>,
}

pub struct PromptManager {
    pub(crate) templates: std::collections::HashMap<String, PromptTemplate>,
}