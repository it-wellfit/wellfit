#![forbid(unsafe_code)]

//! Three-way reconciliation: EKPI decision ↔ event backbone ↔ Wellfit execution.
//!
//! Differences are quarantined into an isolation list — they are never
//! silently corrected. Cross-repo interchange is via JSON contracts only
//! (no path dependency on ekpi-platform), per the unified integration
//! plan (Phase C8).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wellfit_core::audit_integrity::compute_hash;

/// Side A: EKPI governance decision (from approval API or audit event).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EkpiDecision {
    /// Approval task id in ekpi-platform.
    pub task_id: String,
    /// Business reference the decision applies to (e.g., order id, batch id).
    pub business_ref: String,
    /// approved / rejected / escalated.
    pub decision: String,
    /// Actor that made the decision.
    pub actor: String,
    /// Unix seconds.
    pub decided_at_unix: i64,
    /// SHA-256 of the audit chain entry (hex).
    pub audit_chain_sha256: String,
}

/// Side B: Event backbone record (outbox event delivered to Wellfit inbox).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventBackboneRecord {
    /// event_id from event_outbox/event_inbox.
    pub event_id: String,
    /// Business reference carried in the event payload.
    pub business_ref: String,
    /// event_type, e.g. `approval.decided`.
    pub event_type: String,
    /// Decision carried in payload: approved / rejected / escalated.
    pub decision: String,
    /// Unix seconds.
    pub occurred_at_unix: i64,
}

/// Side C: Wellfit execution result (idempotency ledger entry).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WellfitExecution {
    /// Business reference the execution applied to.
    pub business_ref: String,
    /// executed / rejected / isolated.
    pub outcome: String,
    /// Idempotency ledger batch id.
    pub batch_id: String,
    /// Unix seconds.
    pub executed_at_unix: i64,
}

/// Outcome of three-way comparison for one business reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreeWayStatus {
    /// All three sides present and semantically consistent.
    Consistent,
    /// Sides disagree — quarantined.
    Mismatch,
    /// One or more sides are missing the record — quarantined.
    Missing,
}

/// Isolation list entry: a quarantined difference awaiting manual disposition.
/// Quarantine is append-only; entries are never auto-resolved.
#[derive(Debug, Clone, Serialize)]
pub struct IsolationEntry {
    pub business_ref: String,
    pub reason: String,
    pub ekpi: Option<EkpiDecision>,
    pub event: Option<EventBackboneRecord>,
    pub wellfit: Option<WellfitExecution>,
    pub quarantined_at_unix: i64,
}

/// Report of a full three-way reconciliation run.
#[derive(Debug, Clone, Serialize)]
pub struct ThreeWayReport {
    pub total_references: usize,
    pub consistent: usize,
    pub mismatched: usize,
    pub missing: usize,
    pub isolation_list: Vec<IsolationEntry>,
    /// SHA-256 over the sorted isolation reasons + reference set, so two
    /// runs over the same inputs produce the same audit hash.
    pub audit_hash: String,
}

impl ThreeWayReport {
    /// A run is clean when nothing was quarantined.
    pub fn is_clean(&self) -> bool {
        self.isolation_list.is_empty()
    }
}

/// Deterministic three-way reconciler.
pub struct ThreeWayReconciler;

