use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTransform {
    pub trim_whitespace: bool,
    pub max_length: Option<usize>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl Default for MessageTransform {
    fn default() -> Self {
        Self {
            trim_whitespace: true,
            max_length: None,
            prefix: None,
            suffix: None,
        }
    }
}

impl MessageTransform {
    pub fn apply(&self, message: &str) -> String {
        let mut result = message.to_string();

        if self.trim_whitespace {
            result = result.trim().to_string();
        }

        if let Some(max_length) = self.max_length {
            if result.len() > max_length {
                result.truncate(max_length);
                result.push_str("...");
            }
        }

        if let Some(prefix) = &self.prefix {
            result = format!("{}{}", prefix, result);
        }

        if let Some(suffix) = &self.suffix {
            result = format!("{}{}", result, suffix);
        }

        result
    }
}

pub struct TransformPipeline {
    transforms: Vec<MessageTransform>,
}

impl TransformPipeline {
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    pub fn add_transform(&mut self, transform: MessageTransform) {
        self.transforms.push(transform);
    }

    pub fn apply(&self, message: &str) -> String {
        let mut result = message.to_string();
        for transform in &self.transforms {
            result = transform.apply(&result);
        }
        result
    }
}

impl Default for TransformPipeline {
    fn default() -> Self {
        Self::new()
    }
}
