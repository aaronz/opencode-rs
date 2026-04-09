use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CACHE_TTL_SECS: i64 = 3600;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoteConfigCache {
    pub url: String,
    pub content: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub fetched_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub content_hash: String,
}

impl RemoteConfigCache {
    pub fn is_expired(&self) -> bool {
        let expires_at = self
            .expires_at
            .unwrap_or_else(|| self.fetched_at + Duration::seconds(DEFAULT_CACHE_TTL_SECS));
        Utc::now() > expires_at
    }

    pub fn with_default_ttl(mut self) -> Self {
        if self.expires_at.is_none() {
            self.expires_at = Some(self.fetched_at + Duration::seconds(DEFAULT_CACHE_TTL_SECS));
        }
        self
    }
}

fn cache_file_path(url: &str, cache_dir: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    cache_dir.join(format!("remote_config_{}.json", hash))
}

pub fn load_cache(url: &str, cache_dir: &Path) -> Option<RemoteConfigCache> {
    let path = cache_file_path(url, cache_dir);
    let content = fs::read_to_string(path).ok()?;
    let cache: RemoteConfigCache = serde_json::from_str(&content).ok()?;
    if cache.url == url {
        Some(cache.with_default_ttl())
    } else {
        None
    }
}

pub fn save_cache(cache: &RemoteConfigCache, cache_dir: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(cache_dir)?;
    let path = cache_file_path(&cache.url, cache_dir);
    let normalized = cache.clone().with_default_ttl();
    let json = serde_json::to_string_pretty(&normalized)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let cache = RemoteConfigCache {
            url: "https://example.com/.well-known/opencode".to_string(),
            content: "{\"model\":\"x\"}".to_string(),
            etag: Some("abc".to_string()),
            last_modified: Some("Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
            fetched_at: Utc::now(),
            expires_at: None,
            content_hash: "hash".to_string(),
        };

        save_cache(&cache, temp.path()).unwrap();
        let loaded = load_cache(&cache.url, temp.path()).unwrap();

        assert_eq!(loaded.url, cache.url);
        assert_eq!(loaded.content, cache.content);
        assert_eq!(loaded.etag, cache.etag);
        assert_eq!(loaded.last_modified, cache.last_modified);
        assert_eq!(loaded.content_hash, cache.content_hash);
        assert!(loaded.expires_at.is_some());
    }

    #[test]
    fn cache_expiration_check() {
        let expired = RemoteConfigCache {
            url: "https://example.com/.well-known/opencode".to_string(),
            content: "{}".to_string(),
            etag: None,
            last_modified: None,
            fetched_at: Utc::now() - Duration::hours(2),
            expires_at: None,
            content_hash: "hash".to_string(),
        };

        let fresh = RemoteConfigCache {
            fetched_at: Utc::now(),
            ..expired.clone()
        };

        assert!(expired.is_expired());
        assert!(!fresh.is_expired());
    }

    #[test]
    fn test_build_remote_url() {
        assert_eq!(
            "https://example.com/.well-known/opencode",
            build_remote_url("https://example.com")
        );
        assert_eq!(
            "https://example.com/.well-known/opencode",
            build_remote_url("https://example.com/")
        );
    }

    fn build_remote_url(domain: &str) -> String {
        let domain = domain.trim_end_matches('/');
        format!("{}/.well-known/opencode", domain)
    }
}
