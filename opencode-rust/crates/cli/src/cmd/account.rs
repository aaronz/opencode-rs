use clap::{Args, Subcommand};
use opencode_auth::credential_store::CredentialStore;
use opencode_auth::oauth::OAuthFlow;
use serde_json::json;

const DEFAULT_CALLBACK_PORT: u16 = 54345;
const CALLBACK_TIMEOUT_SECS: u64 = 300;

const SUPPORTED_PROVIDERS: &[&str] = &["github", "openai", "anthropic"];

const PROVIDER_CLIENT_IDS: &[(&str, &str)] = &[
    ("github", "Iv1.8a1f8c05dfd1c06e"),
    ("openai", "client_id_openai"),
    ("anthropic", "anthropic_client_id"),
];

const PROVIDER_TOKEN_URLS: &[(&str, &str)] = &[
    ("github", "https://github.com/login/oauth/access_token"),
    ("openai", "https://auth.openai.com/oauth/token"),
    ("anthropic", "https://auth.anthropic.com/oauth/token"),
];

fn get_provider_client_id(provider: &str) -> Option<&'static str> {
    PROVIDER_CLIENT_IDS
        .iter()
        .find(|(p, _)| *p == provider)
        .map(|(_, id)| *id)
}

fn get_provider_token_url(provider: &str) -> Option<&'static str> {
    PROVIDER_TOKEN_URLS
        .iter()
        .find(|(p, _)| *p == provider)
        .map(|(_, url)| *url)
}

#[derive(Args, Debug)]
pub(crate) struct AccountArgs {
    #[arg(long)]
    pub json: bool,

    #[command(subcommand)]
    pub action: AccountAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum AccountAction {
    Login {
        #[arg(long, default_value = "github")]
        provider: String,
    },
    Logout {
        #[arg(long, default_value = "github")]
        provider: String,
    },
    Status,
}

fn run_login(provider: &str) {
    if !SUPPORTED_PROVIDERS.contains(&provider) {
        eprintln!(
            "Unsupported provider: {}. Supported providers: {:?}",
            provider, SUPPORTED_PROVIDERS
        );
        std::process::exit(1);
    }

    let client_id = get_provider_client_id(provider).unwrap_or_default();
    let token_url = get_provider_token_url(provider).unwrap_or_default();

    println!("Starting {} OAuth login flow...", provider);

    let oauth_flow = OAuthFlow::new();

    let redirect_port = DEFAULT_CALLBACK_PORT;

    let (state, verifier) = match oauth_flow.start_browser_login(provider, client_id, redirect_port)
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to start OAuth login: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nWaiting for authentication in browser...");
    println!("(You can also manually visit the URL if browser doesn't open)\n");

    let (code, returned_state) =
        match oauth_flow.run_callback_server_and_wait(redirect_port, CALLBACK_TIMEOUT_SECS) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed waiting for callback: {}", e);
                std::process::exit(1);
            }
        };

    if returned_state != state {
        eprintln!("State mismatch - possible CSRF attack");
        std::process::exit(1);
    }

    let token = match oauth_flow.complete_login(&code, &state, &verifier, client_id, "", token_url)
    {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Failed to exchange code for token: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = oauth_flow.store_token(provider, &token) {
        eprintln!("Failed to store token: {}", e);
        std::process::exit(1);
    }

    println!("✓ {} authentication successful!", provider);
    println!("  Token stored securely.");
}

