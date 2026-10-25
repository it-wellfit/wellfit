#![forbid(unsafe_code)]

//! Configurable validation parameters for the MTRIX engine.
//!
//! These parameters are derived from the authoritative sources (manual L1,
//! field dictionary L2, V6.0 L3). Values that are still pending counterparty
//! confirmation (MTRIX-001, MTRIX-003, MTRIX-024) are exposed here with
//! sensible defaults so they can be overridden without recompiling.

/// Line ending style for output files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Carriage return only (matches delivered files).
    CR,
    /// Line feed only.
    LF,
    /// Carriage return + line feed.
    CRLF,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::CR => "\r",
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
        }
    }
}

/// Header mode for output files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderMode {
    /// Use canonical (technical alias) header.
    Canonical,
    /// Use original (delivered) header.
    Original,
    /// Write both headers (canonical first, then original).
    Both,
}

/// Configuration for MTRIX validation behaviour.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Brazilian postal code (CEP) format pattern.
    /// Default: 8 digits, optional hyphen (XXXXX-XXX).
    /// Pending counterparty confirmation (MTRIX-001).
    pub cp_format_pattern: &'static str,

    /// Columns whose mandatory-ness is undefined (MTRIX-003).
    /// Empty values in these columns produce a VR-15 Warning rather than a rejection.
    pub undefined_mandatory_columns: &'static [&'static str],

    /// Decimal places retained in scaled quantities.
    /// Default: 4 (L2 recommendation, MTRIX-024 pending).
    pub decimal_places: u32,

    /// Whether to validate CP format strictly.
    /// Default: false (pending counterparty confirmation).
    pub cp_format_strict: bool,

    /// Output line ending style.
    /// Default: CR (matches delivered files).
    pub output_line_ending: LineEnding,

    /// Output header mode.
    /// Default: Canonical (technical alias).
    pub output_header_mode: HeaderMode,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            cp_format_pattern: r"^\d{5}-?\d{3}$",
            undefined_mandatory_columns: &[
                "DOCUMENT_TYPE",
                "SALESPERSON_CODE",
                "COUNTRY_ISO",
                "COUNTRY",
                "ZIP_CODE",
            ],
            decimal_places: 4,
            cp_format_strict: false,
            output_line_ending: LineEnding::CR,
            output_header_mode: HeaderMode::Canonical,
        }
    }
}

impl ValidationConfig {
    /// Validate a postal code against the configured pattern.
    pub fn is_valid_cp(&self, value: &str) -> bool {
        if value.is_empty() {
            return true; // empty handled by mandatory check
        }
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.len() == 8
            && value.chars().all(|c| c.is_ascii_digit() || c == '-')
            && value.matches('-').count() <= 1
    }
}
