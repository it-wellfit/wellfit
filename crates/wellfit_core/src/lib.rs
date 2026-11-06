#![forbid(unsafe_code)]

//! Wellfit Core - Common Infrastructure and Sovereign Governance Substrate
//!
//! Provides Master Data Management (MDMHub), SQLite Idempotency Ledger,
//! Desktop Session Mutex, Layer 5 Release Gate, and fixed-point decimal
//! arithmetic used by the MTRIX measures.

pub mod access_control;
pub mod audit_integrity;
pub mod automation;
pub mod config_validator;
pub mod contract;
pub mod currency_precision;
pub mod customer_automation;
pub mod error_codes;
pub mod federation;
pub mod inventory_automation;
pub mod jit_permission;
pub mod ledger;
pub mod mdm;
pub mod mutex;
pub mod release_gate;
pub mod scaled;
pub mod text;

pub use access_control::*;
pub use audit_integrity::*;
pub use automation::*;
pub use config_validator::*;
pub use contract::*;
pub use currency_precision::*;
pub use customer_automation::*;
pub use error_codes::*;
pub use federation::*;
pub use inventory_automation::*;
pub use jit_permission::*;
pub use ledger::*;
pub use mdm::*;
pub use mutex::*;
pub use release_gate::*;
pub use scaled::*;
pub use text::*;
