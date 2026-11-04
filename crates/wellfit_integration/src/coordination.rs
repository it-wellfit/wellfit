#![forbid(unsafe_code)]

//! ERP Coordination — dual-track reconciliation and feature-flag switching.
//!
//! Manages the coordination between the new system (Wellfit) and the
//! legacy system (FlapsErp). Dual-track mode runs both systems in
//! parallel; feature-flag switching allows staged cutover with rollback.

use std::collections::HashMap;
use wellfit_core::audit_integrity::compute_hash;

/// Coordination mode for dual-track execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationMode {
    /// Both systems run in parallel, data is compared continuously.
    DualTrack,
    /// New system is primary, legacy system is shadow.
    ShadowLegacy,
    /// New system is primary, legacy system is read-only.
    LegacyReadOnly,
    /// New system fully owns all operations, legacy system decommissioned.
    Decommissioned,
}

impl CoordinationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoordinationMode::DualTrack => "DUAL_TRACK",
            CoordinationMode::ShadowLegacy => "SHADOW_LEGACY",
            CoordinationMode::LegacyReadOnly => "LEGACY_READ_ONLY",
            CoordinationMode::Decommissioned => "DECOMMISSIONED",
        }
    }
}

/// Reconciliation diff entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationDiff {
    pub key: String,
    pub new_value_hash: String,
    pub legacy_value_hash: String,
    pub is_match: bool,
    pub diff_type: String,
}

/// Dual-track reconciliation result.
#[derive(Debug, Clone)]
pub struct ReconciliationResult {
    pub batch_id: String,
    pub total_records: usize,
    pub matched_records: usize,
    pub diff_records: usize,
    pub match_rate: f64,
    pub diffs: Vec<ReconciliationDiff>,
    pub audit_hash: String,
}

/// Errors from the coordination layer.
#[derive(Debug)]
pub enum CoordinationError {
    InvalidTransition(String),
    ReconciliationFailed(String),
    RollbackFailed(String),
}

impl std::fmt::Display for CoordinationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordinationError::InvalidTransition(msg) => write!(f, "invalid transition: {msg}"),
            CoordinationError::ReconciliationFailed(msg) => {
                write!(f, "reconciliation failed: {msg}")
            }
            CoordinationError::RollbackFailed(msg) => write!(f, "rollback failed: {msg}"),
        }
    }
}

impl std::error::Error for CoordinationError {}

/// ERP Coordination Engine — manages dual-track reconciliation and switching.
pub struct CoordinationEngine {
    mode: CoordinationMode,
    reconciliation_history: Vec<ReconciliationResult>,
    record_hashes: HashMap<String, (String, String)>, // key → (new_hash, legacy_hash)
}

impl CoordinationEngine {
    /// Create a new coordination engine in DualTrack mode.
    pub fn new() -> Self {
        Self {
            mode: CoordinationMode::DualTrack,
            reconciliation_history: Vec::new(),
            record_hashes: HashMap::new(),
        }
    }

    /// Create with a specific initial mode.
    pub fn with_mode(mode: CoordinationMode) -> Self {
        Self {
            mode,
            reconciliation_history: Vec::new(),
            record_hashes: HashMap::new(),
        }
    }

    /// Get the current coordination mode.
    pub fn mode(&self) -> CoordinationMode {
        self.mode
    }

    /// Request a mode transition (validates transition rules).
    ///
    /// Valid transitions:
    /// - DualTrack → ShadowLegacy (new system proven, switch to primary)
    /// - ShadowLegacy → LegacyReadOnly (legacy stable, read-only)
    /// - LegacyReadOnly → Decommissioned (final cutover)
    /// - Any mode → DualTrack (rollback)
    pub fn transition_to(&mut self, target: CoordinationMode) -> Result<(), CoordinationError> {
        let valid = match (self.mode, target) {
            (CoordinationMode::DualTrack, CoordinationMode::ShadowLegacy) => true,
            (CoordinationMode::ShadowLegacy, CoordinationMode::LegacyReadOnly) => true,
            (CoordinationMode::LegacyReadOnly, CoordinationMode::Decommissioned) => true,
            (CoordinationMode::DualTrack, CoordinationMode::DualTrack) => false, // no-op
            (CoordinationMode::ShadowLegacy, CoordinationMode::ShadowLegacy) => false, // no-op
            (CoordinationMode::LegacyReadOnly, CoordinationMode::LegacyReadOnly) => false, // no-op
            (CoordinationMode::Decommissioned, CoordinationMode::Decommissioned) => false, // no-op
            (CoordinationMode::Decommissioned, CoordinationMode::DualTrack) => false, // no rollback
            // Rollback transitions (any → DualTrack, except Decommissioned)
            (_, CoordinationMode::DualTrack) => true,
            // Skip transitions not explicitly allowed
            _ => false,
        };

        if valid {
            self.mode = target;
            Ok(())
        } else {
            Err(CoordinationError::InvalidTransition(format!(
                "cannot transition from {} to {}",
                self.mode.as_str(),
                target.as_str()
            )))
        }
    }

