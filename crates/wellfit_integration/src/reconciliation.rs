#![forbid(unsafe_code)]

//! Reconciliation — data comparison, diff detection, and sync resolution
//! between the new system and legacy system.

use std::collections::HashMap;
use wellfit_core::audit_integrity::{verify_hash_chain, IntegrityCheckResult};

/// Field-level diff for a single record.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDiff {
    pub field_name: String,
    pub new_value: String,
    pub legacy_value: String,
    pub is_match: bool,
}

/// Record-level reconciliation result.
#[derive(Debug, Clone)]
pub struct RecordReconciliation {
    pub record_key: String,
    pub field_diffs: Vec<FieldDiff>,
    pub is_match: bool,
}

/// Reconciliation threshold configuration.
#[derive(Debug, Clone, Copy)]
pub struct ReconciliationConfig {
    /// Minimum match rate required for cutover (default 99.5%).
    pub cutover_threshold: f64,
    /// Maximum allowed diff records for cutover.
    pub max_diff_records: usize,
    /// Whether to fail on missing records.
    pub fail_on_missing: bool,
}

impl Default for ReconciliationConfig {
    fn default() -> Self {
        Self {
            cutover_threshold: 0.995,
            max_diff_records: 5,
            fail_on_missing: true,
        }
    }
}

/// Reconciliation engine for comparing new vs legacy data.
pub struct ReconciliationEngine {
    config: ReconciliationConfig,
}

impl ReconciliationEngine {
    pub fn new(config: ReconciliationConfig) -> Self {
        Self { config }
    }

    /// Compare two records field by field.
    pub fn compare_records(
        &self,
        key: &str,
        new_fields: &HashMap<String, String>,
        legacy_fields: &HashMap<String, String>,
    ) -> RecordReconciliation {
        let mut field_diffs = Vec::new();
        let mut all_match = true;

        // Check all fields from both sides
        let all_fields: std::collections::HashSet<&String> =
            new_fields.keys().chain(legacy_fields.keys()).collect();

        for field_name in all_fields {
            let new_value = new_fields.get(field_name).cloned().unwrap_or_default();
            let legacy_value = legacy_fields.get(field_name).cloned().unwrap_or_default();
            let is_match = new_value == legacy_value;
            if !is_match {
                all_match = false;
            }
            field_diffs.push(FieldDiff {
                field_name: field_name.clone(),
                new_value,
                legacy_value,
                is_match,
            });
        }

        RecordReconciliation {
            record_key: key.to_string(),
            field_diffs,
            is_match: all_match,
        }
    }

    /// Determine if the reconciliation meets the cutover threshold.
    pub fn meets_cutover(&self, result: &crate::coordination::ReconciliationResult) -> bool {
        result.match_rate >= self.config.cutover_threshold
            && result.diff_records <= self.config.max_diff_records
    }

    /// Verify integrity of reconciliation results via hash chain.
    pub fn verify_integrity(
        &self,
        results: &[crate::coordination::ReconciliationResult],
    ) -> IntegrityCheckResult {
        let hashes: Vec<String> = results.iter().map(|r| r.audit_hash.clone()).collect();
        verify_hash_chain(&hashes)
    }
}

impl Default for ReconciliationEngine {
    fn default() -> Self {
        Self::new(ReconciliationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wellfit_core::audit_integrity::compute_hash;

    #[test]
    fn test_compare_records_match() {
        let engine = ReconciliationEngine::default();
        let new_fields = HashMap::from([
            ("name".to_string(), "Alice".to_string()),
            ("age".to_string(), "30".to_string()),
        ]);
        let legacy_fields = HashMap::from([
            ("name".to_string(), "Alice".to_string()),
            ("age".to_string(), "30".to_string()),
        ]);

        let result = engine.compare_records("rec-001", &new_fields, &legacy_fields);
        assert!(result.is_match);
        assert_eq!(result.field_diffs.len(), 2);
        assert!(result.field_diffs.iter().all(|d| d.is_match));
    }

    #[test]
    fn test_compare_records_diff() {
        let engine = ReconciliationEngine::default();
        let new_fields = HashMap::from([
            ("name".to_string(), "Alice".to_string()),
            ("age".to_string(), "31".to_string()),
        ]);
        let legacy_fields = HashMap::from([
            ("name".to_string(), "Alice".to_string()),
            ("age".to_string(), "30".to_string()),
        ]);

        let result = engine.compare_records("rec-001", &new_fields, &legacy_fields);
        assert!(!result.is_match);
        let age_diff = result
            .field_diffs
            .iter()
            .find(|d| d.field_name == "age")
            .unwrap();
        assert!(!age_diff.is_match);
        assert_eq!(age_diff.new_value, "31");
        assert_eq!(age_diff.legacy_value, "30");
    }

    #[test]
    fn test_compare_records_missing_field() {
        let engine = ReconciliationEngine::default();
        let new_fields = HashMap::from([("name".to_string(), "Alice".to_string())]);
        let legacy_fields = HashMap::from([
            ("name".to_string(), "Alice".to_string()),
            ("extra".to_string(), "value".to_string()),
        ]);

        let result = engine.compare_records("rec-001", &new_fields, &legacy_fields);
        assert!(!result.is_match);
        let extra_diff = result
            .field_diffs
            .iter()
            .find(|d| d.field_name == "extra")
            .unwrap();
        assert!(!extra_diff.is_match);
        assert_eq!(extra_diff.new_value, "");
        assert_eq!(extra_diff.legacy_value, "value");
    }

    #[test]
    fn test_meets_cutover() {
        let engine = ReconciliationEngine::default();
        let result = crate::coordination::ReconciliationResult {
            batch_id: "test".to_string(),
            total_records: 100,
            matched_records: 100,
            diff_records: 0,
            match_rate: 1.0,
            diffs: vec![],
            audit_hash: compute_hash("test"),
        };
        assert!(engine.meets_cutover(&result));

        let result_with_diffs = crate::coordination::ReconciliationResult {
            batch_id: "test".to_string(),
            total_records: 100,
            matched_records: 95,
            diff_records: 5,
            match_rate: 0.95,
            diffs: vec![],
            audit_hash: compute_hash("test"),
        };
        assert!(!engine.meets_cutover(&result_with_diffs));
    }

    #[test]
    fn test_reconciliation_config_default() {
        let config = ReconciliationConfig::default();
        assert!((config.cutover_threshold - 0.995).abs() < 0.001);
        assert_eq!(config.max_diff_records, 5);
        assert!(config.fail_on_missing);
    }
}
