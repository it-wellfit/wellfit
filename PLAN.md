# Wellfit MTRIX Spec Alignment & Knowledgebase Build Plan

## Context

5-crate Rust workspace at `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit\` implementing the Alpargatas MTRIX interface (five CSV file types: ALTHCI Customer, ALTHES Inventory, ALTHFV Sales Force, ALTHPD Product, ALTHVD Sales).

## Current State — All Previously Identified Defects FIXED

| Defect | Location | Status |
|--------|----------|--------|
| D-01 VR-05 aggregation bug | `mtrix_engine.rs:730-743` | FIXED — uses `current` value, not re-reading from row |
| D-02 Cross-file line number reporting | `mtrix_engine.rs:831-852` | FIXED — `row_lines.get(offset).copied().unwrap_or(offset + 2)` |
| D-03 Floating-point money parsing | `shopee_engine.rs:124` | FIXED — `parse_money_cents()` pure integer i64 arithmetic |
| D-04 Report integer overflow | `report_automator.rs:46` | FIXED — `checked_add`/`checked_sub` chain |
| D-05 CR-only line endings | `wellfit_core\src\text.rs` | FIXED — `split_records` handles CR/LF/CRLF |
| D-06 Positional fallback in shopee | `shopee_engine.rs` | FIXED — all column access through contract map |
| H-01 MDM `normalize_store_type("")` | `mdm.rs` | FIXED — returns `""`, not `"Online"` |
| H-02 VR-08 conflation | `mtrix_engine.rs` | FIXED — VR-08/Customer, VR-09/Batch, VR-10/Salesperson are separate |
| H-03 Shopee `parse_money_cents` | `shopee_engine.rs` | FIXED — handles negative and scientific notation |

### Verification Status (ALL PASSING)
- `cargo build --workspace` — zero errors
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- `cargo test --workspace --no-fail-fast` — 131 passed, 0 failed
- `wellfit mtrix` — 5 files, 22 errors, 29 warnings, 37 repairs
- `wellfit audit` — 5 files, 56 fields, 15 rules
- `wellfit spec` — complete 56-field dictionary

## Remaining Work

### Phase 1: Remaining P0/P1 Fixes

**T1: LOCAL-048 — Account/Counterparty Cumulative Scope in Ledger**
File: `crates/wellfit_core\src\ledger.rs`
Current: `IdempotencyLedger` tracks per-batch metadata (file_hash, record_count) but does NOT track cumulative balances per customer/account across batches.
Fix:
- Add `cumulative_balance` column to `quarantine_log` or a new `cumulative_balance` table
- Add `MdmCatalog::register_cumulative(batch_id, customer_id, amount_cents)`
- Add `MdmHub::get_cumulative_balance(customer_id: &str) -> Option<i64>`
- Add test: `test_cumulative_balance_accumulates_across_batches()`

### Phase 2: Documentation & Knowledge Base

**T2: LOCAL-053 — Report vs Ledger Count Discrepancy Documentation**
Files: `crates\wellfit_cli\src\main.rs`, `docs/CLI.md`, `AGENTS.md`
The audit report's per-batch counts (errors, warnings) differ from `wellfit status` ledger counts because they measure different scopes (per-batch vs cumulative). Document this distinction clearly.

**T13: Update AGENTS.md** — Document all CLI subcommands, environment variables, release gate states
**T14: Update docs/ARCHITECTURE.md** — Add three-stage pipeline diagram, Release Gate state machine, data flow diagram

---

## 7. Verification Criteria

| Check | Expected |
|-------|----------|
| `cargo build --workspace` | Zero errors |
| `cargo clippy --workspace --all-targets -- -D warnings` | Zero warnings |
| `cargo test --workspace --no-fail-fast` | 131+ passed, 0 failed |
| `wellfit mtrix --dir spec/fixtures/mtrix/delivered` | 5 files, 22 errors, 29 warnings, 37 repairs |
| `wellfit audit --data-dir data` | 5 files, 56 fields, 15 rules |
| `wellfit spec --format markdown` | Complete 56-field dictionary |
| `wellfit shopee --input data/shopee_orders.csv` | 2 orders accepted, 0 isolated |
| `wellfit reports --input data/fuyi_inventory_report.csv --recipient finance@dachen.com.tw` | 4 records processed, 0 isolated |

---

## 8. Next Steps

1. Implement LOCAL-048 (cumulative balance tracking in ledger)
2. Implement LOCAL-049 (release batch claim when rows isolated)
3. Implement LOCAL-053 (document report vs ledger count distinction)
4. Update AGENTS.md, docs/ARCHITECTURE.md, docs/CLI.md
5. Run full verification suite
