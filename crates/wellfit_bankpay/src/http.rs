#![forbid(unsafe_code)]

//! E.Sun Bank API V2.8 integration - HTTP client with TLS client-auth,
//! certificate pinning, and offline mock responses for the synthetic
//! security component (QG-04).
//!
//! Scope note: transport for the two modelled endpoints only
//! (/payment/createTwdPaymentInstruction,
//!  /payment/retrieveTwdPaymentInstructionResult). Other V2.8 endpoints
//! are out of scope until their models exist in api.rs.
//!
//! Invariants honoured: no unsafe; TLS material configured but failed to
//! load blocks requests (no silent downgrade); pin matching is pure and
//! unit-tested.

use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::api::{
    ApiResponse, RsHeader, RsStatus, TwdPaymentRecord, TwdPaymentRequest, TwdPaymentResponseData,
    TwdStatusQueryRequest, TwdStatusResponseData,
};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::DigitallySignedStruct;
use rustls_pki_types::pem::PemObject;
use sha2::{Digest, Sha256};
use wellfit_core::automation::RetryConfig;

/// Inter-attempt sleep function; injected for deterministic tests.
type SleepFn = Arc<dyn Fn(Duration) + Send + Sync>;

/// One HTTP attempt, pre-classified for the retry loop. Keeps the large
/// `ureq::Error` out of function signatures (`clippy::result_large_err`).
enum SendOutcome {
    Response(ureq::Response),
    Transport(String),
    /// HTTP status received; business envelopes may ride on non-2xx codes.
    Status(u16, ureq::Response),
}

// ---------------------------------------------------------------------------
// Client configuration
// ---------------------------------------------------------------------------

/// Configuration for the E.Sun HTTP client.
///
/// Default points at the local security component on port 8443, with no
/// client certificate, no custom CA, and no pinning. Pinning is opt-in.
#[derive(Debug, Clone)]
pub struct EsunHttpClientConfig {
    /// Base URL, e.g. `https://127.0.0.1:8443`. Trailing `/` trimmed.
    pub base_url: String,
    /// PEM file with the client certificate chain (2-way TLS).
    pub client_cert_path: Option<String>,
    /// PEM file with the client private key (2-way TLS).
    pub client_key_path: Option<String>,
    /// PEM file with the CA that signed the server certificate.
    pub ca_cert_path: Option<String>,
    /// Overall request timeout in seconds.
    pub timeout_secs: u64,
    /// Allowed SHA-256 fingerprints (64 hex chars, case-insensitive) of the
    /// server leaf certificate. Empty = no pinning.
    pub pin_server_cert_sha256: Vec<String>,
    /// Retry policy for IDEMPOTENT calls (status queries). Payment creation
    /// (`create_twd_payment`) is NEVER retried automatically: a timed-out
    /// POST may already have been processed by the bank, so the safe path
    /// is dispatch-uncertain reconciliation, not a blind retry (AWS
    /// Builders' Library: side-effecting APIs are not retry-safe without
    /// idempotency).
    pub retry: Option<RetryConfig>,
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
            retry: Some(Self::http_retry()),
        }
    }
}

