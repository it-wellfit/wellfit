#![forbid(unsafe_code)]

//! Assistive Tools — OEKAP cornerstone crate.
//!
//! Provides data transformation, report generation, anomaly detection,
//! and shadow-mode evidence collection for the vendor-independent ERP
//! replacement strategy.

pub mod anomaly;
pub mod capture;
pub mod data_transformer;
pub mod report_generator;
pub mod shadow;

pub use anomaly::*;
pub use capture::*;
pub use data_transformer::*;
pub use report_generator::*;
pub use shadow::*;

/// Unified error type for the assistive-tools layer.
#[derive(Debug)]
pub enum AssistError {
    /// Encoding conversion failed (malformed input or unrepresentable chars).
    Encoding(String),
    /// Data validation failed (missing columns, type mismatch, constraint violation).
    Validation(String),
    /// Format transformation failed.
    Transform(String),
    /// Feature not yet implemented.
    NotImplemented(String),
    /// I/O error.
    Io(std::io::Error),
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
}

impl std::fmt::Display for AssistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssistError::Encoding(msg) => write!(f, "encoding error: {msg}"),
            AssistError::Validation(msg) => write!(f, "validation error: {msg}"),
            AssistError::Transform(msg) => write!(f, "transform error: {msg}"),
            AssistError::NotImplemented(msg) => write!(f, "not implemented: {msg}"),
            AssistError::Io(e) => write!(f, "io error: {e}"),
            AssistError::Json(e) => write!(f, "json error: {e}"),
        }
    }
}

impl std::error::Error for AssistError {}

impl From<std::io::Error> for AssistError {
    fn from(e: std::io::Error) -> Self {
        AssistError::Io(e)
    }
}

impl From<serde_json::Error> for AssistError {
    fn from(e: serde_json::Error) -> Self {
        AssistError::Json(e)
    }
}
