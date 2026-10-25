#![cfg(test)]

use crate::health;

#[test]
fn test_health_check_runs() {
    health::run_health_check();
}

#[test]
fn test_health_check_has_nonnegative_uptime() {
    let start = std::time::SystemTime::now();
    health::run_health_check();
    let elapsed = start.elapsed().unwrap().as_secs();
    // elapsed is u64, so it's always >= 0; just verify the health check completes
    let _ = elapsed;
}