impl EsunHttpClientConfig {
    /// Retry defaults tuned for the LOCAL security component (127.0.0.1):
    /// 3 retries, 200 ms base, 2x multiplier, 2 s cap. The core
    /// `RetryConfig::default_retry()` targets slow remote services
    /// (5 s / 3x / 30 s) and is too slow for a loopback peer.
    pub fn http_retry() -> RetryConfig {
        RetryConfig {
            max_retries: 3,
            base_delay_ms: 200,
            backoff_multiplier: 2.0,
            max_delay_ms: 2000,
        }
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// HTTP client for the E.Sun Bank security component (API V2.8).
pub struct EsunHttpClient {
    /// Client configuration (mirrored for inspection by tests and callers).
    pub config: EsunHttpClientConfig,
    /// TLS configuration built from `config`; `None` when no TLS material
    /// was configured (agent default trust store is then used).
    pub tls_config: Option<Arc<rustls::ClientConfig>>,
    /// Underlying agent; `Some` after [EsunHttpClient::new].
    pub http_client: Option<ureq::Agent>,
    /// Set when TLS material was configured but failed to load. Requests
    /// are refused while this is set (no silent TLS downgrade).
    tls_load_error: Option<String>,
    /// Inter-attempt sleep, injected for deterministic tests. Production
    /// sleeps for real (full-jitter capped exponential backoff).
    sleep: SleepFn,
    /// Entropy counter for jitter; `&self` friendly.
    jitter_state: AtomicU64,
}

impl EsunHttpClient {
    /// Create a client. TLS material is loaded eagerly; a load failure is
    /// recorded and blocks requests instead of silently falling back.
    /// Retries sleep for real between attempts.
    pub fn new(config: EsunHttpClientConfig) -> Self {
        Self::with_sleep(config, Arc::new(std::thread::sleep))
    }

    /// Create a client with an injected inter-attempt sleep function.
    /// Tests pass a no-op (or recording) closure for determinism.
    pub fn with_sleep(config: EsunHttpClientConfig, sleep: SleepFn) -> Self {
        let (tls_config, tls_load_error) = match Self::build_tls_config(&config) {
            Ok(tls) => (tls, None),
            Err(err) => (None, Some(err)),
        };
        let http_client = Some(Self::build_agent(&config, tls_config.as_ref()));
        Self {
            config,
            tls_config,
            http_client,
            tls_load_error,
            sleep,
            jitter_state: AtomicU64::new(0),
        }
    }

    /// Offline client for tests that exercise mock responses only. Never
    /// performs network I/O by itself and never sleeps between retries.
    pub fn new_test_client() -> Self {
        Self::with_sleep(EsunHttpClientConfig::default(), Arc::new(|_| {}))
    }

    /// Whether TLS material was configured but failed to load.
    pub fn tls_load_error(&self) -> Option<&str> {
        self.tls_load_error.as_deref()
    }

    fn build_agent(
        config: &EsunHttpClientConfig,
        tls: Option<&Arc<rustls::ClientConfig>>,
    ) -> ureq::Agent {
        let timeout = Duration::from_secs(config.timeout_secs.max(1));
        let mut builder = ureq::AgentBuilder::new().timeout(timeout);
        if let Some(tls) = tls {
            builder = builder.tls_config(Arc::clone(tls));
        }
        builder.build()
    }

    /// Build the rustls client configuration from the given config.
    ///
    /// Returns `Ok(None)` when no TLS material is configured. Returns `Err`
    /// when material is configured but cannot be read or parsed.
    fn build_tls_config(
        config: &EsunHttpClientConfig,
    ) -> Result<Option<Arc<rustls::ClientConfig>>, String> {
        let client_auth = match (&config.client_cert_path, &config.client_key_path) {
            (Some(cert_path), Some(key_path)) => {
                let cert_pem = std::fs::read(cert_path)
                    .map_err(|e| format!("read client cert {cert_path}: {e}"))?;
                let key_pem = std::fs::read(key_path)
                    .map_err(|e| format!("read client key {key_path}: {e}"))?;
                let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_slice_iter(&cert_pem)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("parse client cert {cert_path}: {e}"))?;
                if certs.is_empty() {
                    return Err(format!(
                        "client cert {cert_path} contains no PEM certificate"
                    ));
                }
                let key = PrivateKeyDer::pem_slice_iter(&key_pem)
                    .next()
                    .ok_or_else(|| format!("client key {key_path} contains no PEM key"))?
                    .map_err(|e| format!("parse client key {key_path}: {e}"))?;
                Some((certs, key))
            }
            (None, None) => None,
            _ => {
                return Err(
                    "client_cert_path and client_key_path must be provided together".to_string(),
                )
            }
        };

        // Nothing configured at all: defer to the agent's default trust
        // store (which bundles public roots), keeping the default config
        // usable for public endpoints.
        if client_auth.is_none()
            && config.ca_cert_path.is_none()
            && config.pin_server_cert_sha256.is_empty()
        {
            return Ok(None);
        }

        // Any explicit TLS material needs a trust anchor; a root-less
        // store can never validate a server chain, so fail here instead
        // of at first handshake.
        let root_store = match &config.ca_cert_path {
            Some(ca_path) => {
                let ca_pem =
                    std::fs::read(ca_path).map_err(|e| format!("read CA cert {ca_path}: {e}"))?;
                let mut store = rustls::RootCertStore::empty();
                for cert in CertificateDer::pem_slice_iter(&ca_pem) {
                    let cert = cert.map_err(|e| format!("parse CA cert {ca_path}: {e}"))?;
                    store
                        .add(cert)
                        .map_err(|e| format!("add CA cert from {ca_path}: {e}"))?;
                }
                store
            }
            None => {
                return Err(
                    "ca_cert_path is required when client auth or pinning is configured; a root-less trust store cannot validate any server"
                        .to_string(),
                )
            }
        };

        if config.pin_server_cert_sha256.is_empty() {
            let builder = rustls::ClientConfig::builder().with_root_certificates(root_store);
            let client_config = match client_auth {
                Some((certs, key)) => builder
                    .with_client_auth_cert(certs, key)
                    .map_err(|e| format!("install client auth: {e}"))?,
                None => builder.with_no_client_auth(),
            };
            Ok(Some(Arc::new(client_config)))
        } else {
            let pins = normalize_pins(&config.pin_server_cert_sha256)?;
            let webpki_verifier =
                rustls::client::WebPkiServerVerifier::builder(Arc::new(root_store))
                    .build()
                    .map_err(|e| format!("build webpki verifier: {e}"))?;
            let pinned_verifier = PinnedCertVerifier {
                pins,
                inner: webpki_verifier,
            };
            let builder = rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(pinned_verifier));
            let client_config = match client_auth {
                Some((certs, key)) => builder
                    .with_client_auth_cert(certs, key)
                    .map_err(|e| format!("install client auth: {e}"))?,
                None => builder.with_no_client_auth(),
            };
            Ok(Some(Arc::new(client_config)))
        }
    }

    /// POST a JSON request and deserialize the `ApiResponse` envelope,
    /// applying the configured retry policy for idempotent calls.
    ///
    /// Non-2xx responses are still parsed (the security component reports
    /// business errors such as `910071` inside the envelope). Only transport
    /// failures, exhausted retries, and undecodable bodies are `Err(String)`.
    pub fn post_json<Req, Resp>(
        &self,
        path: &str,
        source_system: &str,
        req: &Req,
    ) -> Result<ApiResponse<Resp>, String>
    where
        Req: serde::Serialize,
        Resp: serde::de::DeserializeOwned + std::default::Default,
    {
        self.post_json_with_retry(path, source_system, req, self.config.retry.as_ref())
    }

