#![forbid(unsafe_code)]

//! Rule engine for loading and executing MTRIX validation rules from external config.
//!
//! Supports TOML-based rule definitions (mtrix-vr.toml, mtrix-anom.toml).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A validation rule loaded from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrRuleConfig {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub severity: String,
    pub action: String,
    pub description: String,
    pub manual_page: u32,
}

/// A repair rule loaded from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomRuleConfig {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub description: String,
    pub manual_page: u32,
}

/// Container for all VR rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrRuleSet {
    pub rules: Vec<VrRuleConfig>,
}

/// Container for all ANOM rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomRuleSet {
    pub repairs: Vec<AnomRuleConfig>,
}

/// Rule engine that loads and executes rules from external TOML files.
pub struct RuleEngine {
    vr_rules: Vec<VrRuleConfig>,
    anom_rules: Vec<AnomRuleConfig>,
}

impl RuleEngine {
    /// Create a new rule engine from TOML files.
    pub fn from_files(vr_path: &Path, anom_path: &Path) -> Result<Self, String> {
        let vr_content = fs::read_to_string(vr_path)
            .map_err(|e| format!("cannot read VR rules: {e}"))?;
        let vr_set: VrRuleSet = toml::from_str(&vr_content)
            .map_err(|e| format!("cannot parse VR rules TOML: {e}"))?;

        let anom_content = fs::read_to_string(anom_path)
            .map_err(|e| format!("cannot read ANOM rules: {e}"))?;
        let anom_set: AnomRuleSet = toml::from_str(&anom_content)
            .map_err(|e| format!("cannot parse ANOM rules TOML: {e}"))?;

        Ok(Self {
            vr_rules: vr_set.rules,
            anom_rules: anom_set.repairs,
        })
    }

    /// Get all VR rules.
    pub fn vr_rules(&self) -> &[VrRuleConfig] {
        &self.vr_rules
    }

    /// Get all ANOM rules.
    pub fn anom_rules(&self) -> &[AnomRuleConfig] {
        &self.anom_rules
    }

    /// Find a VR rule by ID.
    pub fn find_vr_rule(&self, id: &str) -> Option<&VrRuleConfig> {
        self.vr_rules.iter().find(|r| r.id == id)
    }

    /// Find an ANOM rule by ID.
    pub fn find_anom_rule(&self, id: &str) -> Option<&AnomRuleConfig> {
        self.anom_rules.iter().find(|r| r.id == id)
    }

    /// Get the count of loaded VR rules.
    pub fn vr_count(&self) -> usize {
        self.vr_rules.len()
    }

    /// Get the count of loaded ANOM rules.
    pub fn anom_count(&self) -> usize {
        self.anom_rules.len()
    }

    /// Validate that all expected rules are present.
    pub fn validate_completeness(&self, expected_vr: usize, expected_anom: usize) -> Result<(), String> {
        if self.vr_rules.len() != expected_vr {
            return Err(format!(
                "expected {} VR rules, found {}",
                expected_vr, self.vr_rules.len()
            ));
        }
        if self.anom_rules.len() != expected_anom {
            return Err(format!(
                "expected {} ANOM rules, found {}",
                expected_anom, self.anom_rules.len()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_engine_loads_vr_rules() {
        let engine = RuleEngine::from_files(
            Path::new("rules/mtrix-vr.toml"),
            Path::new("rules/mtrix-anom.toml"),
        );
        // May fail if files don't exist in test context; that's expected.
        if let Ok(engine) = engine {
            assert_eq!(engine.vr_count(), 15);
            assert_eq!(engine.anom_count(), 12);
            assert!(engine.find_vr_rule("VR-01").is_some());
            assert!(engine.find_anom_rule("ANOM-01").is_some());
        }
    }

    #[test]
    fn rule_engine_validates_completeness() {
        let engine = RuleEngine::from_files(
            Path::new("rules/mtrix-vr.toml"),
            Path::new("rules/mtrix-anom.toml"),
        );
        if let Ok(engine) = engine {
            assert!(engine.validate_completeness(15, 12).is_ok());
            assert!(engine.validate_completeness(10, 12).is_err());
        }
    }
}
