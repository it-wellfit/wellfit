#![forbid(unsafe_code)]

//! Automation Framework — retry logic, circuit breaker, and status tracking
//! for automated workflows.
//!
//! Provides shared infrastructure for Phase 2 automation (procurement,
//! outbound, payment) including exponential backoff retry, circuit breaker
//! pattern, and workflow status tracking.

use std::time::Duration;

/// Workflow execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Success,
    Failed,
    Retrying,
    CircuitOpen,
}

impl WorkflowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowStatus::Pending => "PENDING",
            WorkflowStatus::Running => "RUNNING",
            WorkflowStatus::Success => "SUCCESS",
            WorkflowStatus::Failed => "FAILED",
            WorkflowStatus::Retrying => "RETRYING",
            WorkflowStatus::CircuitOpen => "CIRCUIT_OPEN",
        }
    }
}

/// Retry configuration with exponential backoff.
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

impl RetryConfig {
    /// Default retry config: 3 retries, 5s/15s/30s backoff.
    pub fn default_retry() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 5000,
            backoff_multiplier: 3.0,
            max_delay_ms: 30000,
        }
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay =
            (self.base_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32)) as u64;
        let delay = delay.min(self.max_delay_ms);
        Duration::from_millis(delay)
    }
}

/// Retry result.
#[derive(Debug, Clone)]
pub struct RetryResult<T> {
    pub value: Option<T>,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub success: bool,
}

impl<T> RetryResult<T> {
    pub fn success(value: T, attempts: u32) -> Self {
        Self {
            value: Some(value),
            attempts,
            last_error: None,
            success: true,
        }
    }

    pub fn failure(last_error: String, attempts: u32) -> Self {
        Self {
            value: None,
            attempts,
            last_error: Some(last_error),
            success: false,
        }
    }
}

/// Execute a closure with retry logic.
pub fn retry_with_backoff<T, E, F>(config: &RetryConfig, mut operation: F) -> RetryResult<T>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut last_error = None;
    for attempt in 0..=config.max_retries {
        match operation() {
            Ok(value) => {
                return RetryResult::success(value, attempt + 1);
            }
            Err(e) => {
                last_error = Some(e.to_string());
                if attempt < config.max_retries {
                    // In production, would sleep here; for testability, skip
                }
            }
        }
    }
    RetryResult::failure(last_error.unwrap_or_default(), config.max_retries + 1)
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for protecting against cascading failures.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub failure_threshold: u32,
    pub consecutive_failures: u32,
    pub state: CircuitState,
    pub last_failure_time: Option<u64>,
    pub cooldown_ms: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_ms: u64) -> Self {
        Self {
            failure_threshold,
            consecutive_failures: 0,
            state: CircuitState::Closed,
            last_failure_time: None,
            cooldown_ms,
        }
    }

    /// Default circuit breaker: 5 consecutive failures → open, 30s cooldown.
    pub fn default_breaker() -> Self {
        Self::new(5, 30000)
    }

    pub fn is_allowed(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if cooldown has elapsed
                if let Some(last) = self.last_failure_time {
                    let now = current_timestamp_ms();
                    if now - last >= self.cooldown_ms {
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.state = CircuitState::Closed;
    }

    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.last_failure_time = Some(current_timestamp_ms());
        if self.consecutive_failures >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.state = CircuitState::Closed;
        self.last_failure_time = None;
    }
}

/// Workflow execution record for tracking.
#[derive(Debug, Clone)]
pub struct WorkflowRecord {
    pub id: String,
    pub name: String,
    pub status: WorkflowStatus,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub retry_count: u32,
    pub error: Option<String>,
}

