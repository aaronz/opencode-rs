pub struct TestModeConfig {
    pub enabled: bool,
    pub disable_animations: bool,
    pub disable_spinners: bool,
}

impl Default for TestModeConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl TestModeConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("APP_TEST").is_ok();
        Self {
            enabled,
            disable_animations: enabled,
            disable_spinners: enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

pub fn is_test_mode() -> bool {
    std::env::var("APP_TEST").is_ok()
}
