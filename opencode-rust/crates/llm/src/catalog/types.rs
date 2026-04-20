use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCatalog {
    pub providers: BTreeMap<String, ProviderDescriptor>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub source: CatalogSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CatalogSource {
    ModelsDev,
    Config,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name: String,
    pub api_base_url: Option<String>,
    pub docs_url: Option<String>,
    pub env_vars: Vec<String>,
    pub npm_package: Option<String>,
    pub models: BTreeMap<String, ModelDescriptor>,
    pub source: CatalogSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub display_name: String,
    pub family: Option<String>,
    pub provider_id: String,
    pub capabilities: ModelCapabilities,
    pub cost: CostInfo,
    pub limits: LimitInfo,
    pub status: ModelStatus,
    #[serde(default)]
    pub variants: Vec<ModelVariant>,
}

/// Represents a model variant/mode (e.g., thinking mode, extended thinking).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelVariant {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    pub attachment: bool,
    pub reasoning: bool,
    pub tool_call: bool,
    pub temperature: bool,
    pub structured_output: bool,
    pub interleaved: bool,
    pub open_weights: bool,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostInfo {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitInfo {
    pub context: u32,
    pub input: Option<u32>,
    pub output: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelStatus {
    Active,
    Beta,
    Alpha,
    Deprecated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_variant_serialize_deserialize() {
        let variant = ModelVariant {
            name: "thinking".to_string(),
            description: Some("Extended thinking mode".to_string()),
        };

        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: ModelVariant = serde_json::from_str(&json).unwrap();

        assert_eq!(variant, deserialized);
    }

    #[test]
    fn test_model_variant_without_description() {
        let variant = ModelVariant {
            name: "extended".to_string(),
            description: None,
        };

        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: ModelVariant = serde_json::from_str(&json).unwrap();

        assert_eq!(variant, deserialized);
        assert_eq!(deserialized.name, "extended");
        assert!(deserialized.description.is_none());
    }

    #[test]
    fn test_model_descriptor_with_variants() {
        let descriptor = ModelDescriptor {
            id: "gpt-4o".to_string(),
            display_name: "GPT-4o".to_string(),
            family: Some("GPT-4".to_string()),
            provider_id: "openai".to_string(),
            capabilities: ModelCapabilities::default(),
            cost: CostInfo::default(),
            limits: LimitInfo::default(),
            status: ModelStatus::Active,
            variants: vec![
                ModelVariant {
                    name: "thinking".to_string(),
                    description: Some("Extended thinking".to_string()),
                },
                ModelVariant {
                    name: "preview".to_string(),
                    description: None,
                },
            ],
        };

        let json = serde_json::to_string(&descriptor).unwrap();
        let deserialized: ModelDescriptor = serde_json::from_str(&json).unwrap();

        assert_eq!(descriptor.id, deserialized.id);
        assert_eq!(descriptor.variants.len(), 2);
        assert_eq!(descriptor.variants[0].name, "thinking");
        assert_eq!(descriptor.variants[1].name, "preview");
    }

    #[test]
    fn test_model_descriptor_empty_variants() {
        let descriptor = ModelDescriptor {
            id: "gpt-4o".to_string(),
            display_name: "GPT-4o".to_string(),
            family: None,
            provider_id: "openai".to_string(),
            capabilities: ModelCapabilities::default(),
            cost: CostInfo::default(),
            limits: LimitInfo::default(),
            status: ModelStatus::Active,
            variants: vec![],
        };

        let json = serde_json::to_string(&descriptor).unwrap();
        let deserialized: ModelDescriptor = serde_json::from_str(&json).unwrap();

        assert!(deserialized.variants.is_empty());
    }

    #[test]
    fn test_model_variant_equality() {
        let variant1 = ModelVariant {
            name: "test".to_string(),
            description: Some("desc".to_string()),
        };
        let variant2 = ModelVariant {
            name: "test".to_string(),
            description: Some("desc".to_string()),
        };
        let variant3 = ModelVariant {
            name: "different".to_string(),
            description: Some("desc".to_string()),
        };

        assert_eq!(variant1, variant2);
        assert_ne!(variant1, variant3);
    }
}
