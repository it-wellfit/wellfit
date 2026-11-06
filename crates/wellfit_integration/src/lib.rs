#![forbid(unsafe_code)]

//! Wellfit Integration — ERP coordination, dual-track reconciliation, and
//! feature-flag based switching.
//!
//! Provides:
//! - `coordination` module: dual-track reconciliation engine
//! - `reconciliation` module: data comparison, diff detection, sync resolution
//! - `switching` module: feature-flag based cutover, rollback procedures

pub mod coordination;
pub mod reconciliation;
pub mod switching;
pub mod switching_drill;
pub mod three_way;

pub use coordination::*;
pub use reconciliation::*;
pub use switching::*;
pub use switching_drill::*;
pub use three_way::*;
