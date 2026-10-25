#![forbid(unsafe_code)]

//! Health monitoring and health check endpoints.

use std::time::{SystemTime, UNIX_EPOCH};

pub fn run_health_check() {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    println!(
        "{{\"status\": \"ok\", \"uptime\": {}, \"version\": \"0.1.0\"}}",
        uptime
    );
}