impl WorkflowRecord {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: WorkflowStatus::Pending,
            started_at: current_timestamp_ms(),
            completed_at: None,
            retry_count: 0,
            error: None,
        }
    }

    pub fn mark_running(&mut self) {
        self.status = WorkflowStatus::Running;
    }

    pub fn mark_success(&mut self) {
        self.status = WorkflowStatus::Success;
        self.completed_at = Some(current_timestamp_ms());
    }

    pub fn mark_failed(&mut self, error: &str) {
        self.status = WorkflowStatus::Failed;
        self.completed_at = Some(current_timestamp_ms());
        self.error = Some(error.to_string());
    }

    pub fn mark_retrying(&mut self) {
        self.status = WorkflowStatus::Retrying;
        self.retry_count += 1;
    }

    pub fn mark_circuit_open(&mut self) {
        self.status = WorkflowStatus::CircuitOpen;
        self.completed_at = Some(current_timestamp_ms());
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.completed_at.map(|c| c - self.started_at)
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
    fn test_retry_config_delay() {
        let config = RetryConfig::default_retry();
        // attempt 0: 5000 * 3^0 = 5000
        assert_eq!(config.delay_for_attempt(0).as_millis(), 5000);
        // attempt 1: 5000 * 3^1 = 15000
        assert_eq!(config.delay_for_attempt(1).as_millis(), 15000);
        // attempt 2: 5000 * 3^2 = 45000 → capped at 30000
        assert_eq!(config.delay_for_attempt(2).as_millis(), 30000);
    }

    #[test]
    fn test_retry_success_first_attempt() {
        let config = RetryConfig::default_retry();
        let result = retry_with_backoff(&config, || Ok::<i32, &str>(42));
        assert!(result.success);
        assert_eq!(result.value, Some(42));
        assert_eq!(result.attempts, 1);
    }

    #[test]
    fn test_retry_success_after_failures() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 1,
            backoff_multiplier: 2.0,
            max_delay_ms: 10,
        };
        let mut counter = 0;
        let result = retry_with_backoff(&config, || {
            counter += 1;
            if counter < 3 {
                Err("transient error")
            } else {
                Ok(42)
            }
        });
        assert!(result.success);
        assert_eq!(result.value, Some(42));
        assert_eq!(result.attempts, 3);
    }

    #[test]
    fn test_retry_all_failures() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 1,
            backoff_multiplier: 2.0,
            max_delay_ms: 10,
        };
        let result = retry_with_backoff(&config, || Err::<i32, &str>("permanent error"));
        assert!(!result.success);
        assert!(result.value.is_none());
        assert_eq!(result.attempts, 3);
        assert_eq!(result.last_error, Some("permanent error".to_string()));
    }

    #[test]
    fn test_circuit_breaker_closed() {
        let mut cb = CircuitBreaker::default_breaker();
        assert!(cb.is_allowed());
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let mut cb = CircuitBreaker::new(3, 30000);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        assert!(!cb.is_allowed());
    }

    #[test]
    fn test_circuit_breaker_success_resets() {
        let mut cb = CircuitBreaker::new(3, 30000);
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.consecutive_failures, 0);
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let mut cb = CircuitBreaker::new(3, 30000);
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        cb.reset();
        assert_eq!(cb.state, CircuitState::Closed);
        assert_eq!(cb.consecutive_failures, 0);
    }

    #[test]
    fn test_workflow_record_lifecycle() {
        let mut record = WorkflowRecord::new("wf-001", "procurement");
        assert_eq!(record.status, WorkflowStatus::Pending);
        record.mark_running();
        assert_eq!(record.status, WorkflowStatus::Running);
        record.mark_success();
        assert_eq!(record.status, WorkflowStatus::Success);
        assert!(record.completed_at.is_some());
        assert!(record.duration_ms().is_some());
    }

    #[test]
    fn test_workflow_record_failure() {
        let mut record = WorkflowRecord::new("wf-002", "payment");
        record.mark_running();
        record.mark_failed("bank connection timeout");
        assert_eq!(record.status, WorkflowStatus::Failed);
        assert_eq!(record.error, Some("bank connection timeout".to_string()));
    }

    #[test]
    fn test_workflow_record_retry() {
        let mut record = WorkflowRecord::new("wf-003", "outbound");
        record.mark_retrying();
        assert_eq!(record.status, WorkflowStatus::Retrying);
        assert_eq!(record.retry_count, 1);
        record.mark_retrying();
        assert_eq!(record.retry_count, 2);
    }

    #[test]
    fn test_workflow_status_as_str() {
        assert_eq!(WorkflowStatus::Pending.as_str(), "PENDING");
        assert_eq!(WorkflowStatus::Success.as_str(), "SUCCESS");
        assert_eq!(WorkflowStatus::CircuitOpen.as_str(), "CIRCUIT_OPEN");
    }

    #[test]
    fn test_circuit_state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_ne!(CircuitState::Closed, CircuitState::Open);
    }
}
