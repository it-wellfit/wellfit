#![forbid(unsafe_code)]

//! Wellfit Core - Master Data Management Hub (MDMHub) & Dynamic Catalog
//!
//! Canonical master data for the Alpargatas MTRIX interface: the product
//! master, the ERP customer (channel) map and the store type vocabulary.
//!
//! # Provenance (R-D, ADR-0001)
//!
//! Every constant carries the source it came from. Nothing is invented:
//!
//! | Data | Source | Mark |
//! |---|---|---|
//! | Product codes `8`, `16`, `19`, `20` and their EAN13 | ALTHPD product master, delivered bytes | `confirmed` |
//! | Channel codes `995A`..`995D` | ALTHCI customer master, delivered bytes | `confirmed` |
//! | Store type alias `onlinne` -> `Online` | ALTHCI row `995D` | `confirmed` |
//! | Channel codes `995E`..`995G` | none | `unverified`, see [`UNVERIFIED_CHANNELS`] |
//!
//! # Production use
//!
//! [`MdmCatalog::demo_bootstrap`] is a **demo fallback**. Production code must
//! replace it with [`MdmCatalog::register_from_mtrix_product`], which resolves
//! the columns of the product master by name through
//! [`crate::contract::ColumnContract`] and therefore never guesses a column
//! position.
//!
//! # Vendor identifier
//!
//! The original manual (p.10, p.17, p.23) defines `VENDOR_ID` as the first
//! **two** letters of the ISO country code followed by the manufacturer code,
//! while the delivered files carry the manufacturer code alone or with a
//! **three** letter prefix (`USA6063343900011222`). The former
//! `normalize_vendor_id` forced a `USA` prefix onto values that carry none; it
//! is replaced by [`split_vendor_country_prefix`] and
//! [`join_vendor_country_prefix`], which never invent a country code.

use crate::contract::ColumnContract;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Unit of measure of the product master.
///
/// The MTRIX product master carries no unit column, so the loader fixes the
/// unit of measure to the pair, which is how footwear is sold.
pub const UOM_PAIR: &str = "PAIR";

/// `BARCODE_TYPE` value of an EAN13 barcode (manual p.37).
pub const BARCODE_TYPE_EAN13: &str = "1";

/// `BARCODE_TYPE` value of a DUN14 barcode (manual p.37).
pub const BARCODE_TYPE_DUN14: &str = "2";

/// `BARCODE_TYPE` value of any other barcode (manual p.37).
pub const BARCODE_TYPE_OTHER: &str = "3";

/// ERP customer codes confirmed by the authoritative ALTHCI customer master.
///
/// The delivered file pairs `CUSTOMER_ID` with `CUSTOMER_NAME`:
/// `995A`=JDID, `995B`=Lazada, `995C`=Shopee, `995D`=Tiktok Shop.
pub const CONFIRMED_CHANNELS: &[(&str, &str)] = &[
    ("JDID", "995A"),
    ("LAZADA", "995B"),
    ("SHOPEE", "995C"),
    ("TIKTOK SHOP", "995D"),
];

/// ERP customer codes that **no** authoritative source confirms.
///
/// The codes `995E` (MOMO), `995F` (PCHOME) and `995G` (COUPANG) appear in
/// earlier drafts of this module but in none of the delivered files, and the
/// ALTHCI customer master only carries `995A`..`995D`. They are kept as
/// `unverified` placeholders and **must be confirmed with the counterparty**
/// before they are used to route an order to a customer code.
pub const UNVERIFIED_CHANNELS: &[(&str, &str)] =
    &[("MOMO", "995E"), ("PCHOME", "995F"), ("COUPANG", "995G")];

/// The ERP customer code an unknown channel is routed to.
///
/// Legacy behaviour kept for compatibility: [`MdmCatalog::resolve_channel`]
/// routes an unknown channel to Shopee instead of failing. New code must use
/// [`MdmCatalog::resolve_channel_exact`], which reports the unknown channel
/// instead of guessing.
pub const DEFAULT_CHANNEL_FALLBACK: &str = "995C";

/// Extra spellings that resolve to a confirmed channel code.
///
/// These are the names this project used before the codes were confirmed
/// against the delivered ALTHCI; they map to the same ERP customer code.
const CONFIRMED_CHANNEL_ALIASES: &[(&str, &str)] = &[
    ("SHOPEE_TW", "995C"),
    ("蝦皮", "995C"),
    ("TIKTOK", "995D"),
    ("TIKTOK_SHOP", "995D"),
];