    /// POST with an explicit retry policy (`None` = single attempt).
    ///
    /// Retry semantics (AWS Builders' Library, full-jitter capped
    /// exponential backoff):
    /// - transport failures and statuses 429/500-504 are retried up to
    ///   `policy.max_retries`;
    /// - a `Retry-After: <seconds>` header is honoured (capped at
    ///   `policy.max_delay_ms`) instead of the computed backoff;
    /// - business envelopes (any other status) are returned as-is after a
    ///   single attempt.
    pub(crate) fn post_json_with_retry<Req, Resp>(
        &self,
        path: &str,
        source_system: &str,
        req: &Req,
        retry: Option<&RetryConfig>,
    ) -> Result<ApiResponse<Resp>, String>
    where
        Req: serde::Serialize,
        Resp: serde::de::DeserializeOwned + std::default::Default,
    {
        if let Some(err) = &self.tls_load_error {
            return Err(format!(
                "refusing request to {path}: TLS configuration failed and silent fallback is not allowed: {err}"
            ));
        }
        let agent = self
            .http_client
            .as_ref()
            .ok_or_else(|| "HTTP agent not initialised".to_string())?;
        let url = format!("{}{}", self.config.base_url.trim_end_matches('/'), path);
        let json_body =
            serde_json::to_string(req).map_err(|e| format!("serialize request: {e}"))?;

        let mut completed_attempts = 0u32;
        loop {
            completed_attempts += 1;
            match Self::send_once(agent, &url, source_system, &json_body) {
                SendOutcome::Response(response) => return Self::parse_envelope(&url, response),
                SendOutcome::Transport(t) => {
                    let Some(policy) = retry else {
                        return Err(format!("HTTP request to {url} failed: {t}"));
                    };
                    if completed_attempts > policy.max_retries {
                        return Err(format!(
                            "HTTP request to {url} failed after {completed_attempts} attempt(s): {t}"
                        ));
                    }
                    self.sleep_between(policy, completed_attempts - 1);
                }
                SendOutcome::Status(code, response) if retryable_status(code) => {
                    let retry_after_secs = response
                        .header("Retry-After")
                        .and_then(|v| v.parse::<u64>().ok());
                    let Some(policy) = retry else {
                        return Self::parse_envelope(&url, response);
                    };
                    if completed_attempts > policy.max_retries {
                        return Self::parse_envelope(&url, response);
                    }
                    self.sleep_between_with_retry_after(
                        policy,
                        completed_attempts - 1,
                        retry_after_secs,
                    );
                }
                // Business errors arrive as JSON envelopes with non-2xx status.
                SendOutcome::Status(_code, response) => {
                    return Self::parse_envelope(&url, response)
                }
            }
        }
    }

    fn send_once(
        agent: &ureq::Agent,
        url: &str,
        source_system: &str,
        json_body: &str,
    ) -> SendOutcome {
        let mut request = agent.post(url).set("Content-Type", "application/json");
        if !source_system.is_empty() {
            request = request.set("Source-System", source_system);
        }
        match request.send_string(json_body) {
            Ok(response) => SendOutcome::Response(response),
            Err(ureq::Error::Transport(t)) => SendOutcome::Transport(t.to_string()),
            Err(ureq::Error::Status(code, response)) => SendOutcome::Status(code, response),
        }
    }

    fn parse_envelope<Resp>(
        url: &str,
        response: ureq::Response,
    ) -> Result<ApiResponse<Resp>, String>
    where
        Resp: serde::de::DeserializeOwned + std::default::Default,
    {
        let mut body = String::new();
        response
            .into_reader()
            .read_to_string(&mut body)
            .map_err(|e| format!("read response body from {url}: {e}"))?;

        serde_json::from_str(&body).map_err(|e| format!("deserialize response from {url}: {e}"))
    }

    /// Sleep before retry `attempt_index` (0-based): full jitter over the
    /// capped exponential delay.
    fn sleep_between(&self, policy: &RetryConfig, attempt_index: u32) {
        let base = policy.delay_for_attempt(attempt_index);
        let delay = jittered_delay(base, self.next_jitter_state());
        (self.sleep)(delay);
    }

    /// Honour `Retry-After: <seconds>` when present (capped at the policy
    /// maximum); otherwise fall back to the jittered backoff.
    fn sleep_between_with_retry_after(
        &self,
        policy: &RetryConfig,
        attempt_index: u32,
        retry_after_secs: Option<u64>,
    ) {
        let delay = match retry_after_secs {
            Some(secs) => Duration::from_millis(secs.saturating_mul(1000).min(policy.max_delay_ms)),
            None => jittered_delay(
                policy.delay_for_attempt(attempt_index),
                self.next_jitter_state(),
            ),
        };
        (self.sleep)(delay);
    }

    fn next_jitter_state(&self) -> u64 {
        let counter = self.jitter_state.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        counter.rotate_left(17) ^ nanos
    }

    /// POST `/payment/createTwdPaymentInstruction` (TWD payment release).
    pub fn create_twd_payment(
        &self,
        source_system: &str,
        request: &TwdPaymentRequest,
    ) -> Result<ApiResponse<TwdPaymentResponseData>, String> {
        // Non-idempotent: never retried (a timed-out POST may already have
        // been processed). Dispatch-uncertain reconciliation is the engine's
        // safe path (see release.rs / bankpay.rs query_twd_status).
        self.post_json_with_retry(
            "/payment/createTwdPaymentInstruction",
            source_system,
            request,
            None,
        )
    }

