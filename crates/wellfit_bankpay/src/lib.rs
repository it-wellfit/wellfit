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
//! All amounts are carried as integer cents via `ScaledDecimal` (wellfit_core).
//! Nothing is unsafe; every public API is documented.

pub mod api;
pub mod bankpay;
pub mod encoding;
pub mod http;
pub mod media;
pub mod payable_automation;
pub mod query;
pub mod release;

pub use api::*;
pub use bankpay::*;
pub use encoding::*;
pub use http::*;
pub use media::*;
pub use payable_automation::*;
pub use query::*;
pub use release::*;

#[cfg(test)]
mod encoding_regression_tests {
    //! Slice-audit regression: bankpay sources were once committed as UTF-16
    //! LE with BOM (git classified them as binary, blocking text review and
    //! diff). These guards make that class of corruption a test failure.

    #[test]
    fn source_files_are_valid_utf8_without_nul_bytes() {
        for (name, bytes) in [
            ("http.rs", include_bytes!("http.rs").as_slice()),
            ("lib.rs", include_bytes!("lib.rs").as_slice()),
        ] {
            assert!(
                !bytes.contains(&0u8),
                "{name} contains NUL bytes - UTF-16/encoding corruption regression"
            );
            std::str::from_utf8(bytes).unwrap_or_else(|e| panic!("{name} is not valid UTF-8: {e}"));
        }
    }

    #[test]
    fn corrupted_retry_code_module_stays_deleted() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/retry_code.rs");
        assert!(
            !p.exists(),
            "retry_code.rs (UTF-16 corrupted 56-byte stub) must stay deleted"
        );
    }
}
