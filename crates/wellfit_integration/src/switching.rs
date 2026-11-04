#![forbid(unsafe_code)]

//! Switching — feature-flag based cutover and rollback procedures.
//!
//! Manages the staged migration from legacy system to new system through
//! explicit feature-flag transitions, with automatic rollback on failure.

use std::collections::HashMap;
use wellfit_core::audit_integrity::compute_hash;

/// Feature flag for system switching.
#[derive(Debug, Clone)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub rollout_percentage: u32,
    pub rollback_allowed: bool,
}

impl FeatureFlag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: false,
            rollout_percentage: 0,
            rollback_allowed: true,
        }
    }

    pub fn enable(&mut self, percentage: u32) {
        self.enabled = true;
        self.rollout_percentage = percentage.min(100);
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.rollout_percentage = 0;
    }

    pub fn is_active(&self) -> bool {
        self.enabled && self.rollout_percentage > 0
    }
}

/// Switching state record for audit.
#[derive(Debug, Clone)]
pub struct SwitchingRecord {
    pub timestamp: u64,
    pub flag_name: String,
    pub previous_state: bool,
    pub new_state: bool,
    pub reason: String,
    pub hash: String,
}

/// Feature-flag switching controller.
pub struct SwitchingController {
    flags: HashMap<String, FeatureFlag>,
    history: Vec<SwitchingRecord>,
}

impl SwitchingController {
    /// Create a new switching controller with default flags.
    pub fn new() -> Self {
        let mut flags = HashMap::new();

        // Core switching flags
        flags.insert("dual_track".to_string(), FeatureFlag::new("dual_track"));
        flags.insert(
            "shadow_legacy".to_string(),
            FeatureFlag::new("shadow_legacy"),
        );
        flags.insert(
            "legacy_read_only".to_string(),
            FeatureFlag::new("legacy_read_only"),
        );
        flags.insert(
            "decommissioned".to_string(),
            FeatureFlag::new("decommissioned"),
        );

        Self {
            flags,
            history: Vec::new(),
        }
    }

    /// Enable a feature flag at a given rollout percentage.
    pub fn enable_flag(&mut self, flag_name: &str, percentage: u32, reason: &str) {
        if let Some(flag) = self.flags.get_mut(flag_name) {
            let prev = flag.enabled;
            flag.enable(percentage);
            self.record_switch(flag_name, prev, true, reason);
        }
    }

    /// Disable a feature flag (rollback).
    pub fn disable_flag(&mut self, flag_name: &str, reason: &str) {
        if let Some(flag) = self.flags.get_mut(flag_name) {
            if !flag.rollback_allowed {
                eprintln!("warning: rollback not allowed for flag '{}'", flag_name);
                return;
            }
            let prev = flag.enabled;
            flag.disable();
            self.record_switch(flag_name, prev, false, reason);
        }
    }

    /// Check if a flag is active.
    pub fn is_active(&self, flag_name: &str) -> bool {
        self.flags
            .get(flag_name)
            .map(|f| f.is_active())
            .unwrap_or(false)
    }

    /// Get rollout percentage for a flag.
    pub fn rollout_percentage(&self, flag_name: &str) -> u32 {
        self.flags
            .get(flag_name)
            .map(|f| f.rollout_percentage)
            .unwrap_or(0)
    }

    /// Get all flags.
    pub fn flags(&self) -> &HashMap<String, FeatureFlag> {
        &self.flags
    }

    /// Get switching history.
    pub fn history(&self) -> &[SwitchingRecord] {
        &self.history
    }

    /// Rollback all flags to their initial state.
    pub fn rollback_all(&mut self, reason: &str) {
        let flag_names: Vec<String> = self.flags.keys().cloned().collect();
        for name in flag_names {
            let should_disable = self.flags.get(&name).map(|f| f.enabled).unwrap_or(false);
            if should_disable {
                if let Some(flag) = self.flags.get_mut(&name) {
                    let prev = flag.enabled;
                    flag.disable();
                    self.record_switch(&name, prev, false, reason);
                }
            }
        }
    }

