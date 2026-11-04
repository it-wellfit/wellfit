#![forbid(unsafe_code)]

//! Shadow Mode — read-only observation and comparison of vendor vs. our output.

use crate::AssistError;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// A captured observation from shadow mode.
#[derive(Debug, Clone)]
pub struct ShadowCapture {
    pub timestamp: String,
    pub input_hash: String,
    pub vendor_output: String,
    pub our_output: String,
    pub vendor_hash: String,
    pub our_hash: String,
}

impl ShadowCapture {
    pub fn new(input: &str, vendor_output: &str, our_output: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let input_hash = format!("{:x}", hasher.finalize());

        let mut hasher = Sha256::new();
        hasher.update(vendor_output.as_bytes());
        let vendor_hash = format!("{:x}", hasher.finalize());

        let mut hasher = Sha256::new();
        hasher.update(our_output.as_bytes());
        let our_hash = format!("{:x}", hasher.finalize());

        Self {
            timestamp: chrono_now(),
            input_hash,
            vendor_output: vendor_output.to_string(),
            our_output: our_output.to_string(),
            vendor_hash,
            our_hash,
        }
    }

    pub fn matches(&self) -> bool {
        self.vendor_hash == self.our_hash
    }

    pub fn diff_summary(&self) -> String {
        if self.matches() {
            "MATCH: vendor and our output are identical".to_string()
        } else {
            format!(
                "DIFF: vendor_hash={}, our_hash={}",
                self.vendor_hash, self.our_hash
            )
        }
    }
}

/// Shadow mode comparison result.
#[derive(Debug, Clone)]
pub struct ShadowCompare {
    pub captures: Vec<ShadowCapture>,
    pub match_count: usize,
    pub diff_count: usize,
}

impl Default for ShadowCompare {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowCompare {
    pub fn new() -> Self {
        Self {
            captures: Vec::new(),
            match_count: 0,
            diff_count: 0,
        }
    }

    pub fn add(&mut self, capture: ShadowCapture) {
        if capture.matches() {
            self.match_count += 1;
        } else {
            self.diff_count += 1;
        }
        self.captures.push(capture);
    }

    pub fn total(&self) -> usize {
        self.captures.len()
    }

    pub fn match_rate(&self) -> f64 {
        if self.total() == 0 {
            return 0.0;
        }
        self.match_count as f64 / self.total() as f64
    }

    pub fn report(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Shadow Mode Report: {} captures, {} matches, {} diffs, match rate: {:.2}%\n",
            self.total(),
            self.match_count,
            self.diff_count,
            self.match_rate() * 100.0
        ));
        for (i, c) in self.captures.iter().enumerate() {
            out.push_str(&format!(
                "  [{}] {} — {}\n",
                i + 1,
                c.timestamp,
                c.diff_summary()
            ));
        }
        out
    }
}

/// Evidence store for shadow mode captures.
pub struct EvidenceStore {
    pub base_dir: PathBuf,
}

impl EvidenceStore {
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    pub fn save(&self, capture: &ShadowCapture) -> Result<PathBuf, AssistError> {
        let dir = self.base_dir.join("shadow-mode");
        fs::create_dir_all(&dir)?;
        let filename = format!("{}.json", capture.timestamp.replace(':', "-"));
        let path = dir.join(&filename);

        let json = serde_json::json!({
            "timestamp": capture.timestamp,
            "input_hash": capture.input_hash,
            "vendor_output": capture.vendor_output,
            "our_output": capture.our_output,
            "vendor_hash": capture.vendor_hash,
            "our_hash": capture.our_hash,
            "matches": capture.matches(),
        });

        let content = serde_json::to_string_pretty(&json).map_err(AssistError::from)?;
        fs::write(&path, content)?;
        Ok(path)
    }

    pub fn load_all(&self) -> Result<Vec<ShadowCapture>, AssistError> {
        let dir = self.base_dir.join("shadow-mode");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut captures = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;
                let json: serde_json::Value = serde_json::from_str(&content)?;
                captures.push(ShadowCapture {
                    timestamp: json["timestamp"].as_str().unwrap_or("").to_string(),
                    input_hash: json["input_hash"].as_str().unwrap_or("").to_string(),
                    vendor_output: json["vendor_output"].as_str().unwrap_or("").to_string(),
                    our_output: json["our_output"].as_str().unwrap_or("").to_string(),
                    vendor_hash: json["vendor_hash"].as_str().unwrap_or("").to_string(),
                    our_hash: json["our_hash"].as_str().unwrap_or("").to_string(),
                });
            }
        }
        Ok(captures)
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_shadow_capture_match() {
        let capture = ShadowCapture::new("input", "output1", "output1");
        assert!(capture.matches());
        assert!(capture.diff_summary().contains("MATCH"));
    }

    #[test]
    fn test_shadow_capture_diff() {
        let capture = ShadowCapture::new("input", "vendor_out", "our_out");
        assert!(!capture.matches());
        assert!(capture.diff_summary().contains("DIFF"));
    }

    #[test]
    fn test_shadow_compare() {
        let mut compare = ShadowCompare::new();
        compare.add(ShadowCapture::new("in1", "out1", "out1"));
        compare.add(ShadowCapture::new("in2", "v2", "o2"));
        assert_eq!(compare.total(), 2);
        assert_eq!(compare.match_count, 1);
        assert_eq!(compare.diff_count, 1);
        assert!((compare.match_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_shadow_compare_empty() {
        let compare = ShadowCompare::new();
        assert_eq!(compare.total(), 0);
        assert_eq!(compare.match_rate(), 0.0);
    }

    #[test]
    fn test_evidence_store_save_load() {
        let tmp_dir = std::env::temp_dir().join("shadow_test");
        let _ = fs::remove_dir_all(&tmp_dir);
        let store = EvidenceStore::new(tmp_dir.to_str().unwrap());
        let capture = ShadowCapture::new("test_input", "vendor", "vendor");
        let path = store.save(&capture).unwrap();
        assert!(path.exists());

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].matches());

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_shadow_report() {
        let mut compare = ShadowCompare::new();
        compare.add(ShadowCapture::new("in1", "out1", "out1"));
        let report = compare.report();
        assert!(report.contains("1 captures"));
        assert!(report.contains("1 matches"));
    }
}