    /// Record a data point from the new system.
    pub fn record_new(&mut self, key: &str, value: &str) {
        let hash = compute_hash(value);
        let entry = self
            .record_hashes
            .entry(key.to_string())
            .or_insert((String::new(), String::new()));
        entry.0 = hash;
    }

    /// Record a data point from the legacy system.
    pub fn record_legacy(&mut self, key: &str, value: &str) {
        let hash = compute_hash(value);
        let entry = self
            .record_hashes
            .entry(key.to_string())
            .or_insert((String::new(), String::new()));
        entry.1 = hash;
    }

    /// Run reconciliation: compare new vs legacy hashes for all recorded keys.
    pub fn reconcile(&mut self, batch_id: &str) -> ReconciliationResult {
        let total = self.record_hashes.len();
        let mut matched = 0;
        let mut diffs = Vec::new();

        for (key, (new_hash, legacy_hash)) in &self.record_hashes {
            if new_hash.is_empty() && legacy_hash.is_empty() {
                continue;
            }
            let is_match = new_hash == legacy_hash;
            if is_match {
                matched += 1;
            }
            if !is_match {
                diffs.push(ReconciliationDiff {
                    key: key.clone(),
                    new_value_hash: new_hash.clone(),
                    legacy_value_hash: legacy_hash.clone(),
                    is_match,
                    diff_type: if new_hash.is_empty() {
                        "MISSING_IN_NEW".to_string()
                    } else if legacy_hash.is_empty() {
                        "MISSING_IN_LEGACY".to_string()
                    } else {
                        "VALUE_MISMATCH".to_string()
                    },
                });
            }
        }

        let diff_count = diffs.len();
        let match_rate = if total > 0 {
            matched as f64 / total as f64
        } else {
            0.0
        };

        let audit_hash = compute_hash(&format!(
            "reconciliation:batch={}:total={}:matched={}:diffs={}:rate={:.4}",
            batch_id, total, matched, diff_count, match_rate
        ));

        let result = ReconciliationResult {
            batch_id: batch_id.to_string(),
            total_records: total,
            matched_records: matched,
            diff_records: diff_count,
            match_rate,
            diffs,
            audit_hash,
        };

        self.reconciliation_history.push(result.clone());
        result
    }

    /// Get reconciliation history.
    pub fn history(&self) -> &[ReconciliationResult] {
        &self.reconciliation_history
    }

    /// Get the latest reconciliation result.
    pub fn latest_reconciliation(&self) -> Option<&ReconciliationResult> {
        self.reconciliation_history.last()
    }

    /// Check if reconciliation meets the cutover threshold.
    ///
    /// Returns true if match rate >= threshold (default 99.5%).
    pub fn meets_cutover_threshold(&self, threshold: f64) -> bool {
        if let Some(latest) = self.latest_reconciliation() {
            latest.match_rate >= threshold
        } else {
            false
        }
    }

    /// Rollback to DualTrack mode (emergency rollback).
    pub fn rollback(&mut self) -> Result<(), CoordinationError> {
        if self.mode == CoordinationMode::Decommissioned {
            return Err(CoordinationError::RollbackFailed(
                "cannot rollback from Decommissioned".to_string(),
            ));
        }
        self.mode = CoordinationMode::DualTrack;
        Ok(())
    }
}

