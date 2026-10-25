#![forbid(unsafe_code)]

//! Wellfit MTRIX - Specification Registry (Single Source of Truth)
//!
//! Machine-readable mirror of the Alpargatas MTRIX interface contract.
//!
//! ## Source tiers (ADR-0001, R-A / R-B)
//!
//! | Tier | Meaning | Used for |
//! |---|---|---|
//! | [`SourceTier::ManualL1`] | Alpargatas Programmer's Manual, DCA Revision, Sept 2024 (original spec) | field name, type, length, allowed values, page reference |
//! | [`SourceTier::InternalL2`] | Internal recommended spec (MTRIX field dictionary V2) | recommended type/length, issue links |
//! | [`SourceTier::SpecV6L3`] | ALPARGATAS interface spec V6.0 (newest internal consolidation) | superseding evidence |
//! | [`SourceTier::ObservedCsv`] | Actual bytes received from the counterpart system | interface acceptance baseline (R-B) |
//!
//! ## Verification status (IMS document-control convention)
//!
//! `Confirmed` = cross-checked against the original manual page and observed bytes.
//! `ObservedOnly` = seen in the interface bytes but not yet confirmed against the manual.
//! `Deviation` = source header differs from the manual wording; deviation is reported, never silently rewritten.
//! `Pending` = awaiting an official answer (MTRIX-001..030 register).
//! `Unverified` = the authoritative source could not be extracted; never guess a value.
//!
//! Every field carries its provenance so that no assumption can hide inside the engine.

use serde::{Deserialize, Serialize};

/// Which specification layer a field definition was taken from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceTier {
    /// Alpargatas Programmer's Manual (DCA Revision, September 2024).
    ManualL1,
    /// Internal recommended specification (MTRIX field dictionary V2).
    InternalL2,
    /// ALPARGATAS interface specification V6.0.
    SpecV6L3,
    /// Observed in the bytes actually delivered by the counterpart system.
    ObservedCsv,
}

impl SourceTier {
    /// Short, stable label used in generated documentation.
    pub fn label(self) -> &'static str {
        match self {
            SourceTier::ManualL1 => "ManualL1",
            SourceTier::InternalL2 => "InternalL2",
            SourceTier::SpecV6L3 => "SpecV6L3",
            SourceTier::ObservedCsv => "ObservedCsv",
        }
    }
}

/// How well a field definition is corroborated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifyStatus {
    /// Confirmed against the original manual and the observed bytes.
    Confirmed,
    /// Present in the interface bytes; not yet confirmed against the manual.
    ObservedOnly,
    /// Observed wording differs from the manual wording; the deviation is reported.
    Deviation,
    /// Blocked on an official answer (see the MTRIX issue register).
    Pending,
    /// The authoritative source was unavailable; no value was invented.
    Unverified,
}

impl VerifyStatus {
    /// IMS-style verification mark.
    pub fn mark(self) -> &'static str {
        match self {
            VerifyStatus::Confirmed => "confirmed",
            VerifyStatus::ObservedOnly => "observed-only",
            VerifyStatus::Deviation => "deviation",
            VerifyStatus::Pending => "pending",
            VerifyStatus::Unverified => "unverified",
        }
    }
}

/// Data shape of a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldKind {
    /// Character data, carried verbatim to preserve leading zeros.
    Text,
    /// `YYYYMMDD` calendar date.
    Date,
    /// Numeric measure. Carried as a scaled integer, never as a float.
    Number,
    /// Coded value restricted to an explicit value set.
    Code,
}

/// The relational role a field plays across the five files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyRole {
    /// Manufacturer / RUC key (may carry a two-character ISO country prefix).
    Vendor,
    /// Distributor RUC key.
    Dist,
    /// Customer / PoS key.
    Customer,
    /// Second (duplicated) customer column, kept separately, never merged.
    CustomerSecondary,
    /// Product key. BARCODE is primary, PRODUCT_CODE is the fallback.
    Product,
    /// Product barcode (EAN13 / DUN14).
    ProductBarcode,
    /// Salesperson key.
    Salesperson,
    /// Batch / item code (sales key, must exist in Inventory and Product).
    Batch,
    /// No relational role.
    None,
}

