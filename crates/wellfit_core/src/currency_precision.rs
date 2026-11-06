#![forbid(unsafe_code)]

//! Currency and asset precision configuration.
//!
//! Defines the number of decimal places for each currency/asset type,
//! supporting traditional currencies (0-4 dp), forex (4-6 dp),
//! and digital assets (8-18 dp). Never uses f64.

/// Precision configuration for a currency or asset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyPrecision {
    /// Number of decimal places.
    pub decimal_places: u32,
    /// Whether this is a digital asset (crypto).
    pub is_digital_asset: bool,
    /// Scaling factor (10^decimal_places).
    pub scale: u64,
}

impl CurrencyPrecision {
    pub const fn new(decimal_places: u32, is_digital_asset: bool) -> Self {
        Self {
            decimal_places,
            is_digital_asset,
            scale: 10u64.pow(decimal_places),
        }
    }
}

/// Lookup table for currency/asset precision.
/// Traditional currencies: ISO 4217 standard decimal places.
/// Digital assets: per-asset precision (BTC=8, ETH=18, etc.).
pub fn currency_precision(code: &str) -> Option<CurrencyPrecision> {
    match code {
        // Zero decimal places
        "JPY" | "KRW" | "VND" | "ISK" | "CLP" | "PYG" | "UGX" | "RWF" | "BIF" | "DJF" | "GNF"
        | "KMF" | "XAF" | "XOF" | "XPF" => Some(CurrencyPrecision::new(0, false)),

        // Two decimal places (most common)
        "TWD" | "USD" | "EUR" | "GBP" | "CHF" | "CAD" | "AUD" | "NZD" | "SGD" | "HKD" | "CNY"
        | "SEK" | "NOK" | "DKK" | "PLN" | "CZK" | "HUF" | "ILS" | "MXN" | "ZAR" | "THB" | "MYR"
        | "PHP" | "IDR" | "INR" | "BRL" | "TRY" | "RUB" | "AED" | "SAR" | "QAR" | "LBP" | "EGP"
        | "MAD" | "DZD" => Some(CurrencyPrecision::new(2, false)),

        // Three decimal places
        "KWD" | "BHD" | "OMR" | "JOD" | "TND" => Some(CurrencyPrecision::new(3, false)),

        // Digital assets (cryptocurrency)
        "BTC" => Some(CurrencyPrecision::new(8, true)),
        "ETH" => Some(CurrencyPrecision::new(18, true)),
        "USDT" | "USDC" | "DAI" | "BUSD" => Some(CurrencyPrecision::new(6, true)),
        "BNB" => Some(CurrencyPrecision::new(18, true)),
        "XRP" => Some(CurrencyPrecision::new(6, true)),
        "ADA" => Some(CurrencyPrecision::new(6, true)),
        "SOL" => Some(CurrencyPrecision::new(9, true)),
        "DOT" => Some(CurrencyPrecision::new(10, true)),
        "AVAX" => Some(CurrencyPrecision::new(18, true)),
        "MATIC" => Some(CurrencyPrecision::new(18, true)),
        "LINK" => Some(CurrencyPrecision::new(18, true)),
        "UNI" => Some(CurrencyPrecision::new(18, true)),
        "AAVE" => Some(CurrencyPrecision::new(18, true)),
        "ATOM" => Some(CurrencyPrecision::new(6, true)),
        "LTC" => Some(CurrencyPrecision::new(8, true)),
        "BCH" => Some(CurrencyPrecision::new(8, true)),
        "DOGE" => Some(CurrencyPrecision::new(8, true)),
        "TRX" => Some(CurrencyPrecision::new(6, true)),
        "XLM" => Some(CurrencyPrecision::new(7, true)),
        "ALGO" => Some(CurrencyPrecision::new(6, true)),

        _ => None,
    }
}

/// Returns the decimal places for a currency/asset code.
/// Defaults to 2 for unknown currencies.
pub fn decimal_places(code: &str) -> u32 {
    currency_precision(code)
        .map(|p| p.decimal_places)
        .unwrap_or(2)
}

/// Returns true if the code is a digital asset.
pub fn is_digital_asset(code: &str) -> bool {
    currency_precision(code)
        .map(|p| p.is_digital_asset)
        .unwrap_or(false)
}

/// Returns the scale factor for a currency/asset code.
pub fn scale_factor(code: &str) -> u64 {
    currency_precision(code).map(|p| p.scale).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traditional_currencies() {
        assert_eq!(decimal_places("TWD"), 2);
        assert_eq!(decimal_places("USD"), 2);
        assert_eq!(decimal_places("EUR"), 2);
        assert_eq!(decimal_places("JPY"), 0);
        assert_eq!(decimal_places("KWD"), 3);
    }

    #[test]
    fn test_digital_assets() {
        assert_eq!(decimal_places("BTC"), 8);
        assert_eq!(decimal_places("ETH"), 18);
        assert_eq!(decimal_places("USDT"), 6);
        assert_eq!(decimal_places("SOL"), 9);
        assert_eq!(decimal_places("DOT"), 10);
    }

    #[test]
    fn test_is_digital_asset() {
        assert!(is_digital_asset("BTC"));
        assert!(is_digital_asset("ETH"));
        assert!(!is_digital_asset("TWD"));
        assert!(!is_digital_asset("USD"));
    }

    #[test]
    fn test_unknown_defaults_to_2() {
        assert_eq!(decimal_places("UNKNOWN"), 2);
        assert!(!is_digital_asset("UNKNOWN"));
    }

    #[test]
    fn test_scale_factor() {
        assert_eq!(scale_factor("TWD"), 100);
        assert_eq!(scale_factor("BTC"), 100_000_000);
        assert_eq!(scale_factor("ETH"), 1_000_000_000_000_000_000);
    }
}