impl ThreeWayReconciler {
    /// Compare the three sides keyed by `business_ref`.
    ///
    /// Consistency rules:
    /// 1. EKPI decision must equal the event backbone decision.
    /// 2. `approved` on EKPI ⇒ Wellfit outcome must be `executed`.
    /// 3. `rejected` on EKPI ⇒ Wellfit outcome must be `rejected` or
    ///    `isolated`; `executed` is a serious mismatch (unauthorized
    ///    execution of a rejected decision).
    ///
    /// Any rule violation or missing side is quarantined — never corrected.
    pub fn reconcile(
        ekpi: &[EkpiDecision],
        events: &[EventBackboneRecord],
        executions: &[WellfitExecution],
        now_unix: i64,
    ) -> ThreeWayReport {
        let mut ekpi_by_ref: BTreeMap<&str, &EkpiDecision> = BTreeMap::new();
        for d in ekpi {
            ekpi_by_ref.insert(d.business_ref.as_str(), d);
        }
        let mut event_by_ref: BTreeMap<&str, &EventBackboneRecord> = BTreeMap::new();
        for e in events {
            event_by_ref.insert(e.business_ref.as_str(), e);
        }
        let mut exec_by_ref: BTreeMap<&str, &WellfitExecution> = BTreeMap::new();
        for x in executions {
            exec_by_ref.insert(x.business_ref.as_str(), x);
        }

        let mut all_refs: BTreeMap<&str, ()> = BTreeMap::new();
        for k in ekpi_by_ref.keys() {
            all_refs.insert(k, ());
        }
        for k in event_by_ref.keys() {
            all_refs.insert(k, ());
        }
        for k in exec_by_ref.keys() {
            all_refs.insert(k, ());
        }

        let mut consistent = 0usize;
        let mut mismatched = 0usize;
        let mut missing = 0usize;
        let mut isolation_list: Vec<IsolationEntry> = Vec::new();

        for r in all_refs.keys() {
            let ekpi_d = ekpi_by_ref.get(r).copied();
            let event_d = event_by_ref.get(r).copied();
            let exec_d = exec_by_ref.get(r).copied();

            match (ekpi_d, event_d, exec_d) {
                (Some(e), Some(v), Some(x)) => match Self::check_consistent(e, v, x) {
                    Ok(()) => consistent += 1,
                    Err(reason) => {
                        mismatched += 1;
                        isolation_list.push(IsolationEntry {
                            business_ref: (*r).to_string(),
                            reason,
                            ekpi: Some(e.clone()),
                            event: Some(v.clone()),
                            wellfit: Some(x.clone()),
                            quarantined_at_unix: now_unix,
                        });
                    }
                },
                (e, v, x) => {
                    missing += 1;
                    let mut sides = Vec::new();
                    if e.is_none() {
                        sides.push("ekpi");
                    }
                    if v.is_none() {
                        sides.push("event");
                    }
                    if x.is_none() {
                        sides.push("wellfit");
                    }
                    isolation_list.push(IsolationEntry {
                        business_ref: (*r).to_string(),
                        reason: format!("missing side(s): {}", sides.join(", ")),
                        ekpi: e.cloned(),
                        event: v.cloned(),
                        wellfit: x.cloned(),
                        quarantined_at_unix: now_unix,
                    });
                }
            }
        }

        // Deterministic audit hash over the quarantined content.
        let mut material = String::new();
        for entry in &isolation_list {
            material.push_str(&entry.business_ref);
            material.push('|');
            material.push_str(&entry.reason);
            material.push('\n');
        }
        let audit_hash = compute_hash(&material);

        ThreeWayReport {
            total_references: all_refs.len(),
            consistent,
            mismatched,
            missing,
            isolation_list,
            audit_hash,
        }
    }