impl KeyRole {
    /// Whether the field participates in a cross-file referential check.
    pub fn is_key(self) -> bool {
        !matches!(self, KeyRole::None)
    }
}

/// One column of one MTRIX file.
///
/// Only `Serialize` is derived: the tables are static borrowed data, which
/// cannot implement `Deserialize`.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FieldSpec {
    /// 1-based ordinal inside the file.
    pub seq: u8,
    /// Column name as delivered by the counterpart system (acceptance baseline, R-B).
    pub source_name: &'static str,
    /// Column name as written in the original manual (L1).
    pub spec_name: &'static str,
    /// Technical / BI column name (e.g. `ZIP CODE` -> `ZIP_CODE`).
    pub canonical_name: &'static str,
    /// Data shape.
    pub kind: FieldKind,
    /// Declared length in the original manual.
    pub length: u8,
    /// Allowed values for coded fields, `None` when unconstrained.
    pub allowed: Option<&'static [&'static str]>,
    /// Relational role.
    pub key_role: KeyRole,
    /// Specification layer this definition was taken from.
    pub tier: SourceTier,
    /// Corroboration level.
    pub status: VerifyStatus,
    /// Page in the original manual as printed (1-based), when known.
    ///
    /// This is the page cited by the internal field dictionary. It is a
    /// different numbering system from the PDF page index of the extracted
    /// snapshot; use [`manual_pdf_page`] for a reference that can be checked
    /// against the snapshot.
    pub manual_page: Option<u16>,
    /// Open issue in the MTRIX issue register, when the field is disputed.
    pub issue_id: Option<&'static str>,
}

impl FieldSpec {
    /// Whether the observed column name differs from the manual wording.
    pub fn is_deviation(&self) -> bool {
        self.source_name != self.spec_name
    }

    /// Human-readable provenance line for generated documentation.
    pub fn provenance(&self) -> String {
        let page = match self.manual_page {
            Some(p) => format!(" p.{p}"),
            None => String::new(),
        };
        let issue = match self.issue_id {
            Some(i) => format!(" issue {i}"),
            None => String::new(),
        };
        format!(
            "{}{} [{} / {}]{}",
            self.tier.label(),
            page,
            self.status.mark(),
            self.kind_label(),
            issue
        )
    }

    fn kind_label(&self) -> &'static str {
        match self.kind {
            FieldKind::Text => "text",
            FieldKind::Date => "date",
            FieldKind::Number => "number",
            FieldKind::Code => "code",
        }
    }
}

/// The page of the extracted manual snapshot on which each file's field table
/// actually appears.
///
/// The manual exists in two numbering systems. The printed page cited by the
/// internal field dictionary (see [`FieldSpec::manual_page`]) does not align
/// with the PDF page index of the extracted snapshot, and the offset differs
/// per file. Keeping both makes every reference checkable: the PDF page
/// against the snapshot, the printed page against the printed manual.
///
/// The values are derived mechanically by
/// `Knowledge/tools/verification/verify-pages.ps1`, which locates the page
/// carrying the largest number of a file's column names.
pub static MANUAL_PDF_PAGE: &[(&str, u16)] = &[
    ("ALTHCI", 23),
    ("ALTHES", 17),
    ("ALTHFV", 29),
    ("ALTHPD", 36),
    ("ALTHVD", 10),
];

/// The PDF page of a file's field table in the extracted manual snapshot.
pub fn manual_pdf_page(prefix: &str) -> Option<u16> {
    MANUAL_PDF_PAGE
        .iter()
        .find(|(p, _)| p.eq_ignore_ascii_case(prefix))
        .map(|(_, page)| *page)
}

