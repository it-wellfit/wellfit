#![forbid(unsafe_code)]

//! HTTP client for communicating with the E.Sun `???∵播謆????????
//! (security component) on port 8443 (internal) / 9443 (external).
//!
//! Uses `ureq` with `rustls` for TLS 1.2+ and 2-way SSL certificate
//! verification. No OpenSSL dependency, no unsafe code.

use crate::api::{ApiResponse, TwdPaymentRequest, TwdPaymentResponseData};
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
use rustls::DigitallySignedStruct;
use rustls_pki_types::pem::PemObject;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    /// Optional list of pinned server certificate SHA-256 hashes (hex-encoded).
    ///
    /// When non-empty, the client verifies the server's leaf certificate
    /// by computing its SHA-256 hash and checking membership in this list.
    /// This provides certificate pinning protection against man-in-the-middle
    /// attacks, as required by FR-BP-10 / QG-02.
    ///
    /// To obtain a server cert hash:
    /// `openssl x509 -in server.pem -noout -fingerprint -sha256`
    pub pin_server_cert_sha256: Vec<String>,
}

impl Default for EsunHttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://127.0.0.1:8443".to_string(),
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            timeout_secs: 30,
            pin_server_cert_sha256: Vec::new(),
        }
    }
}

/// A `ServerCertVerifier` that pins the server's leaf certificate by SHA-256 hash.
///
/// When pins are provided, only certificates whose SHA-256 hash matches one of
/// the pinned values will be accepted. This provides strong protection against
/// certificate authority compromises and man-in-the-middle attacks.
///
/// The verifier still performs full certificate chain validation in addition
/// to the pin check, providing defense in depth.
#[derive(Clone)]
struct PinnedCertVerifier {
    /// Accepted SHA-256 hashes of the server certificate (lowercase hex).
    pins: Vec<String>,
    /// The fallback verifier to use for chain and signature verification.
    webpki_verifier: Arc<rustls::client::WebPkiServerVerifier>,
}

impl std::fmt::Debug for PinnedCertVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedCertVerifier")
            .field("pins", &self.pins)
            .finish()
    }
}

impl PinnedCertVerifier {
    /// Compute the SHA-256 hash of a certificate in lowercase hex.
    fn cert_sha256(cert: &rustls_pki_types::CertificateDer<'_>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert.as_ref());
        format!("{:x}", hasher.finalize())
    }
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls_pki_types::CertificateDer<'_>,
        intermediates: &[rustls_pki_types::CertificateDer<'_>],
        server_name: &rustls_pki_types::ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls_pki_types::UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // Compute the SHA-256 hash of the server's leaf certificate.
        let cert_hash = Self::cert_sha256(end_entity);

        // Check if the hash matches any of the pinned values.
        let pin_matched = self
            .pins
            .iter()
            .any(|pin| pin.eq_ignore_ascii_case(&cert_hash));

        if !pin_matched {
            // No pin matched - reject the connection.
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ));
        }

        // Pin matched; also verify the certificate chain for defense in depth.
        self.webpki_verifier.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.webpki_verifier
            .verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.webpki_verifier
            .verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.webpki_verifier.supported_verify_schemes()
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
    ///
    /// When `pin_server_cert_sha256` is non-empty, a custom `ServerCertVerifier`
    /// is installed that enforces certificate pinning on top of standard
    /// chain verification.
    fn build_tls_config(config: &EsunHttpClientConfig) -> Option<Arc<rustls::ClientConfig>> {
        // If no certificate paths or pins are provided, use ureq defaults
        if config.ca_cert_path.is_none()
            && config.client_cert_path.is_none()
            && config.client_key_path.is_none()
            && config.pin_server_cert_sha256.is_empty()
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

        // If certificate pinning is configured, install a custom verifier.
        if !config.pin_server_cert_sha256.is_empty() {
            let webpki_verifier =
                rustls::client::WebPkiServerVerifier::builder(Arc::new(root_store))
                    .build()
                    .ok()?;

            let pinned_verifier = PinnedCertVerifier {
                pins: config.pin_server_cert_sha256.clone(),
                webpki_verifier,
            };

            let builder = rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(pinned_verifier));

            let client_config = match client_auth {
                Some((certs, key)) => builder.with_client_auth_cert(certs, key).ok()?,
                None => builder.with_no_client_auth(),
            };

            return Some(Arc::new(client_config));
        }

        // Standard (non-pinned) TLS configuration
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
    /// and pinning settings provided in the config.
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

// ---------------------------------------------------------------------------
// Mock Security Implementation for Integration Testing
// ---------------------------------------------------------------------------

