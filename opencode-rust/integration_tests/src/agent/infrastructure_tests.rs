use crate::common::{MockLLMProvider, MockServer, TempProject, TestConfig};
use opencode_llm::Provider;

#[test]
fn test_infrastructure_temp_project() {
    let project =
        TempProject::with_files(&[("src/main.rs", "fn main() {}"), ("Cargo.toml", "[package]")]);

    project.assert_file_exists("src/main.rs");
    project.assert_file_exists("Cargo.toml");
    project.assert_file_contents("src/main.rs", "fn main() {}");
}

#[test]
fn test_infrastructure_mock_server() {
    let server = MockServer::start();
    let url = server.url("/test");
    assert!(url.contains("127.0.0.1"));
    assert!(url.contains("/test"));
}

#[tokio::test]
async fn test_infrastructure_mock_llm() {
    let provider = MockLLMProvider::new().with_response("test response");

    let result = provider.complete("test prompt", None).await.unwrap();
    assert_eq!(result, "test response");
    assert_eq!(provider.call_count(), 1);
}

#[test]
fn test_infrastructure_test_config() {
    let config = TestConfig::default_for_testing();
    config.create_dirs();
    assert!(config.data_dir.exists());
    assert!(config.config_dir.exists());
}