/// The five file roles of the MTRIX interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileRole {
    /// ALTHCI - Customer / point of sale master.
    Customer,
    /// ALTHES - Inventory snapshot.
    Inventory,
    /// ALTHFV - Sales force / salesperson master.
    SalesForce,
    /// ALTHPD - Product master.
    Product,
    /// ALTHVD - Sales / sellout transactions.
    Sales,
}

impl FileRole {
    /// File prefix used by the counterpart system.
    pub fn prefix(self) -> &'static str {
        match self {
            FileRole::Customer => "ALTHCI",
            FileRole::Inventory => "ALTHES",
            FileRole::SalesForce => "ALTHFV",
            FileRole::Product => "ALTHPD",
            FileRole::Sales => "ALTHVD",
        }
    }

    /// Business name of the file.
    pub fn business_name(self) -> &'static str {
        match self {
            FileRole::Customer => "Customer / PoS",
            FileRole::Inventory => "Inventory",
            FileRole::SalesForce => "Sales Force",
            FileRole::Product => "Product",
            FileRole::Sales => "Sales / Sellout",
        }
    }
}

/// The complete layout of one MTRIX file.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FileSpec {
    /// File prefix (`ALTHCI`, ...).
    pub prefix: &'static str,
    /// Business role.
    pub role: FileRole,
    /// Column definitions, in file order.
    pub columns: &'static [FieldSpec],
}

impl FileSpec {
    /// Number of columns declared for this file.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Expected header line, as delivered by the counterpart system.
    pub fn expected_header(&self) -> String {
        self.columns
            .iter()
            .map(|c| c.source_name)
            .collect::<Vec<_>>()
            .join(";")
    }

    /// Canonical (BI-facing) header line.
    pub fn canonical_header(&self) -> String {
        self.columns
            .iter()
            .map(|c| c.canonical_name)
            .collect::<Vec<_>>()
            .join(";")
    }

    /// Locate a column by its observed source name.
    pub fn index_of(&self, source_name: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|c| c.source_name.eq_ignore_ascii_case(source_name))
    }

    /// Locate a column by its relational role.
    pub fn index_of_role(&self, role: KeyRole) -> Option<usize> {
        self.columns.iter().position(|c| c.key_role == role)
    }
}

// ---------------------------------------------------------------------------
// Field tables
// ---------------------------------------------------------------------------

