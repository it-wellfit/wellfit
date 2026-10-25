# Traceability Matrix — Wellfit

> 文件編號：WFLT-DOC-REQ-001｜狀態：Active｜版本：v1.0｜日期：2026-10-25
> 需求 ID 對應 `Knowledge/issues/mtx-issue-register.md` 與 `Knowledge/specs/mtrix-vr-rules.md`

## 1. MTRIX 驗證規則（VR-01..VR-15）

| 需求 ID | 規則 | 實作位置 | 測試 | 狀態 |
|---|---|---|---|---|
| VR-01 | 檔名格式 | `mtrix_engine.rs` (check_filename), `validators.rs` | `test_filename_pattern_*`, `test_delivered_fixtures_produce_the_documented_findings` | 已實作 |
| VR-02 | 標題比對 | `mtrix_engine.rs` (compare_header), `validators.rs` | `test_header_comparison_accepts_manual_and_alias_wording`, `test_header_mismatch_rejects_only_that_file` | 已實作 |
| VR-03 | 日期合法性 | `validators.rs` (is_valid_yyyymmdd), `mtrix_engine.rs` (validate_row) | `test_calendar_date_validation` | 已實作 |
| VR-04 | DOCUMENT_TYPE 允許值 | `mtrix_engine.rs` (validate_row, format_warnings) | `test_derived_cases_exercise_the_row_rules` | 已實作 |
| VR-05 | 重複交易彙總 | `mtrix_engine.rs` (aggregate_duplicates) | `test_duplicate_transaction_sums_the_quantities`, `test_duplicate_quantity_overflow_fails_instead_of_reusing_prior_value` | 已實作 |
| VR-06 | 負庫存歸零 | `mtrix_engine.rs` (negative_measure) | `test_negative_stock_is_clamped_with_a_warning_rule` | 已實作 |
| VR-07 | EAN13/DUN14 條碼 | `mdm.rs` (is_ean13, is_dun14, validate_barcode) | `test_strict_barcode_validation`, `test_barcode_length_must_match_the_declared_type` | 已實作 |
| VR-08 | 客戶存在性（跨檔） | `mtrix_engine.rs` (apply_cross_file_rules) | `test_derived_cases_exercise_the_row_rules` (case-d) | 已實作 |
| VR-09 | 商品存在性（跨檔） | `mtrix_engine.rs` (apply_cross_file_rules) | `test_derived_cases_exercise_the_row_rules` | 已實作 |
| VR-10 | 業務員存在性（跨檔） | `mtrix_engine.rs` (apply_cross_file_rules) | `test_derived_cases_exercise_the_row_rules` | 已實作 |
| VR-11 | 庫存-商品一致性（跨檔） | `mtrix_engine.rs` (apply_cross_file_rules) | `test_derived_cases_exercise_the_row_rules` | 已實作 |
| VR-12 | 幣別分組彙總 | `mtrix_engine.rs` (currency_groups) | `test_finding_line_numbers_follow_the_delivered_file` (partial) | 部分（缺管理報表核對，G-07） |
| VR-13 | 水位對齊 | `mtrix_engine.rs` | `test_a_quarantined_row_does_not_shift_the_line_numbers` | 已實作 |
| VR-14 | 業務檔孤島客戶 | `mtrix_engine.rs` (apply_cross_file_rules) | `test_derived_cases_exercise_the_row_rules` (case-d) | 已實作 |
| VR-15 | Mandatory 欄位空值 Warning | `mtrix_engine.rs` (format_warnings), `config.rs` | `test_derived_cases_exercise_the_row_rules` | 已實作（清單為 L2 推薦，MTRIX-003 待對接方） |

## 2. 修復規則（ANOM-01..ANOM-12）

| 需求 ID | 修復 | 實作位置 | 測試 |
|---|---|---|---|
| ANOM-01 | 尾端空欄移除 | `repairs.rs` (repair_header) | `test_header_emits_canonical_names` |
| ANOM-02 | 標題正規化 | `repairs.rs` (repair_header) | `test_header_emits_canonical_names` |
| ANOM-03 | 欄位別名對應 | `repairs.rs` (repair_header) | `test_header_comparison_accepts_manual_and_alias_wording` |
| ANOM-04 | 條碼前導零保留 | `mdm.rs` (normalize_barcode), `repairs.rs` | `test_barcode_normalization_keeps_leading_zeros` |
| ANOM-05 | 廠商國碼前綴分離 | `repairs.rs` (split_vendor_country_prefix) | `test_vendor_prefix_is_split_only_when_present`, `test_vendor_prefix_accepts_two_and_three_letter_forms` |
| ANOM-06 | DIST_ID 去遮罩 | `mdm.rs` (normalize_dist_id) | `test_dist_id_dot_mask_is_removed` |
| ANOM-07 | STORE_TYPE 別名 | `mdm.rs` (normalize_store_type) | `test_store_type_alias_onlinne`, `test_empty_store_type_stays_empty` |
| ANOM-08 | 負庫存歸零 | `mtrix_engine.rs` (negative_measure) | `test_negative_stock_is_clamped_with_a_warning_rule` |
| ANOM-09 | 條碼修復不作用於識別碼 | `repairs.rs` | `test_barcode_repair_does_not_touch_identifier_columns` |
| ANOM-10 | 重複交易彙總 | `mtrix_engine.rs` (aggregate_duplicates) | `test_duplicate_transaction_sums_the_quantities` |
| ANOM-11 | 修復日誌 before/after | `repairs.rs` (AppliedRepair, RepairLog) | `test_repaired_output_keeps_the_contract_and_the_fixes` |
| ANOM-12 | 標題 deviations 登錄 | `mtrix_engine.rs` (report.deviations) | `test_deviations_are_flagged_and_never_dropped` |