/// Store type vocabulary.
///
/// The original manual (p.23) calls the column `TYPE OF SHOP` and defines it
/// as the "segment code which the PoS belongs to", but the delivered ALTHCI
/// carries a **name** instead of a code (`Online`, `Onlinne`). The alias table
/// therefore normalises the names the file actually contains; the segment code
/// interpretation stays unconfirmed.
const STORE_TYPE_ALIASES: &[(&str, &str)] = &[
    ("online", "Online"),
    ("onlinne", "Online"),
    ("on-line", "Online"),
    ("retail", "Retail"),
    ("pos", "Retail"),
    ("physical", "Retail"),
];

/// The four products of the authoritative ALTHPD master, as delivered.
///
/// `seller_sku` values are the demo labels the demo order feed uses; they are
/// identifiers, not product facts. The product codes and the EAN13 codes are
/// the authoritative ones.
///
/// | `seller_sku` | product code | EAN13 |
/// |---|---|---|
/// | `HAV-TOP-BLK-39` | `8` | `7898656600087` |
/// | `HAV-TOP-NVY-41` | `16` | `7898656600162` |
/// | `HAV-SLM-GLD-37` | `19` | `7898656600193` |
/// | `HAV-BRZ-WHT-43` | `20` | `7898656600209` |
const DEMO_SKUS: &[(&str, &str, &str)] = &[
    ("HAV-TOP-BLK-39", "8", "7898656600087"),
    ("HAV-TOP-NVY-41", "16", "7898656600162"),
    ("HAV-SLM-GLD-37", "19", "7898656600193"),
    ("HAV-BRZ-WHT-43", "20", "7898656600209"),
];

/// Master data of one product.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkuMetadata {
    /// SKU the sales channel orders under.
    pub seller_sku: String,
    /// Item number of the ERP.
    pub item_no: String,
    /// Barcode of the product (EAN13 or DUN14).
    pub ean13: String,
    /// Product key of the MTRIX product master.
    pub product_code: String,
    /// Product description.
    pub name: String,
    /// Unit of measure.
    pub uom: String,
    /// Cumulative balance of the product across all batches.
    ///
    /// This is updated by [`MdmCatalog::register_cumulative`] whenever a batch
    /// is processed. It represents the running total of quantity for this
    /// product across all batches that have been processed through the system.
    #[serde(default)]
    pub cumulative_balance: i64,
}

/// Dynamic, non-ossified master data catalogue.
///
/// The catalogue holds the product master, the channel map and the store type
/// vocabulary. Every part can be replaced at runtime; nothing in it is a
/// compilation constant except the demo fallback.
#[derive(Debug)]
pub struct MdmCatalog {
    skus_by_code: Arc<RwLock<HashMap<String, SkuMetadata>>>,
    skus_by_ean13: Arc<RwLock<HashMap<String, SkuMetadata>>>,
    channels: Arc<RwLock<HashMap<String, String>>>,
    store_type_aliases: Arc<RwLock<HashMap<String, String>>>,
    conn: Mutex<Connection>,
}

impl Default for MdmCatalog {
    /// A catalogue pre-populated with the demo fallback data.
    fn default() -> Self {
        Self::demo_bootstrap()
    }
}