impl Default for CoordinationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = CoordinationEngine::new();
        assert_eq!(engine.mode(), CoordinationMode::DualTrack);
        assert!(engine.history().is_empty());
    }

    #[test]
    fn test_mode_as_str() {
        assert_eq!(CoordinationMode::DualTrack.as_str(), "DUAL_TRACK");
        assert_eq!(CoordinationMode::ShadowLegacy.as_str(), "SHADOW_LEGACY");
        assert_eq!(
            CoordinationMode::LegacyReadOnly.as_str(),
            "LEGACY_READ_ONLY"
        );
        assert_eq!(CoordinationMode::Decommissioned.as_str(), "DECOMMISSIONED");
    }

    #[test]
    fn test_valid_transition_dual_to_shadow() {
        let mut engine = CoordinationEngine::new();
        assert!(engine.transition_to(CoordinationMode::ShadowLegacy).is_ok());
        assert_eq!(engine.mode(), CoordinationMode::ShadowLegacy);
    }

    #[test]
    fn test_valid_transition_shadow_to_legacy_readonly() {
        let mut engine = CoordinationEngine::with_mode(CoordinationMode::ShadowLegacy);
        assert!(engine
            .transition_to(CoordinationMode::LegacyReadOnly)
            .is_ok());
        assert_eq!(engine.mode(), CoordinationMode::LegacyReadOnly);
    }

    #[test]
    fn test_invalid_transition_dual_to_decommissioned() {
        let mut engine = CoordinationEngine::new();
        // Must go through ShadowLegacy → LegacyReadOnly → Decommissioned
        assert!(engine
            .transition_to(CoordinationMode::Decommissioned)
            .is_err());
    }

    #[test]
    fn test_rollback_to_dual_track() {
        let mut engine = CoordinationEngine::with_mode(CoordinationMode::ShadowLegacy);
        assert!(engine.rollback().is_ok());
        assert_eq!(engine.mode(), CoordinationMode::DualTrack);
    }

    #[test]
    fn test_rollback_from_decommissioned_fails() {
        let mut engine = CoordinationEngine::with_mode(CoordinationMode::Decommissioned);
        assert!(engine.rollback().is_err());
    }

    #[test]
    fn test_reconciliation_all_match() {
        let mut engine = CoordinationEngine::new();
        engine.record_new("key1", "value1");
        engine.record_legacy("key1", "value1");
        engine.record_new("key2", "value2");
        engine.record_legacy("key2", "value2");

        let result = engine.reconcile("batch-001");
        assert_eq!(result.total_records, 2);
        assert_eq!(result.matched_records, 2);
        assert_eq!(result.diff_records, 0);
        assert!((result.match_rate - 1.0).abs() < 0.001);
        assert!(result.diffs.is_empty());
    }

    #[test]
    fn test_reconciliation_with_diffs() {
        let mut engine = CoordinationEngine::new();
        engine.record_new("key1", "value1");
        engine.record_legacy("key1", "value1");
        engine.record_new("key2", "value2");
        engine.record_legacy("key2", "different_value");

        let result = engine.reconcile("batch-001");
        assert_eq!(result.total_records, 2);
        assert_eq!(result.matched_records, 1);
        assert_eq!(result.diff_records, 1);
        assert!((result.match_rate - 0.5).abs() < 0.001);
        assert_eq!(result.diffs.len(), 1);
        assert_eq!(result.diffs[0].key, "key2");
        assert_eq!(result.diffs[0].diff_type, "VALUE_MISMATCH");
    }

    #[test]
    fn test_reconciliation_missing_in_new() {
        let mut engine = CoordinationEngine::new();
        engine.record_legacy("key1", "value1");
        // No record_new for key1

        let result = engine.reconcile("batch-001");
        assert_eq!(result.diff_records, 1);
        assert_eq!(result.diffs[0].diff_type, "MISSING_IN_NEW");
    }

    #[test]
    fn test_meets_cutover_threshold() {
        let mut engine = CoordinationEngine::new();
        engine.record_new("key1", "value1");
        engine.record_legacy("key1", "value1");
        engine.record_new("key2", "value2");
        engine.record_legacy("key2", "value2");
        engine.reconcile("batch-001");

        assert!(engine.meets_cutover_threshold(0.995));
        assert!(engine.meets_cutover_threshold(1.0));
        assert!(!engine.meets_cutover_threshold(1.1));
    }

    #[test]
    fn test_reconciliation_history() {
        let mut engine = CoordinationEngine::new();
        engine.record_new("key1", "v1");
        engine.record_legacy("key1", "v1");
        engine.reconcile("batch-001");

        engine.record_new("key2", "v2");
        engine.record_legacy("key2", "v2");
        engine.reconcile("batch-002");

        assert_eq!(engine.history().len(), 2);
        assert_eq!(
            engine.latest_reconciliation().unwrap().batch_id,
            "batch-002"
        );
    }

    #[test]
    fn test_audit_hash_non_empty() {
        let mut engine = CoordinationEngine::new();
        engine.record_new("key1", "v1");
        engine.record_legacy("key1", "v1");
        let result = engine.reconcile("batch-001");
        assert!(!result.audit_hash.is_empty());
        assert_eq!(result.audit_hash.len(), 64);
    }

    #[test]
    fn test_coordination_error_display() {
        let err = CoordinationError::InvalidTransition("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = CoordinationError::RollbackFailed("cannot rollback".to_string());
        assert!(err.to_string().contains("cannot rollback"));
    }
}
