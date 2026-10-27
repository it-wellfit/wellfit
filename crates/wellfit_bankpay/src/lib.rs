#![forbid(unsafe_code)]

//! Wellfit BankPay - E.Sun Bank enterprise net-banking interface for the
//! Fuyi ERP (FlapsErp) accounts payable voucher wave.
//!
//! Provides:
//! - Encoding conversion between internal UTF-8 and Taiwan bank/ERP
//!   encodings (CP950 / Big5 / Big5-HKSCS).
//! - Export pipeline: ERP voucher -> encrypted bank media file -> bank.
//! - Import pipeline: bank result file -> voucher status update -> ERP.
//! - Idempotency ledger integration (file batches, transaction keys).
//! - Release gate (two-way door verify, one-way door dispatch).
//! - Traceable query: voucher <-> bank reference <-> batch <-> file SHA-256.
//!
//! All amounts are carried as integer cents via [`crate::scaled::ScaledDecimal`].
//! Nothing is unsafe; every public API is documented.

pub mod bankpay;
pub mod encoding;
pub mod query;
pub mod release;

pub use bankpay::*;
pub use encoding::*;
pub use query::*;
pub use release::*;
