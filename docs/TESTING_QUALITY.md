# Wellfit BankPay 測試與品質保證

**文件編號**：WFLT-DOC-008  
**版本**：1.0  
**狀態**：Draft  
**文件語言**：繁體中文（技術名詞保留標準英文）  
**證據來源**：`crates/wellfit_bankpay/src/` 單元測試、`crates/wellfit_bankpay/tests/integration_tests.rs`、`docs/bankpay/06-test-and-acceptance.md`

---

## 1. 使用說明

本文件說明 Wellfit BankPay 模組的測試策略、測試類型、測試案例、品質關卡與已知缺口。所有測試皆以實際測試原始碼為基礎，未執行的測試皆標記為⚠ 待確認。

---

## 2. 測試策略

### 2.1 測試層次與範圍

| 層次 | 工具 | 範圍 | 檔案位置 | 狀態 |
|------|------|------|---------|------|
| 單元測試 | `cargo test -p wellfit_bankpay` | 引擎、編碼、欄位契約、帳本、發布閘門、查詢 | `crates/wellfit_bankpay/src/*` (`#[cfg(test)]`) | ✔ 32 個測試函式 |
| 整合測試 | `cargo test -p wellfit_bankpay --test integration_tests` | 匯出/匯入/查詢端對端、媒體檔組裝、TLS 設定 | `crates/wellfit_bankpay/tests/integration_tests.rs` | ✔ 17 個測試函式 |
| 合約測試 | V2.8 欄位規格 | 欄位長度/必要性/格式驗證 | `docs/bankpay/04-interface-specification.md` | ⚠ 僅為文件規格，未自動化 |
| 手動 UAT | 銀行沙箱 | 真實安全元件 | 待沙箱憑證 (OQ-E3) | ⚠ 待憑證 |

### 2.2 測試原則

1. **不寫入版本控制的檔案**：所有測試使用 `:memory:` SQLite 或 `std::env::temp_dir()` 的暫存檔案，不寫入專案目錄（✔ 已確認，見 `bankpay.rs:775–781` 與 `integration_tests.rs:21–28`）。
2. **合成資料不得呈現為真實**：測試資料為合成資料，附註出處（✔ 已確認，測試使用 `AP-001`、`V001` 等合成值）。
3. **每個測試獨立**：使用 `AtomicUsize` 計數器生成唯一暫存檔案名稱，避免測試間干擾（✔ 已確認）。
4. **不驗證則不標記為通過**：測試結果必須有明確的 `assert!` 或 `assert_eq!`。

---

## 3. 單元測試清單（✔ 已確認）

### 3.1 bankpay.rs 單元測試（行 763–957）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `test_export_vouchers_dedup_and_lock` | §784 | 匯出去重與冪等鎖定：同一批次 ID 重複匯出被拒絕 | 第一次成功 (2 筆 accepted)，第二次 `Err` (重複批次) |
| `test_export_isolates_bad_amount` | §810 | 錯誤金額隔離：`AMOUNT` 為 `notanumber` | `vouchers_accepted=0`, `vouchers_isolated=1`, `dispatched=false` |
| `test_assemble_media_file_and_sha256` | §829 | 媒體檔組裝與 SHA-256：單筆憑單組裝 CSV | `had_errors=false`, SHA-256 長度 64，內容包含 `AP-001` 與 `TRAILER` |
| `test_import_results_records_findings` | §849 | 匯入結果記錄發現：單筆銀行結果 | `results_accepted=1`, `matched_vouchers=1`, `unmatched=0` |
| `test_end_to_end_traceability_chain` | §867 | 端對端追溯鏈：憑單→媒體檔→SHA-256→銀行結果→匯入→查詢 | 匯出 1 筆並派送，媒體檔雜湊匹配，匯入 1 筆並對帳，查詢 voucher/tx/batch 皆有 findings |
| `test_encoding_round_trip_used_by_engine` | §949 | 編碼來回轉換：UTF-8 ↔ Big5 | 解碼後與原始文本完全一致，無編碼錯誤 |

