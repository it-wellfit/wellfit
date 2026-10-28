#![forbid(unsafe_code)]

//! Wellfit MTRIX - Alpargatas MTRIX 56-Column Specification & Validation Engine
//!
//! The crate is organised around the interface contract rather than around the
//! files that happen to arrive:
//!
//! - [`spec`] holds the machine-readable contract: five files, 56 columns, each
//!   column carrying its provenance and verification status.
//! - [`repairs`] holds the numbered repair rules that may be applied, each one
//!   bound to the source that authorises it.
//! - [`validators`] holds the validation rules VR-01 .. VR-12 and the report
//!   model.
//! - [`mtrix_engine`] orchestrates accept, repair and validate over a drop.

pub mod config;
pub mod mtrix_engine;
pub mod pipeline;
pub mod repairs;
pub mod rule_engine;
pub mod spec;
pub mod validators;

pub use config::*;
pub use mtrix_engine::*;
pub use pipeline::*;
pub use repairs::*;
pub use rule_engine::*;
pub use spec::*;
pub use validators::*;