    /// Record a switching action with audit hash.
    fn record_switch(&mut self, flag_name: &str, prev: bool, new_state: bool, reason: &str) {
        let timestamp = current_timestamp_ms();
        let data = format!(
            "{}:{}:{}:{}:{}",
            timestamp, flag_name, prev, new_state, reason
        );
        let hash = compute_hash(&data);

        self.history.push(SwitchingRecord {
            timestamp,
            flag_name: flag_name.to_string(),
            previous_state: prev,
            new_state,
            reason: reason.to_string(),
            hash,
        });
    }
}

impl Default for SwitchingController {
    fn default() -> Self {
        Self::new()
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
    fn test_flag_new() {
        let flag = FeatureFlag::new("test_flag");
        assert!(!flag.enabled);
        assert_eq!(flag.rollout_percentage, 0);
        assert!(flag.rollback_allowed);
        assert!(!flag.is_active());
    }

    #[test]
    fn test_flag_enable() {
        let mut flag = FeatureFlag::new("test_flag");
        flag.enable(50);
        assert!(flag.enabled);
        assert_eq!(flag.rollout_percentage, 50);
        assert!(flag.is_active());
    }

    #[test]
    fn test_flag_enable_caps_at_100() {
        let mut flag = FeatureFlag::new("test_flag");
        flag.enable(150);
        assert_eq!(flag.rollout_percentage, 100);
    }

    #[test]
    fn test_flag_disable() {
        let mut flag = FeatureFlag::new("test_flag");
        flag.enable(100);
        assert!(flag.is_active());
        flag.disable();
        assert!(!flag.enabled);
        assert!(!flag.is_active());
    }

    #[test]
    fn test_controller_new() {
        let controller = SwitchingController::new();
        assert!(!controller.is_active("dual_track"));
        assert!(!controller.is_active("shadow_legacy"));
        assert!(controller.history().is_empty());
    }

    #[test]
    fn test_controller_enable_flag() {
        let mut controller = SwitchingController::new();
        controller.enable_flag("dual_track", 100, "initial enable");
        assert!(controller.is_active("dual_track"));
        assert_eq!(controller.rollout_percentage("dual_track"), 100);
        assert_eq!(controller.history().len(), 1);
    }

    #[test]
    fn test_controller_disable_flag() {
        let mut controller = SwitchingController::new();
        controller.enable_flag("dual_track", 100, "enable");
        controller.disable_flag("dual_track", "rollback");
        assert!(!controller.is_active("dual_track"));
        assert_eq!(controller.history().len(), 2);
    }

    #[test]
    fn test_controller_rollback_all() {
        let mut controller = SwitchingController::new();
        controller.enable_flag("dual_track", 100, "enable");
        controller.enable_flag("shadow_legacy", 50, "enable");
        assert_eq!(controller.history().len(), 2);

        controller.rollback_all("emergency rollback");
        assert!(!controller.is_active("dual_track"));
        assert!(!controller.is_active("shadow_legacy"));
        assert_eq!(controller.history().len(), 4);
    }

    #[test]
    fn test_controller_staged_rollout() {
        let mut controller = SwitchingController::new();

        // Stage 1: dual-track at 10%
        controller.enable_flag("dual_track", 10, "stage 1: 10% rollout");
        assert_eq!(controller.rollout_percentage("dual_track"), 10);

        // Stage 2: increase to 50%
        controller.enable_flag("dual_track", 50, "stage 2: 50% rollout");
        assert_eq!(controller.rollout_percentage("dual_track"), 50);

        // Stage 3: full rollout
        controller.enable_flag("dual_track", 100, "stage 3: full rollout");
        assert_eq!(controller.rollout_percentage("dual_track"), 100);
    }

    #[test]
    fn test_switching_record_hash() {
        let mut controller = SwitchingController::new();
        controller.enable_flag("dual_track", 100, "test");
        let record = controller.history().first().unwrap();
        assert!(!record.hash.is_empty());
        assert_eq!(record.hash.len(), 64);
        assert_eq!(record.flag_name, "dual_track");
        assert!(!record.previous_state);
        assert!(record.new_state);
    }
}