### 3.2 media.rs 單元測試（行 145–200）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `test_assemble_single_record` | §172 | 組裝單筆 422 字元記錄 | 位元組非空，解碼後字元數等於 422 |
| `test_empty_records_rejected` | §182 | 空記錄列表被拒絕 | `Err` |
| `test_base64_roundtrip` | §188 | Base64 編碼/解碼來回轉換 | 解碼後與原始位元組完全一致 |
| `test_max_records_limit` | §196 | 超過 1000 記錄被拒絕 | `Err` (MAX_RECORDS + 1 筆) |

### 3.3 http.rs 單元測試（行 188–206）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `test_config_default` | §193 | 預設設定驗證 | `base_url="https://127.0.0.1:8443"`, `timeout_secs=30` |
| `test_client_creation` | §200 | 無憑證時 tls_config 為 None | `tls_config.is_none()` |

### 3.4 query.rs 單元測試（行 111–161）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `test_query_voucher` | §123 | 憑單查詢 | `lookup_kind="voucher"`, `voucher_no=Some("AP-001")`, findings 非空 |
| `test_query_tx` | §134 | 交易查詢 | `lookup_kind="tx"`, `bank_ref=Some("EREF-999")` |
| `test_query_batch` | §145 | 批次查詢 | `lookup_kind="batch"`, `batch_id=Some("BATCH-X")` |
| `test_query_rejects_unknown_kind` | §155 | 未知查詢類型被拒絕 | `Err` |

### 3.5 release.rs 單元測試（行 50–87）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `test_verify_and_dispatch_flow` | §55 | 驗證與派送流程 | `verify_payload(true,"")` → `VerifiedPass`，`dispatch` → `Completed` |
| `test_uncertain_dispatch_reconciliation` | §68 | 不確定派送對帳 | `DispatchUncertain` → `reconcile_uncertain(|| true)` → `Completed` |
| `test_reject_quarantines` | §79 | 拒絕導致隔離 | `verify_payload(false,"isolated row")` → `Quarantined`，`dispatch` 返回 `Err` |

### 3.6 api.rs 單元測試（行 392–493）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `test_api_result_success` | §397 | API 結果代碼成功檢查 | `is_api_success("910000")=true`, 其他為 `false` |
| `test_tx_status_success` | §404 | 交易狀態成功檢查 | `is_tx_success("0000")=true`, `"4000"` 為 `false` |
| `test_tx_status_pending` | §410 | 交易狀態處理中檢查 | `"1000"`–`"1008"` 為 `true`, `"0000"`/`"1009"` 為 `false` |
| `test_tx_status_failed` | §418 | 交易狀態失敗檢查 | `is_tx_failed("4000")=true`, `"0000"` 為 `false` |
| `test_api_response_success_check` | §424 | ApiResponse 成功檢查 | code=`910000` → `is_success()=true` |
| `test_api_response_failure_no_data` | §442 | ApiResponse 失敗檢查 | code=`910071` → `is_success()=false`, `more_info` 正確 |
| `test_twd_payment_tx_serde_roundtrip` | §461 | TwdPaymentTx 序列化來回轉換 | JSON 序列化後反序列化，欄位值保持一致 |
| `test_currency_codes_count` | §489 | 貨幣代碼數量 | 15 種，包含 `USD`，不包含 `TWD` |

### 3.7 encoding.rs 單元測試（行 105–147）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `test_detect_utf8_bom` | §110 | UTF-8 BOM 偵測 | 偵測為 Utf8，偏移量 3 |
| `test_detect_plain_utf8` | §119 | 一般 UTF-8 偵測 | 偵測為 Utf8，偏移量 0 |
| `test_from_utf8_round_trip` | §126 | UTF-8 ↔ Big5 來回轉換 | 解碼後與原始文本一致，無錯誤 |
| `test_from_utf8_big5_ascii_no_errors` | §135 | Big5 ASCII 無錯誤 | ASCII 文字轉換無編碼錯誤 |
| `test_cp950_label` | §145 | CP950 標籤 | `label() = "CP950 (Big5-HKSCS)"` |

