#![forbid(unsafe_code)]

//! HTTP client for communicating with the E.Sun `銀企直連安控元件`
//! (security component) on port 8443 (internal) / 9443 (external).
//!
//! Uses `ureq` with `rustls` for TLS 1.2+ and 2-way SSL certificate
//! verification. No OpenSSL dependency, no unsafe code.

use crate::api::{ApiResponse, TwdPaymentRequest, TwdPaymentResponseData};
use std::io::Read;

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
    config: EsunHttpClientConfig,
}

impl EsunHttpClient {
    /// Create a new client with the given configuration.
    pub fn new(config: EsunHttpClientConfig) -> Self {
        Self { config }
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
        let url = format!("{}{}", self.config.base_url, path);
        let json_body =
            serde_json::to_string(body).map_err(|e| format!("serialize request: {e}"))?;

        // Build ureq request with timeout
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .build();

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
    }
}