impl MdmCatalog {
    /// Create an empty catalogue registry.
    pub fn new() -> Self {
        let conn = Connection::open_in_memory().expect("in-memory SQLite must open");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cumulative_balance (
                customer_id TEXT PRIMARY KEY,
                balance_cents INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            )",
        )
        .expect("the cumulative_balance table must be created");
        Self {
            skus_by_code: Arc::new(RwLock::new(HashMap::new())),
            skus_by_ean13: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            store_type_aliases: Arc::new(RwLock::new(HashMap::new())),
            conn: Mutex::new(conn),
        }
    }

    /// Create the **demo fallback** catalogue.
    ///
    /// The catalogue carries the four products of the authoritative ALTHPD
    /// master (product codes and EAN13 confirmed against the delivered bytes)
    /// and the channel map of the ALTHCI customer master.
    ///
    /// This is a demo fallback, not production master data. A production
    /// deployment must replace it with
    /// [`MdmCatalog::register_from_mtrix_product`], which loads the real
    /// product master; the demo entries are placeholders whose descriptions
    /// are not sourced (the delivered `DESCRIPTION` column is empty).
    ///
    /// The channel map contains the codes of [`UNVERIFIED_CHANNELS`]; they are
    /// marked as unverified and must be confirmed with the counterparty.
    pub fn demo_bootstrap() -> Self {
        let catalog = Self::new();

        for (seller_sku, product_code, ean13) in DEMO_SKUS {
            catalog.register_sku(SkuMetadata {
                seller_sku: (*seller_sku).to_string(),
                item_no: (*product_code).to_string(),
                ean13: (*ean13).to_string(),
                product_code: (*product_code).to_string(),
                name: format!("Demo product {product_code}"),
                uom: UOM_PAIR.to_string(),
                cumulative_balance: 0,
            });
        }

        for (channel, code) in CONFIRMED_CHANNELS {
            catalog.register_channel(channel, code);
        }
        for (channel, code) in CONFIRMED_CHANNEL_ALIASES {
            catalog.register_channel(channel, code);
        }
        for (channel, code) in UNVERIFIED_CHANNELS {
            catalog.register_channel(channel, code);
        }

        for (alias, canonical) in STORE_TYPE_ALIASES {
            catalog.register_store_type_alias(alias, canonical);
        }

        catalog
    }

    /// Register a SKU dynamically at runtime.
    ///
    /// The barcode is normalised with [`normalize_barcode`] before it is
    /// stored and indexed, so a lookup succeeds whether the caller passes the
    /// delivered value (`7898656600087 `) or the clean one.
    pub fn register_sku(&self, sku: SkuMetadata) {
        let clean_barcode = normalize_barcode(&sku.ean13);
        let mut sku = sku;
        sku.ean13 = clean_barcode;

        let mut by_code = self.skus_by_code.write().unwrap();
        let mut by_ean = self.skus_by_ean13.write().unwrap();

        if !sku.ean13.is_empty() {
            by_ean.insert(sku.ean13.clone(), sku.clone());
        }
        if !sku.seller_sku.is_empty() {
            by_code.insert(sku.seller_sku.clone(), sku.clone());
        }
        if !sku.product_code.is_empty() {
            by_code.insert(sku.product_code.clone(), sku);
        }
    }

    /// Register a channel mapping dynamically.
    pub fn register_channel(&self, channel_alias: &str, erp_code: &str) {
        let mut channels = self.channels.write().unwrap();
        channels.insert(
            channel_alias.trim().to_uppercase(),
            erp_code.trim().to_string(),
        );
    }

    /// Register a store type normalization alias.
    pub fn register_store_type_alias(&self, alias: &str, canonical: &str) {
        let mut aliases = self.store_type_aliases.write().unwrap();
        aliases.insert(alias.trim().to_lowercase(), canonical.trim().to_string());
    }

    /// Lookup SKU metadata by SKU or product code.
    pub fn lookup_sku(&self, sku_key: &str) -> Option<SkuMetadata> {
        let clean = sku_key.trim();
        self.skus_by_code.read().unwrap().get(clean).cloned()
    }

    /// Reverse lookup SKU metadata by barcode.
    pub fn lookup_by_ean13(&self, ean13: &str) -> Option<SkuMetadata> {
        let clean = normalize_barcode(ean13);
        if clean.is_empty() {
            return None;
        }
        self.skus_by_ean13.read().unwrap().get(&clean).cloned()
    }

    /// Resolve a channel alias to its ERP customer code, without guessing.
    ///
    /// Returns `None` when the alias is unknown, so a caller can refuse the
    /// order instead of routing it to the wrong customer code.
    pub fn resolve_channel_exact(&self, channel_ref: &str) -> Option<String> {
        let upper = channel_ref.trim().to_uppercase();
        self.channels.read().unwrap().get(&upper).cloned()
    }

    /// Resolve a channel alias to its ERP customer code, falling back to Shopee.
    ///
    /// The fallback is legacy behaviour kept for compatibility: an unknown
    /// channel is routed to [`DEFAULT_CHANNEL_FALLBACK`] instead of failing.
    /// That is a silent mis-route, so new code must use
    /// [`MdmCatalog::resolve_channel_exact`].
    pub fn resolve_channel(&self, channel_ref: &str) -> String {
        self.resolve_channel_exact(channel_ref)
            .unwrap_or_else(|| DEFAULT_CHANNEL_FALLBACK.to_string())
    }

    /// Normalize store type using registered alias rules or titlecase fallback.
    ///
    /// The manual calls the column `TYPE OF SHOP` (segment code, p.23) while
    /// the delivered data carries a name; see [`STORE_TYPE_ALIASES`].
    ///
    /// An empty value stays empty. The mandatory-ness of the column is not
    /// defined by the manual (MTRIX-003) and the code table is unknown
    /// (MTRIX-029, "show the source code"), so inventing a value would put a
    /// fabricated store type into the delivered data.
    pub fn normalize_store_type(&self, store_type: &str) -> String {
        let clean = store_type.trim();
        let lower = clean.to_lowercase();
        let aliases = self.store_type_aliases.read().unwrap();
        if let Some(canonical) = aliases.get(&lower) {
            return canonical.clone();
        }
        if clean.is_empty() {
            return String::new();
        }
        let mut chars = clean.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Load SKUs dynamically from JSON array string.
    pub fn load_from_json(&self, json_str: &str) -> Result<usize, String> {
        let skus: Vec<SkuMetadata> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse catalog JSON: {e}"))?;
        let count = skus.len();
        for sku in skus {
            self.register_sku(sku);
        }
        Ok(count)
    }

    /// Load SKUs dynamically from structured CSV string.
    ///
    /// Fixed layout, declared by this function: `seller_sku,item_no,ean13,
    /// product_code,name,uom`. This is **not** the MTRIX file layout; use
    /// [`MdmCatalog::register_from_mtrix_product`] for the product master,
    /// which resolves its columns by name.
    pub fn load_from_csv(&self, csv_str: &str) -> Result<usize, String> {
        let mut count = 0;
        for line in csv_str.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 6 {
                let sku = SkuMetadata {
                    seller_sku: parts[0].trim().to_string(),
                    item_no: parts[1].trim().to_string(),
                    ean13: parts[2].trim().to_string(),
                    product_code: parts[3].trim().to_string(),
                    name: parts[4].trim().to_string(),
                    uom: parts[5].trim().to_string(),
                    cumulative_balance: 0,
                };
                self.register_sku(sku);
                count += 1;
            }
        }
        Ok(count)
    }

    /// Register every product of an MTRIX product master (ALTHPD) file.
    ///
    /// The header is resolved **by name** with [`mtrix_product_contract`]: a
    /// missing required column is an error and no column position is ever
    /// assumed. Each row becomes one [`SkuMetadata`]:
    ///
    /// | Field | Source column |
    /// |---|---|
    /// | `seller_sku`, `item_no`, `product_code` | `PRODUCT_CODE` |
    /// | `ean13` | `BARCODE` |
    /// | `name` | `DESCRIPTION` |
    /// | `uom` | fixed [`UOM_PAIR`] |
    ///
    /// The optional columns are validated against the original manual
    /// (p.37, p.39): `PACKAGE_TYPE` in `0|1`, `BARCODE_TYPE` in `1|2|3` with
    /// the barcode length implied by the type (`1` EAN13 = 13 digits, `2`
    /// DUN14 = 14 digits), `PRODUCT_STATUS` in `A|I`, `COUNTRY_ISO` as a three
    /// letter code, `VENDOR_ID` as an optional two or three letter country
    /// prefix followed by the manufacturer code and `DIST_ID` as an unmasked
    /// RUC.
    ///
    /// Every row is registered, including the products whose `PRODUCT_STATUS`
    /// is `I` (inactive): the catalogue reports what the file carries, and the
    /// caller decides whether an inactive product may be sold.
    ///
    /// Returns the number of registered products.
    pub fn register_from_mtrix_product(&self, csv: &str) -> Result<usize, String> {
        let mut lines = csv
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty());

        let (header_line, header) = lines
            .next()
            .ok_or_else(|| "MTRIX product master: the file is empty".to_string())?;

        let separator = detect_separator(header).ok_or_else(|| {
            format!("MTRIX product master line {header_line}: no delimiter found")
        })?;

        let header_names = crate::contract::split_header(header, separator);
        let name_refs: Vec<&str> = header_names.iter().map(|name| name.as_str()).collect();
        let map = mtrix_product_contract().resolve(&name_refs)?;

        let idx_product_code = map.index("PRODUCT_CODE")?;
        let idx_barcode = map.index("BARCODE")?;
        let idx_description = map.index("DESCRIPTION")?;
        let idx_package_type = map.optional_index("PACKAGE_TYPE");
        let idx_barcode_type = map.optional_index("BARCODE_TYPE");
        let idx_product_status = map.optional_index("PRODUCT_STATUS");
        let idx_country_iso = map.optional_index("COUNTRY_ISO");
        let idx_vendor_id = map.optional_index("VENDOR_ID");
        let idx_dist_id = map.optional_index("DIST_ID");

        let mut count = 0usize;
        for (line_no, line) in lines {
            let values: Vec<&str> = line.split(separator).map(str::trim).collect();

            let product_code = cell(&values, idx_product_code).to_string();
            if product_code.is_empty() {
                return Err(format!(
                    "MTRIX product master line {line_no}: PRODUCT_CODE is empty"
                ));
            }

            let barcode = normalize_barcode(cell(&values, idx_barcode));
            if barcode.is_empty() {
                return Err(format!(
                    "MTRIX product master line {line_no}: BARCODE is empty"
                ));
            }

            if let Some(index) = idx_package_type {
                let value = cell(&values, index);
                if !value.is_empty() && value != "0" && value != "1" {
                    return Err(format!(
                        "MTRIX product master line {line_no}: PACKAGE_TYPE '{value}' is neither 0 nor 1"
                    ));
                }
            }

            if let Some(index) = idx_barcode_type {
                let value = cell(&values, index);
                if !value.is_empty() {
                    validate_barcode(&barcode, value).map_err(|reason| {
                        format!("MTRIX product master line {line_no}: {reason}")
                    })?;
                }
            }

            if let Some(index) = idx_product_status {
                let value = cell(&values, index);
                if !value.is_empty() && value != "A" && value != "I" {
                    return Err(format!(
                        "MTRIX product master line {line_no}: PRODUCT_STATUS '{value}' is neither A nor I"
                    ));
                }
            }

            if let Some(index) = idx_country_iso {
                let value = cell(&values, index);
                if !value.is_empty()
                    && (value.len() != 3 || !value.chars().all(|c| c.is_ascii_alphabetic()))
                {
                    return Err(format!(
                        "MTRIX product master line {line_no}: COUNTRY_ISO '{value}' is not an ISO 3 letter code"
                    ));
                }
            }

            if let Some(index) = idx_vendor_id {
                let value = cell(&values, index);
                if !value.is_empty() {
                    let (code, _prefix) = split_vendor_country_prefix(value);
                    if code.is_empty() || !code.chars().all(|c| c.is_ascii_digit()) {
                        return Err(format!(
                            "MTRIX product master line {line_no}: VENDOR_ID '{value}' is not a country prefix followed by the manufacturer code"
                        ));
                    }
                }
            }

            if let Some(index) = idx_dist_id {
                let value = cell(&values, index);
                if !value.is_empty() {
                    if !value.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        return Err(format!(
                            "MTRIX product master line {line_no}: DIST_ID '{value}' carries characters other than digits and dot masks"
                        ));
                    }
                    if normalize_dist_id(value).is_empty() {
                        return Err(format!(
                            "MTRIX product master line {line_no}: DIST_ID '{value}' is empty once the masks are removed"
                        ));
                    }
                }
            }

            let sku = SkuMetadata {
                seller_sku: product_code.clone(),
                item_no: product_code.clone(),
                ean13: barcode,
                product_code,
                name: cell(&values, idx_description).to_string(),
                uom: UOM_PAIR.to_string(),
                cumulative_balance: 0,
            };
            self.register_sku(sku);
            count += 1;
        }

        Ok(count)
    }

    /// Register a cumulative balance change for a customer/product.
    ///
    /// The amount is added to any existing balance for the same customer.
    /// All amounts are tracked in integer cents.
    pub fn register_cumulative(
        &self,
        _batch_id: &str,
        customer_id: &str,
        amount_cents: i64,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO cumulative_balance (customer_id, balance_cents, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(customer_id) DO UPDATE SET
                balance_cents = balance_cents + ?2,
                updated_at = ?3
            "#,
            params![customer_id, amount_cents, now],
        )?;
        Ok(())
    }

    /// Get the cumulative balance for a customer.
    ///
    /// Returns `None` if no balance has been recorded for the customer.
    pub fn get_cumulative_balance(
        &self,
        customer_id: &str,
    ) -> Result<Option<i64>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT balance_cents FROM cumulative_balance WHERE customer_id = ?1")?;
        let mut rows = stmt.query(params![customer_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }
}