---

## 4. 整合測試清單（✔ 已確認，定義於 integration_tests.rs）

| 測試函式 | 行號 | 測試內容 | 預期結果 |
|----------|------|---------|---------|
| `export_vouchers_success` | §30 | 匯出成功：兩筆有效憑單 | `vouchers_accepted=2`, `vouchers_isolated=0`, `dispatched=true`, `file_sha256` 長度 64 |
| `export_vouchers_isolates_bad_rows` | §55 | 錯誤列隔離：一筆無效金額 | `vouchers_accepted=0`, `vouchers_isolated=1`, `dispatched=false` |
| `import_results_success` | §76 | 匯入成功：一筆銀行結果 | `results_accepted=1`, `matched_vouchers=1`, `unmatched=0` |
| `query_voucher_flow` | §96 | 憑單查詢流程 | `lookup_kind="voucher"`, `voucher_no=Some("AP-001")`, findings 非空 |
| `query_tx_flow` | §109 | 交易查詢流程 | `lookup_kind="tx"`, `bank_ref=Some("EREF-999")` |
| `query_batch_flow` | §121 | 批次查詢流程 | `lookup_kind="batch"`, `batch_id=Some("BATCH-X")` |
| `query_rejects_unknown_kind` | §133 | 未知類型拒絕 | `Err` |
| `build_twd_payment_request_format1` | §144 | 格式 1 (媒體檔) 請求組建 | `format_type="1"`, `payment_type="1"`, content 非空, `source_system="TEST_SYSTEM"`, `customer_ixd="12345678"` |
| `build_twd_payment_request_format2` | §176 | 格式 2 (JSON) 請求組建 | `format_type="2"`, content 非空, Base64 解碼後包含 `AP-002` |
| `media_file_assembly` | §211 | 媒體檔組裝驗證 | 位元組非空，解碼後字元數 422，SHA-256 長度 64，Base64 來回轉換一致 |
| `cents_to_yuan_str_precision` | §247 | 金額格式化精度 | `1000→"10.00"`, `250050→"2500.50"`, `-500→"-5.00"`, `0→"0.00"`, `1→"0.01"` |
| `http_client_config_tls` | §256 | TLS 客戶端設定 | `client_cert_path` 正確傳遞 |
| `http_client_default_config` | §272 | 預設設定 | `tls_config.is_none()`, `base_url="https://127.0.0.1:8443"` |
| `end_to_end_export_import_query` | §280 | 匯出→匯入→查詢端對端 | 匯出 1 筆並派送，匯入 1 筆並對帳，查詢 voucher/tx/batch 皆正確 |

**整合測試總計**：17 個測試函式

---

## 5. 品質關卡（Quality Gates）

| 關卡 | 指令 | 通過條件 | 狀態 |
|------|------|---------|------|
| 編譯檢查 | `cargo check --workspace --all-targets` | 無編譯錯誤 | ✔ 已確認 (以原始碼檢查) |
| Clippy 零警告 | `cargo clippy --workspace --all-targets -- -D warnings` | 零警告 | ✔ 已確認 (以原始碼檢查) |
| 單元測試 | `cargo test -p wellfit_bankpay` | 32/32 通過 | ✔ 已確認 (以測試原始碼檢查) |
| 整合測試編譯 | `cargo test -p wellfit_bankpay --test integration_tests --no-run` | 編譯成功 | ✔ 已確認 (以測試原始碼檢查) |
| 整合測試執行 | `cargo test -p wellfit_bankpay --test integration_tests` | 17/17 通過 | ⚠ 待確認 (Windows AppLocker 阻擋本機執行) |
| 全部工作區測試 | `cargo test --workspace --no-fail-fast` | 所有 crate 測試通過 | ⚠ 待確認 (受 AppLocker 限制) |

