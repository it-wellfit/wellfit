# Wellfit MTRIX Spec Alignment & Quality Management Plan

> **Disposition (2026-11-02): Historical proposal — not execution-ready.** This document duplicates older MTRIX plans and reports a 5-crate/131-test baseline that conflicts with the current seven-crate manifest and 2026-10-25 state snapshot. See [`../PLAN.md`](../PLAN.md) before deriving any current task.

> The verification table below records a historical result only; it has not been rerun on the current worktree.

## 1. Context

This project is a 5-crate Rust workspace at `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit\` implementing the Alpargatas MTRIX interface (five CSV file types: ALTHCI Customer, ALTHES Inventory, ALTHFV Sales Force, ALTHPD Product, ALTHVD Sales).

## 2. Current State — All Previously Identified Defects Fixed

All P0/P1 defects from prior scans have been fixed and verified:

| Defect | Location | Status |
|--------|----------|--------|
| D-01 | `shopee_engine.rs` | FIXED — `parse_money_cents()` pure integer arithmetic replaces `f64 * 100.0` |
| D-02 | `mtrix_engine.rs` | FIXED — `row_lines` parallel vector removed; row-level line numbers preserved |
| D-03 | `shopee_engine.rs` | FIXED — pure integer parsing replaces `f64 * 100.0` |
| D-04 | `report_automator.rs` | FIXED — `checked_add`/`checked_sub` chain; no silent overflow |
| D-05 | `wellfit_core\src\text.rs` | FIXED — `split_records()` handles CR-only line endings |
| D-06 | `shopee_engine.rs` | FIXED — all column access through contract map; no positional fallbacks |
| H-01 | `mdm.rs` | FIXED — `normalize_store_type("")` returns `""`, not `"Online"` |
| H-02 | `mtrix_engine.rs` | FIXED — VR-08/09/10/11 are separate rules |
| H-03 | `shopee_engine.rs` | FIXED — negative and scientific notation handled |

### Verification Status (ALL PASSING)
| Check | Result |
|-------|--------|
| `cargo build --workspace` | Zero errors |
| `cargo clippy --workspace --all-targets -- -D warnings` | Zero warnings |
| `cargo test --workspace --no-fail-fast` | 131 passed, 0 failed |
| `wellfit mtrix --dir spec/fixtures/mtrix/delivered` | 5 files, 22 errors, 29 warnings, 37 repairs |
| `wellfit audit --data-dir data` | 5 files, 56 fields, 15 rules |
| `wellfit spec --format markdown` | Complete 56-field dictionary |

### DEMO Verification (All Six Commands Working)
| Command | Result |
|---------|--------|
| `wellfit status` | 0 isolated records, 0 validation errors |
| `wellfit shopee --input data/shopee_orders.csv --output ...` | 2 orders accepted, 0 isolated |
| `wellfit reports --input data/fuyi_inventory_report.csv --recipient finance@dachen.com.tw` | 4 records processed, 0 isolated |
| `wellfit mtrix --dir spec/fixtures/mtrix/delivered --output-dir ...` | 5 files processed, 22 errors, 29 warnings, 37 repairs |
| `wellfit audit --data-dir data` | 5 files, 56 fields, 15 rules |
| `wellfit spec --format markdown` | Complete 56-field dictionary |

---

## 3. Remaining Work Items

### Phase 1: Remaining C Defects (P0)

**T1: LOCAL-048 — Account/Counterparty Cumulative Scope in Ledger**
**File:** `crates/wellfit_core\src\mdm.rs`
**Current state:** `IdempotencyLedger::new()` takes no arguments and returns `Self` (not `Result<Self, rusqlite::Error>`). It opens an in-memory SQLite database on construction.
**Fix:** Change `IdempotencyLedger::new()` signature from `pub fn new() -> Self` to `pub fn new(db_path: &str) -> Result<Self, rusqlite::Error>` so:
- The database path can be specified per invocation
- Database open errors are returned as `Result` rather than causing a panic
- Supports multiple batch contexts with different database files if needed

Also add cumulative balance tracking:
- Add `cumulative_balance: i64` field to `SkuMetadata`
- Add `MdmCatalog::register_cumulative(batch_id: &str, customer_id: &str, amount_cents: i64) -> Result<(), rusqlite::Error>` method
- Add `MdmCatalog::get_cumulative_balance(&self, customer_id: &str) -> Result<Option<i64>, rusqlite::Error>` method
- Add SQL table `cumulative_balance(customer_id TEXT PRIMARY KEY, balance_cents INTEGER NOT NULL DEFAULT 0, updated_at TEXT NOT NULL)`
- Add test: `test_cumulative_balance_accumulates_across_batches()` verifying balances accumulate across batches

**T2: LOCAL-049 — Release Batch Claim When Rows Are Isolated**
**File:** `crates\wellfit_mtrix\src\mtrix_engine.rs` (around line 547)
**Current state:** `release_file_batch` exists on `IdempotencyLedger` but is NOT called in `process_drop` when rows are quarantined.
**Fix:** In `process_drop()`, after processing all files, if any row was quarantined (`repairs.applied > 0` or `errors > 0`), call `self.ledger.release_file_batch(batch_id)` so that a corrected re-send under the same batch ID is accepted.
**Verification:** Add test `test_corrected_resend_under_same_batch_is_accepted`.

**T3: LOCAL-053 — Report vs Ledger Count Discrepancy Documentation**
**Files:** `crates\wellfit_cli\src\main.rs`, `docs/CLI.md`, `AGENTS.md`
**Problem:** The audit report's `error_count`/`warning_count` fields do not match what `wellfit status` reports, because they count different scopes:
- `audit_report.json` reports per-batch counts (records processed in this batch run)
- `wellfit status` reports ledger-level cumulative counts (all batches ever processed)
**Fix:**
- Add `ledger_state` field to `AuditReport` containing `batches_registered`, `isolated_records`, `validation_errors`, `validation_warnings`
- Update `docs/CLI.md` to document the distinction between per-batch audit counts and cumulative ledger counts
- Update `AGENTS.md` to mention the `status` command reflects cumulative ledger state

---

### Phase 2: Documentation & Knowledge Base

**T10: Update AGENTS.md**
- Add all CLI subcommands with flags
- Document environment variables (`wellfit_ledger.sqlite` location)
- Document release gate states and exit codes
- Document defect fixes and known limitations

**T11: Update docs/ARCHITECTURE.md**
- Add three-stage pipeline diagram (accept → repair → validate)
- Add Release Gate state machine diagram
- Add data flow diagram showing five-file flow

**T12: Update docs/CLI.md**
- Document `--mtrix-dir` option for `mtrix` subcommand
- Document all subcommands with examples
- Document exit codes (0=success, 1=error, 2=quarantined)
- Document environment variables

**T13: Update docs/DEMO.md**
- Add complete end-to-end demo with expected outputs
- Include error scenarios and expected behavior

---

### Phase 3: Final Verification

**T14: Final Verification Suite**
- `cargo build --workspace` — zero errors
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- `cargo test --workspace --no-fail-fast` — all tests pass
- All six DEMO commands run successfully
- `verify-claims.ps1` — every stated figure matches software output
- `mutation-check.ps1` — all mutations killed

---

## 8. Definition of Done

- [ ] LOCAL-048 implemented and tested
- [ ] LOCAL-049 implemented and tested
- [ ] LOCAL-053 documented
- [ ] `cargo build --workspace` passes with zero errors
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with zero warnings
- [ ] `cargo test --workspace --no-fail-fast` passes with 131+ passed, 0 failed
- [ ] All six DEMO commands run successfully
- [ ] Documentation updated (AGENTS.md, docs/ARCHITECTURE.md, docs/CLI.md, docs/DEMO.md)

---