/// Customer / PoS master - 16 columns (manual p.19).
pub const ALTHCI_COLUMNS: &[FieldSpec] = &[
    FieldSpec {
        seq: 1,
        source_name: "VENDOR_ID",
        spec_name: "VENDOR_ID",
        canonical_name: "VENDOR_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Vendor,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 2,
        source_name: "DIST_ID",
        spec_name: "DIST_ID",
        canonical_name: "DIST_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Dist,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 3,
        source_name: "CUSTOMER_ID",
        spec_name: "CUSTOMER_ID",
        canonical_name: "CUSTOMER_ID",
        kind: FieldKind::Text,
        length: 18,
        allowed: None,
        key_role: KeyRole::Customer,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 4,
        source_name: "CUSTOMER_NAME",
        spec_name: "CUSTOMER_NAME",
        canonical_name: "CUSTOMER_NAME",
        kind: FieldKind::Text,
        length: 40,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(19),
        issue_id: Some("MTRIX-008"),
    },
    FieldSpec {
        seq: 5,
        source_name: "ADRESS",
        spec_name: "ADDRESS",
        canonical_name: "ADDRESS",
        kind: FieldKind::Text,
        length: 40,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 6,
        source_name: "DISTRICT",
        spec_name: "DISTRICT",
        canonical_name: "DISTRICT",
        kind: FieldKind::Text,
        length: 30,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 7,
        source_name: "ZIP_CODE",
        spec_name: "ZIP CODE",
        canonical_name: "ZIP_CODE",
        kind: FieldKind::Text,
        length: 9,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 8,
        source_name: "CITY",
        spec_name: "CITY",
        canonical_name: "CITY",
        kind: FieldKind::Text,
        length: 30,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 9,
        source_name: "STATE",
        spec_name: "STATE",
        canonical_name: "STATE",
        kind: FieldKind::Text,
        length: 30,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(19),
        issue_id: Some("MTRIX-010"),
    },
    FieldSpec {
        seq: 10,
        source_name: "SALES_CONTACT",
        spec_name: "SALES_CONTACT",
        canonical_name: "SALES_CONTACT",
        kind: FieldKind::Text,
        length: 20,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 11,
        source_name: "PHONE_NUMBERS",
        spec_name: "PHONE_NUMBERS",
        canonical_name: "PHONE_NUMBERS",
        kind: FieldKind::Text,
        length: 40,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
    FieldSpec {
        seq: 12,
        source_name: "CUSTOMER_ID",
        spec_name: "CUSTOMER_ID",
        canonical_name: "CUSTOMER_ID_2",
        kind: FieldKind::Text,
        length: 18,
        allowed: None,
        key_role: KeyRole::CustomerSecondary,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(19),
        issue_id: Some("MTRIX-009"),
    },
    FieldSpec {
        seq: 13,
        source_name: "ROUTE",
        spec_name: "ROUTE",
        canonical_name: "ROUTE",
        kind: FieldKind::Text,
        length: 10,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(19),
        issue_id: Some("MTRIX-028"),
    },
    FieldSpec {
        seq: 14,
        source_name: "STORE_TYPE",
        spec_name: "STORE_TYPE",
        canonical_name: "STORE_TYPE",
        kind: FieldKind::Text,
        length: 10,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(19),
        issue_id: Some("MTRIX-029"),
    },
    FieldSpec {
        seq: 15,
        source_name: "PROPORTION",
        spec_name: "PROPORTION",
        canonical_name: "PROPORTION",
        kind: FieldKind::Number,
        length: 6,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(19),
        issue_id: Some("MTRIX-030"),
    },
    FieldSpec {
        seq: 16,
        source_name: "COUNTRY_ISO",
        spec_name: "COUNTRY_ISO",
        canonical_name: "COUNTRY_ISO",
        kind: FieldKind::Text,
        length: 3,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(19),
        issue_id: None,
    },
];

/// Inventory snapshot - 6 columns (manual p.15).
pub const ALTHES_COLUMNS: &[FieldSpec] = &[
    FieldSpec {
        seq: 1,
        source_name: "INVENTORY_DATE",
        spec_name: "INVENTORY_DATE",
        canonical_name: "INVENTORY_DATE",
        kind: FieldKind::Date,
        length: 8,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(15),
        issue_id: None,
    },
    FieldSpec {
        seq: 2,
        source_name: "VENDOR_ID",
        spec_name: "VENDOR_ID",
        canonical_name: "VENDOR_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Vendor,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(15),
        issue_id: None,
    },
    FieldSpec {
        seq: 3,
        source_name: "DIST_ID",
        spec_name: "DIST_ID",
        canonical_name: "DIST_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Dist,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(15),
        issue_id: None,
    },
    FieldSpec {
        seq: 4,
        source_name: "BATCH_CODE",
        spec_name: "BATCH_CODE",
        canonical_name: "BATCH_CODE",
        kind: FieldKind::Text,
        length: 8,
        allowed: None,
        key_role: KeyRole::Batch,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(15),
        issue_id: Some("MTRIX-007"),
    },
    FieldSpec {
        seq: 5,
        source_name: "QUANTITY_STOCK",
        spec_name: "QUANTITY_STOCK",
        canonical_name: "QUANTITY_STOCK",
        kind: FieldKind::Number,
        length: 8,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(15),
        issue_id: None,
    },
    FieldSpec {
        seq: 6,
        source_name: "COUNTRY_ISO",
        spec_name: "COUNTRY_ISO",
        canonical_name: "COUNTRY_ISO",
        kind: FieldKind::Text,
        length: 3,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(15),
        issue_id: None,
    },
];

/// Sales force master - 10 columns (manual p.22).
pub const ALTHFV_COLUMNS: &[FieldSpec] = &[
    FieldSpec {
        seq: 1,
        source_name: "VENDOR_ID",
        spec_name: "VENDOR_ID",
        canonical_name: "VENDOR_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Vendor,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 2,
        source_name: "DIST_ID",
        spec_name: "DIST_ID",
        canonical_name: "DIST_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Dist,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 3,
        source_name: "CUSTOMER_ID",
        spec_name: "CUSTOMER ID",
        canonical_name: "CUSTOMER_ID",
        kind: FieldKind::Text,
        length: 18,
        allowed: None,
        key_role: KeyRole::Customer,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 4,
        source_name: "MANAGER_CODE",
        spec_name: "MANAGER_ID",
        canonical_name: "MANAGER_ID",
        kind: FieldKind::Text,
        length: 13,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 5,
        source_name: "MANAGER_NAME",
        spec_name: "MANAGER_NAME",
        canonical_name: "MANAGER_NAME",
        kind: FieldKind::Text,
        length: 50,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 6,
        source_name: "SUPERVISOR_ID",
        spec_name: "SUPERVISOR_ID",
        canonical_name: "SUPERVISOR_ID",
        kind: FieldKind::Text,
        length: 13,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 7,
        source_name: "SUPERVISOR_NAME",
        spec_name: "SUPERVISOR_NAME",
        canonical_name: "SUPERVISOR_NAME",
        kind: FieldKind::Text,
        length: 50,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 8,
        source_name: "SALESPERSON_CODE",
        spec_name: "SALESPERSON_ID",
        canonical_name: "SALESPERSON_ID",
        kind: FieldKind::Text,
        length: 20,
        allowed: None,
        key_role: KeyRole::Salesperson,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(22),
        issue_id: Some("MTRIX-006"),
    },
    FieldSpec {
        seq: 9,
        source_name: "SALESPERSON_NAME",
        spec_name: "SALESPERSON_NAME",
        canonical_name: "SALESPERSON_NAME",
        kind: FieldKind::Text,
        length: 50,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(22),
        issue_id: None,
    },
    FieldSpec {
        seq: 10,
        source_name: "COUNTRY",
        spec_name: "COUNTRY_ISO",
        canonical_name: "COUNTRY_ISO",
        kind: FieldKind::Text,
        length: 3,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(22),
        issue_id: None,
    },
];

/// Product master - 12 columns (manual p.25-26).
pub const ALTHPD_COLUMNS: &[FieldSpec] = &[
    FieldSpec {
        seq: 1,
        source_name: "TRANSACTION_DATE",
        spec_name: "TRANSACTION_DATE",
        canonical_name: "TRANSACTION_DATE",
        kind: FieldKind::Date,
        length: 8,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(25),
        issue_id: Some("MTRIX-011"),
    },
    FieldSpec {
        seq: 2,
        source_name: "DIST_ID",
        spec_name: "DIST_ID",
        canonical_name: "DIST_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Dist,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: None,
    },
    FieldSpec {
        seq: 3,
        source_name: "VENDOR_ID",
        spec_name: "VENDOR_ID",
        canonical_name: "VENDOR_ID",
        kind: FieldKind::Text,
        length: 18,
        allowed: None,
        key_role: KeyRole::Vendor,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(25),
        issue_id: Some("MTRIX-020"),
    },
    FieldSpec {
        seq: 4,
        source_name: "VENDOR_COMPANY_NAME",
        spec_name: "VENDOR_COMPANY_NAME",
        canonical_name: "VENDOR_COMPANY_NAME",
        kind: FieldKind::Text,
        length: 40,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: None,
    },
    FieldSpec {
        seq: 5,
        source_name: "PRODUCT_CODE",
        spec_name: "PRODUCT_CODE",
        canonical_name: "PRODUCT_CODE",
        kind: FieldKind::Text,
        length: 40,
        allowed: None,
        key_role: KeyRole::Product,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: None,
    },
    FieldSpec {
        seq: 6,
        source_name: "PACKAGE_TYPE",
        spec_name: "PACKAGE_TYPE",
        canonical_name: "PACKAGE_TYPE",
        kind: FieldKind::Code,
        length: 1,
        allowed: Some(&["0", "1"]),
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: Some("MTRIX-013"),
    },
    FieldSpec {
        seq: 7,
        source_name: "BARCODE",
        spec_name: "BARCODE",
        canonical_name: "BARCODE",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::ProductBarcode,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(25),
        issue_id: Some("MTRIX-012"),
    },
    FieldSpec {
        seq: 8,
        source_name: "BARCODE_TYPE",
        spec_name: "BARCODE_TYPE",
        canonical_name: "BARCODE_TYPE",
        kind: FieldKind::Code,
        length: 1,
        allowed: Some(&["1", "2", "3"]),
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: Some("MTRIX-013"),
    },
    FieldSpec {
        seq: 9,
        source_name: "DESCRIPTION",
        spec_name: "DESCRIPTION",
        canonical_name: "DESCRIPTION",
        kind: FieldKind::Text,
        length: 30,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: None,
    },
    FieldSpec {
        seq: 10,
        source_name: "PRODUCT_DIV",
        spec_name: "PRODUCT_DIV",
        canonical_name: "PRODUCT_DIV",
        kind: FieldKind::Text,
        length: 20,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: None,
    },
    FieldSpec {
        seq: 11,
        source_name: "PRODUCT_STATUS",
        spec_name: "PRODUCT_STATUS",
        canonical_name: "PRODUCT_STATUS",
        kind: FieldKind::Code,
        length: 1,
        allowed: Some(&["A", "I"]),
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(25),
        issue_id: Some("MTRIX-013"),
    },
    FieldSpec {
        seq: 12,
        source_name: "COUNTRY_ISO",
        spec_name: "COUNTRY_ISO",
        canonical_name: "COUNTRY_ISO",
        kind: FieldKind::Text,
        length: 3,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(26),
        issue_id: None,
    },
];

/// Sales / sellout transactions - 12 columns (manual p.11).
pub const ALTHVD_COLUMNS: &[FieldSpec] = &[
    FieldSpec {
        seq: 1,
        source_name: "VENDOR_ID",
        spec_name: "VENDOR_ID",
        canonical_name: "VENDOR_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Vendor,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 2,
        source_name: "DIST_ID",
        spec_name: "DIST_ID",
        canonical_name: "DIST_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Dist,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 3,
        source_name: "CUSTOMER_ID",
        spec_name: "CUSTOMER_ID",
        canonical_name: "CUSTOMER_ID",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Customer,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 4,
        source_name: "DATE",
        spec_name: "DATE",
        canonical_name: "DATE",
        kind: FieldKind::Date,
        length: 8,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 5,
        source_name: "DOCUMENT_NUMBER",
        spec_name: "DOCUMENT_NUMBER",
        canonical_name: "DOCUMENT_NUMBER",
        kind: FieldKind::Text,
        length: 20,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 6,
        source_name: "BATCH_CODE",
        spec_name: "BATCH_CODE",
        canonical_name: "BATCH_CODE",
        kind: FieldKind::Text,
        length: 14,
        allowed: None,
        key_role: KeyRole::Batch,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 7,
        source_name: "QUANTITY",
        spec_name: "QUANTITY",
        canonical_name: "QUANTITY",
        kind: FieldKind::Number,
        length: 8,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 8,
        source_name: "SALES_PRICE",
        spec_name: "SALES_PRICE",
        canonical_name: "SALES_PRICE",
        kind: FieldKind::Number,
        length: 10,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(11),
        issue_id: Some("MTRIX-005"),
    },
    FieldSpec {
        seq: 9,
        source_name: "SALESPERSON_CODE",
        spec_name: "SALESPERSON_CODE",
        canonical_name: "SALESPERSON_CODE",
        kind: FieldKind::Text,
        length: 20,
        allowed: None,
        key_role: KeyRole::Salesperson,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::ObservedOnly,
        manual_page: Some(11),
        issue_id: Some("MTRIX-006"),
    },
    FieldSpec {
        seq: 10,
        source_name: "DOCUMENT_TYPE",
        spec_name: "DOCUMENT_TYPE",
        canonical_name: "DOCUMENT_TYPE",
        kind: FieldKind::Code,
        length: 1,
        allowed: Some(&["N", "B", "T", "E", "F"]),
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Confirmed,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 11,
        source_name: "ZIP_CODE",
        spec_name: "ZIP CODE",
        canonical_name: "ZIP_CODE",
        kind: FieldKind::Text,
        length: 8,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(11),
        issue_id: None,
    },
    FieldSpec {
        seq: 12,
        source_name: "COUNTRY_ISO",
        spec_name: "COUNTRY ISO",
        canonical_name: "COUNTRY_ISO",
        kind: FieldKind::Text,
        length: 3,
        allowed: None,
        key_role: KeyRole::None,
        tier: SourceTier::ManualL1,
        status: VerifyStatus::Deviation,
        manual_page: Some(11),
        issue_id: None,
    },
];

/// The complete MTRIX contract: five files, 56 columns.
pub static MTRIX_SPEC: &[FileSpec] = &[
    FileSpec {
        prefix: "ALTHCI",
        role: FileRole::Customer,
        columns: ALTHCI_COLUMNS,
    },
    FileSpec {
        prefix: "ALTHES",
        role: FileRole::Inventory,
        columns: ALTHES_COLUMNS,
    },
    FileSpec {
        prefix: "ALTHFV",
        role: FileRole::SalesForce,
        columns: ALTHFV_COLUMNS,
    },
    FileSpec {
        prefix: "ALTHPD",
        role: FileRole::Product,
        columns: ALTHPD_COLUMNS,
    },
    FileSpec {
        prefix: "ALTHVD",
        role: FileRole::Sales,
        columns: ALTHVD_COLUMNS,
    },
];

/// Total number of declared columns across the five files.
pub const TOTAL_FIELDS: usize = 56;

/// Look up a file specification by prefix.
pub fn spec_by_prefix(prefix: &str) -> Option<&'static FileSpec> {
    MTRIX_SPEC
        .iter()
        .find(|f| f.prefix.eq_ignore_ascii_case(prefix))
}

