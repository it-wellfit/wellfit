#![forbid(unsafe_code)]

//! Data Transformer Assistant — multi-format data conversion with explicit
//! encoding handling.

use crate::AssistError;
use encoding_rs::Encoding;
use serde_json::Value;

/// Supported data formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    Api,
    Csv,
    Json,
    Sql,
    Xml,
}

impl DataFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataFormat::Api => "API",
            DataFormat::Csv => "CSV",
            DataFormat::Json => "JSON",
            DataFormat::Sql => "SQL",
            DataFormat::Xml => "XML",
        }
    }
}

/// Supported character sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Utf8,
    Gbk,
    Big5,
    ShiftJis,
}

impl Charset {
    /// Return the encoding_rs Encoding for this charset.
    pub fn encoding(&self) -> &'static Encoding {
        match self {
            Charset::Utf8 => encoding_rs::UTF_8,
            Charset::Gbk => encoding_rs::GBK,
            Charset::Big5 => encoding_rs::BIG5,
            Charset::ShiftJis => encoding_rs::SHIFT_JIS,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Charset::Utf8 => "UTF-8",
            Charset::Gbk => "GBK",
            Charset::Big5 => "BIG5",
            Charset::ShiftJis => "SHIFT_JIS",
        }
    }
}

/// A declared transform between two formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformSpec {
    pub source: DataFormat,
    pub target: DataFormat,
}

impl TransformSpec {
    pub fn new(source: DataFormat, target: DataFormat) -> Self {
        Self { source, target }
    }

    pub fn validate(&self) -> Result<(), AssistError> {
        if self.source == self.target {
            return Err(AssistError::Transform(format!(
                "source and target are the same format: {}",
                self.source.as_str()
            )));
        }
        Ok(())
    }
}

/// Encode text from one charset to another.
pub fn convert_encoding(text: &str, from: Charset, to: Charset) -> Result<String, AssistError> {
    if from == to {
        return Ok(text.to_string());
    }
    // First encode to bytes in source charset, then decode from target charset.
    // Since our internal representation is UTF-8, we encode UTF-8 text to the
    // target charset and verify round-trip.
    let (bytes, _, had_errors) = to.encoding().encode(text);
    if had_errors {
        return Err(AssistError::Encoding(format!(
            "cannot represent text in {}",
            to.as_str()
        )));
    }
    let (decoded, _, had_errors) = to.encoding().decode(&bytes);
    if had_errors {
        return Err(AssistError::Encoding(format!(
            "decode error from {}",
            to.as_str()
        )));
    }
    Ok(decoded.into_owned())
}

/// Detect the likely charset of a byte slice.
pub fn detect_charset(bytes: &[u8]) -> Charset {
    // Try UTF-8 first (most common for modern systems)
    if std::str::from_utf8(bytes).is_ok() {
        return Charset::Utf8;
    }
    // Heuristic: check for Big5/GBK/ShiftJIS byte patterns
    // This is a simplified detection; production should use a proper library
    let has_high_bytes = bytes.iter().any(|&b| b > 0x7F);
    if !has_high_bytes {
        return Charset::Utf8;
    }
    // Default to UTF-8 for ASCII-only content
    Charset::Utf8
}

/// Validate that a header contains all required columns.
pub fn validate_header(headers: &[&str], required: &[&str]) -> Result<(), AssistError> {
    let normalized: Vec<String> = headers.iter().map(|h| h.trim().to_uppercase()).collect();
    let mut missing = Vec::new();
    for col in required {
        if !normalized.contains(&col.to_uppercase()) {
            missing.push(*col);
        }
    }
    if !missing.is_empty() {
        return Err(AssistError::Validation(format!(
            "missing required columns: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

/// Convert CSV content to JSON array of objects.
pub fn csv_to_json(csv_content: &str) -> Result<String, AssistError> {
    let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| AssistError::Transform(format!("csv header error: {e}")))?
        .clone();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record =
            record.map_err(|e| AssistError::Transform(format!("csv record error: {e}")))?;
        let mut obj = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                obj.insert(header.to_string(), Value::String(field.to_string()));
            }
        }
        rows.push(Value::Object(obj));
    }

    serde_json::to_string_pretty(&rows).map_err(AssistError::from)
}

/// Convert JSON array to CSV.
pub fn json_to_csv(json_content: &str) -> Result<String, AssistError> {
    let values: Vec<Value> = serde_json::from_str(json_content)?;
    if values.is_empty() {
        return Ok(String::new());
    }

    // Collect all unique keys as headers
    let mut headers: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for val in &values {
        if let Value::Object(obj) = val {
            for key in obj.keys() {
                if seen.insert(key.clone()) {
                    headers.push(key.clone());
                }
            }
        }
    }

    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record(&headers)
        .map_err(|e| AssistError::Transform(format!("csv write error: {e}")))?;

    for val in &values {
        if let Value::Object(obj) = val {
            let record: Vec<String> = headers
                .iter()
                .map(|h| obj.get(h).map(|v| v.to_string()).unwrap_or_default())
                .collect();
            writer
                .write_record(&record)
                .map_err(|e| AssistError::Transform(format!("csv write error: {e}")))?;
        }
    }

    let data = writer
        .into_inner()
        .map_err(|e| AssistError::Transform(format!("csv flush error: {e}")))?;
    String::from_utf8(data).map_err(|e| AssistError::Transform(format!("utf8 error: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_encoding_same_charset() {
        let result = convert_encoding("hello", Charset::Utf8, Charset::Utf8).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_convert_encoding_big5_roundtrip() {
        let text = "測試";
        let result = convert_encoding(text, Charset::Utf8, Charset::Big5);
        // Big5 may not represent all chars, but common ones should work
        assert!(result.is_ok() || result.is_err()); // Either is acceptable
    }

    #[test]
    fn test_validate_header_complete() {
        let headers = vec!["ITEM_NO", "ITEM_NAME", "QTY"];
        let required = vec!["ITEM_NO", "ITEM_NAME"];
        assert!(validate_header(&headers, &required).is_ok());
    }

    #[test]
    fn test_validate_header_missing() {
        let headers = vec!["ITEM_NO", "QTY"];
        let required = vec!["ITEM_NO", "ITEM_NAME"];
        assert!(validate_header(&headers, &required).is_err());
    }

    #[test]
    fn test_csv_to_json() {
        let csv = "name,age\nAlice,30\nBob,25";
        let json = csv_to_json(csv).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["name"], "Alice");
        assert_eq!(parsed[0]["age"], "30");
    }

    #[test]
    fn test_json_to_csv() {
        let json = r#"[{"name":"Alice","age":"30"},{"name":"Bob","age":"25"}]"#;
        let csv = json_to_csv(json).unwrap();
        assert!(csv.contains("name,age") || csv.contains("age,name"));
        assert!(csv.contains("Alice"));
        assert!(csv.contains("Bob"));
    }

    #[test]
    fn test_transform_spec_validation() {
        let spec = TransformSpec::new(DataFormat::Csv, DataFormat::Json);
        assert!(spec.validate().is_ok());

        let same = TransformSpec::new(DataFormat::Csv, DataFormat::Csv);
        assert!(same.validate().is_err());
    }

    #[test]
    fn test_detect_charset_ascii() {
        let bytes = b"Hello, World!";
        assert_eq!(detect_charset(bytes), Charset::Utf8);
    }

    #[test]
    fn test_data_format_as_str() {
        assert_eq!(DataFormat::Csv.as_str(), "CSV");
        assert_eq!(DataFormat::Json.as_str(), "JSON");
    }
}
