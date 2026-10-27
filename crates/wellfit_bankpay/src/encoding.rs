#![forbid(unsafe_code)]

//! Encoding conversion between internal UTF-8 and Taiwan bank/ERP encodings.
//!
//! Taiwan banking and ERP exchanges commonly use CP950 (Microsoft Code Page
//! 950) or Big5. Rust's standard library does not ship these codecs, so this
//! module relies on the [`encoding_rs`] crate, which provides Big5 and
//! Big5-HKSCS. CP950 is treated as Big5-HKSCS compatible: the two encodings
//! share the common Traditional Chinese repertoire used in banking data, and
//! CP950-specific extensions are rare in accounts-payable voucher files. This
//! mapping is documented here so it can be revisited when the bank supplies
//! its exact code page.

use encoding_rs::Encoding;

/// Supported bank/ERP exchange encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BankEncoding {
    /// Internal canonical encoding.
    #[default]
    Utf8,
    /// Microsoft Code Page 950 (mapped to Big5-HKSCS).
    Cp950,
    /// Big5.
    Big5,
    /// Big5-HKSCS (Hong Kong Supplementary Character Set).
    Big5Hkscs,
}

impl BankEncoding {
    /// The [`encoding_rs::Encoding`] for this variant, resolved by label.
    pub fn encoding(self) -> &'static Encoding {
        match self {
            BankEncoding::Utf8 => {
                Encoding::for_label(b"UTF-8").expect("UTF-8 must be available in encoding_rs")
            }
            BankEncoding::Cp950 => Encoding::for_label(b"Big5-HKSCS")
                .expect("Big5-HKSCS must be available in encoding_rs"),
            BankEncoding::Big5 => {
                Encoding::for_label(b"Big5").expect("Big5 must be available in encoding_rs")
            }
            BankEncoding::Big5Hkscs => Encoding::for_label(b"Big5-HKSCS")
                .expect("Big5-HKSCS must be available in encoding_rs"),
        }
    }

    /// MIME / IANA label for this encoding.
    pub fn label(self) -> &'static str {
        match self {
            BankEncoding::Utf8 => "UTF-8",
            BankEncoding::Cp950 => "CP950 (Big5-HKSCS)",
            BankEncoding::Big5 => "Big5 (Big5-HKSCS)",
            BankEncoding::Big5Hkscs => "Big5-HKSCS",
        }
    }
}

/// Detect the encoding of a byte buffer.
///
/// Detection order:
/// 1. Byte-order mark (UTF-8, UTF-16 LE, UTF-16 BE).
/// 2. Otherwise defaults to UTF-8 (the internal canonical encoding).
///
/// Returns the detected encoding and the byte offset after the BOM (if any).
pub fn detect_encoding(bytes: &[u8]) -> (BankEncoding, usize) {
    if bytes.starts_with(b"\xEF\xBB\xBF") {
        return (BankEncoding::Utf8, 3);
    }
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        return (BankEncoding::Utf8, 2);
    }
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        return (BankEncoding::Utf8, 2);
    }
    (BankEncoding::Utf8, 0)
}

/// Convert a byte buffer to a UTF-8 `String`.
///
/// The caller supplies the source encoding (or `None` to auto-detect via
/// [`detect_encoding`]). Invalid byte sequences are replaced with the
/// replacement character rather than failing, so a malformed bank file is
/// still readable for review and quarantine.
pub fn to_utf8(bytes: &[u8], source: Option<BankEncoding>) -> (String, BankEncoding) {
    let (encoding, bom_offset) = match source {
        Some(enc) => (enc, 0),
        None => detect_encoding(bytes),
    };
    let payload = &bytes[bom_offset..];
    let (decoded, _encoding_used, _had_errors) = encoding.encoding().decode(payload);
    (decoded.into_owned(), encoding)
}

/// Convert a UTF-8 `String` to bytes in the target encoding.
///
/// Characters that cannot be represented in the target encoding are replaced
/// with the encoding's replacement byte sequence. Callers should inspect the
/// returned `had_errors` flag and quarantine files that lost characters.
pub fn from_utf8(text: &str, target: BankEncoding) -> (Vec<u8>, bool) {
    let encoding = target.encoding();
    let (encoded, _encoding_used, had_errors) = encoding.encode(text);
    (encoded.into_owned(), had_errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8_bom() {
        let bytes = b"\xEF\xBB\xBFhello";
        let (enc, offset) = detect_encoding(bytes);
        assert_eq!(enc, BankEncoding::Utf8);
        assert_eq!(offset, 3);
    }

    #[test]
    fn test_detect_plain_utf8() {
        let bytes = b"hello world";
        let (enc, offset) = detect_encoding(bytes);
        assert_eq!(enc, BankEncoding::Utf8);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_from_utf8_round_trip() {
        let text = "AP-001,V001,1000,ACCT-1,2026-11-03,INV-100";
        let (bytes, had_errors) = from_utf8(text, BankEncoding::Big5);
        assert!(!had_errors);
        let (decoded, detected) = to_utf8(&bytes, Some(BankEncoding::Big5));
        assert_eq!(detected, BankEncoding::Big5);
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_from_utf8_big5_ascii_no_errors() {
        let text = "AP-001,V001,1000";
        let (bytes, had_errors) = from_utf8(text, BankEncoding::Big5);
        assert!(!had_errors);
        let (decoded, _detected) = to_utf8(&bytes, Some(BankEncoding::Big5));
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_cp950_label() {
        assert_eq!(BankEncoding::Cp950.label(), "CP950 (Big5-HKSCS)");
    }
}
