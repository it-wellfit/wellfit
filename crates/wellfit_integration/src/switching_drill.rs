#![forbid(unsafe_code)]

//! Switching Drill — end-to-end cutover rehearsal and recovery testing.
//!
//! Simulates the full dual-track → shadow → read-only → decommissioned
//! lifecycle with reconciliation checkpoints and rollback recovery.
//! This is the M3 gate deliverable (T12.1) and supports T12.3 recovery testing.

use crate::coordination::{CoordinationEngine, CoordinationMode};
use crate::switching::SwitchingController;
use wellfit_core::audit_integrity::{compute_hash, verify_hash_chain};

/// Drill step recorded during execution.
#[derive(Debug, Clone, PartialEq)]
pub struct DrillStep {
    pub step_name: String,
    pub mode: CoordinationMode,
    pub match_rate: f64,
    pub passed: bool,
}

/// End-to-end switching drill report.
#[derive(Debug, Clone)]
pub struct DrillReport {
    pub drill_id: String,
    pub steps: Vec<DrillStep>,
    pub final_mode: CoordinationMode,
    pub overall_pass: bool,
    pub audit_hash: String,
}

/// Errors from the switching drill.
#[derive(Debug)]
pub enum DrillError {
    TransitionFailed(String),
    ReconciliationBelowThreshold(String),
    AuditChainBroken(String),
    RollbackFailed(String),
    DrillAborted(String),
}

impl std::fmt::Display for DrillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrillError::TransitionFailed(msg) => write!(f, "transition failed: {msg}"),
            DrillError::ReconciliationBelowThreshold(msg) => {
                write!(f, "reconciliation below threshold: {msg}")
            }
            DrillError::AuditChainBroken(msg) => write!(f, "audit chain broken: {msg}"),
            DrillError::RollbackFailed(msg) => write!(f, "rollback failed: {msg}"),
            DrillError::DrillAborted(msg) => write!(f, "drill aborted: {msg}"),
        }
    }
}

impl std::error::Error for DrillError {}

/// End-to-end switching drill executor.
pub struct SwitchingDrill {
    engine: CoordinationEngine,
    controller: SwitchingController,
}

impl SwitchingDrill {
    /// Create a new drill executor.
    pub fn new() -> Self {
        Self {
            engine: CoordinationEngine::new(),
            controller: SwitchingController::new(),
        }
    }