/// The column contract of the MTRIX product master (ALTHPD).
///
/// Required columns: `PRODUCT_CODE`, `BARCODE`, `DESCRIPTION`.
/// Optional columns: `PACKAGE_TYPE`, `BARCODE_TYPE`, `PRODUCT_STATUS`,
/// `COUNTRY_ISO`, `VENDOR_ID`, `DIST_ID`.
pub fn mtrix_product_contract() -> ColumnContract {
    ColumnContract::new(
        "MTRIX product master (ALTHPD)",
        &["PRODUCT_CODE", "BARCODE", "DESCRIPTION"],
        &[
            "PACKAGE_TYPE",
            "BARCODE_TYPE",
            "PRODUCT_STATUS",
            "COUNTRY_ISO",
            "VENDOR_ID",
            "DIST_ID",
        ],
    )
}

/// Trim a barcode without touching its content.
///
/// The delivered product master carries the barcode with a trailing space
/// (`7898656600087 `), so surrounding whitespace and trailing `;` delimiters
/// are removed. Nothing else is touched: leading zeros survive and an
/// identifier that is not numeric stays as it is. Stripping every non digit
/// would corrupt both.
pub fn normalize_barcode(barcode: &str) -> String {
    barcode.trim().trim_end_matches(';').trim().to_string()
}