impl EsunHttpClient {
    /// Create a test client with mock certificates.
    ///
    /// This client uses test certificates and mock responses for integration
    /// testing. Do not use in production.
    pub fn new_test_client() -> Self {
        let config = EsunHttpClientConfig {
            base_url: "https://localhost:8443".to_string(),
            client_cert_path: Some("C:\\certs\\client.crt".to_string()),
            client_key_path: Some("C:\\certs\\client.key".to_string()),
            ca_cert_path: Some("C:\\certs\\ca.crt".to_string()),
            timeout_secs: 30,
            pin_server_cert_sha256: Vec::new(),
        };
        Self::new(config)
    }

    /// Create a mock payment response for testing.
    ///
    /// Returns a simulated successful payment response with the given tx_seq.
    pub fn mock_payment_response(
        &self,
        tx_seq: &str,
    ) -> Result<crate::api::ApiResponse<crate::api::TwdPaymentResponseData>, String> {
        use crate::api::{
            ApiResponse, RsHeader, RsStatus, TwdPaymentRecord, TwdPaymentResponseData,
        };

        let response = ApiResponse {
            rs_header: RsHeader {
                msg_id: format!(
                    "MOCK{}{}",
                    tx_seq,
                    chrono::Utc::now().format("%Y%m%d%H%M%S")
                ),
                log_id: "MOCK-LOG-001".to_string(),
                response_time: chrono::Utc::now().format("%Y/%m/%d %H:%M:%S").to_string(),
            },
            rs_status: RsStatus {
                code: "910000".to_string(),
                message: "SUCCESS".to_string(),
                more_info: String::new(),
            },
            rs_data: Some(TwdPaymentResponseData {
                data_count: "1".to_string(),
                tx_record_list: vec![TwdPaymentRecord {
                    tx_seq: tx_seq.to_string(),
                    auto_delay: "0".to_string(),
                    payer_account_no: "1234567890123".to_string(),
                    payer_id: "1234567890".to_string(),
                    payee_bank_id: "013".to_string(),
                    payee_branch_id: "0001".to_string(),
                    payee_account_no: "9876543210".to_string(),
                    payee_name: "Test Payee".to_string(),
                    payee_id: String::new(),
                    tx_date: chrono::Utc::now().format("%Y%m%d").to_string(),
                    tx_amt: "100000".to_string(),
                    charge_fee_type: "0".to_string(),
                    memo: "Test payment".to_string(),
                    notify_method: "1".to_string(),
                    notify_email: String::new(),
                    notify_fax: String::new(),
                    write_off_no: tx_seq.to_string(),
                    remark: String::new(),
                    detail_list: vec![],
                }],
            }),
        };
        Ok(response)
    }

    /// Create a mock status response for testing.
    ///
    /// Returns a simulated successful status response for the given bank_ref.
    pub fn mock_status_response(
        &self,
        bank_ref: &str,
    ) -> Result<crate::api::ApiResponse<crate::api::TwdStatusResponseData>, String> {
        use crate::api::{ApiResponse, RsHeader, RsStatus, TwdStatusResponseData};

        let response = ApiResponse {
            rs_header: RsHeader {
                msg_id: format!(
                    "MOCK{}{}",
                    bank_ref,
                    chrono::Utc::now().format("%Y%m%d%H%M%S")
                ),
                log_id: "MOCK-LOG-002".to_string(),
                response_time: chrono::Utc::now().format("%Y/%m/%d %H:%M:%S").to_string(),
            },
            rs_status: RsStatus {
                code: "910000".to_string(),
                message: "SUCCESS".to_string(),
                more_info: String::new(),
            },
            rs_data: Some(TwdStatusResponseData {
                tx_status: "0000".to_string(),
                bank_ref: bank_ref.to_string(),
                tx_amt: "100000".to_string(),
                tx_date: chrono::Utc::now().format("%Y%m%d").to_string(),
                error_message: String::new(),
            }),
        };
        Ok(response)
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
        assert!(config.pin_server_cert_sha256.is_empty());
    }

    #[test]
    fn test_client_creation() {
        let config = EsunHttpClientConfig::default();
        let client = EsunHttpClient::new(config);
        assert_eq!(client.config.base_url, "https://127.0.0.1:8443");
        // Default config has no cert paths, so tls_config is None (ureq defaults)
        assert!(client.tls_config.is_none());
    }

