#![forbid(unsafe_code)]

//! Wellfit Core - SQLite Idempotency Ledger (IdempotencyLedger)
//!
//! Guarantees zero duplicate batches, atomic transaction locks, and structured quarantine logging.

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineRecord {
    pub id: i64,
    pub batch_id: String,
    pub filename: String,
    pub line_number: usize,
    pub raw_value: String,
    pub error_code: String,
    pub error_reason: String,
    pub quarantined_at: String,
}

/// Severity of a recorded finding (MTRIX-016 two-tier model).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    /// The row or file cannot be accepted.
    Error,
    /// Accepted with a recorded caveat.
    Warning,
    /// Observation only.
    Info,
}

impl FindingSeverity {
    /// Stored representation.
    pub fn as_str(self) -> &'static str {
        match self {
            FindingSeverity::Error => "ERROR",
            FindingSeverity::Warning => "WARNING",
            FindingSeverity::Info => "INFO",
        }
    }

    /// Parse a stored representation.
    pub fn from_str_lossy(value: &str) -> Self {
        match value.to_ascii_uppercase().as_str() {
            "WARNING" => FindingSeverity::Warning,
            "INFO" => FindingSeverity::Info,
            _ => FindingSeverity::Error,
        }
    }
}

/// A validation finding that did not isolate a row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingRecord {
    pub id: i64,
    pub batch_id: String,
    pub rule_code: String,
    pub severity: String,
    pub filename: String,
    pub line_number: usize,
    pub key_value: String,
    pub reason: String,
    pub created_at: String,
}

/// A validation finding that is about to be written to the ledger.
///
/// The ledger stores two kinds of record: rows that were isolated (the row
/// could not be accepted) and findings that did not isolate anything. Both
/// carry the same attributes, so they share one input type.
#[derive(Debug, Clone, Copy)]
pub struct Finding<'a> {
    /// Batch the finding belongs to.
    pub batch_id: &'a str,
    /// Rule that produced the finding (VR-xx / ANOM-xx / error code).
    pub rule_code: &'a str,
    /// Severity of the finding.
    pub severity: FindingSeverity,
    /// File the finding was raised against.
    pub filename: &'a str,
    /// One based line number in the source file.
    pub line_number: usize,
    /// The key value the finding is about.
    pub key_value: &'a str,
    /// Human readable explanation.
    pub reason: &'a str,
}

/// How a ledger claims batches and order keys.
///
/// The production ingestion policy is [`BatchScope::Exclusive`]: the content of
/// a file is ingested once and never again. A verification run is different -
/// the same file has to be verifiable as often as the operator wants, without
/// the ledger turning the second run into a false rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchScope {
    /// The content hash identifies the batch. Ingesting the same content twice
    /// is refused.
    Exclusive,
    /// The batch identity includes the run, so the same content may be verified
    /// repeatedly. The true content hash is still recorded.
    PerRun,
}

pub struct IdempotencyLedger {
    conn: Mutex<Connection>,
    scope: BatchScope,
}

impl IdempotencyLedger {
    /// Open a ledger with the production ingestion policy.
    pub fn new(db_path: &str) -> Result<Self> {
        Self::new_scoped(db_path, BatchScope::Exclusive)
    }

