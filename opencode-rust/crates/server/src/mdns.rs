use std::collections::HashMap;

use mdns_sd::{ServiceDaemon, ServiceInfo};
use opencode_core::config::ServerConfig;

const SERVICE_TYPE: &str = "_opencode._tcp.local.";
const DEFAULT_DOMAIN: &str = "opencode.local";

pub struct MdnsService {
    daemon: ServiceDaemon,
    fullname: String,
}

impl MdnsService {
    pub(crate) fn start(config: &ServerConfig) -> Result<Self, std::io::Error> {
        let port = config.port.unwrap_or(4096);
        let host = config
            .hostname
            .clone()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let domain = normalize_domain(config.mdns_domain.as_deref())?;
        let instance_name = domain.trim_end_matches(".local");

        let daemon = ServiceDaemon::new().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to initialize mDNS daemon: {}", e),
            )
        })?;

        let properties = HashMap::from([
            ("domain".to_string(), domain.clone()),
            ("service".to_string(), "opencode".to_string()),
        ]);

        let service = ServiceInfo::new(
            SERVICE_TYPE,
            instance_name,
            &format!("{}.local.", host.trim_end_matches('.')),
            host,
            port,
            Some(properties),
        )
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to create mDNS service info: {}", e),
            )
        })?;

        let fullname = service.get_fullname().to_string();
        daemon.register(service).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to register mDNS service: {}", e),
            )
        })?;

        Ok(Self { daemon, fullname })
    }

    pub(crate) fn stop(self) {
        let _ = self.daemon.unregister(&self.fullname);
        let _ = self.daemon.shutdown();
    }
}

fn normalize_domain(domain: Option<&str>) -> Result<String, std::io::Error> {
    let value = domain
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(DEFAULT_DOMAIN)
        .to_string();

    if !value.ends_with(".local") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid mDNS domain '{}': must end with .local", value),
        ));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_domain_defaults_when_empty() {
        assert_eq!(normalize_domain(None).unwrap(), "opencode.local");
        assert_eq!(normalize_domain(Some("   ")).unwrap(), "opencode.local");
    }

    #[test]
    fn normalize_domain_rejects_non_local() {
        let err = normalize_domain(Some("opencode.test")).unwrap_err();
        assert!(err.to_string().contains("must end with .local"));
    }

    #[test]
    fn normalize_domain_accepts_local() {
        assert_eq!(
            normalize_domain(Some("my-opencode.local")).unwrap(),
            "my-opencode.local"
        );
    }
}