    /// POST `/payment/retrieveTwdPaymentInstructionResult` (TWD status query).
    pub fn retrieve_twd_status(
        &self,
        source_system: &str,
        request: &TwdStatusQueryRequest,
    ) -> Result<ApiResponse<TwdStatusResponseData>, String> {
        // Idempotent read: safe to retry on transport failures and
        // transient statuses (429, 500-504).
        self.post_json_with_retry(
            "/payment/retrieveTwdPaymentInstructionResult",
            source_system,
            request,
            self.config.retry.as_ref(),
        )
    }

    // -----------------------------------------------------------------------
    // Offline mock security component (QG-04)
    // -----------------------------------------------------------------------

    /// Synthetic payment-creation response for offline lifecycle tests.
    /// `mock_` prefix per invariant: synthetic data is never presented as a
    /// real bank response.
    pub fn mock_payment_response(
        &self,
        tx_seq: &str,
    ) -> Result<ApiResponse<TwdPaymentResponseData>, String> {
        if tx_seq.is_empty() {
            return Err("mock_payment_response: tx_seq must not be empty".to_string());
        }
        let record = TwdPaymentRecord {
            tx_seq: tx_seq.to_string(),
            ..Default::default()
        };
        Ok(ApiResponse {
            rs_header: RsHeader {
                msg_id: "MOCK".to_string(),
                log_id: "MOCK".to_string(),
                response_time: "1970/01/01 00:00:00".to_string(),
            },
            rs_status: RsStatus {
                code: "910000".to_string(),
                message: "SUCCESS".to_string(),
                more_info: String::new(),
            },
            rs_data: Some(TwdPaymentResponseData {
                data_count: "1".to_string(),
                tx_record_list: vec![record],
            }),
        })
    }

    /// Synthetic status-query response for offline lifecycle tests.
    pub fn mock_status_response(
        &self,
        bank_ref: &str,
    ) -> Result<ApiResponse<TwdStatusResponseData>, String> {
        if bank_ref.is_empty() {
            return Err("mock_status_response: bank_ref must not be empty".to_string());
        }
        Ok(ApiResponse {
            rs_header: RsHeader {
                msg_id: "MOCK".to_string(),
                log_id: "MOCK".to_string(),
                response_time: "1970/01/01 00:00:00".to_string(),
            },
            rs_status: RsStatus {
                code: "910000".to_string(),
                message: "SUCCESS".to_string(),
                more_info: String::new(),
            },
            rs_data: Some(TwdStatusResponseData {
                tx_status: "0000".to_string(),
                bank_ref: bank_ref.to_string(),
                tx_amt: String::new(),
                tx_date: String::new(),
                error_message: String::new(),
            }),
        })
    }
}

// ---------------------------------------------------------------------------
// Certificate pinning (QR-02 hardening, previously a stub)
// ---------------------------------------------------------------------------

/// Normalize configured pins: trim, lowercase, require 64 hex characters.
fn normalize_pins(pins: &[String]) -> Result<Vec<String>, String> {
    let mut normalized = Vec::with_capacity(pins.len());
    for pin in pins {
        let trimmed = pin.trim().to_ascii_lowercase();
        if trimmed.len() != 64 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "pinned server certificate SHA-256 must be 64 hex characters, got {pin:?}"
            ));
        }
        normalized.push(trimmed);
    }
    Ok(normalized)
}

/// Whether the SHA-256 of a leaf certificate DER matches any pin.
///
/// Empty `pins` returns `true` (safety net; the verifier is only installed
/// when pins are configured). Matching is case-insensitive defensively.
fn pin_cert_der_matches(pins: &[String], leaf_der: &[u8]) -> bool {
    if pins.is_empty() {
        return true;
    }
    let digest = Sha256::digest(leaf_der);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    pins.iter().any(|pin| pin.trim().eq_ignore_ascii_case(&hex))
}

/// Server certificate verifier that delegates to webpki and additionally
/// enforces SHA-256 leaf-certificate pinning (E.Sun QR-02).
#[derive(Debug)]
struct PinnedCertVerifier {
    pins: Vec<String>,
    inner: Arc<rustls::client::WebPkiServerVerifier>,
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // Full webpki chain validation first; pinning is additive.
        self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )?;
        if !pin_cert_der_matches(&self.pins, end_entity.as_ref()) {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::ApplicationVerificationFailure,
            ));
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

// ---------------------------------------------------------------------------
// Retry classification and jitter (AWS Builders' Library: full-jitter
// capped exponential backoff; retry 429/500-504 only, idempotent calls only)
// ---------------------------------------------------------------------------

/// Whether an HTTP status code is worth retrying for an idempotent call.
///
/// 429 (rate limited) and 500-504 (transient server-side failures; for the
/// local security component a 500-class usually means it is restarting).
/// Other 4xx are permanent client errors; 5xx outside 500-504 (501, 505,
/// ...) are permanent protocol conditions.
fn retryable_status(code: u16) -> bool {
    matches!(code, 429 | 500 | 502 | 503 | 504)
}

