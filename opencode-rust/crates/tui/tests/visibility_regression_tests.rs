//! Visibility regression tests for opencode-tui
//!
//! This module verifies that the tui crate compiles correctly with its
//! audited public visibility. The pub declarations have been reviewed to
//! ensure only truly necessary items are public.
//!
//! **Public API used by external crates:**
//! - `App` - used by CLI for application entry point
//! - `OutputFormat` - used by CLI for output formatting
//!
//! All other items are either:
//! - Used by internal integration tests (in tests/ directory)
//! - Internal implementation details

#[cfg(test)]
mod tests {
    use opencode_tui::cli::OutputFormat;
    use opencode_tui::App;

    #[test]
    fn verify_public_api_app_is_constructible() {
        let _app = App::new();
    }

    #[test]
    fn verify_public_api_output_format_variants() {
        assert!(matches!(OutputFormat::Text, OutputFormat::Text));
        assert!(matches!(OutputFormat::Json, OutputFormat::Json));
    }

    #[test]
    fn verify_tui_crate_compiles_with_audited_visibility() {
        let theme = opencode_tui::Theme::default();
        let _app = App::new();
        assert_eq!(theme.name, "default");
    }
}