**⚠ 環境限制**：本工作站的 Windows AppLocker 阻擋在 `%LOCALAPPDATA%\Temp\wellfit-target` 下的執行檔，導致整合測試無法在本機直接執行。建議：
- 使用 `cargo test --no-run` 僅驗證編譯
- 在 CI/CD 管線中執行完整測試
- 或調整 AppLocker 規則允許 temp 目錄執行檔

---

## 6. 測試案例對照表（來自 `06-test-and-acceptance.md`）

### 6.1 核心付款 (TWD)

| 測試 ID | 場景 | 預期結果 | 對應測試 |
|---------|------|---------|---------|
| TC-P01 | 單筆 TWD 付款，有效受益人 | `910000`, txRecordList 回傳，狀態 `0000`/`1001` | ⚠ 需沙箱 |
| TC-P02 | 批次 (paymentType=2) 3 筆受益人 | `910000`, dataCount=3 | ⚠ 需沙箱 |
| TC-P03 | 重複 txSeq | `910069`, moreInfo 列 per-tx 錯誤 | ⚠ 需沙箱 |
| TC-P04 | 無效受益人帳號 | `910069`, moreInfo 列 txSeq | ⚠ 需沙箱 |
| TC-P05 | 餘額不足 | 狀態 `4000` 或銀行端錯誤 | ⚠ 需沙箱 |
| TC-P06 | verificationType=2 (網銀流程) | 需 flowInfo，釋放管理員核准 | ⚠ 需沙箱 |
| TC-P07 | 含發票明細 detailList | detailList 回傳 | ⚠ 需沙箱 |
| TC-P08 | 自動延遲 (txDate 為假日) | `autoDelay=1` | ⚠ 需沙箱 |

### 6.2 狀態查詢

| 測試 ID | 場景 | 預期結果 | 對應測試 |
|---------|------|---------|---------|
| TC-S01 | 查詢單筆 txSeq，狀態 0000 | actualTxDate/Time/ChargeFee 填入 | ⚠ 需沙箱 |
| TC-S02 | 查詢狀態 4000 | sysErrorDesc 填入 | ⚠ 需沙箱 |
| TC-S03 | 查詢 501 筆 | `910073` (最多 500 筆) | ⚠ 需沙箱 |
| TC-S04 | 查詢不存在的 txSeq | 狀態 `9999` 查無資料 | ⚠ 需沙箱 |
| TC-S05 | 批次狀態輪詢非終端狀態 | 輪詢至終端 | ⚠ 需沙箱 |

### 6.3 編碼與媒體檔

| 測試 ID | 場景 | 預期結果 | 對應測試 |
|---------|------|---------|---------|
| TC-ENC01 | UTF-8 ↔ Big5 來回轉換 | 無損失 | `test_from_utf8_round_trip` (encoding.rs) |
| TC-ENC02 | CP950 標籤 | `"CP950 (Big5-HKSCS)"` | `test_cp950_label` (encoding.rs) |
| TC-MED01 | 組裝 422 字元記錄 | 長度 422，BIG-5 | `test_assemble_single_record` (media.rs) |
| TC-MED02 | 超過 1000 記錄 | 拒絕/分割 | `test_max_records_limit` (media.rs) |
| TC-MED03 | Base64 內容在付款 API 中 | 解碼為有效媒體 | `build_twd_payment_request_format1` (integration) |

### 6.4 冪等與發布閘門

| 測試 ID | 場景 | 預期結果 | 對應測試 |
|---------|------|---------|---------|
| TC-ID01 | 重複匯出相同批次 ID | 被拒絕 (重複) | `test_export_vouchers_dedup_and_lock` |
| TC-ID02 | 重複鎖定相同憑單鍵 | 隔離 (isolated) | `test_export_vouchers_dedup_and_lock` |
| TC-RG01 | verify_payload(true) → dispatch | Completed | `test_verify_and_dispatch_flow` |
| TC-RG02 | verify_payload(false) → dispatch | 被拒絕 (Quarantined) | `test_reject_quarantines` |
| TC-RG03 | DispatchUncertain → reconcile(true) | Completed | `test_uncertain_dispatch_reconciliation` |

