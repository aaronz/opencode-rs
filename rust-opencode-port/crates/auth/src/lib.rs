pub mod password;
pub mod jwt;
pub mod manager;
pub mod credential_store;
pub mod credential_ref;
pub mod oauth;

pub use manager::AuthManager;
pub use credential_store::{Credential, CredentialStore};
pub use credential_ref::{
    CredentialRef, CredentialResolutionError, CredentialResolver,
    DefaultCredentialResolver, CredentialStoreEntry, CredentialType,
};
pub use oauth::{AuthUrl, CodeVerifier, OAuthError, OAuthFlow, OAuthSession, OAuthSessionManager, OAuthToken, State};
