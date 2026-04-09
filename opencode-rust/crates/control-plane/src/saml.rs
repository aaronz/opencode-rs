use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAuthnRequest {
    pub id: String,
    pub issuer: String,
    pub assertion_consumer_service_url: String,
    pub request_xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertion {
    pub id: String,
    pub issuer: String,
    pub subject: String,
    pub attributes: HashMap<String, String>,
    pub session_index: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlResponse {
    pub saml_response: String,
    pub relay_state: Option<String>,
}

pub struct SamlAuthnRequestBuilder {
    issuer: String,
    acs_url: String,
    #[allow(dead_code)]
    entity_id: String,
}

impl SamlAuthnRequestBuilder {
    pub fn new(issuer: String, acs_url: String, entity_id: String) -> Self {
        Self {
            issuer,
            acs_url,
            entity_id,
        }
    }

    pub fn build(self) -> SamlAuthnRequest {
        let id = format!("_id-{}", uuid::Uuid::new_v4());
        let issue_instant = chrono::Utc::now().to_rfc3339();

        let request_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<samlp:AuthnRequest 
    xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
    xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
    ID="{id}"
    Version="2.0"
    IssueInstant="{issue_instant}"
    AssertionConsumerServiceURL="{acs_url}"
    ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
    <saml:Issuer>{issuer}</saml:Issuer>
    <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" AllowCreate="true"/>
</samlp:AuthnRequest>"#,
            id = id,
            issue_instant = issue_instant,
            acs_url = self.acs_url,
            issuer = self.issuer
        );

        SamlAuthnRequest {
            id,
            issuer: self.issuer,
            assertion_consumer_service_url: self.acs_url,
            request_xml,
        }
    }
}

pub fn decode_saml_response(
    response: &str,
    _certificate: Option<&str>,
) -> Result<SamlAssertion, SamlError> {
    info!(event = "saml_response_decode_attempt");

    let decoded = BASE64.decode(response).map_err(|e| {
        warn!(event = "saml_decode_failed", error = %e);
        SamlError::DecodeError(e.to_string())
    })?;

    let xml_string = String::from_utf8(decoded).map_err(|e| {
        warn!(event = "saml_invalid_utf8", error = %e);
        SamlError::InvalidXml(e.to_string())
    })?;

    info!(event = "saml_response_decoded", xml_len = xml_string.len());

    // In production, this would:
    // 1. Parse the XML
    // 2. Verify the signature using the certificate
    // 3. Validate conditions (NotBefore, NotOnOrAfter)
    // 4. Extract subject and attributes
    // For now, return a mock assertion
    let assertion = SamlAssertion {
        id: "mock-assertion-id".to_string(),
        issuer: "https://idp.example.com".to_string(),
        subject: "user@example.com".to_string(),
        attributes: HashMap::new(),
        session_index: None,
    };

    info!(
        event = "saml_assertion_extracted",
        subject = assertion.subject
    );
    Ok(assertion)
}

#[derive(Debug, Clone)]
pub enum SamlError {
    DecodeError(String),
    InvalidXml(String),
    SignatureValidationFailed(String),
    CertificateError(String),
    MissingAssertion,
}

impl std::fmt::Display for SamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecodeError(msg) => write!(f, "SAML decode error: {}", msg),
            Self::InvalidXml(msg) => write!(f, "SAML invalid XML: {}", msg),
            Self::SignatureValidationFailed(msg) => {
                write!(f, "SAML signature validation failed: {}", msg)
            }
            Self::CertificateError(msg) => write!(f, "SAML certificate error: {}", msg),
            Self::MissingAssertion => write!(f, "SAML assertion missing"),
        }
    }
}

impl std::error::Error for SamlError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saml_authn_request_build() {
        let builder = SamlAuthnRequestBuilder::new(
            "https://sp.example.com".to_string(),
            "https://sp.example.com/saml/acs".to_string(),
            "https://sp.example.com".to_string(),
        );
        let request = builder.build();
        assert!(request.request_xml.contains("AuthnRequest"));
        assert!(request.request_xml.contains("samlp:"));
    }

    #[test]
    fn test_saml_error_display() {
        let err = SamlError::MissingAssertion;
        assert!(err.to_string().contains("missing"));
    }
}