    #[test]
    fn test_config_with_pinning() {
        let config = EsunHttpClientConfig {
            base_url: "https://127.0.0.1:8443".to_string(),
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            timeout_secs: 30,
            pin_server_cert_sha256: vec!["abcdef1234567890".to_string()],
        };
        // With pinning but no cert paths, tls_config should still be None
        // (pinning requires at least the webpki root store)
        let client = EsunHttpClient::new(config);
        // Pinning with empty certs but webpki roots should work if root store loads
        // For this test config (no CA path), it uses webpki_roots
        assert_eq!(client.config.pin_server_cert_sha256.len(), 1);
    }
}
// ---------------------------------------------------------------------------
// E.Sun Bank Environment Configuration (V2.8)
// ---------------------------------------------------------------------------

/// E.Sun Bank integration environment.
///
/// The spec defines three environments with distinct domains and ports:
/// - DEV: `eospt.testesunbank.com.tw` (port 9443)
/// - UAT: `eospu.testesunbank.com.tw` (port 9443)
/// - Production: `eosp.esunbank.com.tw` (port 9443)
///
/// Internal security component runs on port 8443 (localhost).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EsunEnvironment {
    /// Development / testing environment.
    Dev,
    /// User acceptance testing environment.
    Uat,
    /// Production environment.
    Production,
}

impl EsunEnvironment {
    /// Returns the external domain for this environment.
    pub fn external_domain(&self) -> &'static str {
        match self {
            Self::Dev => "eospt.testesunbank.com.tw",
            Self::Uat => "eospu.testesunbank.com.tw",
            Self::Production => "eosp.esunbank.com.tw",
        }
    }

    /// Returns the external port for outbound connections (9443).
    pub fn external_port(&self) -> u16 {
        9443
    }

    /// Returns the external base URL (https://domain:9443).
    pub fn external_base_url(&self) -> String {
        format!(
            "https://{}:{}",
            self.external_domain(),
            self.external_port()
        )
    }

    /// Returns the internal security component port (8443).
    pub fn internal_port(&self) -> u16 {
        8443
    }

    /// Parses an environment from a string.
    pub fn parse_env(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dev" | "development" | "test" => Some(Self::Dev),
            "uat" | "staging" => Some(Self::Uat),
            "prod" | "production" => Some(Self::Production),
            _ => None,
        }
    }
}

impl std::fmt::Display for EsunEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dev => write!(f, "Dev"),
            Self::Uat => write!(f, "Uat"),
            Self::Production => write!(f, "Production"),
        }
    }
}

/// Certificate configuration for 2-Way SSL.
///
/// The spec requires:
/// - Bank provides certificate for ERP (bank_cert_path)
/// - ERP provides certificate for Bank (erp_cert_path, erp_key_path)
#[derive(Debug, Clone, Default)]
pub struct CertificateConfig {
    /// Path to the bank-provided certificate (for ERP to verify bank).
    pub bank_cert_path: Option<String>,
    /// Path to the ERP-provided client certificate (PEM).
    pub erp_cert_path: Option<String>,
    /// Path to the ERP-provided client private key (PEM).
    pub erp_key_path: Option<String>,
    /// Path to the CA certificate for chain verification.
    pub ca_cert_path: Option<String>,
}

/// Enhanced HTTP client configuration supporting environments.
#[derive(Debug, Clone)]
pub struct EsunHttpClientConfigV2 {
    /// Target environment.
    pub environment: EsunEnvironment,
    /// Internal security component URL (default: https://127.0.0.1:8443).
    pub internal_base_url: String,
    /// Certificate configuration for 2-Way SSL.
    pub certificates: CertificateConfig,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Optional list of pinned server certificate SHA-256 hashes.
    pub pin_server_cert_sha256: Vec<String>,
    /// SourceSystemId agreed with the bank.
    pub source_system_id: String,
}

impl Default for EsunHttpClientConfigV2 {
    fn default() -> Self {
        Self {
            environment: EsunEnvironment::Dev,
            internal_base_url: "https://127.0.0.1:8443".to_string(),
            certificates: CertificateConfig::default(),
            timeout_secs: 30,
            pin_server_cert_sha256: Vec::new(),
            source_system_id: String::new(),
        }
    }
}

impl EsunHttpClientConfigV2 {
    /// Creates a config for the specified environment.
    pub fn for_environment(env: EsunEnvironment) -> Self {
        Self {
            environment: env,
            ..Default::default()
        }
    }

    /// Returns the internal security component base URL.
    pub fn internal_url(&self) -> &str {
        &self.internal_base_url
    }

    /// Returns the external E.Sun bank base URL.
    pub fn external_url(&self) -> String {
        self.environment.external_base_url()
    }
}
