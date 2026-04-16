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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_transform_default() {
        let transform = MessageTransform::default();
        assert!(transform.trim_whitespace);
        assert!(transform.max_length.is_none());
        assert!(transform.prefix.is_none());
        assert!(transform.suffix.is_none());
    }

    #[test]
    fn test_message_transform_apply_no_op() {
        let transform = MessageTransform::default();
        let result = transform.apply("hello world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_message_transform_trim_whitespace() {
        let transform = MessageTransform {
            trim_whitespace: true,
            ..Default::default()
        };
        let result = transform.apply("  hello world  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_message_transform_no_trim() {
        let transform = MessageTransform {
            trim_whitespace: false,
            ..Default::default()
        };
        let result = transform.apply("  hello world  ");
        assert_eq!(result, "  hello world  ");
    }

    #[test]
    fn test_message_transform_max_length_truncate() {
        let transform = MessageTransform {
            trim_whitespace: false,
            max_length: Some(10),
            ..Default::default()
        };
        let result = transform.apply("hello world");
        assert_eq!(result, "hello worl...");
    }

    #[test]
    fn test_message_transform_max_length_no_truncate() {
        let transform = MessageTransform {
            trim_whitespace: false,
            max_length: Some(20),
            ..Default::default()
        };
        let result = transform.apply("hello world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_message_transform_max_length_exact() {
        let transform = MessageTransform {
            trim_whitespace: false,
            max_length: Some(11),
            ..Default::default()
        };
        let result = transform.apply("hello world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_message_transform_prefix() {
        let transform = MessageTransform {
            trim_whitespace: false,
            max_length: None,
            prefix: Some("[".to_string()),
            suffix: None,
        };
        let result = transform.apply("hello");
        assert_eq!(result, "[hello");
    }

    #[test]
    fn test_message_transform_suffix() {
        let transform = MessageTransform {
            trim_whitespace: false,
            max_length: None,
            prefix: None,
            suffix: Some("]".to_string()),
        };
        let result = transform.apply("hello");
        assert_eq!(result, "hello]");
    }

    #[test]
    fn test_message_transform_prefix_and_suffix() {
        let transform = MessageTransform {
            trim_whitespace: false,
            max_length: None,
            prefix: Some("<".to_string()),
            suffix: Some(">".to_string()),
        };
        let result = transform.apply("hello");
        assert_eq!(result, "<hello>");
    }

    #[test]
    fn test_message_transform_all_options() {
        let transform = MessageTransform {
            trim_whitespace: true,
            max_length: Some(10),
            prefix: Some("[".to_string()),
            suffix: Some("]".to_string()),
        };
        let result = transform.apply("  hello world  ");
        assert_eq!(result, "[hello worl...]");
    }

    #[test]
    fn test_transform_pipeline_new() {
        let pipeline = TransformPipeline::new();
        assert!(pipeline.transforms.is_empty());
    }

    #[test]
    fn test_transform_pipeline_add_transform() {
        let mut pipeline = TransformPipeline::new();
        pipeline.add_transform(MessageTransform::default());
        assert_eq!(pipeline.transforms.len(), 1);
    }

    #[test]
    fn test_transform_pipeline_apply_empty() {
        let pipeline = TransformPipeline::new();
        let result = pipeline.apply("hello");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_transform_pipeline_apply_single_transform() {
        let mut pipeline = TransformPipeline::new();
        pipeline.add_transform(MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: None,
            suffix: None,
        });
        let result = pipeline.apply("  hello  ");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_transform_pipeline_apply_multiple_transforms() {
        let mut pipeline = TransformPipeline::new();
        pipeline.add_transform(MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: Some("[".to_string()),
            suffix: None,
        });
        pipeline.add_transform(MessageTransform {
            trim_whitespace: false,
            max_length: Some(8),
            prefix: None,
            suffix: Some("]".to_string()),
        });
        let result = pipeline.apply("  hello world  ");
        assert_eq!(result, "[hello...]");
    }

    #[test]
    fn test_transform_pipeline_chained_trimming() {
        let mut pipeline = TransformPipeline::new();
        pipeline.add_transform(MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: None,
            suffix: None,
        });
        pipeline.add_transform(MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: None,
            suffix: None,
        });
        let result = pipeline.apply("  hello  ");
        assert_eq!(result, "hello");
    }
}
