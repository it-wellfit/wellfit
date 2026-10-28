#![forbid(unsafe_code)]

//! Configuration schema validation for `wellfit.toml`.
//!
//! Validates that a Wellfit configuration file parses as TOML and contains the
//! required sections, keys and value types/ranges. Returns a list of
//! [`ConfigError`]s so callers can report every problem at once instead of
//! failing on the first one.

use serde::Deserialize;
use toml::Value;

/// A single configuration validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    /// Dotted key path, e.g. `database.path`.
    pub key: String,
    /// Human-readable description of the problem.
    pub message: String,
}

impl ConfigError {
    pub fn new(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.key, self.message)
    }
}

/// Schema for the `[database]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

fn default_pool_size() -> u32 {
    4
}

/// Schema for the `[mtrix]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct MtrixConfig {
    pub drop_dir: String,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default = "default_strict")]
    pub strict: bool,
}

fn default_strict() -> bool {
    false
}

/// Schema for the `[sync]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
    #[serde(default = "default_retry")]
    pub retry_max_attempts: u32,
}

fn default_interval() -> u64 {
    300
}
fn default_max_concurrent() -> u32 {
    4
}
fn default_retry() -> u32 {
    3
}

/// Top-level configuration schema.
#[derive(Debug, Clone, Deserialize)]
pub struct WellfitConfig {
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub mtrix: Option<MtrixConfig>,
    #[serde(default)]
    pub sync: Option<SyncConfig>,
}

/// Validate a TOML configuration string against the Wellfit schema.
///
/// Returns `Ok(())` when the document is valid, or a list of all detected
/// errors. An empty document is considered valid (all sections optional).
pub fn validate_config(content: &str) -> Result<(), Vec<ConfigError>> {
    let mut errors: Vec<ConfigError> = Vec::new();

    // Parse as TOML first.
    let value: Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            errors.push(ConfigError::new("<root>", format!("TOML parse error: {e}")));
            return Err(errors);
        }
    };

    if !value.is_table() {
        errors.push(ConfigError::new(
            "<root>",
            "top-level value must be a table",
        ));
        return Err(errors);
    }

    // Deserialise into the typed schema to catch type mismatches.
    let cfg: WellfitConfig = match toml::from_str(content) {
        Ok(c) => c,
        Err(e) => {
            errors.push(ConfigError::new("<root>", format!("schema error: {e}")));
            return Err(errors);
        }
    };

    // Validate database section.
    if let Some(db) = &cfg.database {
        if db.path.trim().is_empty() {
            errors.push(ConfigError::new("database.path", "must not be empty"));
        }
        if db.pool_size == 0 || db.pool_size > 64 {
            errors.push(ConfigError::new(
                "database.pool_size",
                "must be between 1 and 64",
            ));
        }
    }

    // Validate mtrix section.
    if let Some(m) = &cfg.mtrix {
        if m.drop_dir.trim().is_empty() {
            errors.push(ConfigError::new("mtrix.drop_dir", "must not be empty"));
        }
        if let Some(out) = &m.output_dir {
            if out.trim().is_empty() {
                errors.push(ConfigError::new(
                    "mtrix.output_dir",
                    "must not be empty when set",
                ));
            }
        }
    }

    // Validate sync section.
    if let Some(s) = &cfg.sync {
        if s.interval_seconds == 0 {
            errors.push(ConfigError::new(
                "sync.interval_seconds",
                "must be greater than 0",
            ));
        }
        if s.max_concurrent == 0 || s.max_concurrent > 32 {
            errors.push(ConfigError::new(
                "sync.max_concurrent",
                "must be between 1 and 32",
            ));
        }
        if s.retry_max_attempts > 10 {
            errors.push(ConfigError::new(
                "sync.retry_max_attempts",
                "must not exceed 10",
            ));
        }
    }

    // Unknown top-level keys are allowed but reported as info? We keep strict
    // on the documented sections only.

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_document_is_valid() {
        assert!(validate_config("").is_ok());
    }

    #[test]
    fn valid_full_config() {
        let toml = r#"
[database]
path = "data/wellfit.sqlite"
pool_size = 8

[mtrix]
drop_dir = "spec/fixtures/mtrix/delivered"
output_dir = "data/out"
strict = true

[sync]
interval_seconds = 120
max_concurrent = 2
retry_max_attempts = 5
"#;
        assert!(validate_config(toml).is_ok());
    }

    #[test]
    fn invalid_toml_reports_parse_error() {
        let err = validate_config("this is not = toml [").unwrap_err();
        assert!(!err.is_empty());
        assert!(err[0].key == "<root>");
    }

    #[test]
    fn empty_database_path_rejected() {
        let toml = r#"
[database]
path = "   "
"#;
        let err = validate_config(toml).unwrap_err();
        assert!(err.iter().any(|e| e.key == "database.path"));
    }

    #[test]
    fn zero_pool_size_rejected() {
        let toml = r#"
[database]
path = "x.sqlite"
pool_size = 0
"#;
        let err = validate_config(toml).unwrap_err();
        assert!(err.iter().any(|e| e.key == "database.pool_size"));
    }

    #[test]
    fn sync_interval_zero_rejected() {
        let toml = r#"
[sync]
interval_seconds = 0
"#;
        let err = validate_config(toml).unwrap_err();
        assert!(err.iter().any(|e| e.key == "sync.interval_seconds"));
    }

    #[test]
    fn sync_max_concurrent_out_of_range_rejected() {
        let toml = r#"
[sync]
max_concurrent = 0
"#;
        let err = validate_config(toml).unwrap_err();
        assert!(err.iter().any(|e| e.key == "sync.max_concurrent"));
    }

    #[test]
    fn type_mismatch_reported() {
        let toml = r#"
[database]
path = 42
"#;
        let err = validate_config(toml).unwrap_err();
        assert!(!err.is_empty());
    }
}