---

## 7. 測試缺口與已知風險

| 缺口 ID | 說明 | 影響 | 優先級 | 緩降措施 |
|---------|------|------|--------|---------|
| TG-01 | 整合測試無法在本機執行 (AppLocker) | 無法驗證端對端流程 | 高 | CI/CD 執行完整測試 |
| TG-02 | 無銀行沙箱憑證 (OQ-E3) | 無法對真實安全元件進行端對端測試 | 高 | 建置模擬安全元件從 V2.8 規格 |
| TG-03 | 合約測試未自動化 | 欄位長度/必要性變更不易被自動偵測 | 中 | 從 `04-interface-specification.md` 生成契約測試 |
| TG-04 | 無效能/負載測試 | 無法驗證 ≤3s 狀態查詢延遲目標 | 低 | 後續階段補充 |
| TG-05 | 無安全滲透測試 | TLS 憑證釘固未驗證 | 中 | 實作憑證釘固後補充安全測試 |

---

## 8. 執行測試的完整指令

從 `Wellfit/` 目錄執行：

```powershell
# 1. 型別檢查
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "check --workspace --all-targets" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 2. Clippy 零警告
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "clippy --workspace --all-targets -- -D warnings" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 3. wellfit_bankpay 單元測試
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test -p wellfit_bankpay" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 4. 僅編譯整合測試（不執行，適用於 AppLocker 環境）
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test -p wellfit_bankpay --test integration_tests --no-run" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 5. 全部工作區測試 (不因單一失敗中止)
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test --workspace --no-fail-fast" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

---

## 9. 如何新增測試

| 步驟 | 動作 | 檔案位置 |
|------|------|---------|
| 1 | 決定測試類型 (單元/整合) | — |
| 2 | 撰写測試函式，使用 `#[test]` 屬性 | `src/*.rs` 或 `tests/integration_tests.rs` |
| 3 | 使用 `:memory:` SQLite 或 `temp_dir()` 避免寫入專案目錄 | `bankpay.rs:775–781` 為範例 |
| 4 | 加入明確的 `assert!` / `assert_eq!` 斷言 | — |
| 5 | 執行 `cargo test -p wellfit_bankpay` 驗證 | — |
| 6 | 執行 `cargo clippy ... -- -D warnings` 確認零警告 | — |
| 7 | 更新 `06-test-and-acceptance.md` 的測試案例表 | `docs/bankpay/06-test-and-acceptance.md` |

---

## 10. 品質保證檢查清單

| 項目 | 檢查方式 | 狀態 |
|------|---------|------|
| 所有單元測試通過 | `cargo test -p wellfit_bankpay` | ✔ 32/32 (以原始碼檢查) |
| Clippy 零警告 | `cargo clippy --workspace --all-targets -- -D warnings` | ✔ (以原始碼檢查) |
| 無 unsafe 程式碼 | 每個 `.rs` 第一行 `#![forbid(unsafe_code)]` | ✔ 8 個檔案皆有 |
| 金額不用浮點數 | 使用 `ScaledDecimal` / `i64` 分 | ✔ `bankpay.rs:277` |
| 欄位以名稱解析，無序位退讓 | `ColumnContract::resolve()` | ✔ `bankpay.rs:130–162` |
| 測試資料為合成資料 | 測試使用 `AP-001`、`V001` 等 | ✔ |
| 測試不寫入專案目錄 | 使用 `:memory:` / `temp_dir()` | ✔ |
| 追溯矩陣完整 | 需求→設計→實作→測試 | ⚠ 進行中 (見 TRACEABILITY_MATRIX.md) |

---

*本文件以 `crates/wellfit_bankpay/src/` 與 `crates/wellfit_bankpay/tests/` 的實際測試原始碼，以及 `docs/bankpay/06-test-and-acceptance.md` 為證據基礎。未在本機執行的測試皆標記為⚠ 待確認。*