    /// Open a ledger with an explicit claim policy.
    ///
    /// [`BatchScope::PerRun`] is used by the verification pipeline: the audit
    /// re-reads the same files on purpose, so the ledger has to accept the same
    /// content again while still recording that the content was seen before.
    pub fn new_scoped(db_path: &str, scope: BatchScope) -> Result<Self> {
        let conn = if db_path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            Connection::open(db_path)?
        };
        let ledger = Self {
            conn: Mutex::new(conn),
            scope,
        };
        ledger.init_db()?;
        Ok(ledger)
    }

    /// The claim policy of this ledger.
    pub fn scope(&self) -> BatchScope {
        self.scope
    }

    pub fn in_memory() -> Result<Self> {
        Self::new(":memory:")
    }

    fn init_db(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA busy_timeout = 5000;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS file_batches (
                batch_id TEXT PRIMARY KEY,
                source_type TEXT NOT NULL,
                filename TEXT NOT NULL,
                file_hash_sha256 TEXT NOT NULL UNIQUE,
                content_sha256 TEXT NOT NULL DEFAULT '',
                record_count INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS transaction_keys (
                tx_key TEXT PRIMARY KEY,
                batch_id TEXT NOT NULL,
                order_no TEXT NOT NULL,
                amount_cents INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(batch_id) REFERENCES file_batches(batch_id)
            );
            CREATE TABLE IF NOT EXISTS quarantine_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                raw_value TEXT NOT NULL,
                error_code TEXT NOT NULL,
                error_reason TEXT NOT NULL,
                quarantined_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS validation_findings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id TEXT NOT NULL,
                rule_code TEXT NOT NULL,
                severity TEXT NOT NULL,
                filename TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                key_value TEXT NOT NULL,
                reason TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_findings_batch ON validation_findings(batch_id);
            CREATE INDEX IF NOT EXISTS idx_findings_rule ON validation_findings(rule_code);
            CREATE TABLE IF NOT EXISTS cumulative_balance (
                customer_id TEXT PRIMARY KEY,
                balance_cents INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_cumulative_balance_customer ON cumulative_balance(customer_id);
            "#
        )?;
        // Backward compatible migration: databases created by earlier versions do
        // not carry the severity / rule code columns.
        Self::ensure_column(
            &conn,
            "quarantine_log",
            "severity",
            "TEXT NOT NULL DEFAULT 'ERROR'",
        )?;
        Self::ensure_column(
            &conn,
            "quarantine_log",
            "rule_code",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        // The content hash is kept next to the claim hash, so a verification run
        // that re-reads a file still leaves the original content traceable.
        Self::ensure_column(
            &conn,
            "file_batches",
            "content_sha256",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        // Batches registered before the split carried the content hash in the
        // claim column; the production policy made the two identical.
        conn.execute(
            "UPDATE file_batches SET content_sha256 = file_hash_sha256 WHERE content_sha256 = ''",
            [],
        )?;
        Ok(())
    }

    /// Add a column when the table does not already carry it.
    fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name.eq_ignore_ascii_case(column) {
                return Ok(());
            }
        }
        conn.execute_batch(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {definition};"
        ))?;
        Ok(())
    }

    pub fn compute_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Register a file batch and claim its content in the ledger.
    ///
    /// Returns `false` when the batch identifier or the content is already
    /// known. With [`BatchScope::PerRun`] the claim is scoped to the run, so
    /// the same content can be verified again; the true content hash is stored
    /// in `content_sha256` either way.
    pub fn register_file_batch(
        &self,
        batch_id: &str,
        source_type: &str,
        filename: &str,
        content: &[u8],
        record_count: usize,
    ) -> Result<bool> {
        let content_hash = Self::compute_sha256(content);
        let file_hash = match self.scope {
            BatchScope::Exclusive => content_hash.clone(),
            BatchScope::PerRun => {
                let mut hasher = Sha256::new();
                hasher.update(b"PER-RUN\0");
                hasher.update(batch_id.as_bytes());
                hasher.update(b"\0");
                hasher.update(content_hash.as_bytes());
                format!("{:x}", hasher.finalize())
            }
        };
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        let res = conn.execute(
            r#"
            INSERT INTO file_batches (batch_id, source_type, filename, file_hash_sha256, content_sha256, record_count, status, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'PROCESSING', ?7)
            "#,
            params![
                batch_id,
                source_type,
                filename,
                file_hash,
                content_hash,
                record_count as i64,
                now
            ],
        );
        match res {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Ok(false) // Duplicate batch hash or duplicate batch_id
            }
            Err(e) => Err(e),
        }
    }

    /// The content hash of a registered batch, independent of the claim policy.
    pub fn content_hash_of(&self, batch_id: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT content_sha256 FROM file_batches WHERE batch_id = ?1")?;
        let mut rows = stmt.query(params![batch_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn check_and_lock_order(
        &self,
        order_no: &str,
        batch_id: &str,
        amount_cents: i64,
    ) -> Result<bool> {
        // With the per run scope the key carries the run, so a verification run
        // does not collide with the production key of the same order number.
        let tx_key = self.order_key(order_no, batch_id);
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        let res = conn.execute(
            r#"
            INSERT INTO transaction_keys (tx_key, batch_id, order_no, amount_cents, status, created_at)
            VALUES (?1, ?2, ?3, ?4, 'LOCKED', ?5)
            "#,
            params![tx_key, batch_id, order_no.trim(), amount_cents, now],
        );
        match res {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Release a batch claim so that a corrected re-send under the same
    /// identifier is accepted.
    ///
    /// A wave that had to isolate rows produced a defective file. Keeping the
    /// claim would force the sender to invent a new batch identifier for the
    /// corrected file, which the operational documentation does not ask for.
    pub fn release_file_batch(&self, batch_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM transaction_keys WHERE batch_id = ?1",
            params![batch_id],
        )?;
        conn.execute(
            "DELETE FROM file_batches WHERE batch_id = ?1",
            params![batch_id],
        )?;
        Ok(())
    }

    pub fn count_file_batches(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM file_batches")?;
        let count: i64 = stmt.query_row((), |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn log_quarantine(
        &self,
        batch_id: &str,
        filename: &str,
        line_number: usize,
        raw_value: &str,
        error_code: &str,
        error_reason: &str,
    ) -> Result<()> {
        self.log_quarantine_with_severity(Finding {
            batch_id,
            rule_code: error_code,
            severity: FindingSeverity::Error,
            filename,
            line_number,
            key_value: raw_value,
            reason: error_reason,
        })
    }

    /// Record an isolated row together with the rule that isolated it.
    pub fn log_quarantine_with_severity(&self, finding: Finding<'_>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO quarantine_log
                (batch_id, filename, line_number, raw_value, error_code, error_reason, quarantined_at, severity, rule_code)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                finding.batch_id,
                finding.filename,
                finding.line_number as i64,
                finding.key_value,
                finding.rule_code,
                finding.reason,
                now,
                finding.severity.as_str(),
                finding.rule_code
            ],
        )?;
        Ok(())
    }

    /// Record a validation finding that did not isolate a row.
    pub fn log_finding(&self, finding: Finding<'_>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO validation_findings
                (batch_id, rule_code, severity, filename, line_number, key_value, reason, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                finding.batch_id,
                finding.rule_code,
                finding.severity.as_str(),
                finding.filename,
                finding.line_number as i64,
                finding.key_value,
                finding.reason,
                now
            ],
        )?;
        Ok(())
    }

    /// Read back the findings of a batch, or of every batch.
    pub fn get_findings(&self, batch_id: Option<&str>) -> Result<Vec<FindingRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if batch_id.is_some() {
            conn.prepare(
                r#"
                SELECT id, batch_id, rule_code, severity, filename, line_number, key_value, reason, created_at
                FROM validation_findings WHERE batch_id = ?1 ORDER BY id
                "#,
            )?
        } else {
            conn.prepare(
                r#"
                SELECT id, batch_id, rule_code, severity, filename, line_number, key_value, reason, created_at
                FROM validation_findings ORDER BY id
                "#,
            )?
        };
        let rows = if let Some(bid) = batch_id {
            stmt.query_map(params![bid], Self::map_finding_row)?
        } else {
            stmt.query_map([], Self::map_finding_row)?
        };
        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }
        Ok(items)
    }

    fn map_finding_row(row: &rusqlite::Row) -> Result<FindingRecord> {
        Ok(FindingRecord {
            id: row.get(0)?,
            batch_id: row.get(1)?,
            rule_code: row.get(2)?,
            severity: row.get(3)?,
            filename: row.get(4)?,
            line_number: row.get::<_, i64>(5)? as usize,
            key_value: row.get(6)?,
            reason: row.get(7)?,
            created_at: row.get(8)?,
        })
    }

    pub fn get_quarantine_items(&self, batch_id: Option<&str>) -> Result<Vec<QuarantineRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if batch_id.is_some() {
            conn.prepare(
                r#"
                SELECT id, batch_id, filename, line_number, raw_value, error_code, error_reason, quarantined_at
                FROM quarantine_log WHERE batch_id = ?1 ORDER BY id
                "#,
            )?
        } else {
            conn.prepare(
                r#"
                SELECT id, batch_id, filename, line_number, raw_value, error_code, error_reason, quarantined_at
                FROM quarantine_log ORDER BY id
                "#,
            )?
        };

        let rows = if let Some(bid) = batch_id {
            stmt.query_map(params![bid], Self::map_quarantine_row)?
        } else {
            stmt.query_map([], Self::map_quarantine_row)?
        };

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }
        Ok(items)
    }

    fn map_quarantine_row(row: &rusqlite::Row) -> Result<QuarantineRecord> {
        Ok(QuarantineRecord {
            id: row.get(0)?,
            batch_id: row.get(1)?,
            filename: row.get(2)?,
            line_number: row.get::<_, i64>(3)? as usize,
            raw_value: row.get(4)?,
            error_code: row.get(5)?,
            error_reason: row.get(6)?,
            quarantined_at: row.get(7)?,
        })
    }

    /// Whether an order was already processed.
    ///
    /// The lookup follows the claim policy of the ledger, so a verification run
    /// does not report the production key of an order as its own.
    pub fn is_order_processed(&self, order_no: &str) -> Result<bool> {
        let tx_key = self.order_key(order_no, "");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT 1 FROM transaction_keys WHERE tx_key = ?1")?;
        let exists = stmt.exists(params![tx_key])?;
        Ok(exists)
    }

    /// The transaction key of an order under the claim policy of the ledger.
    fn order_key(&self, order_no: &str, batch_id: &str) -> String {
        match self.scope {
            BatchScope::Exclusive => format!("ORD:{}", order_no.trim()),
            BatchScope::PerRun => format!("RUN:{batch_id}:ORD:{}", order_no.trim()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_deduplication() {
        let ledger = IdempotencyLedger::in_memory().unwrap();
        let content = b"ORDER_NO,TOTAL\n123,500\n";
        assert!(ledger
            .register_file_batch("B1", "SHOPEE", "orders.csv", content, 1)
            .unwrap());
        // Same content -> duplicate
        assert!(!ledger
            .register_file_batch("B2", "SHOPEE", "orders.csv", content, 1)
            .unwrap());
    }

    #[test]
    fn test_order_locking() {
        let ledger = IdempotencyLedger::in_memory().unwrap();
        ledger
            .register_file_batch("B1", "SHOPEE", "orders.csv", b"sample", 1)
            .unwrap();
        assert!(ledger.check_and_lock_order("ORD-001", "B1", 1000).unwrap());
        assert!(!ledger.check_and_lock_order("ORD-001", "B1", 1000).unwrap());
        assert!(ledger.is_order_processed("ORD-001").unwrap());
        assert!(!ledger.is_order_processed("ORD-999").unwrap());
    }

    #[test]
    fn test_quarantine_logging() {
        let ledger = IdempotencyLedger::in_memory().unwrap();
        ledger
            .log_quarantine(
                "B1",
                "test.csv",
                2,
                "bad_line",
                "ERR_BAD_SYNTAX",
                "Failed arithmetic",
            )
            .unwrap();
        let items = ledger.get_quarantine_items(Some("B1")).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].error_code, "ERR_BAD_SYNTAX");
    }

    #[test]
    fn test_per_run_scope_accepts_the_same_content_again() {
        let ledger = IdempotencyLedger::new_scoped(":memory:", BatchScope::PerRun).unwrap();
        let content = b"ORDER_NO,TOTAL\n123,500\n";
        assert!(ledger
            .register_file_batch("RUN-1", "SHOPEE", "orders.csv", content, 1)
            .unwrap());
        assert!(ledger
            .register_file_batch("RUN-2", "SHOPEE", "orders.csv", content, 1)
            .unwrap());
        // The content hash stays the one of the file, so the evidence of what
        // was verified is not lost by the scoped claim.
        let expected = IdempotencyLedger::compute_sha256(content);
        assert_eq!(
            ledger.content_hash_of("RUN-1").unwrap().as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(
            ledger.content_hash_of("RUN-2").unwrap().as_deref(),
            Some(expected.as_str())
        );
    }

    #[test]
    fn test_per_run_scope_keys_orders_by_run() {
        let ledger = IdempotencyLedger::new_scoped(":memory:", BatchScope::PerRun).unwrap();
        ledger
            .register_file_batch("RUN-1", "SHOPEE", "orders.csv", b"sample", 1)
            .unwrap();
        ledger
            .register_file_batch("RUN-2", "SHOPEE", "orders.csv", b"sample", 1)
            .unwrap();
        assert!(ledger
            .check_and_lock_order("ORD-001", "RUN-1", 1000)
            .unwrap());
        // The same order number in another run is not a duplicate.
        assert!(ledger
            .check_and_lock_order("ORD-001", "RUN-2", 1000)
            .unwrap());
        // Inside one run the order is still claimed once.
        assert!(!ledger
            .check_and_lock_order("ORD-001", "RUN-1", 1000)
            .unwrap());
    }

    #[test]
    fn test_exclusive_scope_still_refuses_the_same_content() {
        let ledger = IdempotencyLedger::in_memory().unwrap();
        let content = b"ORDER_NO,TOTAL\n123,500\n";
        assert!(ledger
            .register_file_batch("RUN-1", "SHOPEE", "orders.csv", content, 1)
            .unwrap());
        assert!(!ledger
            .register_file_batch("RUN-2", "SHOPEE", "orders.csv", content, 1)
            .unwrap());
    }
}
