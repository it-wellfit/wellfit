#![forbid(unsafe_code)]

//! Anomaly Detection — pure Rust statistical anomaly detection.

use std::collections::HashMap;

/// Anomaly score for a single observation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnomalyScore {
    pub index: usize,
    pub score: f64,
    pub is_outlier: bool,
}

/// Trait for anomaly detectors.
pub trait AnomalyDetector {
    fn detect(&self, series: &[f64]) -> Vec<AnomalyScore>;
}

/// Z-score based anomaly detector.
#[derive(Debug, Clone, Copy)]
pub struct ZScoreDetector {
    pub threshold: f64,
}

impl ZScoreDetector {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }
}

impl AnomalyDetector for ZScoreDetector {
    fn detect(&self, series: &[f64]) -> Vec<AnomalyScore> {
        if series.len() < 2 {
            return series
                .iter()
                .enumerate()
                .map(|(i, _)| AnomalyScore {
                    index: i,
                    score: 0.0,
                    is_outlier: false,
                })
                .collect();
        }

        let n = series.len() as f64;
        let mean = series.iter().sum::<f64>() / n;
        let variance = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        series
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                let score = if std_dev > 0.0 {
                    (val - mean).abs() / std_dev
                } else {
                    0.0
                };
                AnomalyScore {
                    index: i,
                    score,
                    is_outlier: score > self.threshold,
                }
            })
            .collect()
    }
}

/// Moving average based anomaly detector.
#[derive(Debug, Clone, Copy)]
pub struct MovingAverageDetector {
    pub window_size: usize,
    pub threshold: f64,
}

impl MovingAverageDetector {
    pub fn new(window_size: usize, threshold: f64) -> Self {
        Self {
            window_size,
            threshold,
        }
    }
}

impl AnomalyDetector for MovingAverageDetector {
    fn detect(&self, series: &[f64]) -> Vec<AnomalyScore> {
        if series.is_empty() || self.window_size == 0 {
            return vec![];
        }

        let mut scores = Vec::with_capacity(series.len());

        for (i, &val) in series.iter().enumerate() {
            let start = i.saturating_sub(self.window_size);
            let window = &series[start..i];
            if window.is_empty() {
                scores.push(AnomalyScore {
                    index: i,
                    score: 0.0,
                    is_outlier: false,
                });
                continue;
            }
            let mean = window.iter().sum::<f64>() / window.len() as f64;
            let deviation = (val - mean).abs();
            let score = deviation;
            scores.push(AnomalyScore {
                index: i,
                score,
                is_outlier: score > self.threshold,
            });
        }

        scores
    }
}

/// Extract statistical features from a series.
pub fn extract_features(series: &[f64]) -> HashMap<String, f64> {
    let mut features = HashMap::new();
    if series.is_empty() {
        return features;
    }

    let n = series.len() as f64;
    let mean = series.iter().sum::<f64>() / n;
    let variance = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    // Skewness
    let skewness = if std_dev > 0.0 {
        series
            .iter()
            .map(|x| ((x - mean) / std_dev).powi(3))
            .sum::<f64>()
            / n
    } else {
        0.0
    };

    // Kurtosis
    let kurtosis = if std_dev > 0.0 {
        series
            .iter()
            .map(|x| ((x - mean) / std_dev).powi(4))
            .sum::<f64>()
            / n
            - 3.0
    } else {
        0.0
    };

    features.insert("mean".to_string(), mean);
    features.insert("variance".to_string(), variance);
    features.insert("std_dev".to_string(), std_dev);
    features.insert(
        "min".to_string(),
        series.iter().cloned().fold(f64::INFINITY, f64::min),
    );
    features.insert(
        "max".to_string(),
        series.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    );
    features.insert("skewness".to_string(), skewness);
    features.insert("kurtosis".to_string(), kurtosis);
    features.insert("count".to_string(), n);

    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zscore_detects_outlier() {
        let mut series = vec![10.0; 19];
        series.push(100.0);
        let detector = ZScoreDetector::new(3.0);
        let scores = detector.detect(&series);
        assert_eq!(scores.len(), 20);
        assert!(
            scores[19].is_outlier,
            "outlier at index 19 should be detected"
        );
        assert!(!scores[0].is_outlier, "normal value should not be outlier");
    }

    #[test]
    fn test_zscore_constant_series() {
        let series = vec![5.0; 10];
        let detector = ZScoreDetector::new(3.0);
        let scores = detector.detect(&series);
        assert!(scores.iter().all(|s| !s.is_outlier));
        assert!(scores.iter().all(|s| s.score == 0.0));
    }

    #[test]
    fn test_zscore_short_series() {
        let series = vec![1.0];
        let detector = ZScoreDetector::new(3.0);
        let scores = detector.detect(&series);
        assert_eq!(scores.len(), 1);
        assert!(!scores[0].is_outlier);
    }

    #[test]
    fn test_zscore_empty_series() {
        let series: Vec<f64> = vec![];
        let detector = ZScoreDetector::new(3.0);
        let scores = detector.detect(&series);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_moving_average() {
        let series = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        let detector = MovingAverageDetector::new(3, 10.0);
        let scores = detector.detect(&series);
        assert_eq!(scores.len(), 5);
    }

    #[test]
    fn test_extract_features() {
        let series = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let features = extract_features(&series);
        assert!(features.contains_key("mean"));
        assert!(features.contains_key("variance"));
        assert!(features.contains_key("std_dev"));
        assert!(features.contains_key("min"));
        assert!(features.contains_key("max"));
        assert!(features.contains_key("skewness"));
        assert!(features.contains_key("kurtosis"));
        assert!(features.contains_key("count"));
        assert_eq!(features["mean"], 3.0);
        assert_eq!(features["min"], 1.0);
        assert_eq!(features["max"], 5.0);
        assert_eq!(features["count"], 5.0);
    }

    #[test]
    fn test_extract_features_empty() {
        let series: Vec<f64> = vec![];
        let features = extract_features(&series);
        assert!(features.is_empty());
    }
}
