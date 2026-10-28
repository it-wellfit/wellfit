#![forbid(unsafe_code)]

//! HTTP client for communicating with the E.Sun `銀企直連安控元件`
//! (security component) on port 8443 (internal) / 9443 (external).
//!
//! Uses `ureq` with `rustls` for TLS 1.2+ and 2-way SSL certificate
//! verification. No OpenSSL dependency, no unsafe code.

use crate::api::{ApiResponse, TwdPaymentRequest, TwdPaymentResponseData};
use rustls_pki_types::pem::PemObject;
use std::io::Read;
use std::sync::Arc;

/// Configuration for the E.Sun security component HTTP client.
#[derive(Debug, Clone)]
pub struct EsunHttpClientConfig {
    /// Base URL of the security component (e.g. `https://127.0.0.1:8443`).
    pub base_url: String,
    /// Path to the client certificate (PEM) for 2-way SSL.
    pub client_cert_path: Option<String>,
    /// Path to the client private key (PEM) for 2-way SSL.
    pub client_key_path: Option<String>,
    /// Path to the CA certificate (PEM) for server verification.
    pub ca_cert_path: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for EsunHttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://127.0.0.1:8443".to_string(),
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            timeout_secs: 30,
        }
    }
}

/// HTTP client for E.Sun security component API calls.
pub struct EsunHttpClient {
    pub config: EsunHttpClientConfig,
    pub tls_config: Option<Arc<rustls::ClientConfig>>,
}

impl EsunHttpClient {
    /// Create a new client with the given configuration.
    pub fn new(config: EsunHttpClientConfig) -> Self {
        let tls_config = Self::build_tls_config(&config);
        Self { config, tls_config }
    }

    /// Build a rustls ClientConfig from the certificate paths in the config.
    ///
    /// Returns None if no certificate paths are provided or if loading fails,
    /// in which case the default ureq TLS configuration is used.
    fn build_tls_config(config: &EsunHttpClientConfig) -> Option<Arc<rustls::ClientConfig>> {
        // If no certificate paths are provided, use ureq defaults
        if config.ca_cert_path.is_none()
            && config.client_cert_path.is_none()
            && config.client_key_path.is_none()
        {
            return None;
        }

        // Load CA certificate if provided, otherwise use webpki roots
        let root_store = if let Some(ca_path) = &config.ca_cert_path {
            let ca_pem = std::fs::read(ca_path).ok()?;
            let ca_certs: Vec<rustls_pki_types::CertificateDer> =
                rustls_pki_types::CertificateDer::pem_slice_iter(&ca_pem)
                    .filter_map(|r| r.ok())
                    .collect();
            let mut store = rustls::RootCertStore::empty();
            for cert in ca_certs {
                store.add(cert).ok()?;
            }
            store
        } else {
            // Use webpki roots as default
            rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            }
        };

        // Load client certificate and key if provided
        let client_auth = match (&config.client_cert_path, &config.client_key_path) {
            (Some(cert_path), Some(key_path)) => {
                let cert_pem = std::fs::read(cert_path).ok()?;
                let key_pem = std::fs::read(key_path).ok()?;
                let certs: Vec<rustls_pki_types::CertificateDer> =
                    rustls_pki_types::CertificateDer::pem_slice_iter(&cert_pem)
                        .filter_map(|r| r.ok())
                        .collect();
                let key = rustls_pki_types::PrivateKeyDer::try_from(key_pem).ok()?;
                Some((certs, key))
            }
            _ => None,
        };

        let builder = rustls::ClientConfig::builder().with_root_certificates(root_store);

        let client_config = match client_auth {
            Some((certs, key)) => builder.with_client_auth_cert(certs, key).ok()?,
            None => builder.with_no_client_auth(),
        };

        Some(Arc::new(client_config))
    }

    /// POST a JSON request and deserialize the response.
    ///
    /// In production, this uses ureq with rustls for 2-way SSL.
    /// The actual TLS configuration depends on the certificate paths
    /// provided in the config.
    pub fn post_json<Req, Resp>(
        &self,
        path: &str,
        source_system: &str,
        body: &Req,
    ) -> Result<ApiResponse<Resp>, String>
    where
        Req: serde::Serialize,
        Resp: serde::de::DeserializeOwned + Default,
    {
        // Enforce HTTPS for production security
        if !self.config.base_url.starts_with("https://")
            && !self.config.base_url.starts_with("http://127.0.0.1")
            && !self.config.base_url.starts_with("http://localhost")
        {
            return Err(format!(
                "base URL must use HTTPS in production: {}",
                self.config.base_url
            ));
        }
        let url = format!("{}{}", self.config.base_url, path);
        let json_body =
            serde_json::to_string(body).map_err(|e| format!("serialize request: {e}"))?;

        // Build ureq request with timeout and TLS config
        let mut builder = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs));
        if let Some(tls) = &self.tls_config {
            builder = builder.tls_config(tls.clone());
        }
        let agent = builder.build();

        let response = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .set("Source-System", source_system)
            .send_string(&json_body)
            .map_err(|e| format!("HTTP request to {url} failed: {e}"))?;

        let mut body_str = String::new();
        response
            .into_reader()
            .read_to_string(&mut body_str)
            .map_err(|e| format!("read response body: {e}"))?;

        let parsed: ApiResponse<Resp> =
            serde_json::from_str(&body_str).map_err(|e| format!("deserialize response: {e}"))?;

        Ok(parsed)
    }

    /// TWD payment release API call.
    pub fn create_twd_payment(
        &self,
        source_system: &str,
        request: &TwdPaymentRequest,
    ) -> Result<ApiResponse<TwdPaymentResponseData>, String> {
        self.post_json(
            "/payment/createTwdPaymentInstruction",
            source_system,
            request,
        )
    }

    /// TWD transaction status query API call.
    pub fn retrieve_twd_status(
        &self,
        source_system: &str,
        request: &crate::api::TwdStatusQueryRequest,
    ) -> Result<ApiResponse<crate::api::TwdStatusResponseData>, String> {
        self.post_json(
            "/payment/retrieveTwdPaymentInstructionResult",
            source_system,
            request,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = EsunHttpClientConfig::default();
        assert_eq!(config.base_url, "https://127.0.0.1:8443");
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_client_creation() {
        let config = EsunHttpClientConfig::default();
        let client = EsunHttpClient::new(config);
        assert_eq!(client.config.base_url, "https://127.0.0.1:8443");
        // Default config has no cert paths, so tls_config is None (ureq defaults)
        assert!(client.tls_config.is_none());
    }
}