/// Look up a file specification by business role.
pub fn spec_by_role(role: FileRole) -> &'static FileSpec {
    MTRIX_SPEC
        .iter()
        .find(|f| f.role == role)
        .expect("every file role is declared in MTRIX_SPEC")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_five_files_declared() {
        assert_eq!(MTRIX_SPEC.len(), 5);
    }

    #[test]
    fn test_column_counts_match_manual() {
        // Sales 12, Inventory 6, Customer 16, Sales Force 10, Product 12
        assert_eq!(spec_by_role(FileRole::Sales).column_count(), 12);
        assert_eq!(spec_by_role(FileRole::Inventory).column_count(), 6);
        assert_eq!(spec_by_role(FileRole::Customer).column_count(), 16);
        assert_eq!(spec_by_role(FileRole::SalesForce).column_count(), 10);
        assert_eq!(spec_by_role(FileRole::Product).column_count(), 12);
    }

    #[test]
    fn test_total_field_count_is_56() {
        let total: usize = MTRIX_SPEC.iter().map(|f| f.column_count()).sum();
        assert_eq!(total, TOTAL_FIELDS);
        assert_eq!(total, 56);
    }

    #[test]
    fn test_sequence_numbers_are_contiguous() {
        for file in MTRIX_SPEC {
            for (i, col) in file.columns.iter().enumerate() {
                assert_eq!(
                    col.seq as usize,
                    i + 1,
                    "{} column {}",
                    file.prefix,
                    col.seq
                );
            }
        }
    }

    #[test]
    fn test_customer_has_duplicate_customer_id_kept_separate() {
        let ci = spec_by_role(FileRole::Customer);
        let primary = ci
            .index_of_role(KeyRole::Customer)
            .expect("primary customer key");
        let secondary = ci
            .index_of_role(KeyRole::CustomerSecondary)
            .expect("secondary customer key");
        assert_ne!(primary, secondary);
        // Both keep the source wording; only the canonical alias differs (MTRIX-009).
        assert_eq!(ci.columns[primary].source_name, "CUSTOMER_ID");
        assert_eq!(ci.columns[secondary].source_name, "CUSTOMER_ID");
        assert_eq!(ci.columns[secondary].canonical_name, "CUSTOMER_ID_2");
    }

    #[test]
    fn test_sales_force_retains_supervisor_columns() {
        let fv = spec_by_role(FileRole::SalesForce);
        let names: Vec<&str> = fv.columns.iter().map(|c| c.source_name).collect();
        assert!(names.contains(&"SUPERVISOR_ID"));
        assert!(names.contains(&"SUPERVISOR_NAME"));
        assert!(names.contains(&"MANAGER_CODE"));
        assert!(names.contains(&"SALESPERSON_CODE"));
    }

    #[test]
    fn test_inventory_layout_matches_observed_bytes() {
        let es = spec_by_role(FileRole::Inventory);
        assert_eq!(
            es.expected_header(),
            "INVENTORY_DATE;VENDOR_ID;DIST_ID;BATCH_CODE;QUANTITY_STOCK;COUNTRY_ISO"
        );
        assert_eq!(es.index_of("BATCH_CODE"), Some(3));
        assert_eq!(es.index_of("QUANTITY_STOCK"), Some(4));
    }

    #[test]
    fn test_sales_layout_matches_observed_bytes() {
        let vd = spec_by_role(FileRole::Sales);
        assert_eq!(vd.index_of("CUSTOMER_ID"), Some(2));
        assert_eq!(vd.index_of("BATCH_CODE"), Some(5));
        assert_eq!(vd.index_of("DOCUMENT_TYPE"), Some(9));
        assert_eq!(vd.index_of("COUNTRY_ISO"), Some(11));
    }

    #[test]
    fn test_product_layout_matches_observed_bytes() {
        let pd = spec_by_role(FileRole::Product);
        assert_eq!(pd.index_of("DIST_ID"), Some(1));
        assert_eq!(pd.index_of("VENDOR_ID"), Some(2));
        assert_eq!(pd.index_of("BARCODE"), Some(6));
    }

    #[test]
    fn test_allowed_values_are_declared() {
        let pd = spec_by_role(FileRole::Product);
        assert_eq!(
            pd.columns[pd.index_of("PACKAGE_TYPE").unwrap()].allowed,
            Some(&["0", "1"][..])
        );
        assert_eq!(
            pd.columns[pd.index_of("BARCODE_TYPE").unwrap()].allowed,
            Some(&["1", "2", "3"][..])
        );
        assert_eq!(
            pd.columns[pd.index_of("PRODUCT_STATUS").unwrap()].allowed,
            Some(&["A", "I"][..])
        );
        let vd = spec_by_role(FileRole::Sales);
        assert_eq!(
            vd.columns[vd.index_of("DOCUMENT_TYPE").unwrap()].allowed,
            Some(&["N", "B", "T", "E", "F"][..])
        );
    }

    #[test]
    fn test_deviations_are_flagged_and_never_dropped() {
        let mut seen = HashSet::new();
        for file in MTRIX_SPEC {
            for col in file.columns {
                if col.is_deviation() {
                    seen.insert((file.prefix, col.source_name, col.spec_name));
                    assert_ne!(
                        col.status,
                        VerifyStatus::Unverified,
                        "a deviation must still be sourced"
                    );
                }
            }
        }
        // Eight wording deviations between the manual and the delivered bytes.
        assert_eq!(seen.len(), 8, "unexpected deviation set: {seen:?}");
    }

    #[test]
    fn test_every_field_declares_provenance() {
        for file in MTRIX_SPEC {
            for col in file.columns {
                assert!(!col.provenance().is_empty());
                assert!(
                    col.manual_page.is_some(),
                    "{} {}",
                    file.prefix,
                    col.source_name
                );
            }
        }
    }
}
