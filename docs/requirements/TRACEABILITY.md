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
|-----------|------|------|------|
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

## 6. BankPay 需求追溯（E.Sun V2.8）

| 需求 ID | 需求 | 設計文件 | 介面規格 | 實作位置 | 測試 | 狀態 |
|---------|------|----------|----------|----------|------|------|
| BP-01 | TWD 付款放行 | `docs/bankpay/03-technical-design.md` §2 | `docs/bankpay/04-interface-specification.md` §2 | `crates/wellfit_bankpay/src/bankpay.rs` `export_vouchers` / `build_twd_payment_request` | `test_export_vouchers_dedup_and_lock`, `test_export_isolates_bad_amount`, `test_assemble_media_file_and_sha256` | 已實作（mock 可測） |
| BP-02 | TWD 交易狀態查詢 | `docs/bankpay/03-technical-design.md` §2 | `docs/bankpay/04-interface-specification.md` §3 | `crates/wellfit_bankpay/src/http.rs` `retrieve_twd_status` / `crates/wellfit_bankpay/src/bankpay.rs` `query_twd_status` | `test_query_voucher`, `test_query_tx`, `test_query_batch` | 已實作（mock 可測） |
| BP-03 | 交易明細查詢 | `docs/bankpay/03-technical-design.md` §2 | `docs/bankpay/04-interface-specification.md` §4 | 未整合進 `bankpay.rs`（HTTP client 現有 `post_json` 可複用） | — | P3 後續 |
| BP-04 | FX 預檢與放行 | `docs/bankpay/04-interface-specification.md` §5-6 | `docs/bankpay/04-interface-specification.md` §5-6 | `api.rs` 已定義 `validateFxPaymentInstruction` 模型，未整合進引擎 | — | P3 後續 |
| BP-05 | 帳戶餘額查詢 | `docs/bankpay/04-interface-specification.md` §7 | `docs/bankpay/04-interface-specification.md` §7 | 未整合進 `bankpay.rs` | — | P3 後續 |
| BP-06 | 付款結果列印 | `docs/bankpay/04-interface-specification.md` §9 | `docs/bankpay/04-interface-specification.md` §9 | 未整合進 `bankpay.rs` | — | P3 後續 |
| BP-07 | 媒體檔 422-char BIG-5 | `docs/bankpay/03-technical-design.md` §3 | `docs/bankpay/04-interface-specification.md` §1 | `crates/wellfit_bankpay/src/media.rs` `assemble_twd_media_file` | `test_assemble_media_file_and_sha256`, `test_base64_roundtrip` | 已實作 |
| BP-08 | 編碼轉換 UTF-8↔CP950/Big5 | `docs/bankpay/03-technical-design.md` §1 | `docs/bankpay/04-interface-specification.md` §1 | `crates/wellfit_bankpay/src/encoding.rs` `to_utf8` / `from_utf8` | `test_detect_utf8_bom`, `test_from_utf8_round_trip`, `test_cp950_label` | 已實作 |
| BP-09 | 冪等鍵防重複 | `docs/bankpay/03-technical-design.md` §5 | `docs/bankpay/08-open-questions-and-decisions.md` ADR-BP-003 | `crates/wellfit_bankpay/src/bankpay.rs` `export_vouchers` (idempotency lock) | `test_export_vouchers_dedup_and_lock` | 已實作 |
| BP-10 | 發布閘門狀態機 | `docs/bankpay/03-technical-design.md` §6 | `docs/bankpay/05-security-and-audit.md` §6.3 | `crates/wellfit_bankpay/src/release.rs` / `crates/wellfit_core/src/release_gate.rs` | `test_verify_and_dispatch_flow`, `test_uncertain_dispatch_reconciliation`, `test_reject_quarantines` | 已實作 |
| BP-11 | 追溯查詢 | `docs/bankpay/03-technical-design.md` §9 | `docs/bankpay/04-interface-specification.md` §1 | `crates/wellfit_bankpay/src/query.rs` `query` | `test_query_voucher`, `test_query_tx`, `test_query_batch`, `test_query_rejects_unknown_kind` | 已實作 |
| BP-12 | 稽核 Finding | `docs/bankpay/03-technical-design.md` §5 | `docs/bankpay/05-security-and-audit.md` §6 | `crates/wellfit_bankpay/src/bankpay.rs` `log_finding` / `log_quarantine` | `test_import_results_records_findings` | 已實作 |
| BP-13 | API 結果代碼 | `docs/bankpay/05-security-and-audit.md` §4 | `docs/bankpay/04-interface-specification.md` §11 | `crates/wellfit_bankpay/src/api.rs` `api_result` | `test_api_result_success`, `test_api_response_failure_no_data` | 已實作 |
| BP-14 | 交易狀態代碼 | `docs/bankpay/05-security-and-audit.md` §5 | `docs/bankpay/04-interface-specification.md` §11 | `crates/wellfit_bankpay/src/api.rs` `tx_status` | `test_tx_status_success`, `test_tx_status_pending`, `test_tx_status_failed` | 已實作 |

## 7. BankPay 核心不變量

| 不變量 | 實作位置 | 測試 |
|--------|----------|------|
| 零 `unsafe` | 每個 `.rs` 第一行 `#![forbid(unsafe_code)]` | clippy + cargo check |
| Clippy 零警告 | `cargo clippy --workspace --all-targets -- -D warnings` | CI |
| 金額不用浮點 | `ScaledDecimal`（`wellfit_core::scaled`） | `test_no_binary_floating_point_drift` |
| 欄位以名稱解析 | `ColumnContract`（`wellfit_core::contract`） | `test_never_falls_back_to_a_position` |
| 批次冪等（批次 ID + SHA-256） | `IdempotencyLedger`（`wellfit_core::ledger`） | `test_the_same_drop_cannot_be_ingested_twice` |
| API 結果代碼 910000=成功 | `api.rs` `api_result::is_success` | `test_api_result_success` |
| 媒體檔固定長度 422 | `media.rs` `assemble_twd_media_file` | `test_assemble_media_file_and_sha256` |

## 8. 未完成 / 受阻需求（BankPay）

| 需求/議題 | 編號 | 狀態 | 阻礙 |
|-----------|------|------|------|
| HTTP 客戶端憑證釘選 | BP-TLS | 待實作 | `http.rs` `EsunHttpClient` 有 `client_cert_path` 等欄位，但 `post_json` 尚未載入 rustls 憑證；需銀行憑證可用後實作 |
| FlapsErp COM 介面 | OQ-E3 | 待確認 | 需向 ERP 廠商取得 ProgID/方法，或確認 HTTP API 可取代 |
| 沙箱認證與 UAT | OQ-E3 | 待銀行交付 | E.Sun 帳戶經理提供 SourceSystemId、ClientId、啟用碼、憑證 |
| 整合測試（mock 元件） | BP-INT | 待建置 | 根據 V2.8 規格建構 mock 安全元件供 CI 測試 |
| FX 波次整合 | BP-04, BP-05, BP-06 | P3 後續 | TWD 優先；FX 為後續波次 |