/// Whether a barcode is a well formed EAN13 (13 ASCII digits).
pub fn is_ean13(barcode: &str) -> bool {
    let clean = normalize_barcode(barcode);
    clean.len() == 13 && clean.chars().all(|c| c.is_ascii_digit())
}

/// Whether a barcode is a well formed DUN14 (14 ASCII digits).
pub fn is_dun14(barcode: &str) -> bool {
    let clean = normalize_barcode(barcode);
    clean.len() == 14 && clean.chars().all(|c| c.is_ascii_digit())
}

/// Validate a barcode against the barcode type declared in the product master.
///
/// `BARCODE_TYPE` is `1` for EAN13 (13 digits), `2` for DUN14 (14 digits) and
/// `3` for any other code (manual p.37, validation rules p.39).
pub fn validate_barcode(barcode: &str, barcode_type: &str) -> Result<(), String> {
    let clean = normalize_barcode(barcode);
    match barcode_type.trim() {
        BARCODE_TYPE_EAN13 => {
            if is_ean13(&clean) {
                Ok(())
            } else {
                Err(format!(
                    "barcode '{clean}' is not a valid EAN13 (13 digits)"
                ))
            }
        }
        BARCODE_TYPE_DUN14 => {
            if is_dun14(&clean) {
                Ok(())
            } else {
                Err(format!(
                    "barcode '{clean}' is not a valid DUN14 (14 digits)"
                ))
            }
        }
        BARCODE_TYPE_OTHER => {
            if clean.is_empty() {
                Err("barcode is empty".to_string())
            } else {
                Ok(())
            }
        }
        other => Err(format!(
            "BARCODE_TYPE '{other}' is none of 1 (EAN13), 2 (DUN14) or 3 (other)"
        )),
    }
}

