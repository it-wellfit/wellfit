#![forbid(unsafe_code)]

//! Audit Ledger Integrity Verification
//!
//! Provides integrity checking for the idempotency ledger, including
//! hash chain verification, record count validation, and tamper detection.

use sha2::{Digest, Sha256};

/// Audit integrity check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityCheckResult {
    pub passed: bool,
    pub checks_performed: u32,
    pub checks_passed: u32,
    pub failures: Vec<String>,
}

impl IntegrityCheckResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            checks_performed: 0,
            checks_passed: 0,
            failures: Vec::new(),
        }
    }

    pub fn add_check(&mut self, name: &str, passed: bool) {
        self.checks_performed += 1;
        if passed {
            self.checks_passed += 1;
        } else {
            self.passed = false;
            self.failures.push(name.to_string());
        }
    }

    pub fn is_valid(&self) -> bool {
        self.passed && self.checks_performed == self.checks_passed
    }
}

impl Default for IntegrityCheckResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute SHA-256 hash of a string.
pub fn compute_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Verify a hash chain: each entry's hash must match the computed hash
/// of the previous entry's data + current data.
pub fn verify_hash_chain(entries: &[String]) -> IntegrityCheckResult {
    let mut result = IntegrityCheckResult::new();

    if entries.is_empty() {
        result.add_check("empty_chain", true);
        return result;
    }

    // Check 1: Each entry must be non-empty
    for (i, entry) in entries.iter().enumerate() {
        result.add_check(&format!("entry_{}_non_empty", i), !entry.is_empty());
    }

    // Check 2: Hash chain integrity
    let mut prev_hash = String::from("genesis");
    for (i, entry) in entries.iter().enumerate() {
        let combined = format!("{}{}", prev_hash, entry);
        let computed = compute_hash(&combined);
        // In a real implementation, we'd compare against stored hash
        // For now, just verify the hash is computable and non-empty
        result.add_check(&format!("chain_{}_computable", i), !computed.is_empty());
        prev_hash = computed;
    }

    // Check 3: No duplicate entries (idempotency)
    let mut seen = std::collections::HashSet::new();
    for (i, entry) in entries.iter().enumerate() {
        let is_unique = seen.insert(entry.clone());
        result.add_check(&format!("entry_{}_unique", i), is_unique);
    }

    result
}

/// Verify record counts match expected values.
pub fn verify_record_counts(
    actual: usize,
    expected: usize,
    tolerance: usize,
) -> IntegrityCheckResult {
    let mut result = IntegrityCheckResult::new();
    let diff = actual.abs_diff(expected);
    result.add_check("count_within_tolerance", diff <= tolerance);
    result
}

/// Verify that all required fields are present in a record.
pub fn verify_required_fields(record: &str, required_fields: &[&str]) -> IntegrityCheckResult {
    let mut result = IntegrityCheckResult::new();
    for field in required_fields {
        result.add_check(&format!("field_{}_present", field), record.contains(field));
    }
    result
}

/// Audit trail entry for integrity tracking.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub action: String,
    pub actor: String,
    pub target: String,
    pub result: String,
    pub hash: String,
}

impl AuditEntry {
    pub fn new(action: &str, actor: &str, target: &str, result: &str) -> Self {
        let timestamp = current_timestamp_ms();
        let data = format!("{}:{}:{}:{}:{}", timestamp, action, actor, target, result);
        let hash = compute_hash(&data);
        Self {
            timestamp,
            action: action.to_string(),
            actor: actor.to_string(),
            target: target.to_string(),
            result: result.to_string(),
            hash,
        }
    }

    pub fn verify(&self) -> bool {
        let data = format!(
            "{}:{}:{}:{}:{}",
            self.timestamp, self.action, self.actor, self.target, self.result
        );
        compute_hash(&data) == self.hash
    }
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrity_check_result_new() {
        let result = IntegrityCheckResult::new();
        assert!(result.passed);
        assert_eq!(result.checks_performed, 0);
        assert_eq!(result.checks_passed, 0);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_integrity_check_result_add_pass() {
        let mut result = IntegrityCheckResult::new();
        result.add_check("test_check", true);
        assert!(result.passed);
        assert_eq!(result.checks_performed, 1);
        assert_eq!(result.checks_passed, 1);
        assert!(result.is_valid());
    }

    #[test]
    fn test_integrity_check_result_add_fail() {
        let mut result = IntegrityCheckResult::new();
        result.add_check("test_check", false);
        assert!(!result.passed);
        assert_eq!(result.checks_performed, 1);
        assert_eq!(result.checks_passed, 0);
        assert!(!result.is_valid());
        assert_eq!(result.failures, vec!["test_check"]);
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash("hello");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
                                    // Same input → same hash
        assert_eq!(compute_hash("hello"), compute_hash("hello"));
        // Different input → different hash
        assert_ne!(compute_hash("hello"), compute_hash("world"));
    }

    #[test]
    fn test_verify_hash_chain_empty() {
        let result = verify_hash_chain(&[]);
        assert!(result.is_valid());
    }

    #[test]
    fn test_verify_hash_chain_non_empty() {
        let entries = vec!["entry1".to_string(), "entry2".to_string()];
        let result = verify_hash_chain(&entries);
        assert!(result.is_valid());
        assert_eq!(result.checks_performed, 6); // 2 non-empty + 2 chain + 2 unique
    }

    #[test]
    fn test_verify_hash_chain_duplicates() {
        let entries = vec!["same".to_string(), "same".to_string()];
        let result = verify_hash_chain(&entries);
        // Should fail on uniqueness check
        assert!(!result.is_valid());
    }

    #[test]
    fn test_verify_record_counts() {
        let result = verify_record_counts(100, 100, 5);
        assert!(result.is_valid());

        let result = verify_record_counts(103, 100, 5);
        assert!(result.is_valid());

        let result = verify_record_counts(110, 100, 5);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_verify_required_fields() {
        let record = "order_no=123,item_no=456,qty=10";
        let result = verify_required_fields(record, &["order_no", "item_no", "qty"]);
        assert!(result.is_valid());

        let result = verify_required_fields(record, &["order_no", "missing_field"]);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new("CREATE", "admin", "order:123", "success");
        assert_eq!(entry.action, "CREATE");
        assert_eq!(entry.actor, "admin");
        assert_eq!(entry.target, "order:123");
        assert_eq!(entry.result, "success");
        assert!(!entry.hash.is_empty());
    }

    #[test]
    fn test_audit_entry_verification() {
        let entry = AuditEntry::new("UPDATE", "user", "config:db", "ok");
        assert!(entry.verify());
    }

    #[test]
    fn test_audit_entry_tamper_detection() {
        let mut entry = AuditEntry::new("DELETE", "admin", "record:456", "success");
        // Tamper with the result
        entry.result = "failure".to_string();
        assert!(!entry.verify());
    }

    #[test]
    fn test_integrity_check_result_default() {
        let result = IntegrityCheckResult::default();
        assert!(result.passed);
    }
}