/// Full-jitter capped exponential backoff: uniform in `[0, base]`.
/// Deterministic given `state` (xorshift64*), which keeps tests reproducible.
fn jittered_delay(base: Duration, state: u64) -> Duration {
    if base.is_zero() {
        return base;
    }
    let mut x = state | 1;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    let r = x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11; // 53 random bits
    let frac = r as f64 / (1u64 << 53) as f64;
    Duration::from_secs_f64(base.as_secs_f64() * frac)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_tls_material() {
        let client = EsunHttpClient::new(EsunHttpClientConfig::default());
        assert!(client.tls_config.is_none());
        assert!(client.tls_load_error().is_none());
        assert!(client.http_client.is_some());
        assert_eq!(client.config.base_url, "https://127.0.0.1:8443");
        assert_eq!(client.config.timeout_secs, 30);
        assert!(client.config.pin_server_cert_sha256.is_empty());
    }

    #[test]
    fn missing_cert_files_block_requests_instead_of_downgrading() {
        let client = EsunHttpClient::new(EsunHttpClientConfig {
            client_cert_path: Some("Z:/definitely/missing/client.pem".to_string()),
            client_key_path: Some("Z:/definitely/missing/client.key".to_string()),
            ..EsunHttpClientConfig::default()
        });
        let err = client
            .tls_load_error()
            .expect("missing client cert files must produce a TLS load error");
        assert!(
            err.contains("client cert"),
            "error must name the failing material: {err}"
        );
        let result: Result<ApiResponse<serde_json::Value>, String> = client.post_json(
            "/payment/createTwdPaymentInstruction",
            "TEST-SYS",
            &serde_json::json!({}),
        );
        let err = result.expect_err("requests must be refused after TLS load failure");
        assert!(
            err.contains("refusing request"),
            "refusal message expected, got: {err}"
        );
    }

    #[test]
    fn half_configured_client_auth_is_rejected() {
        let client = EsunHttpClient::new(EsunHttpClientConfig {
            client_cert_path: Some("cert.pem".to_string()),
            client_key_path: None,
            ..EsunHttpClientConfig::default()
        });
        let err = client
            .tls_load_error()
            .expect("cert without key must be rejected");
        assert!(
            err.contains("together"),
            "pairing error expected, got: {err}"
        );
    }

    #[test]
    fn pinning_without_ca_is_rejected_at_construction() {
        let client = EsunHttpClient::new(EsunHttpClientConfig {
            pin_server_cert_sha256: vec!["ab".repeat(32)],
            ..EsunHttpClientConfig::default()
        });
        let err = client
            .tls_load_error()
            .expect("pinning without ca_cert_path must fail at construction");
        assert!(err.contains("ca_cert_path"), "got: {err}");
    }

    #[test]
    fn normalize_pins_accepts_and_normalizes() {
        let pins = vec![
            "  ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789  ".to_string(),
        ];
        let normalized = normalize_pins(&pins).expect("valid uppercase hex pin with spaces");
        assert_eq!(
            normalized[0],
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        );
    }

    #[test]
    fn normalize_pins_rejects_bad_input() {
        assert!(normalize_pins(&[]).is_ok());
        assert!(normalize_pins(&["abc".to_string()]).is_err());
        assert!(
            normalize_pins(&["g".repeat(64)]).is_err(),
            "non-hex rejected"
        );
        assert!(
            normalize_pins(&["a".repeat(63)]).is_err(),
            "63 chars rejected"
        );
    }

    #[test]
    fn pin_matching_is_case_insensitive() {
        let der = b"synthetic-leaf-der-bytes";
        let hex: String = Sha256::digest(der)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(pin_cert_der_matches(std::slice::from_ref(&hex), der));
        assert!(pin_cert_der_matches(&[hex.to_ascii_uppercase()], der));
        assert!(!pin_cert_der_matches(&[hex], b"different-der"));
    }

    #[test]
    fn empty_pins_is_a_safety_net_match() {
        assert!(pin_cert_der_matches(&[], b"anything"));
    }

    #[test]
    fn mock_payment_response_matches_e_sun_envelope() {
        let client = EsunHttpClient::new_test_client();
        let resp = client
            .mock_payment_response("IOB20261102001-0001")
            .expect("mock payment response");
        assert!(resp.is_success(), "code must be 910000");
        let data = resp.rs_data.expect("rs_data present");
        assert_eq!(data.data_count, "1");
        assert_eq!(data.tx_record_list.len(), 1);
        assert_eq!(data.tx_record_list[0].tx_seq, "IOB20261102001-0001");
    }

    #[test]
    fn mock_payment_response_rejects_empty_tx_seq() {
        let client = EsunHttpClient::new_test_client();
        assert!(client.mock_payment_response("").is_err());
    }

    #[test]
    fn mock_status_response_matches_e_sun_envelope() {
        let client = EsunHttpClient::new_test_client();
        let resp = client
            .mock_status_response("ETRACE-9001")
            .expect("mock status response");
        assert!(resp.is_success(), "code must be 910000");
        let data = resp.rs_data.expect("rs_data present");
        assert_eq!(data.tx_status, "0000");
        assert_eq!(data.bank_ref, "ETRACE-9001");
    }

    #[test]
    fn mock_status_response_rejects_empty_bank_ref() {
        let client = EsunHttpClient::new_test_client();
        assert!(client.mock_status_response("").is_err());
    }

    // -------------------------------------------------------------------
    // Retry semantics over a real loopback socket (no TLS material)
    // -------------------------------------------------------------------

    use crate::api::{FlowInfo, RqHeader, TwdPaymentRequestData, TwdStatusQueryRequestData};
    use std::collections::VecDeque;
    use std::sync::Mutex;

    enum MockAction {
        CloseConnection,
        Respond(String),
    }

    struct MockServer {
        port: u16,
        requests: Arc<std::sync::atomic::AtomicU32>,
    }

    impl MockServer {
        /// Start a raw loopback HTTP mock. Actions are consumed per
        /// connection in order; exhausted queues close the connection.
        fn start(actions: Vec<MockAction>) -> Self {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock server");
            let port = listener.local_addr().expect("local addr").port();
            let queue: Arc<Mutex<VecDeque<MockAction>>> =
                Arc::new(Mutex::new(VecDeque::from(actions)));
            let requests = Arc::new(std::sync::atomic::AtomicU32::new(0));
            {
                let queue = Arc::clone(&queue);
                let requests = Arc::clone(&requests);
                std::thread::spawn(move || {
                    for stream in listener.incoming().flatten() {
                        requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let action = queue.lock().expect("mock queue lock").pop_front();
                        let mut stream = stream;
                        match action {
                            Some(MockAction::CloseConnection) | None => drop(stream),
                            Some(MockAction::Respond(raw)) => {
                                drain_request(&mut stream);
                                let _ = std::io::Write::write_all(&mut stream, raw.as_bytes());
                                let _ = std::io::Write::flush(&mut stream);
                            }
                        }
                    }
                });
            }
            Self { port, requests }
        }

        fn base_url(&self) -> String {
            format!("http://127.0.0.1:{}", self.port)
        }

        fn request_count(&self) -> u32 {
            self.requests.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    /// Read the full HTTP request (headers plus Content-Length body) before
    /// responding. Closing a socket with unread inbound data makes Windows
    /// send RST, which discards the response on the client (os error 10053).
    fn drain_request(stream: &mut std::net::TcpStream) {
        let deadline = std::time::Instant::now() + Duration::from_millis(2000);
        let mut buf: Vec<u8> = Vec::with_capacity(2048);
        let mut scratch = [0u8; 2048];
        let mut headers_end: Option<usize> = None;
        loop {
            if std::time::Instant::now() >= deadline {
                break;
            }
            let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
            match std::io::Read::read(stream, &mut scratch) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&scratch[..n]);
                    if headers_end.is_none() {
                        headers_end = find_subsequence(&buf, b"\r\n\r\n").map(|p| p + 4);
                    }
                    if let Some(he) = headers_end {
                        match content_length(&buf[..he]) {
                            Some(cl) if buf.len() >= he + cl => break,
                            Some(_) => {}
                            None => break, // no body expected
                        }
                    }
                }
                Err(_) => {
                    // Read timeout: keep polling until headers + body are in
                    // or the deadline passes.
                    if let Some(he) = headers_end {
                        if let Some(cl) = content_length(&buf[..he]) {
                            if buf.len() >= he + cl {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.len() > haystack.len() {
            return None;
        }
        haystack.windows(needle.len()).position(|w| w == needle)
    }

    fn content_length(headers: &[u8]) -> Option<usize> {
        let text = std::str::from_utf8(headers).ok()?;
        text.split("\r\n").find_map(|line| {
            let value = line
                .strip_prefix("Content-Length:")
                .or_else(|| line.strip_prefix("content-length:"))?;
            value.trim().parse::<usize>().ok()
        })
    }

    fn status_envelope_http(status_line: &str, envelope_code: &str) -> String {
        let body = serde_json::json!({
            "rsHeader": {
                "msgId": "MOCK", "logId": "MOCK", "responseTime": "1970/01/01 00:00:00"
            },
            "rsStatus": {"code": envelope_code, "message": "SUCCESS", "moreInfo": ""},
            "rsData": {
                "txStatus": "0000", "bankRef": "ETRACE-9001",
                "txAmt": "", "txDate": "", "errorMessage": ""
            }
        })
        .to_string();
        format!(
            "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
    }

    fn sample_rq_header() -> RqHeader {
        RqHeader {
            msg_id: "12345678202611060000001".to_string(),
            request_time: "2026/11/06 10:00:00".to_string(),
            customer_ixd: "12345678".to_string(),
            source_system: "ERP-BANKPAY".to_string(),
        }
    }

    fn sample_status_query() -> TwdStatusQueryRequest {
        TwdStatusQueryRequest {
            rq_header: sample_rq_header(),
            rq_data: TwdStatusQueryRequestData {
                payment_no: "PAY12345678".to_string(),
                tx_seq: "IOB20261102001-0001".to_string(),
            },
        }
    }

    fn sample_payment_request() -> TwdPaymentRequest {
        TwdPaymentRequest {
            rq_header: sample_rq_header(),
            rq_data: TwdPaymentRequestData {
                format_type: "2".to_string(),
                payment_type: "2".to_string(),
                verification_type: "1".to_string(),
                flow_info: FlowInfo::default(),
                content: "{}".to_string(),
            },
        }
    }

    fn no_sleep_client(base_url: String) -> EsunHttpClient {
        EsunHttpClient::with_sleep(
            EsunHttpClientConfig {
                base_url,
                ..EsunHttpClientConfig::default()
            },
            Arc::new(|_| {}),
        )
    }

    #[test]
    fn retryable_status_covers_only_transient_codes() {
        assert!(retryable_status(429));
        assert!(retryable_status(500));
        assert!(retryable_status(502));
        assert!(retryable_status(503));
        assert!(retryable_status(504));
        assert!(!retryable_status(400));
        assert!(!retryable_status(401));
        assert!(!retryable_status(404));
        assert!(!retryable_status(501));
        assert!(!retryable_status(505));
    }

    #[test]
    fn jittered_delay_stays_within_base_and_is_deterministic() {
        let base = Duration::from_millis(500);
        let d1 = jittered_delay(base, 0x0DDB_1A5E_5BAD_5EED);
        let d2 = jittered_delay(base, 0x0DDB_1A5E_5BAD_5EED);
        assert_eq!(d1, d2, "same state must be deterministic");
        assert!(d1 <= base, "full jitter must stay in [0, base]");
        assert_eq!(jittered_delay(Duration::ZERO, 1), Duration::ZERO);
        let values: Vec<Duration> = (1u64..=8).map(|s| jittered_delay(base, s)).collect();
        assert!(values.iter().all(|d| *d <= base));
        assert!(
            values.iter().any(|d| *d != values[0]),
            "jitter must spread retries across time"
        );
    }

    #[test]
    fn status_query_retries_transport_failures_until_success() {
        let server = MockServer::start(vec![
            MockAction::CloseConnection,
            MockAction::CloseConnection,
            MockAction::Respond(status_envelope_http("HTTP/1.1 200 OK", "910000")),
        ]);
        let client = no_sleep_client(server.base_url());
        let resp = client
            .retrieve_twd_status("ERP-BANKPAY", &sample_status_query())
            .expect("third attempt must succeed");
        assert!(resp.is_success(), "envelope code must be 910000");
        assert_eq!(
            server.request_count(),
            3,
            "two transient failures plus one success"
        );
    }

    #[test]
    fn status_query_retry_exhaustion_reports_attempts() {
        let server = MockServer::start(vec![
            MockAction::CloseConnection,
            MockAction::CloseConnection,
            MockAction::CloseConnection,
            MockAction::CloseConnection,
        ]);
        let client = no_sleep_client(server.base_url());
        let err = client
            .retrieve_twd_status("ERP-BANKPAY", &sample_status_query())
            .expect_err("all attempts fail");
        assert!(
            err.contains("after 4 attempt"),
            "must report 1 + 3 retries attempts: {err}"
        );
        assert_eq!(server.request_count(), 4);
    }

    #[test]
    fn payment_creation_is_never_retried() {
        let server = MockServer::start(vec![
            MockAction::CloseConnection,
            MockAction::CloseConnection,
            MockAction::CloseConnection,
            MockAction::CloseConnection,
        ]);
        let client = no_sleep_client(server.base_url());
        let err = client
            .create_twd_payment("ERP-BANKPAY", &sample_payment_request())
            .expect_err("transport failure without retry");
        assert!(
            !err.contains("after"),
            "single attempt must not report retry exhaustion: {err}"
        );
        assert_eq!(server.request_count(), 1, "non-idempotent POST: no retry");
    }

    #[test]
    fn http_429_is_retried_then_succeeds() {
        let server = MockServer::start(vec![
            MockAction::Respond(status_envelope_http(
                "HTTP/1.1 429 Too Many Requests",
                "910000",
            )),
            MockAction::Respond(status_envelope_http("HTTP/1.1 200 OK", "910000")),
        ]);
        let client = no_sleep_client(server.base_url());
        let resp = client
            .retrieve_twd_status("ERP-BANKPAY", &sample_status_query())
            .expect("retry after 429 must succeed");
        assert!(resp.is_success());
        assert_eq!(server.request_count(), 2);
    }

    #[test]
    fn http_4xx_is_not_retried_and_envelope_is_returned() {
        let server = MockServer::start(vec![MockAction::Respond(status_envelope_http(
            "HTTP/1.1 400 Bad Request",
            "910071",
        ))]);
        let client = no_sleep_client(server.base_url());
        let resp = client
            .retrieve_twd_status("ERP-BANKPAY", &sample_status_query())
            .expect("business envelope must be returned as Ok");
        assert!(!resp.is_success(), "910071 is a business error");
        assert_eq!(resp.rs_status.code, "910071");
        assert_eq!(server.request_count(), 1, "4xx is never retried");
    }

    #[test]
    fn retry_after_header_is_respected_and_capped() {
        // Retry-After: 7 seconds, but policy caps the sleep at 2 s
        // (max_delay_ms of http_retry()).
        let seen: Arc<Mutex<Vec<Duration>>> = Arc::new(Mutex::new(Vec::new()));
        let seen_for_sleep = Arc::clone(&seen);
        let server = MockServer::start(vec![
            MockAction::Respond(
                "HTTP/1.1 503 Service Unavailable\r\nRetry-After: 7\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_string(),
            ),
            MockAction::Respond(status_envelope_http("HTTP/1.1 200 OK", "910000")),
        ]);
        let client = EsunHttpClient::with_sleep(
            EsunHttpClientConfig {
                base_url: server.base_url(),
                ..EsunHttpClientConfig::default()
            },
            Arc::new(move |d: Duration| {
                seen_for_sleep.lock().expect("seen lock").push(d);
            }),
        );
        let resp = client
            .retrieve_twd_status("ERP-BANKPAY", &sample_status_query())
            .expect("retry after 503 must succeed");
        assert!(resp.is_success());
        let slept = seen.lock().expect("seen lock").clone();
        assert_eq!(
            slept,
            vec![Duration::from_millis(2000)],
            "Retry-After 7 s must be honoured but capped at max_delay_ms"
        );
    }
}

#[cfg(test)]
mod slice_audit_tests {
    //! Slice-audit tests (Test Engineer report): retry policy bounds,
    //! payment-never-retried invariant, retryable classification, jitter
    //! bounds and certificate pinning. All local - no network, no bank.

    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn client_with_sleep_counter(sleeps: Arc<AtomicUsize>) -> EsunHttpClient {
        EsunHttpClient::with_sleep(
            EsunHttpClientConfig {
                base_url: "http://127.0.0.1:1".to_string(), // connection refused
                retry: Some(EsunHttpClientConfig::http_retry()),
                ..Default::default()
            },
            Arc::new(move |_| {
                sleeps.fetch_add(1, Ordering::SeqCst);
            }),
        )
    }

    #[test]
    fn idempotent_status_query_retries_three_times_then_fails() {
        let sleeps = Arc::new(AtomicUsize::new(0));
        let client = client_with_sleep_counter(sleeps.clone());
        let result = client.post_json_with_retry::<serde_json::Value, serde_json::Value>(
            "/status/none",
            "TEST",
            &serde_json::json!({}),
            Some(&EsunHttpClientConfig::http_retry()),
        );
        assert!(result.is_err(), "transport errors must surface as Err");
        // http_retry(): max_retries = 3 -> attempts 1..=3 sleep, attempt 4 aborts.
        assert_eq!(sleeps.load(Ordering::SeqCst), 3, "3 sleeps = 3 retries");
    }

    #[test]
    fn payment_creation_is_never_retried_on_transport_error() {
        // Invariant: create_twd_payment passes retry = None. A timed-out POST
        // may already have been processed by the bank - a blind retry risks
        // double payment. Assert zero sleeps (zero retries) on transport error.
        let sleeps = Arc::new(AtomicUsize::new(0));
        let sleeps_counter = sleeps.clone();
        let client = EsunHttpClient::with_sleep(
            EsunHttpClientConfig {
                base_url: "http://127.0.0.1:1".to_string(),
                retry: None,
                ..Default::default()
            },
            Arc::new(move |_| {
                sleeps_counter.fetch_add(1, Ordering::SeqCst);
            }),
        );
        let result = client.post_json_with_retry::<serde_json::Value, serde_json::Value>(
            "/payment/createTwdPaymentInstruction",
            "TEST",
            &serde_json::json!({}),
            None,
        );
        assert!(result.is_err());
        assert_eq!(
            sleeps.load(Ordering::SeqCst),
            0,
            "payment creation must never retry (double-payment risk)"
        );
    }

    #[test]
    fn retryable_status_classification() {
        for code in [429u16, 500, 502, 503, 504] {
            assert!(retryable_status(code), "{code} must be retryable");
        }
        for code in [200u16, 400, 401, 403, 404] {
            assert!(!retryable_status(code), "{code} must not be retryable");
        }
    }

    #[test]
    fn jittered_delay_stays_within_base_and_handles_zero() {
        assert_eq!(jittered_delay(Duration::ZERO, 42), Duration::ZERO);
        for state in [0u64, 1, 42, u64::MAX] {
            let d = jittered_delay(Duration::from_millis(200), state);
            assert!(d <= Duration::from_millis(200), "full jitter cap: {d:?}");
        }
    }

    #[test]
    fn normalize_pins_trims_lowercases_and_rejects_bad_shapes() {
        // An 8-char pin can never be valid - trim does not relax the length rule.
        assert!(normalize_pins(&[" ABCD1234 ".to_string()]).is_err());
        let ok = normalize_pins(&[
            "  Ab23456789012345678901234567890123456789012345678901234567890123  ".to_string(),
        ])
        .expect("valid hex");
        assert_eq!(
            ok[0],
            "ab23456789012345678901234567890123456789012345678901234567890123"
        );
    }

    #[test]
    fn pin_cert_der_matches_verifies_digest() {
        let der = vec![1u8, 2, 3, 4];
        use sha2::Digest;
        let hex: String = sha2::Sha256::digest(&der)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(pin_cert_der_matches(std::slice::from_ref(&hex), &der));
        assert!(
            pin_cert_der_matches(&["  ".to_string() + &hex.to_uppercase() + "  "], &der),
            "case/space tolerant"
        );
        assert!(
            !pin_cert_der_matches(&["deadbeef".to_string()], &der),
            "wrong digest rejected"
        );
        assert!(
            pin_cert_der_matches(&[], &der),
            "empty pins = no pinning = allow"
        );
    }

    #[test]
    fn timeout_secs_degrades_to_error_not_hang() {
        // Local listener that accepts but never responds: the 1s client
        // timeout must surface as Err (not a hang).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let handle = std::thread::spawn(move || {
            let (_s, _) = listener.accept().expect("accept");
            std::thread::sleep(std::time::Duration::from_secs(5));
        });
        let client = EsunHttpClient::new(EsunHttpClientConfig {
            base_url: format!("https://127.0.0.1:{port}"),
            timeout_secs: 1,
            ..Default::default()
        });
        let result = client.post_json_with_retry::<serde_json::Value, serde_json::Value>(
            "/timeout/probe",
            "TEST",
            &serde_json::json!({}),
            None,
        );
        assert!(result.is_err(), "timeout must degrade to Err");
        drop(handle);
    }
}