/// Split a vendor identifier into its manufacturer code and country prefix.
///
/// The original manual (p.10, p.17, p.23) defines `VENDOR_ID` as the first
/// **two** letters of the ISO country code followed by the manufacturer code,
/// while the delivered inventory file carries a **three** letter prefix
/// (`USA6063343900011222`). Both forms are accepted and the prefix is
/// returned separately, never silently discarded.
///
/// Returns `(code, prefix)`: the manufacturer code and the country prefix.
/// A value that carries no country prefix is returned unchanged with an empty
/// prefix.
pub fn split_vendor_country_prefix(value: &str) -> (String, String) {
    let clean = value.trim();
    let bytes = clean.as_bytes();
    let prefix_len = if bytes.len() > 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1].is_ascii_alphabetic()
        && bytes[2].is_ascii_alphabetic()
        && bytes[3].is_ascii_digit()
    {
        3
    } else if bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1].is_ascii_alphabetic()
        && bytes[2].is_ascii_digit()
    {
        2
    } else {
        0
    };
    if prefix_len == 0 {
        return (clean.to_string(), String::new());
    }
    let (prefix, code) = match (clean.get(..prefix_len), clean.get(prefix_len..)) {
        (Some(prefix), Some(code)) => (prefix, code),
        _ => return (clean.to_string(), String::new()),
    };
    if code.len() < 12 || !code.chars().all(|c| c.is_ascii_digit()) {
        return (clean.to_string(), String::new());
    }
    (code.to_string(), prefix.to_uppercase())
}

/// Rebuild a vendor identifier from its manufacturer code and country prefix.
///
/// The prefix length is preserved as given: the canonical form of the manual
/// is two letters, the delivered file uses three.
pub fn join_vendor_country_prefix(code: &str, prefix: &str) -> String {
    format!("{}{}", prefix.trim().to_uppercase(), code.trim())
}

/// Strip dot masks and whitespace from `DIST_ID`.
///
/// The original manual (p.23) requires the "DISTRIBUTOR RUC CODE WITHOUT
/// MASKS", so `02.986.485.7023.000` becomes `029864857023000`. Leading zeros
/// are preserved.
pub fn normalize_dist_id(dist_id: &str) -> String {
    dist_id.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Detect the delimiter of a delimited header line.
///
/// The MTRIX files are semicolon delimited; a comma delimited header is
/// accepted as well. The delimiter is detected from the header only, never
/// assumed.
fn detect_separator(header: &str) -> Option<char> {
    let semicolons = header.matches(';').count();
    let commas = header.matches(',').count();
    if semicolons > 0 && semicolons >= commas {
        Some(';')
    } else if commas > 0 {
        Some(',')
    } else {
        None
    }
}

/// Read one cell of a split row, tolerating a short row.
fn cell<'a>(values: &[&'a str], index: usize) -> &'a str {
    values.get(index).copied().unwrap_or("")
}

