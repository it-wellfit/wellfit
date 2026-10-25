#![forbid(unsafe_code)]

//! Wellfit Core - Federated Cross-Project Data Highway Contracts
//!
//! Implements clean-room RFC-2020-12 data contract serialization and SHA-256 digital
//! envelopes, enabling 1+1+1+1 > 16 super-additive value across Website Detective,
//! AI Bidding ERP, Mendao OS, and Wellfit ERP without compile-time coupling.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub indicator_name: String,
    pub epistemic_tier: String,
    pub value_observed: serde_json::Value,
    pub source_uri: String,
    pub observed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEvidenceBundle {
    pub bundle_id: String,
    pub target_domain: String,
    pub issued_at: String,
    pub evidence_count: usize,
    pub items: Vec<EvidenceItem>,
    pub digital_signature_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwardedTenderHandover {
    pub handover_id: String,
    pub tender_id: String,
    pub buyer_org: String,
    pub contract_amount_cents: i64,
    pub awarded_items: Vec<TenderItem>,
    pub digital_signature_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenderItem {
    pub item_code: String,
    pub description: String,
    pub quantity: i64,
    pub unit_price_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalRealityFeedback {
    pub feedback_id: String,
    pub reporting_system: String, // "WELLFIT_ERP"
    pub batch_id: String,
    pub total_processed_volume_cents: i64,
    pub error_quarantine_count: usize,
    pub reported_at: String,
    pub digital_signature_sha256: String,
}

impl OperationalRealityFeedback {
    pub fn new(
        feedback_id: String,
        batch_id: String,
        total_processed_volume_cents: i64,
        error_quarantine_count: usize,
    ) -> Self {
        let reported_at = chrono::Utc::now().to_rfc3339();
        let payload = format!(
            "{}:{}:{}:{}:{}",
            feedback_id,
            batch_id,
            total_processed_volume_cents,
            error_quarantine_count,
            reported_at
        );
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let digital_signature_sha256 = format!("{:x}", hasher.finalize());

        Self {
            feedback_id,
            reporting_system: "WELLFIT_ERP".to_string(),
            batch_id,
            total_processed_volume_cents,
            error_quarantine_count,
            reported_at,
            digital_signature_sha256,
        }
    }
}