## 3. 核心不變量

| 不變量 | 實作位置 | 測試 |
|---|---|---|
| EX-06 金額守恆（整數分） | `shopee_engine.rs` (verify_line_sum, parse_money_cents) | `test_ex06_sum_mismatch`, `test_money_parsing_is_integer_only` |
| 庫存恆等式 | `report_automator.rs` | `test_inventory_math_mismatch`, `test_valid_inventory_report` |
| 56 欄契約 | `spec.rs` (MTRIX_SPEC, TOTAL_FIELDS) | `test_total_field_count_is_56`, `test_five_files_declared` |
| 欄位以名稱解析（無序位退讓） | `contract.rs` (ColumnContract::resolve) | `test_never_falls_back_to_a_position`, `test_columns_are_resolved_by_name_and_never_by_position` |
| CR-only 行尾支援 | `text.rs` (split_lines) | `test_carriage_return_terminated_files_are_processed` |
| 隔離列行號不偏移 | `mtrix_engine.rs` (row_lines) | `test_a_quarantined_row_does_not_shift_the_line_numbers` |
| 批次冪等（批次 ID + SHA-256） | `ledger.rs` (register_file_batch) | `test_the_same_drop_cannot_be_ingested_twice`, `test_a_corrected_drop_can_be_resent_under_the_same_batch` |
| 金額零浮點 | `scaled.rs` (ScaledDecimal), `shopee_engine.rs` | `test_no_binary_floating_point_drift`, `test_erp_import_csv_formats_cents_without_floating_point` |
| 跨檔 finding 與帳本一致 | `mtrix_engine.rs` (record_finding) | `test_every_reported_finding_is_recorded_in_the_ledger` |

## 4. 工程品質

| 項目 | 實作位置 | 測試/驗證 |
|---|---|---|
| 固定檢查入口 | `scripts/check.ps1` | green（fmt + clippy + tests + fixtures + golden parity） |
| 變動檢查（mutation） | `scripts/mutation-check.ps1` | green（tracked files byte-identical） |
| 備份與還原 | `git bundle` + `Documents\wellfit-backups\` | bundle verify + restore drill 成功 |
| CI 工作流 | `.github/workflows/ci.yml`, `security.yml` | 定義（未執行於遠端） |
| 故障注入 | `mtrix_engine.rs` tests (fault_injection_*) | 3 tests passed |
| 安全掃描 | `deny.toml`, `cargo audit`, `trufflehog` | CI 定義 |

## 5. 未完成 / 受阻需求

| 需求/議題 | 編號 | 狀態 | 阻礙 |
|---|---|---|---|
| CP 格式驗證 | C-01 / MTRIX-001 | 待對接方 | CP 格式未定義 |
| VR-12 管理報表核對 | C-02 / G-07 | 待對接方 | 管理報表來源未取得 |
| 逐欄 Mandatory 清單 | C-03 / MTRIX-003 | 待對接方 | 清單不可得 |
| 小數位數統一 | C-04 / MTRIX-024 | 待裁決 | 2 位 vs 4 位 |
| 廠商前綴長度 | C-05 / G-11 | 待對接方 | 2 vs 3 字母 |
| 輸出 CR/LF 與標題政策 | C-06 | 已決策（CR default, Canonical header） | — |
| scaled.rs 文件說明 | C-07 | 已完成 | — |
| ALTHCI 第 12 欄狀態 | C-08 | 已完成（ObservedOnly） | — |
| 管線接線 | Phase 2 | 未開始 | 需重構 process_drop |
| 規則/欄位外置化 | Phase 3 | 未開始 | 需設計 TOML/JSON schema |

## 6. 追溯鏈
需求 → 規格（`Knowledge/specs/`）→ 架構（`docs/ARCHITECTURE.md`）→ Ticket（`PLAN.md`）→ 程式碼（`crates/`）→ 測試（`#[test]`）→ 驗收證據（`scripts/check.ps1` golden parity）→ 版本（git tag `v1.0.6`）

## 7. 更新條件
本矩陣應在每次新增/修改需求、規則或測試時同步更新。每次版本發布後應重新驗證 golden parity。