static GLOBAL_CATALOG: OnceLock<MdmCatalog> = OnceLock::new();

/// Entry point to the shared master data.
pub struct MdmHub;

impl MdmHub {
    /// Access the shared global catalogue.
    ///
    /// The catalogue starts as the **demo fallback**; a production deployment
    /// replaces its content with [`MdmCatalog::register_from_mtrix_product`].
    pub fn global_catalog() -> &'static MdmCatalog {
        GLOBAL_CATALOG.get_or_init(MdmCatalog::demo_bootstrap)
    }

    /// Strip dot masks and whitespace from `DIST_ID`.
    ///
    /// The original manual (p.23) requires the "DISTRIBUTOR RUC CODE WITHOUT
    /// MASKS", so `02.986.485.7023.000` becomes `029864857023000`.
    pub fn normalize_dist_id(dist_id: &str) -> String {
        normalize_dist_id(dist_id)
    }

    /// Fix known typos in store types using the global catalogue.
    pub fn normalize_store_type(store_type: &str) -> String {
        Self::global_catalog().normalize_store_type(store_type)
    }

    /// Trim a barcode without touching its content.
    ///
    /// Only whitespace and trailing `;` delimiters are removed, so leading
    /// zeros survive and a non numeric identifier is not corrupted.
    pub fn normalize_barcode(barcode: &str) -> String {
        normalize_barcode(barcode)
    }

    /// Validate a barcode against the barcode type of the product master.
    pub fn validate_barcode(barcode: &str, barcode_type: &str) -> Result<(), String> {
        validate_barcode(barcode, barcode_type)
    }

    /// Split a vendor identifier into its manufacturer code and country prefix.
    pub fn split_vendor_country_prefix(value: &str) -> (String, String) {
        split_vendor_country_prefix(value)
    }

    /// Rebuild a vendor identifier from its manufacturer code and country prefix.
    pub fn join_vendor_country_prefix(code: &str, prefix: &str) -> String {
        join_vendor_country_prefix(code, prefix)
    }

    /// Resolve an external store / channel identifier to an ERP customer code.
    ///
    /// An unknown channel falls back to [`DEFAULT_CHANNEL_FALLBACK`]; use
    /// [`MdmHub::resolve_channel_client_exact`] to detect an unknown channel.
    pub fn resolve_channel_client(channel_ref: &str) -> String {
        Self::global_catalog().resolve_channel(channel_ref)
    }

    /// Resolve an external store / channel identifier to an ERP customer code.
    ///
    /// Returns `None` when the channel is unknown instead of guessing.
    pub fn resolve_channel_client_exact(channel_ref: &str) -> Option<String> {
        Self::global_catalog().resolve_channel_exact(channel_ref)
    }

    /// Lookup SKU metadata in the canonical catalogue.
    pub fn lookup_sku(sku_key: &str) -> Option<SkuMetadata> {
        Self::global_catalog().lookup_sku(sku_key)
    }

    /// Reverse lookup by barcode.
    pub fn lookup_by_ean13(ean13: &str) -> Option<SkuMetadata> {
        Self::global_catalog().lookup_by_ean13(ean13)
    }

    /// Cumulative balance, in integer cents, of a customer across all batches.
    ///
    /// Aggregates every amount recorded through
    /// [`MdmCatalog::register_cumulative`] for `customer_id`. Returns `None`
    /// when no balance has been recorded yet.
    pub fn get_cumulative_balance(customer_id: &str) -> Option<i64> {
        Self::global_catalog()
            .get_cumulative_balance(customer_id)
            .ok()
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_dist_id() {
        assert_eq!(
            MdmHub::normalize_dist_id("02.986.485.7023.000"),
            "029864857023000"
        );
    }

    #[test]
    fn test_normalize_store_type() {
        assert_eq!(MdmHub::normalize_store_type("Onlinne"), "Online");
        assert_eq!(MdmHub::normalize_store_type("retail"), "Retail");
        assert_eq!(MdmHub::normalize_store_type("pos"), "Retail");
    }

    #[test]
    fn test_normalize_barcode_keeps_leading_zeros() {
        assert_eq!(
            MdmHub::normalize_barcode(" 7898656600087 ;;\n"),
            "7898656600087"
        );
        assert_eq!(
            MdmHub::normalize_barcode("007898656600087 "),
            "007898656600087"
        );
        assert_eq!(MdmHub::normalize_barcode("ABC-123"), "ABC-123");
    }

    #[test]
    fn test_resolve_channel_client() {
        assert_eq!(MdmHub::resolve_channel_client("Shopee"), "995C");
        assert_eq!(MdmHub::resolve_channel_client("蝦皮"), "995C");
        assert_eq!(MdmHub::resolve_channel_client("MOMO"), "995E");
        assert_eq!(
            MdmHub::resolve_channel_client_exact("MOMO").as_deref(),
            Some("995E")
        );
        assert!(MdmHub::resolve_channel_client_exact("No Such Channel").is_none());
    }

    #[test]
    fn test_sku_lookups() {
        let meta = MdmHub::lookup_sku("HAV-TOP-BLK-39").expect("SKU must exist");
        assert_eq!(meta.item_no, "8");
        assert_eq!(meta.ean13, "7898656600087");

        let rev = MdmHub::lookup_by_ean13("7898656600087").expect("Barcode must resolve");
        assert_eq!(rev.seller_sku, "HAV-TOP-BLK-39");
    }

    #[test]
    fn test_dynamic_catalog_registration_and_loading() {
        let catalog = MdmCatalog::new();
        assert!(catalog.lookup_sku("HAV-NEW-2027").is_none());

        catalog.register_sku(SkuMetadata {
            seller_sku: "HAV-NEW-2027".to_string(),
            item_no: "99".to_string(),
            ean13: "7899999999999".to_string(),
            product_code: "99".to_string(),
            name: "Havaianas Future Flip-Flop 2027".to_string(),
            uom: "PAIR".to_string(),
            cumulative_balance: 0,
        });

        let found = catalog
            .lookup_sku("HAV-NEW-2027")
            .expect("Must find dynamic SKU");
        assert_eq!(found.item_no, "99");

        let found_ean = catalog
            .lookup_by_ean13("7899999999999")
            .expect("Must find by EAN");
        assert_eq!(found_ean.seller_sku, "HAV-NEW-2027");

        // Dynamic channel
        catalog.register_channel("CYBERBIZ", "995H");
        assert_eq!(catalog.resolve_channel("Cyberbiz"), "995H");
        assert_eq!(
            catalog.resolve_channel_exact("Cyberbiz").as_deref(),
            Some("995H")
        );

        // Load JSON
        let json_data = r#"[
            {
                "seller_sku": "HAV-JSON-01",
                "item_no": "101",
                "ean13": "7891010101010",
                "product_code": "101",
                "name": "JSON Model",
                "uom": "PAIR"
            }
        ]"#;
        let count = catalog.load_from_json(json_data).unwrap();
        assert_eq!(count, 1);
        assert_eq!(catalog.lookup_sku("HAV-JSON-01").unwrap().item_no, "101");
    }

    #[test]
    fn test_vendor_country_prefix_round_trip() {
        let (code, prefix) = split_vendor_country_prefix("USA6063343900011222");
        assert_eq!(code, "6063343900011222");
        assert_eq!(prefix, "USA");
        assert_eq!(
            join_vendor_country_prefix(&code, &prefix),
            "USA6063343900011222"
        );

        let (code, prefix) = split_vendor_country_prefix("BR6063343900011222");
        assert_eq!(code, "6063343900011222");
        assert_eq!(prefix, "BR");

        let (code, prefix) = split_vendor_country_prefix("6063343900011222");
        assert_eq!(code, "6063343900011222");
        assert!(prefix.is_empty());
    }

    #[test]
    fn test_cumulative_balance_accumulates_across_batches() {
        let catalog = MdmCatalog::new();
        assert_eq!(catalog.get_cumulative_balance("995C").unwrap(), None);

        catalog
            .register_cumulative("B-001", "995C", 1_000)
            .expect("first credit must be recorded");
        assert_eq!(catalog.get_cumulative_balance("995C").unwrap(), Some(1_000));

        // A later batch for the same customer accumulates instead of overwriting.
        catalog
            .register_cumulative("B-002", "995C", 500)
            .expect("second credit must accumulate");
        assert_eq!(catalog.get_cumulative_balance("995C").unwrap(), Some(1_500));

        // Each customer keeps an independent balance.
        assert_eq!(catalog.get_cumulative_balance("995A").unwrap(), None);
    }

    #[test]
    fn test_validate_barcode_against_declared_type() {
        assert!(validate_barcode("7898656600087", BARCODE_TYPE_EAN13).is_ok());
        assert!(validate_barcode("7898656600087", BARCODE_TYPE_DUN14).is_err());
        assert!(validate_barcode("78986566000870", BARCODE_TYPE_DUN14).is_ok());
        assert!(validate_barcode("7898656600087", BARCODE_TYPE_OTHER).is_ok());
        assert!(validate_barcode("7898656600087", "9").is_err());
    }
}
