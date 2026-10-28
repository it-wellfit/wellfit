#![forbid(unsafe_code)]

//! Wellfit error code registry (ERR-001 .. ERR-099).
//!
//! Every user-facing failure in the Wellfit CLI is reported with a stable
//! numeric code so that operators, support engineers and automated tooling can
//! map a message back to a documented cause and remediation without parsing
//! free-form text. The codes are grouped by subsystem:
//!
//! - `ERR-001..009`  - argument / invocation errors
//! - `ERR-010..019`  - file system and I/O errors
//! - `ERR-020..029`  - ledger / database errors
//! - `ERR-030..039`  - MTRIX validation errors
//! - `ERR-040..049`  - Shopee inbound errors
//! - `ERR-050..059`  - report extraction errors
//! - `ERR-060..069`  - bankpay errors
//! - `ERR-070..079`  - access control / JIT errors
//! - `ERR-080..089`  - sync daemon errors
//! - `ERR-090..099`  - configuration / environment errors

use std::fmt;

/// A stable Wellfit error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode {
    /// Numeric identifier (1..=99).
    pub number: u8,
}

impl ErrorCode {
    /// Construct a code from its numeric value. Returns `None` for values
    /// outside the supported range (1..=99).
    pub const fn new(number: u8) -> Option<Self> {
        if number == 0 || number > 99 {
            None
        } else {
            Some(Self { number })
        }
    }

    /// Format as `ERR-XXX`.
    pub fn label(self) -> String {
        format!("ERR-{:03}", self.number)
    }

    /// Human readable description of the code.
    pub fn description(self) -> &'static str {
        match self.number {
            1 => "invalid or missing command-line argument",
            2 => "unknown subcommand",
            3 => "missing required argument",
            4 => "conflicting arguments supplied",
            5 => "invalid argument value",
            6 => "help requested",
            7 => "version requested",
            8 => "too many positional arguments",
            9 => "unexpected flag",
            10 => "file not found",
            11 => "directory not found",
            12 => "permission denied",
            13 => "I/O error while reading file",
            14 => "I/O error while writing file",
            15 => "file already exists",
            16 => "path is not a file",
            17 => "path is not a directory",
            18 => "file too large",
            19 => "encoding error",
            20 => "cannot open ledger database",
            21 => "database schema mismatch",
            22 => "duplicate batch rejected",
            23 => "order already processed",
            24 => "transaction failed",
            25 => "constraint violation",
            26 => "query failed",
            27 => "cannot close database",
            28 => "corrupt database file",
            29 => "migration failed",
            30 => "MTRIX file discovery failed",
            31 => "MTRIX header mismatch",
            32 => "MTRIX row rejected",
            33 => "MTRIX cross-file referential failure",
            34 => "MTRIX duplicate transaction",
            35 => "MTRIX negative inventory",
            36 => "MTRIX date format invalid",
            37 => "MTRIX value set violation",
            38 => "MTRIX record count mismatch",
            39 => "MTRIX water-level misalignment",
            40 => "Shopee CSV parse error",
            41 => "Shopee missing required column",
            42 => "Shopee invalid order status",
            43 => "Shopee duplicate order",
            44 => "Shopee customer not found",
            45 => "Shopee product not found",
            46 => "Shopee quantity invalid",
            47 => "Shopee price invalid",
            48 => "Shopee import failed",
            49 => "Shopee export failed",
            50 => "report CSV parse error",
            51 => "report missing required column",
            52 => "report invalid quantity",
            53 => "report invalid price",
            54 => "report duplicate record",
            55 => "report reconciliation failed",
            56 => "report extraction failed",
            57 => "report output write failed",
            58 => "report format unsupported",
            59 => "report schema mismatch",
            60 => "bankpay export failed",
            61 => "bankpay import failed",
            62 => "bankpay voucher not found",
            63 => "bankpay duplicate voucher",
            64 => "bankpay encoding error",
            65 => "bankpay dispatch failed",
            66 => "bankpay result mismatch",
            67 => "bankpay release gate rejected",
            68 => "bankpay query failed",
            69 => "bankpay media file invalid",
            70 => "access denied",
            71 => "role not found",
            72 => "permission not found",
            73 => "segregation of duties violation",
            74 => "JIT request not found",
            75 => "JIT grant expired",
            76 => "JIT approval failed",
            77 => "JIT revocation failed",
            78 => "audit log read failed",
            79 => "classification mismatch",
            80 => "sync configuration invalid",
            81 => "sync connection failed",
            82 => "sync checkpoint failed",
            83 => "sync device not found",
            84 => "sync transfer failed",
            85 => "sync retry exhausted",
            86 => "sync bandwidth limit exceeded",
            87 => "sync discovery failed",
            88 => "sync health check failed",
            89 => "sync shutdown failed",
            90 => "configuration file not found",
            91 => "configuration parse error",
            92 => "configuration schema violation",
            93 => "missing required configuration key",
            94 => "invalid configuration value",
            95 => "environment variable missing",
            96 => "toolchain unavailable",
            97 => "target directory unavailable",
            98 => "application control policy blocked execution",
            99 => "internal error",
            _ => "unknown error",
        }
    }

    pub fn is_config(self) -> bool {
        self.number >= 90
    }

    pub fn is_io(self) -> bool {
        (10..=19).contains(&self.number)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.label(), self.description())
    }
}

pub const fn code(n: u8) -> ErrorCode {
    match ErrorCode::new(n) {
        Some(c) => c,
        None => panic!("error code out of range"),
    }
}

pub fn all_codes() -> Vec<ErrorCode> {
    (1u8..=99u8).map(|n| ErrorCode { number: n }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_has_a_description() {
        for c in all_codes() {
            assert!(
                !c.description().is_empty(),
                "empty description for {}",
                c.label()
            );
            assert_ne!(
                c.description(),
                "unknown error",
                "undefined code {}",
                c.label()
            );
        }
    }

    #[test]
    fn labels_are_zero_padded() {
        assert_eq!(code(1).label(), "ERR-001");
        assert_eq!(code(42).label(), "ERR-042");
        assert_eq!(code(99).label(), "ERR-099");
    }

    #[test]
    fn new_rejects_out_of_range() {
        assert!(ErrorCode::new(0).is_none());
        assert!(ErrorCode::new(100).is_none());
        assert!(ErrorCode::new(1).is_some());
        assert!(ErrorCode::new(99).is_some());
    }

    #[test]
    fn display_includes_label_and_description() {
        let s = code(10).to_string();
        assert!(s.starts_with("ERR-010:"));
        assert!(s.contains("file not found"));
    }

    #[test]
    fn all_codes_count_is_99() {
        assert_eq!(all_codes().len(), 99);
    }

    #[test]
    fn config_range_check() {
        assert!(code(90).is_config());
        assert!(!code(10).is_config());
    }
}
