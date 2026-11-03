#![forbid(unsafe_code)]

//! Wellfit Core - Column Contract Enforcement
//!
//! A column contract declares the columns a file must carry and resolves them
//! by name. Resolution **fails** when a required column is absent.
//!
//! This module exists because the earlier implementation resolved columns with
//! `header.iter().position(...)` and fell back to a hard coded ordinal when the
//! name was not found. A missing column therefore did not fail: the parser read
//! whatever happened to sit at that position, which silently corrupted data
//! and made any "strict column validation" claim untrue. A contract has no
//! positional fallback by construction.

use std::collections::{HashMap, HashSet};

/// The columns a file must carry, and where they live.
#[derive(Debug, Clone, Default)]
pub struct ColumnContract {
    name: String,
    required: Vec<String>,
    optional: Vec<String>,
}

/// The resolved positions of the contract columns.
#[derive(Debug, Clone, Default)]
pub struct ColumnMap {
    indices: HashMap<String, usize>,
    header: Vec<String>,
}

impl ColumnContract {
    /// Declare a contract.
    ///
    /// `required` columns must be present; `optional` columns are resolved when
    /// present and reported as absent otherwise.
    pub fn new(name: &str, required: &[&str], optional: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            required: required.iter().map(|c| c.to_string()).collect(),
            optional: optional.iter().map(|c| c.to_string()).collect(),
        }
    }

    /// Name of the contract, used in error messages.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The required column names.
    pub fn required(&self) -> &[String] {
        &self.required
    }

    /// The optional column names.
    pub fn optional(&self) -> &[String] {
        &self.optional
    }

    /// Resolve the contract against a header line.
    ///
    /// Returns an error naming every missing required column. Matching is case
    /// insensitive and ignores surrounding whitespace.
    pub fn resolve(&self, headers: &[&str]) -> Result<ColumnMap, String> {
        let normalised: Vec<String> = headers.iter().map(|h| h.trim().to_uppercase()).collect();
        let mut seen = HashSet::with_capacity(normalised.len());
        let duplicates: Vec<&str> = headers
            .iter()
            .zip(normalised.iter())
            .filter_map(|(original, normalized)| {
                if seen.insert(normalized) {
                    None
                } else {
                    Some(*original)
                }
            })
            .collect();
        if !duplicates.is_empty() {
            return Err(format!(
                "{}: duplicate column name(s): {}",
                self.name,
                duplicates.join(", ")
            ));
        }

        let mut indices = HashMap::new();
        let mut missing: Vec<&str> = Vec::new();

        for column in &self.required {
            match normalised.iter().position(|h| h == &column.to_uppercase()) {
                Some(index) => {
                    indices.insert(column.clone(), index);
                }
                None => missing.push(column),
            }
        }
        for column in &self.optional {
            if let Some(index) = normalised.iter().position(|h| h == &column.to_uppercase()) {
                indices.insert(column.clone(), index);
            }
        }

        if !missing.is_empty() {
            return Err(format!(
                "{}: required column(s) missing: {} (header: {})",
                self.name,
                missing.join(", "),
                headers.join(", ")
            ));
        }

        Ok(ColumnMap {
            indices,
            header: headers.iter().map(|h| h.trim().to_string()).collect(),
        })
    }
}

impl ColumnMap {
    /// Position of a required column.
    pub fn index(&self, column: &str) -> Result<usize, String> {
        self.indices
            .get(column)
            .copied()
            .ok_or_else(|| format!("column '{column}' was not part of the resolved contract"))
    }

    /// Position of a column, when it is present.
    pub fn optional_index(&self, column: &str) -> Option<usize> {
        self.indices.get(column).copied()
    }

    /// The header line the contract was resolved against.
    pub fn header(&self) -> &[String] {
        &self.header
    }

    /// Number of columns in the source header.
    pub fn column_count(&self) -> usize {
        self.header.len()
    }
}

/// Split a header line into its column names.
pub fn split_header(line: &str, separator: char) -> Vec<String> {
    line.split(separator)
        .map(|s| s.trim().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> ColumnContract {
        ColumnContract::new(
            "fuyi inventory report",
            &[
                "ITEM_NO",
                "ITEM_NAME",
                "WAREHOUSE_CODE",
                "BEGIN_INV",
                "IN_QTY",
                "OUT_QTY",
                "ADJ_QTY",
                "END_INV",
            ],
            &[],
        )
    }

    #[test]
    fn test_resolves_a_complete_header() {
        let header = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV";
        let names: Vec<&str> = header.split(',').collect();
        let map = report().resolve(&names).expect("contract must resolve");
        assert_eq!(map.index("ITEM_NO").unwrap(), 0);
        assert_eq!(map.index("END_INV").unwrap(), 7);
        assert_eq!(map.column_count(), 8);
    }

    #[test]
    fn test_fails_when_a_required_column_is_missing() {
        let header = "ITEM_NO,ITEM_NAME,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV";
        let names: Vec<&str> = header.split(',').collect();
        let error = report().resolve(&names).unwrap_err();
        assert!(error.contains("WAREHOUSE_CODE"), "{error}");
    }

    #[test]
    fn test_never_falls_back_to_a_position() {
        // A header that is complete but in a different order still resolves by
        // name, so the value is read from the right column.
        let header = "END_INV,ITEM_NO,BEGIN_INV,ADJ_QTY,IN_QTY,ITEM_NAME,OUT_QTY,WAREHOUSE_CODE";
        let names: Vec<&str> = header.split(',').collect();
        let map = report().resolve(&names).unwrap();
        assert_eq!(map.index("ITEM_NO").unwrap(), 1);
        assert_eq!(map.index("END_INV").unwrap(), 0);
    }

    #[test]
    fn test_optional_columns_are_absent_without_failing() {
        let contract = ColumnContract::new("t", &["A"], &["B"]);
        let map = contract.resolve(&["A"]).unwrap();
        assert!(map.optional_index("B").is_none());
        assert!(map.index("B").is_err());
    }

    #[test]
    fn test_header_matching_is_case_insensitive() {
        let map = report()
            .resolve(&[
                "item_no",
                "ITEM_NAME",
                "warehouse_code",
                "begin_inv",
                "in_qty",
                "out_qty",
                "adj_qty",
                "end_inv",
            ])
            .unwrap();
        assert_eq!(map.index("ITEM_NO").unwrap(), 0);
    }

    #[test]
    fn test_rejects_duplicate_headers_after_normalization() {
        let error = report()
            .resolve(&[
                "ITEM_NO",
                "item_no",
                "WAREHOUSE_CODE",
                "BEGIN_INV",
                "IN_QTY",
                "OUT_QTY",
                "ADJ_QTY",
                "END_INV",
            ])
            .unwrap_err();
        assert!(error.contains("duplicate column"), "{error}");
    }
}
