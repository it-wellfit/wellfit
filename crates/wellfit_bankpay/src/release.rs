#![forbid(unsafe_code)]

//! Release-gate integration for BankPay.
//!
//! Wraps [`wellfit_core::release_gate::ReleaseGate`] with BankPay-specific
//! transitions and the uncertain-dispatch reconciliation flow.

use wellfit_core::release_gate::{ReleaseGate, ReleaseState};

/// BankPay release state alias.
pub type BankPayReleaseState = ReleaseState;

/// Evaluate a bank payable payload before dispatch (two-way door).
///
/// Returns the resulting release state.
pub fn verify_payload(gate: &mut ReleaseGate, is_valid: bool, reason: &str) -> ReleaseState {
    gate.verify_payload(is_valid, reason)
}

/// Dispatch a verified payload (one-way door). `transport` performs the actual
/// bank delivery. Returns whether dispatch completed.
pub fn dispatch_payload(
    gate: &mut ReleaseGate,
    transport: impl FnOnce() -> Result<bool, String>,
    recipients: Option<&[String]>,
) -> Result<bool, String> {
    gate.dispatch(transport, recipients)
}

/// Reconcile a `DispatchUncertain` state by checking the remote status.
///
/// `status_check` returns `true` when the bank confirmed delivery.
pub fn reconcile_uncertain(
    gate: &mut ReleaseGate,
    status_check: impl FnOnce() -> bool,
) -> ReleaseState {
    gate.reconcile_uncertain_dispatch(status_check)
}

/// Whether the gate is in a terminal state (Completed / Failed).
pub fn is_terminal(state: ReleaseState) -> bool {
    matches!(state, ReleaseState::Completed | ReleaseState::Failed)
}

/// Whether the gate awaits reconciliation (DispatchUncertain).
pub fn is_uncertain(state: ReleaseState) -> bool {
    state == ReleaseState::DispatchUncertain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_and_dispatch_flow() {
        let mut gate = ReleaseGate::default();
        assert_eq!(
            verify_payload(&mut gate, true, ""),
            ReleaseState::VerifiedPass
        );
        let result = dispatch_payload(&mut gate, || Ok(true), None);
        assert_eq!(result, Ok(true));
        assert!(is_terminal(gate.state));
        assert_eq!(gate.state, ReleaseState::Completed);
    }

    #[test]
    fn test_uncertain_dispatch_reconciliation() {
        let mut gate = ReleaseGate::default();
        verify_payload(&mut gate, true, "");
        let _ = dispatch_payload(&mut gate, || Err("timeout".to_string()), None);
        assert!(is_uncertain(gate.state));
        let state = reconcile_uncertain(&mut gate, || true);
        assert_eq!(state, ReleaseState::Completed);
        assert!(is_terminal(state));
    }

    #[test]
    fn test_reject_quarantines() {
        let mut gate = ReleaseGate::default();
        assert_eq!(
            verify_payload(&mut gate, false, "isolated row"),
            ReleaseState::Quarantined
        );
        let result = dispatch_payload(&mut gate, || Ok(true), None);
        assert!(result.is_err());
    }
}