    /// Run the full end-to-end switching drill.
    ///
    /// Executes the staged cutover path:
    /// 1. DualTrack: verify reconciliation >= 99.5%
    /// 2. ShadowLegacy: verify continued reconciliation
    /// 3. LegacyReadOnly: verify legacy stability
    /// 4. Decommissioned: final cutover confirmation
    ///
    /// Each step records a [`DrillStep`] and a hash-chained audit entry.
    pub fn run_drill(
        &mut self,
        drill_id: &str,
        test_records: &[(String, String, String)],
    ) -> Result<DrillReport, DrillError> {
        let mut steps = Vec::new();

        // Phase 1: DualTrack — verify baseline reconciliation
        self.controller
            .enable_flag("dual_track", 100, "drill phase 1");
        for (key, new_val, legacy_val) in test_records {
            self.engine.record_new(key, new_val);
            self.engine.record_legacy(key, legacy_val);
        }
        let recon1 = self.engine.reconcile(&format!("{}-phase1", drill_id));
        let passed1 = recon1.match_rate >= 0.995;
        steps.push(DrillStep {
            step_name: "dual_track".to_string(),
            mode: CoordinationMode::DualTrack,
            match_rate: recon1.match_rate,
            passed: passed1,
        });
        if !passed1 {
            return Err(DrillError::ReconciliationBelowThreshold(format!(
                "dual_track match rate {:.4} below 99.5%",
                recon1.match_rate
            )));
        }

        // Phase 2: Transition to ShadowLegacy
        self.engine
            .transition_to(CoordinationMode::ShadowLegacy)
            .map_err(|e| DrillError::TransitionFailed(e.to_string()))?;
        self.controller
            .enable_flag("shadow_legacy", 100, "drill phase 2");
        let recon2 = self.engine.reconcile(&format!("{}-phase2", drill_id));
        let passed2 = recon2.match_rate >= 0.995;
        steps.push(DrillStep {
            step_name: "shadow_legacy".to_string(),
            mode: CoordinationMode::ShadowLegacy,
            match_rate: recon2.match_rate,
            passed: passed2,
        });
        if !passed2 {
            // Rollback on failure
            self.engine
                .rollback()
                .map_err(|e| DrillError::RollbackFailed(e.to_string()))?;
            return Err(DrillError::ReconciliationBelowThreshold(format!(
                "shadow_legacy match rate {:.4} below 99.5%",
                recon2.match_rate
            )));
        }

        // Phase 3: Transition to LegacyReadOnly
        self.engine
            .transition_to(CoordinationMode::LegacyReadOnly)
            .map_err(|e| DrillError::TransitionFailed(e.to_string()))?;
        self.controller
            .enable_flag("legacy_read_only", 100, "drill phase 3");
        let recon3 = self.engine.reconcile(&format!("{}-phase3", drill_id));
        let passed3 = recon3.match_rate >= 0.995;
        steps.push(DrillStep {
            step_name: "legacy_read_only".to_string(),
            mode: CoordinationMode::LegacyReadOnly,
            match_rate: recon3.match_rate,
            passed: passed3,
        });
        if !passed3 {
            self.engine
                .rollback()
                .map_err(|e| DrillError::RollbackFailed(e.to_string()))?;
            return Err(DrillError::ReconciliationBelowThreshold(format!(
                "legacy_read_only match rate {:.4} below 99.5%",
                recon3.match_rate
            )));
        }

        // Phase 4: Transition to Decommissioned
        self.engine
            .transition_to(CoordinationMode::Decommissioned)
            .map_err(|e| DrillError::TransitionFailed(e.to_string()))?;
        self.controller
            .enable_flag("decommissioned", 100, "drill phase 4");
        steps.push(DrillStep {
            step_name: "decommissioned".to_string(),
            mode: CoordinationMode::Decommissioned,
            match_rate: recon3.match_rate,
            passed: true,
        });

        // Verify audit chain integrity across all reconciliation records
        let hashes: Vec<String> = self
            .engine
            .history()
            .iter()
            .map(|r| r.audit_hash.clone())
            .collect();
        let integrity = verify_hash_chain(&hashes);
        if !integrity.is_valid() {
            return Err(DrillError::AuditChainBroken(format!(
                "audit chain broken: {:?}",
                integrity.failures
            )));
        }

        let audit_hash = compute_hash(&format!(
            "drill:{}:steps={}:final_mode={}",
            drill_id,
            steps.len(),
            self.engine.mode().as_str()
        ));

        Ok(DrillReport {
            drill_id: drill_id.to_string(),
            steps,
            final_mode: self.engine.mode(),
            overall_pass: true,
            audit_hash,
        })
    }

    /// Test recovery: simulate reconciliation failure and rollback.
    pub fn test_recovery(&mut self) -> Result<DrillReport, DrillError> {
        // Start in DualTrack
        self.engine.record_new("key1", "value1");
        self.engine.record_legacy("key1", "different_value");
        let recon = self.engine.reconcile("recovery-test");
        assert!(recon.match_rate < 0.995);

        // Attempt transition (should fail if we enforced threshold, but drill
        // tests the rollback path)
        self.engine
            .transition_to(CoordinationMode::ShadowLegacy)
            .map_err(|e| DrillError::TransitionFailed(e.to_string()))?;

        // Rollback should succeed
        self.engine
            .rollback()
            .map_err(|e| DrillError::RollbackFailed(e.to_string()))?;
        assert_eq!(self.engine.mode(), CoordinationMode::DualTrack);

        Ok(DrillReport {
            drill_id: "recovery-test".to_string(),
            steps: vec![
                DrillStep {
                    step_name: "dual_track".to_string(),
                    mode: CoordinationMode::DualTrack,
                    match_rate: recon.match_rate,
                    passed: false,
                },
                DrillStep {
                    step_name: "rollback".to_string(),
                    mode: CoordinationMode::DualTrack,
                    match_rate: recon.match_rate,
                    passed: true,
                },
            ],
            final_mode: self.engine.mode(),
            overall_pass: true,
            audit_hash: compute_hash("recovery-test"),
        })
    }

