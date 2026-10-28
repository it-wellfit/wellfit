#![forbid(unsafe_code)]

//! Wellfit Core - Fixed-point decimal arithmetic (zero floating point).
//!
//! The MTRIX field dictionary declares `QUANTITY` and `SALES_PRICE` as
//! `DECIMAL(18,4)` (MTRIX-024) while the original manual leaves the number of
//! decimal places undefined (MTRIX-005). Because the platform invariant forbids
//! floating point, all measures are carried as a scaled `i64`: the value equals
//! `units / 10^4`.
//!
//! The original text is always preserved by the caller, so no precision is lost
//! relative to the source file; the scaled form exists only for arithmetic.

use serde::{Deserialize, Serialize};

/// A decimal number carried as a scaled integer. Never uses `f64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScaledDecimal {
    units: i64,
}

impl ScaledDecimal {
    /// Number of decimal places retained.
    pub const SCALE: u32 = 4;
    /// Scaling factor of the retained decimal places.
    pub const FACTOR: i64 = 10_000;
    /// Additive identity.
    pub const ZERO: ScaledDecimal = ScaledDecimal { units: 0 };

    /// Build from an already scaled integer.
    pub fn from_units(units: i64) -> Self {
        Self { units }
    }

    /// The scaled integer representation.
    pub fn units(&self) -> i64 {
        self.units
    }

    /// Whether the value is below zero.
    pub fn is_negative(&self) -> bool {
        self.units < 0
    }

    /// Add two values, returning `None` on overflow.
    pub fn checked_add(&self, other: &ScaledDecimal) -> Option<ScaledDecimal> {
        self.units.checked_add(other.units).map(ScaledDecimal::from_units)
    }

    /// Parse a decimal literal without floating point.
    ///
    /// Accepts optional sign, an integer part, and up to four fractional digits.
    /// Extra fractional digits beyond [`ScaledDecimal::SCALE`] are truncated and
    /// the original string stays available to the caller for reporting.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("empty numeric value".to_string());
        }
        let (int_part, frac_part) = match trimmed.split_once('.') {
            Some((a, b)) => (a, b),
            None => (trimmed, ""),
        };
        let negative = int_part.starts_with('-');
        let int_digits: String = int_part
            .trim_start_matches(['-', '+'])
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let mut frac_digits: String = frac_part.chars().filter(|c| c.is_ascii_digit()).collect();
        // Anything that is not a digit, a sign or a decimal point is not a
        // number. Silently dropping it would turn "1e5" into 15.
        let significant: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
        let only_numeric = significant
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == '+' || c == '.');
        if !only_numeric || significant.matches('.').count() > 1 {
            return Err(format!("not a numeric value: '{raw}'"));
        }
        if int_digits.is_empty() && frac_digits.is_empty() {
            return Err(format!("not a numeric value: '{raw}'"));
        }
        frac_digits.truncate(Self::SCALE as usize);
        while (frac_digits.len() as u32) < Self::SCALE {
            frac_digits.push('0');
        }
        let int_value: i64 = if int_digits.is_empty() {
            0
        } else {
            int_digits
                .parse::<i64>()
                .map_err(|e| format!("invalid integer part '{raw}': {e}"))?
        };
        let frac_value: i64 = if frac_digits.is_empty() {
            0
        } else {
            frac_digits
                .parse::<i64>()
                .map_err(|e| format!("invalid fractional part '{raw}': {e}"))?
        };
        let magnitude = int_value
            .checked_mul(Self::FACTOR)
            .and_then(|v| v.checked_add(frac_value))
            .ok_or_else(|| format!("numeric overflow: '{raw}'"))?;
        Ok(ScaledDecimal {
            units: if negative { -magnitude } else { magnitude },
        })
    }

    /// Exact plain-text representation with the retained decimal places.
    pub fn to_plain_string(&self) -> String {
        let negative = self.units < 0;
        let magnitude = self.units.unsigned_abs();
        let whole = magnitude / Self::FACTOR as u64;
        let frac = magnitude % Self::FACTOR as u64;
        let sign = if negative { "-" } else { "" };
        format!("{sign}{whole}.{frac:04}")
    }
}

impl std::fmt::Display for ScaledDecimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_plain_string())
    }
}

impl Default for ScaledDecimal {
    fn default() -> Self {
        ScaledDecimal::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parses_plain_integers_without_floats() {
        let v = ScaledDecimal::parse("450").unwrap();
        assert_eq!(v.units(), 4_500_000);
        assert_eq!(v.to_plain_string(), "450.0000");
    }

    #[test]
    fn test_parses_four_decimal_places_exactly() {
        let v = ScaledDecimal::parse("123.4567").unwrap();
        assert_eq!(v.units(), 1_234_567);
        assert_eq!(v.to_plain_string(), "123.4567");
    }

    #[test]
    fn test_no_binary_floating_point_drift() {
        // 0.1 + 0.2 must be exactly 0.3 in scaled arithmetic.
        let a = ScaledDecimal::parse("0.1").unwrap();
        let b = ScaledDecimal::parse("0.2").unwrap();
        let c = ScaledDecimal::parse("0.3").unwrap();
        assert_eq!(a.checked_add(&b).unwrap(), c);
    }

    #[test]
    fn test_handles_negative_values() {
        let v = ScaledDecimal::parse("-12").unwrap();
        assert!(v.is_negative());
        assert_eq!(v.units(), -120_000);
        assert_eq!(v.to_plain_string(), "-12.0000");
    }

    #[test]
    fn test_truncates_extra_fractional_digits() {
        let v = ScaledDecimal::parse("1.234567").unwrap();
        assert_eq!(v.units(), 12_345);
    }

    #[test]
    fn test_rejects_non_numeric_input() {
        assert!(ScaledDecimal::parse("abc").is_err());
        assert!(ScaledDecimal::parse("").is_err());
    }

    #[test]
    fn test_rejects_a_number_with_a_hidden_letter() {
        // "1e5" must not be read as 15.
        assert!(ScaledDecimal::parse("1e5").is_err());
        assert!(ScaledDecimal::parse("12abc").is_err());
        assert!(ScaledDecimal::parse("1.2.3").is_err());
    }
}
