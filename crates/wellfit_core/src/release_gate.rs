#![forbid(unsafe_code)]

//! Wellfit Core - Layer 5 Release Gate & Quarantine State Machine
//!
//! Enforces Two-Way Door vs One-Way Door boundaries, recipient whitelist, and status check before retry.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseState {
    DraftGenerated,
    VerifiedPass,
    Quarantined,
    AuthorizedDispatch,
    DispatchUncertain,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    pub state: ReleaseState,
    pub detail: String,
}

pub struct ReleaseGate {
    pub state: ReleaseState,
    pub allowed_domains: Vec<String>,
    pub history: Vec<TransitionRecord>,
}

impl Default for ReleaseGate {
    fn default() -> Self {
        Self::new(vec![
            "dachen.com.tw".to_string(),
            "wellfit.tw".to_string(),
            "alpargatas.com".to_string(),
        ])
    }
}

impl ReleaseGate {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self {
            state: ReleaseState::DraftGenerated,
            allowed_domains,
            history: vec![TransitionRecord {
                state: ReleaseState::DraftGenerated,
                detail: "Initialized in draft state".to_string(),
            }],
        }
    }

    pub fn from_domains(domains: &[&str]) -> Self {
        Self::new(domains.iter().map(|d| d.to_string()).collect())
    }

    pub fn add_allowed_domain(&mut self, domain: &str) {
        let clean = domain.trim().to_lowercase();
        if !self
            .allowed_domains
            .iter()
            .any(|d| d.eq_ignore_ascii_case(&clean))
        {
            self.allowed_domains.push(clean);
        }
    }

    /// Two-Way Door transition: Evaluate verification results.
    pub fn verify_payload(&mut self, is_valid: bool, reason: &str) -> ReleaseState {
        if is_valid {
            self.state = ReleaseState::VerifiedPass;
            self.log_transition(
                self.state,
                "Payload passed all contract checks.".to_string(),
            );
        } else {
            self.state = ReleaseState::Quarantined;
            self.log_transition(self.state, format!("Payload quarantined: {reason}"));
        }
        self.state
    }

    /// Validate recipient email addresses against corporate whitelist.
    pub fn validate_recipients(&self, recipients: &[String]) -> Result<(), String> {
        if recipients.is_empty() {
            return Err("Recipient list cannot be empty".to_string());
        }
        for r in recipients {
            let clean = r.trim().to_lowercase();
            // The address form is validated before the domain is taken: an
            // address with more than one '@' must not pass the allow list
            // because its last segment happens to be an allowed domain.
            let mut parts = clean.split('@');
            let local = parts.next().unwrap_or("");
            let domain = parts.next().unwrap_or("");
            if local.is_empty() || parts.next().is_some() || domain.is_empty() {
                return Err(format!("Recipient '{r}' is not a valid address"));
            }
            if !self
                .allowed_domains
                .iter()
                .any(|d| d.eq_ignore_ascii_case(domain))
            {
                return Err(format!(
                    "Recipient domain '{domain}' not in authorized whitelist: {:?}",
                    self.allowed_domains
                ));
            }
        }
        Ok(())
    }

    /// One-Way Door transition: Execute dispatch with uncertainty protection.
    /// Requires state to be VerifiedPass or AuthorizedDispatch.
    pub fn dispatch<F>(
        &mut self,
        transport_fn: F,
        recipients: Option<&[String]>,
    ) -> Result<bool, String>
    where
        F: FnOnce() -> Result<bool, String>,
    {
        if self.state != ReleaseState::VerifiedPass
            && self.state != ReleaseState::AuthorizedDispatch
        {
            return Err(format!(
                "Cannot dispatch from unverified state: {:?}",
                self.state
            ));
        }

        if let Some(recs) = recipients {
            self.validate_recipients(recs)?;
        }

        self.state = ReleaseState::AuthorizedDispatch;
        self.log_transition(self.state, "Dispatch initiated.".to_string());

        match transport_fn() {
            Ok(true) => {
                self.state = ReleaseState::Completed;
                self.log_transition(self.state, "Dispatch successfully completed.".to_string());
                Ok(true)
            }
            Ok(false) => {
                self.state = ReleaseState::Failed;
                self.log_transition(self.state, "Transport reported failure.".to_string());
                Ok(false)
            }
            Err(e) if e.contains("timeout") || e.contains("Timeout") => {
                self.state = ReleaseState::DispatchUncertain;
                self.log_transition(
                    self.state,
                    format!("Timeout encountered: {e}. Entering DISPATCH_UNCERTAIN."),
                );
                Err(e)
            }
            Err(e) => {
                self.state = ReleaseState::Failed;
                self.log_transition(self.state, format!("Dispatch failed with error: {e}"));
                Err(e)
            }
        }
    }

    /// Perform status check before retry to prevent duplicate dispatch/orders.
    pub fn reconcile_uncertain_dispatch<F>(&mut self, status_check_fn: F) -> ReleaseState
    where
        F: FnOnce() -> bool,
    {
        if self.state != ReleaseState::DispatchUncertain {
            return self.state;
        }

        if status_check_fn() {
            self.state = ReleaseState::Completed;
            self.log_transition(
                self.state,
                "Status check confirmed remote delivery; resolved to COMPLETED.".to_string(),
            );
        } else {
            self.state = ReleaseState::VerifiedPass;
            self.log_transition(
                self.state,
                "Status check confirmed remote never received; ready for safe retry.".to_string(),
            );
        }
        self.state
    }

    fn log_transition(&mut self, state: ReleaseState, detail: String) {
        self.history.push(TransitionRecord { state, detail });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist_validation() {
        let gate = ReleaseGate::default();
        assert!(gate
            .validate_recipients(&["finance@dachen.com.tw".to_string()])
            .is_ok());
        assert!(gate
            .validate_recipients(&["attacker@evil.com".to_string()])
            .is_err());
    }

    #[test]
    fn test_verify_payload_transitions() {
        let mut gate = ReleaseGate::default();
        assert_eq!(gate.verify_payload(true, ""), ReleaseState::VerifiedPass);
        assert_eq!(
            gate.verify_payload(false, "bad arithmetic"),
            ReleaseState::Quarantined
        );
    }

    #[test]
    fn test_uncertain_dispatch_and_reconciliation() {
        let mut gate = ReleaseGate::default();
        gate.verify_payload(true, "");
        let timeout_fn =
            || -> Result<bool, String> { Err("Connection timeout to SMTP".to_string()) };
        let _ = gate.dispatch(timeout_fn, None);
        assert_eq!(gate.state, ReleaseState::DispatchUncertain);

        gate.reconcile_uncertain_dispatch(|| true);
        assert_eq!(gate.state, ReleaseState::Completed);
    }
}