    fn check_consistent(
        ekpi: &EkpiDecision,
        event: &EventBackboneRecord,
        exec: &WellfitExecution,
    ) -> Result<(), String> {
        if ekpi.decision != event.decision {
            return Err(format!(
                "decision conflict: ekpi={} event={}",
                ekpi.decision, event.decision
            ));
        }
        match ekpi.decision.as_str() {
            "approved" => {
                if exec.outcome != "executed" {
                    return Err(format!(
                        "approved decision but wellfit outcome={}",
                        exec.outcome
                    ));
                }
            }
            "rejected" => {
                if exec.outcome == "executed" {
                    return Err("rejected decision was executed by wellfit".to_string());
                }
            }
            "escalated" => {
                // Escalated tasks must not have been executed yet.
                if exec.outcome == "executed" {
                    return Err("escalated decision was executed before human ruling".to_string());
                }
            }
            other => return Err(format!("unknown ekpi decision `{other}`")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ekpi(r: &str, decision: &str) -> EkpiDecision {
        EkpiDecision {
            task_id: format!("task-{r}"),
            business_ref: r.to_string(),
            decision: decision.to_string(),
            actor: "approver-1".to_string(),
            decided_at_unix: 1_000,
            audit_chain_sha256: "abc123".to_string(),
        }
    }

    fn event(r: &str, decision: &str) -> EventBackboneRecord {
        EventBackboneRecord {
            event_id: format!("evt-{r}"),
            business_ref: r.to_string(),
            event_type: "approval.decided".to_string(),
            decision: decision.to_string(),
            occurred_at_unix: 1_001,
        }
    }

    fn exec(r: &str, outcome: &str) -> WellfitExecution {
        WellfitExecution {
            business_ref: r.to_string(),
            outcome: outcome.to_string(),
            batch_id: format!("batch-{r}"),
            executed_at_unix: 1_002,
        }
    }

    #[test]
    fn all_consistent() {
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o1", "approved")],
            &[event("o1", "approved")],
            &[exec("o1", "executed")],
            2_000,
        );
        assert_eq!(report.total_references, 1);
        assert_eq!(report.consistent, 1);
        assert!(report.is_clean());
    }

    #[test]
    fn decision_conflict_is_quarantined() {
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o1", "approved")],
            &[event("o1", "rejected")],
            &[exec("o1", "executed")],
            2_000,
        );
        assert_eq!(report.mismatched, 1);
        assert_eq!(report.isolation_list.len(), 1);
        assert!(report.isolation_list[0]
            .reason
            .contains("decision conflict"));
    }

    #[test]
    fn rejected_but_executed_is_quarantined() {
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o2", "rejected")],
            &[event("o2", "rejected")],
            &[exec("o2", "executed")],
            2_000,
        );
        assert_eq!(report.mismatched, 1);
        assert!(report.isolation_list[0]
            .reason
            .contains("rejected decision was executed"));
    }

    #[test]
    fn missing_side_is_quarantined() {
        // Event missing.
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o3", "approved")],
            &[],
            &[exec("o3", "executed")],
            2_000,
        );
        assert_eq!(report.missing, 1);
        assert!(report.isolation_list[0]
            .reason
            .contains("missing side(s): event"));

        // Wellfit missing.
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o4", "approved")],
            &[event("o4", "approved")],
            &[],
            2_000,
        );
        assert_eq!(report.missing, 1);
        assert!(report.isolation_list[0]
            .reason
            .contains("missing side(s): wellfit"));
    }

    #[test]
    fn escalated_must_not_be_executed() {
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o5", "escalated")],
            &[event("o5", "escalated")],
            &[exec("o5", "executed")],
            2_000,
        );
        assert_eq!(report.mismatched, 1);
        assert!(report.isolation_list[0].reason.contains("escalated"));
    }

    #[test]
    fn rejected_with_isolated_wellfit_is_consistent() {
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o6", "rejected")],
            &[event("o6", "rejected")],
            &[exec("o6", "isolated")],
            2_000,
        );
        assert_eq!(report.consistent, 1);
        assert!(report.is_clean());
    }

    #[test]
    fn audit_hash_deterministic() {
        let a = ThreeWayReconciler::reconcile(
            &[ekpi("o1", "approved")],
            &[],
            &[exec("o1", "executed")],
            2_000,
        );
        let b = ThreeWayReconciler::reconcile(
            &[ekpi("o1", "approved")],
            &[],
            &[exec("o1", "executed")],
            9_999,
        );
        assert_eq!(a.audit_hash, b.audit_hash);
    }

    #[test]
    fn json_roundtrip_contracts() {
        let d = ekpi("o7", "approved");
        let json = serde_json::to_string(&d).unwrap();
        let back: EkpiDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);

        let e = event("o7", "approved");
        let json = serde_json::to_string(&e).unwrap();
        let back: EventBackboneRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e);

        let x = exec("o7", "executed");
        let json = serde_json::to_string(&x).unwrap();
        let back: WellfitExecution = serde_json::from_str(&json).unwrap();
        assert_eq!(back, x);
    }

    #[test]
    fn duplicate_refs_last_wins_deterministically() {
        let report = ThreeWayReconciler::reconcile(
            &[ekpi("o1", "approved"), ekpi("o1", "approved")],
            &[event("o1", "approved")],
            &[exec("o1", "executed")],
            2_000,
        );
        assert_eq!(report.total_references, 1);
        assert_eq!(report.consistent, 1);
    }
}