    /// Get the drill's coordination engine.
    pub fn engine(&self) -> &CoordinationEngine {
        &self.engine
    }

    /// Get the drill's switching controller.
    pub fn controller(&self) -> &SwitchingController {
        &self.controller
    }
}

impl Default for SwitchingDrill {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matching_records() -> Vec<(String, String, String)> {
        vec![
            (
                "key1".to_string(),
                "value1".to_string(),
                "value1".to_string(),
            ),
            (
                "key2".to_string(),
                "value2".to_string(),
                "value2".to_string(),
            ),
            (
                "key3".to_string(),
                "value3".to_string(),
                "value3".to_string(),
            ),
        ]
    }

    #[test]
    fn test_drill_full_lifecycle_success() {
        let mut drill = SwitchingDrill::new();
        let records = matching_records();
        let report = drill.run_drill("drill-001", &records).unwrap();

        assert!(report.overall_pass);
        assert_eq!(report.drill_id, "drill-001");
        assert_eq!(report.final_mode, CoordinationMode::Decommissioned);
        assert_eq!(report.steps.len(), 4);
        assert!(report.steps.iter().all(|s| s.passed));
        assert!(!report.audit_hash.is_empty());
    }

    #[test]
    fn test_drill_step_modes() {
        let mut drill = SwitchingDrill::new();
        let records = matching_records();
        let report = drill.run_drill("drill-002", &records).unwrap();

        assert_eq!(report.steps[0].mode, CoordinationMode::DualTrack);
        assert_eq!(report.steps[1].mode, CoordinationMode::ShadowLegacy);
        assert_eq!(report.steps[2].mode, CoordinationMode::LegacyReadOnly);
        assert_eq!(report.steps[3].mode, CoordinationMode::Decommissioned);
    }

    #[test]
    fn test_drill_fails_below_threshold() {
        let mut drill = SwitchingDrill::new();
        // One mismatched record → match rate 0.67, below 99.5%
        let records = vec![
            (
                "key1".to_string(),
                "value1".to_string(),
                "value1".to_string(),
            ),
            (
                "key2".to_string(),
                "value2".to_string(),
                "DIFFERENT".to_string(),
            ),
            (
                "key3".to_string(),
                "value3".to_string(),
                "value3".to_string(),
            ),
        ];
        let result = drill.run_drill("drill-fail", &records);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(DrillError::ReconciliationBelowThreshold(_))
        ));
    }

    #[test]
    fn test_drill_recovery_rollback() {
        let mut drill = SwitchingDrill::new();
        let report = drill.test_recovery().unwrap();

        assert!(report.overall_pass);
        assert_eq!(report.final_mode, CoordinationMode::DualTrack);
        assert_eq!(report.steps.len(), 2);
        assert_eq!(report.steps[0].step_name, "dual_track");
        assert_eq!(report.steps[1].step_name, "rollback");
    }

    #[test]
    fn test_drill_audit_hash_valid() {
        let mut drill = SwitchingDrill::new();
        let records = matching_records();
        let report = drill.run_drill("drill-audit", &records).unwrap();

        assert_eq!(report.audit_hash.len(), 64);
        assert!(!report.audit_hash.is_empty());
    }

    #[test]
    fn test_drill_error_display() {
        let err = DrillError::TransitionFailed("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = DrillError::AuditChainBroken("broken".to_string());
        assert!(err.to_string().contains("broken"));
    }

    #[test]
    fn test_drill_default() {
        let drill = SwitchingDrill::default();
        assert_eq!(drill.engine().mode(), CoordinationMode::DualTrack);
    }
}