fn run_logout(provider: &str) {
    if !SUPPORTED_PROVIDERS.contains(&provider) {
        eprintln!(
            "Unsupported provider: {}. Supported providers: {:?}",
            provider, SUPPORTED_PROVIDERS
        );
        std::process::exit(1);
    }

    let credential_store = CredentialStore::new();

    match credential_store.load(provider) {
        Ok(Some(_)) => {
            if let Err(e) = credential_store.delete(provider) {
                eprintln!("Failed to remove credentials: {}", e);
                std::process::exit(1);
            }
            println!("✓ {} logout successful", provider);
        }
        Ok(None) => {
            println!("Not logged in to {}", provider);
        }
        Err(e) => {
            eprintln!("Error checking credentials: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_status(json_output: bool) {
    let credential_store = CredentialStore::new();
    let oauth_flow = OAuthFlow::new();

    let mut statuses = Vec::new();
    let mut any_logged_in = false;

    for provider in SUPPORTED_PROVIDERS {
        let status = match credential_store.load(provider) {
            Ok(Some(..)) => {
                any_logged_in = true;
                let token_info = oauth_flow.load_token_for_provider(provider).ok().flatten();
                let is_valid = token_info
                    .as_ref()
                    .map(|t| !t.is_expired())
                    .unwrap_or(false);
                let expires_at = token_info.as_ref().map(|t| t.expires_at().to_rfc3339());

                serde_json::json!({
                    "provider": provider,
                    "logged_in": true,
                    "valid": is_valid,
                    "expires_at": expires_at,
                })
            }
            Ok(None) => {
                serde_json::json!({
                    "provider": provider,
                    "logged_in": false,
                    "valid": false,
                })
            }
            Err(e) => {
                serde_json::json!({
                    "provider": provider,
                    "logged_in": false,
                    "error": e.to_string(),
                })
            }
        };
        statuses.push(status);
    }

    if json_output {
        let result = json!({
            "action": "status",
            "logged_in": any_logged_in,
            "accounts": statuses,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
    } else {
        println!("Account Status:");
        println!("===============");
        for status in &statuses {
            let provider = status
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let logged_in = status
                .get("logged_in")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let valid = status
                .get("valid")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if logged_in {
                if valid {
                    println!("  {}: ✓ logged in", provider);
                } else {
                    println!("  {}: ⚠ logged in (token expired)", provider);
                }
            } else {
                println!("  {}: ✗ not logged in", provider);
            }
        }
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_args_login() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Login {
                provider: "github".to_string(),
            },
        };
        match args.action {
            AccountAction::Login { ref provider } => assert_eq!(provider, "github"),
            _ => panic!("Expected Login"),
        }
    }

    #[test]
    fn test_account_args_logout() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Logout {
                provider: "openai".to_string(),
            },
        };
        match args.action {
            AccountAction::Logout { ref provider } => assert_eq!(provider, "openai"),
            _ => panic!("Expected Logout"),
        }
    }

    #[test]
    fn test_account_args_status() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Status,
        };
        match args.action {
            AccountAction::Status => {}
            _ => panic!("Expected Status"),
        }
    }

    #[test]
    fn test_account_args_with_json() {
        let args = AccountArgs {
            json: true,
            action: AccountAction::Login {
                provider: "github".to_string(),
            },
        };
        assert!(args.json);
    }

    #[test]
    fn test_supported_providers() {
        assert!(SUPPORTED_PROVIDERS.contains(&"github"));
        assert!(SUPPORTED_PROVIDERS.contains(&"openai"));
        assert!(SUPPORTED_PROVIDERS.contains(&"anthropic"));
        assert!(!SUPPORTED_PROVIDERS.contains(&"unknown"));
    }

    #[test]
    fn test_get_provider_client_id() {
        assert_eq!(
            get_provider_client_id("github"),
            Some("Iv1.8a1f8c05dfd1c06e")
        );
        assert_eq!(get_provider_client_id("openai"), Some("client_id_openai"));
        assert_eq!(
            get_provider_client_id("anthropic"),
            Some("anthropic_client_id")
        );
        assert_eq!(get_provider_client_id("unknown"), None);
    }

    #[test]
    fn test_get_provider_token_url() {
        assert_eq!(
            get_provider_token_url("github"),
            Some("https://github.com/login/oauth/access_token")
        );
        assert_eq!(
            get_provider_token_url("openai"),
            Some("https://auth.openai.com/oauth/token")
        );
        assert_eq!(
            get_provider_token_url("anthropic"),
            Some("https://auth.anthropic.com/oauth/token")
        );
        assert_eq!(get_provider_token_url("unknown"), None);
    }

    #[test]
    fn test_login_default_provider() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Login {
                provider: "github".to_string(),
            },
        };
        if let AccountAction::Login { provider } = args.action {
            assert_eq!(provider, "github");
        }
    }

    #[test]
    fn test_logout_default_provider() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Logout {
                provider: "github".to_string(),
            },
        };
        if let AccountAction::Logout { provider } = args.action {
            assert_eq!(provider, "github");
        }
    }

    #[test]
    fn test_status_json_format() {
        let args = AccountArgs {
            json: true,
            action: AccountAction::Status,
        };
        assert!(args.json);
    }

    #[test]
    fn test_anthropic_provider_support() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Login {
                provider: "anthropic".to_string(),
            },
        };
        match args.action {
            AccountAction::Login { ref provider } => assert_eq!(provider, "anthropic"),
            _ => panic!("Expected Login with anthropic"),
        }
    }

    #[test]
    fn test_all_providers_have_client_ids() {
        for provider in SUPPORTED_PROVIDERS {
            assert!(
                get_provider_client_id(provider).is_some(),
                "Provider {} should have a client ID",
                provider
            );
        }
    }

    #[test]
    fn test_all_providers_have_token_urls() {
        for provider in SUPPORTED_PROVIDERS {
            assert!(
                get_provider_token_url(provider).is_some(),
                "Provider {} should have a token URL",
                provider
            );
        }
    }

    #[test]
    fn test_multi_provider_status_check() {
        let credential_store = CredentialStore::new();
        for provider in SUPPORTED_PROVIDERS {
            let result = credential_store.load(provider);
            assert!(
                result.is_ok(),
                "Loading {} credentials should succeed",
                provider
            );
        }
    }

    #[test]
    fn test_login_stores_credentials() {
        use chrono::Utc;
        use opencode_auth::oauth::{OAuthFlow, OAuthToken};
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let store = opencode_auth::credential_store::CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(tmp.path().to_path_buf());
        let flow = OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let token = OAuthToken {
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: Some("read".to_string()),
            received_at: Utc::now(),
        };

        flow.store_token("github", &token).unwrap();

        let loaded = flow.load_token_for_provider("github").unwrap().unwrap();
        assert_eq!(loaded.access_token, "test-access-token");
        assert_eq!(loaded.refresh_token.as_deref(), Some("test-refresh-token"));
    }

    #[test]
    fn test_logout_clears_credentials() {
        use chrono::Utc;
        use opencode_auth::oauth::{OAuthFlow, OAuthToken};
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let store_path = tmp.path().join("credentials.enc.json");
        let key_path = tmp.path().join("credentials.key");
        let store = opencode_auth::credential_store::CredentialStore::with_paths(
            store_path.clone(),
            key_path.clone(),
        );
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(tmp.path().to_path_buf());
        let flow = OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let token = OAuthToken {
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: Some("read".to_string()),
            received_at: Utc::now(),
        };

        flow.store_token("github", &token).unwrap();
        let before = flow.load_token_for_provider("github").unwrap();
        assert!(before.is_some());

        let store2 = opencode_auth::credential_store::CredentialStore::with_paths(
            store_path.clone(),
            key_path,
        );
        store2.delete("github").unwrap();
        drop(store2);
        let after = flow.load_token_for_provider("github").unwrap();
        assert!(
            after.is_none(),
            "Expected credential to be deleted after logout"
        );
    }

    #[test]
    fn test_status_shows_auth_state() {
        use chrono::Utc;
        use opencode_auth::oauth::{OAuthFlow, OAuthToken};
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let store = opencode_auth::credential_store::CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(tmp.path().to_path_buf());
        let flow = OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let token = OAuthToken {
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: Some("read".to_string()),
            received_at: Utc::now(),
        };

        flow.store_token("github", &token).unwrap();

        let loaded = flow.load_token_for_provider("github").unwrap().unwrap();
        assert!(!loaded.is_expired());
        assert_eq!(loaded.access_token, "test-access-token");
    }
}

pub(crate) fn run(args: AccountArgs) {
    match args.action {
        AccountAction::Login { provider } => {
            run_login(&provider);
        }
        AccountAction::Logout { provider } => {
            run_logout(&provider);
        }
        AccountAction::Status => {
            run_status(args.json);
        }
    }
}
