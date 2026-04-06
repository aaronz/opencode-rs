use std::path::PathBuf;

pub struct TestConfig {
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl TestConfig {
    pub fn default_for_testing() -> Self {
        let temp_base =
            std::env::temp_dir().join(format!("opencode-test-{}", uuid::Uuid::new_v4()));
        Self {
            api_key: "test-api-key".to_string(),
            model: "gpt-4o".to_string(),
            temperature: 0.0,
            data_dir: temp_base.join("data"),
            config_dir: temp_base.join("config"),
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = key.to_string();
        self
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn apply_env(&self) {
        std::env::set_var("OPENCODE_API_KEY", &self.api_key);
        std::env::set_var("OPENCODE_MODEL", &self.model);
        std::env::set_var("OPENCODE_TEMPERATURE", self.temperature.to_string());
        std::env::set_var("OPENCODE_DATA_DIR", &self.data_dir);
        std::env::set_var("OPENCODE_CONFIG_DIR", &self.config_dir);
    }

    pub fn create_dirs(&self) {
        std::fs::create_dir_all(&self.data_dir).expect("Failed to create data dir");
        std::fs::create_dir_all(&self.config_dir).expect("Failed to create config dir");
    }

    pub fn cleanup(&self) {
        if self.data_dir.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_dir_all(&self.data_dir);
        }
        if self.config_dir.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_dir_all(&self.config_dir);
        }
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TestConfig::default_for_testing();
        assert_eq!(config.api_key, "test-api-key");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, 0.0);
    }

    #[test]
    fn test_config_builder() {
        let config = TestConfig::default_for_testing()
            .with_api_key("custom-key")
            .with_model("custom-model")
            .with_temperature(0.5);

        assert_eq!(config.api_key, "custom-key");
        assert_eq!(config.model, "custom-model");
        assert_eq!(config.temperature, 0.5);
    }

    #[test]
    fn test_config_creates_dirs() {
        let config = TestConfig::default_for_testing();
        config.create_dirs();
        assert!(config.data_dir.exists());
        assert!(config.config_dir.exists());
    }
}
