use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct ProviderFilter {
    disabled: HashSet<String>,
    enabled: HashSet<String>,
}

impl ProviderFilter {
    pub fn new(disabled: Vec<String>, enabled: Vec<String>) -> Self {
        Self {
            disabled: disabled
                .into_iter()
                .map(|provider| provider.to_lowercase())
                .collect(),
            enabled: enabled
                .into_iter()
                .map(|provider| provider.to_lowercase())
                .collect(),
        }
    }

    pub fn is_allowed(&self, provider_id: &str) -> bool {
        let normalized = provider_id.to_lowercase();

        if self.disabled.contains(&normalized) {
            return false;
        }

        if self.enabled.is_empty() {
            return true;
        }

        self.enabled.contains(&normalized)
    }

    pub fn filter_available(&self, providers: Vec<String>) -> Vec<String> {
        providers
            .into_iter()
            .filter(|provider| self.is_allowed(provider))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ProviderFilter;

    #[test]
    fn blacklist_priority_over_whitelist() {
        let filter = ProviderFilter::new(
            vec!["openai".to_string()],
            vec!["openai".to_string(), "anthropic".to_string()],
        );

        assert!(!filter.is_allowed("openai"));
        assert!(filter.is_allowed("anthropic"));
    }

    #[test]
    fn empty_whitelist_allows_non_blacklisted() {
        let filter = ProviderFilter::new(vec!["openai".to_string()], vec![]);

        assert!(!filter.is_allowed("openai"));
        assert!(filter.is_allowed("anthropic"));
    }

    #[test]
    fn both_sets_blacklist_wins() {
        let filter = ProviderFilter::new(
            vec!["anthropic".to_string()],
            vec!["anthropic".to_string(), "openai".to_string()],
        );

        assert!(!filter.is_allowed("anthropic"));
        assert!(filter.is_allowed("openai"));
    }
}
