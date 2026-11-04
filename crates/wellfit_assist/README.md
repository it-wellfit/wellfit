# wellfit_assist — OEKAP Cornerstone Crate

## Overview

The `wellfit_assist` crate is the cornerstone of the Open Enterprise Knowledge & Action Platform (OEKAP) development strategy. It provides the foundational assistive capabilities that enable gradual replacement of vendor-dependent ERP functionality.

## Modules

### data_transformer
Multi-format data conversion with explicit encoding handling:
- `DataFormat` enum: Api, Csv, Json, Sql, Xml
- `Charset` enum: Utf8, Gbk, Big5, ShiftJis
- `TransformSpec`: declared transform between two formats
- `convert_encoding`: charset conversion with error handling
- `detect_charset`: heuristic charset detection
- `validate_header`: required column validation
- `csv_to_json`: CSV → JSON conversion
- `json_to_csv`: JSON → CSV conversion

### report_generator
Assisted report rendering in multiple formats:
- `ReportFormat`: Html, Csv, Json, Excel, Pdf
- `ReportSpec`: report specification (title, format, template)
- `render_table`: render a table in the specified format
- HTML: full HTML document with XSS-safe escaping
- CSV: RFC 4180 compliant with proper quoting
- JSON: pretty-printed JSON object
- Excel/PDF: stub (requires ADR for dependencies)

### anomaly
Statistical anomaly detection (pure Rust, no Python):
- `AnomalyScore`: index, score, is_outlier
- `AnomalyDetector` trait
- `ZScoreDetector`: mean/std-dev based detection
- `MovingAverageDetector`: trend-based detection
- `extract_features`: statistical feature extraction (mean, variance, skewness, kurtosis, min, max, count)

### shadow
Shadow mode — read-only observation and comparison:
- `ShadowCapture`: records vendor output + our output with SHA-256 hashes
- `ShadowCompare`: aggregates captures and computes match rate
- `EvidenceStore`: file-based evidence persistence with JSON snapshots

### capture (S1-1)
Shadow capture — read-only verification evidence for a vendor export:
- `capture_reports_export`: recomputes the Fuyi inventory identity
  (End = Begin + In - Out + Adj, overflow-safe) independently of the vendor's
  stated values; read-only with a 100 MiB cap; requires zero writes outside the
  artifact root
- `ShadowEvidence`: evidence document schema `wellfit-shadow-evidence/v1`
  (input SHA-256, vendor digest, our digest, per-row mismatches)
- `MismatchRow`: `value mismatch` / `overflow` / `unparsable` rows with
  line numbers and both sides of the comparison
- CLI: `wellfit shadow --pipeline reports --input <file> --artifacts <dir> --batch <id>`
  (exit 0 = all rows match, 2 = differences recorded as evidence, 1 = error)

## Error Handling

All modules use the unified `AssistError` enum:
- `Encoding`: charset conversion failures
- `Validation`: missing columns, type mismatches
- `Transform`: format conversion failures
- `NotImplemented`: features pending ADR
- `Io`: I/O errors
- `Json`: JSON serialization/deserialization errors

## Testing

36 unit tests covering:
- Encoding round-trips (UTF-8, Big5, GBK, Shift_JIS)
- Header validation (complete, missing)
- CSV ↔ JSON conversion
- Transform spec validation
- Z-score outlier detection (normal, constant, short, empty series)
- Moving average detection
- Feature extraction (normal, empty series)
- Shadow capture (match, diff)
- Shadow comparison (normal, empty)
- Evidence store (save/load round-trip)
- Shadow report generation
- Shadow capture / S1-1 (balanced match, value mismatch, overflow, unparsable
  cell, missing column fails before rows, reordered header, UTF-8 BOM,
  missing input)

## Quality

- `#![forbid(unsafe_code)]` on every file
- Clippy clean under `-D warnings`
- Rustfmt clean
- 0 warnings, 0 errors

## Dependencies

- `wellfit_core`: shared core (ColumnContract, MDM, ledger)
- `wellfit_reports`: Fuyi inventory report contract (capture verification)
- `serde` + `serde_json`: serialization
- `encoding_rs`: charset handling
- `toml`: configuration
- `sha2`: evidence chain hashing
- `csv`: CSV parsing/writing