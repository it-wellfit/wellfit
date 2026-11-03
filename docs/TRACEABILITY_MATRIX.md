# Wellfit BankPay 追溯矩陣

**文件編號**：WFLT-DOC-010  
**版本**：1.0  
**狀態**：Draft  
**文件語言**：繁體中文（技術名詞保留標準英文）  
**證據來源**：`docs/bankpay/01-problem-and-scope.md`、`docs/bankpay/03-technical-design.md`、`docs/bankpay/04-interface-specification.md`、`docs/bankpay/06-test-and-acceptance.md`、`crates/wellfit_bankpay/src/`、`crates/wellfit_bankpay/tests/`

> **狀態／證據限制（2026-11-02）：**此矩陣描述本機程式碼與測試案例的對應，不等於測試已在目前工作樹執行，也不代表銀行端已驗收。品質審查 [`QUALITY_REVIEW_REPORT.md`](QUALITY_REVIEW_REPORT.md) 的 `QR-06` 記錄 CLI 匯出 transport closure 僅回傳 `Ok(true)`、沒有實際網路派送；因此 FR-BP-01 的本機組裝／引擎狀態不可標示為已完成付款整合。FR-BP-02 亦僅代表本機查詢引擎存在，銀行端到端仍需沙箱驗證。

---

## 1. 使用說明

本矩陣旨在展示需求、設計、實作、測試與驗收標準之間的雙向關聯。資料列是文件所記錄的對應，不代表每個需求都有完整實作、已執行測試或驗收；缺少證據的狀態必須維持未驗證。

**追溯方向**：
- **順向追溯**：需求 → 設計 → 實作 → 測試 → 驗收（檢查每個階段是否有對應證據）
- **逆向追溯**：測試 → 實作 → 設計 → 需求（檢查測試是否有來源依據）

---

## 2. 需求 → 設計 → 實作 → 測試追溯表

### 2.1 核心功能需求 (FR-BP-*)

| 需求 ID | 需求描述 | 來源文件 | 設計位置 | 實作位置 | 測試位置 | 驗收標準 | 狀態 |
|---------|---------|---------|---------|---------|---------|---------|------|
| FR-BP-01 | TWD 付款發放 (單筆/批次) | `01-problem-and-scope.md` §2.1 #1 | `03-technical-design.md` §2 | `bankpay.rs:510` (`export_vouchers`), `bankpay.rs:404` (`build_twd_payment_request`), `http.rs:162` (`create_twd_payment`) | `integration_tests.rs:30` (`export_vouchers_success`), `integration_tests.rs:144` (`build_twd_payment_request_format1`), `integration_tests.rs:176` (`build_twd_payment_request_format2`) | TC-P01, TC-P02 | △ 本機請求組裝／引擎存在；CLI 未實際網路派送，銀行付款未完成驗證 |
| FR-BP-02 | TWD 交易狀態查詢 | `01-problem-and-scope.md` §2.1 #2 | `03-technical-design.md` §2 | `bankpay.rs:327` (`query_twd_status`), `http.rs:175` (`retrieve_twd_status`) | `integration_tests.rs:96` (`query_voucher_flow`), `integration_tests.rs:109` (`query_tx_flow`) | TC-S01, TC-S04 | △ 本機查詢引擎存在；銀行端對端需沙箱驗證 |
| FR-BP-03 | 媒體檔組裝 (422 字元 BIG-5, formatType=1) | `03-technical-design.md` §3 | `03-technical-design.md` §3 | `media.rs:23` (`assemble_twd_media_file`), `media.rs:59` (`TwdMediaRecord`) | `integration_tests.rs:211` (`media_file_assembly`), `media.rs:172` (`test_assemble_single_record`) | TC-MED01, TC-MED02 | ✔ 已實作 |
| FR-BP-04 | 編碼轉換 (UTF-8 ↔ Big5/CP950) | `03-technical-design.md` §1 | `03-technical-design.md` §1 | `encoding.rs:84` (`to_utf8`), `encoding.rs:99` (`from_utf8`) | `encoding.rs:126` (`test_from_utf8_round_trip`), `bankpay.rs:949` (`test_encoding_round_trip_used_by_engine`) | TC-ENC01, TC-ENC02 | ✔ 已實作 |
| FR-BP-05 | 冪等鎖定 (防止重複付款) | `03-technical-design.md` §5 | `03-technical-design.md` §5 | `bankpay.rs:559–566` (`check_and_lock_order`) | `bankpay.rs:784` (`test_export_vouchers_dedup_and_lock`), `integration_tests.rs:30` | TC-ID01, TC-ID02 | ✔ 已實作 |
| FR-BP-06 | 發布閘門 (兩向門驗證 + 單向門派送) | `02-business-workflow.md` §7 | `03-technical-design.md` §6 | `release.rs:16` (`verify_payload`), `release.rs:22` (`dispatch_payload`), `bankpay.rs:601–636` | `release.rs:55` (`test_verify_and_dispatch_flow`), `release.rs:79` (`test_reject_quarantines`) | TC-RG01, TC-RG02 | ✔ 已實作 |
| FR-BP-07 | DispatchUncertain 對帳 | `02-business-workflow.md` §7 | `03-technical-design.md` §6 | `release.rs:33` (`reconcile_uncertain`) | `release.rs:68` (`test_uncertain_dispatch_reconciliation`) | TC-RG03 | ✔ 已實作 |
| FR-BP-08 | 追溯查詢 (voucher/tx/batch) | `03-technical-design.md` §9 | `03-technical-design.md` §9 | `query.rs:37` (`query`), `query.rs:14` (`QueryResult`) | `integration_tests.rs:96` (`query_voucher_flow`), `integration_tests.rs:109` (`query_tx_flow`), `integration_tests.rs:121` (`query_batch_flow`), `bankpay.rs:867` (`test_end_to_end_traceability_chain`) | — | ✔ 已實作 |
| FR-BP-09 | 錯誤行隔離與檢查記錄 | `03-technical-design.md` §7 | `03-technical-design.md` §7 | `bankpay.rs:576–589` (匯出隔離), `bankpay.rs:725–737` (匯入隔離) | `bankpay.rs:810` (`test_export_isolates_bad_amount`), `integration_tests.rs:55` (`export_vouchers_isolates_bad_rows`) | — | ✔ 已實作 |
| FR-BP-10 | TLS 2-Way SSL 客戶端 | `05-security-and-audit.md` §2 | `03-technical-design.md` §1 | `http.rs:42` (`EsunHttpClient`), `http.rs:58` (`build_tls_config`) | `http.rs:193` (`test_config_default`), `http.rs:200` (`test_client_creation`), `integration_tests.rs:256` (`http_client_config_tls`) | — | △ 本機 client/config 程式碼存在；握手、正式憑證與銀行端連線未驗證；憑證釘固未實作 |

### 2.2 外幣 (FX) 功能需求 (本階段外)

| 需求 ID | 需求描述 | 來源文件 | 設計位置 | 實作位置 | 測試位置 | 狀態 |
|---------|---------|---------|---------|---------|---------|------|
| FR-BP-11 | FX 預檢核 (validateFxPaymentInstruction) | `01-problem-and-scope.md` §2.1 #4 | `04-interface-specification.md` §5 | ⚠ 未實作 (僅 api.rs 定義欄位) | ⚠ 未實作 | 🚫 本階段外 |
| FR-BP-12 | FX 付款發放 (createFxPaymentInstruction) | `01-problem-and-scope.md` §2.1 #5 | `04-interface-specification.md` §6 | ⚠ 未實作 | ⚠ 未實作 | 🚫 本階段外 |
| FR-BP-13 | FX 交易狀態查詢 | `01-problem-and-scope.md` §2.1 #6 | `04-interface-specification.md` §7 | ⚠ 未實作 | ⚠ 未實作 | 🚫 本階段外 |
| FR-BP-14 | 對帳單查詢 (retrieveStatements) | `01-problem-and-scope.md` §2.1 #3 | `04-interface-specification.md` §4 | ⚠ 未實作 | ⚠ 未實作 | 🚫 本階段外 |
| FR-BP-15 | 帳戶餘額查詢 (retrieveAccountBalance) | `01-problem-and-scope.md` §2.1 #7 | `04-interface-specification.md` §8 | ⚠ 未實作 | ⚠ 未實作 | 🚫 本階段外 |
| FR-BP-16 | 付款結果列印 (retrieveTwdPaymentInstructionPrintResult) | `01-problem-and-scope.md` §2.1 #8 | `04-interface-specification.md` §9 | ⚠ 未實作 | ⚠ 未實作 | 🚫 本階段外 |

---

## 3. 業務規則 → 實作追溯表

| 業務規則 ID | 規則描述 | 來源文件 | 實作位置 | 驗證測試 | 狀態 |
|-------------|---------|---------|---------|---------|------|
| BR-1 | `msgId` = 統編 (8–10) + YYYYMMDD + 7 碼流水 | `02-business-workflow.md` §6 | `bankpay.rs:413–418` (`build_twd_payment_request` msgId 組建) | `integration_tests.rs:144` (檢查 `rq_header` 組建) | ✔ 已實作 |
| BR-2 | txDate ≤ 系統日期且狀態不在 {0000, 4000} 必須重新查詢 | `02-business-workflow.md` §6 | ⚠ 未實作 (查詢邏輯在 `query_twd_status` 但未實作重新查詢規則) | ⚠ 未實作 | ⚠ 待實作 |
| BR-3 | `txSeq` 必須每個 ERP 付款草稿唯一 | `02-business-workflow.md` §6 | `bankpay.rs:392` (`voucher_to_twd_tx` txSeq 組建), `bankpay.rs:559` (冪等鎖定鍵) | `bankpay.rs:784` (去重測試) | ✔ 已實作 |
| BR-4 | FX paymentType=2 需相同付款人帳戶、幣別、日期 | `02-business-workflow.md` §6 | ⚠ 未實作 (FX 階段外) | ⚠ 未實作 | 🚫 本階段外 |
| BR-5 | 對帳單分頁：續而回傳筆數 == 500 且 code == 910000 | `02-business-workflow.md` §6 | ⚠ 未實作 | ⚠ 未實作 | 🚫 本階段外 |
| BR-6 | 列印 API 限 5 筆 per call | `02-business-workflow.md` §6 | ⚠ 未實作 | ⚠ 未實作 | 🚫 本階段外 |

---

## 4. 錯誤代碼 → 處理邏輯追溯表

| 結果代碼 | 意義 | 來源文件 | 處理邏輯位置 | 測試覆蓋 | 狀態 |
|-----------|------|---------|-------------|---------|------|
| `910000` | 訊息回覆成功 | `05-security-and-audit.md` §4 | `api.rs:378` (`is_api_success`), `bankpay.rs:482` | `api.rs:397` (`test_api_result_success`) | ✔ |
| `910069` | 交易資料檢核有誤 | `05-security-and-audit.md` §4 | ⚠ 未實作 (moreInfo 解析邏輯) | ⚠ 未實作 | ⚠ 待實作 |
| `910073` | 超過查詢上限 500 筆 | `05-security-and-audit.md` §4 | ⚠ 未實作 | ⚠ 未實作 | ⚠ 待實作 |
| `999999` | 未知錯誤 | `05-security-and-audit.md` §4 | `api.rs:380` (`is_api_success` 返回 false) | `api.rs:397` | ✔ |
| `0000` | 交易成功 | `02-business-workflow.md` §5 | `api.rs:346` (`is_tx_success`) | `api.rs:404` (`test_tx_status_success`) | ✔ |
| `4000` | 交易失敗 | `02-business-workflow.md` §5 | `api.rs:359` (`is_tx_failed`) | `api.rs:418` (`test_tx_status_failed`) | ✔ |
| `9999` | 查無資料 | `02-business-workflow.md` §5 | `api.rs:341` (`NO_DATA` 常數) | ⚠ 未直接測試 | ⚠ 待測試 |
| `1000–1008` | 處理中/待放行 | `02-business-workflow.md` §5 | `api.rs:351` (`is_tx_pending`) | `api.rs:410` (`test_tx_status_pending`) | ✔ |

---

## 5. 安全控制 → 實作追溯表

| 控制項 | 說明 | 來源文件 | 實作位置 | 驗證方式 | 狀態 |
|--------|------|---------|---------|---------|------|
| TLS 1.2+ | 傳輸加密 | `05-security-and-audit.md` §2 | `http.rs:58` (`build_tls_config` 使用 rustls) | `http.rs:200` (`test_client_creation`) | ✔ |
| 2-Way SSL | 客戶端 + 伺服器憑證 | `05-security-and-audit.md` §2 | `http.rs:87–99` (`client_auth` 邏輯) | `integration_tests.rs:256` (`http_client_config_tls`) | ✔ (設定層)，⚠ 端對端需沙箱 |
| 憑證釘固 | 驗證伺服器憑證指紋 | `05-security-and-audit.md` §2 | ⚠ 未實作 (listed as gap) | ⚠ 未實作 | ⚠ 待實作 |
| eToken 簽名 | 委託安全元件 | `05-security-and-audit.md` §3 | ⚠ 委託給安全元件，ERP 不處理 | ⚠ 無法測試 (需 eToken) | 🚫 委託給安全元件 |
| Source-System 標頭 | 携帶約定識別碼 | `05-security-and-audit.md` §2 | `http.rs:145` (`.set("Source-System", source_system)`) | ⚠ 未直接測試 | ✔ 已實作 |
| Idempotency | 防止重複付款 | `05-security-and-audit.md` §6 | `bankpay.rs:559–566` (`check_and_lock_order`) | `bankpay.rs:784` (去重測試) | ✔ |
| SHA-256 內容雜湊 | 批次完整性 | `05-security-and-audit.md` §6 | `bankpay.rs:267` (`media_file_sha256`), `bankpay.rs:594` | `bankpay.rs:829` (SHA-256 測試) | ✔ |
| 稽軌跡 (Finding) | 所有驗證事件記錄 | `05-security-and-audit.md` §6 | `bankpay.rs:578`, `bankpay.rs:709`, `bankpay.rs:727`, `query.rs:90` | `bankpay.rs:849` (import findings 測試) | ✔ |
| RBAC | 財務/釋放管理員角色 | `05-security-and-audit.md` §7 | `wellfit_core::access_control` (共享 crate) | `wellfit_core` 測試 | ✔ (共享核心) |

---

## 6. 測試 → 需求逆向追溯表

| 測試 ID | 測試名稱 | 驗證的需求/規則 | 實作位置 | 狀態 |
|---------|---------|--------------|---------|------|
| `export_vouchers_success` | 匯出成功 | FR-BP-01, FR-BP-05 | `bankpay.rs:510` | ✔ |
| `export_vouchers_isolates_bad_rows` | 錯誤列隔離 | FR-BP-09 | `bankpay.rs:576–589` | ✔ |
| `import_results_success` | 匯入成功 | FR-BP-01 (匯入半) | `bankpay.rs:659` | ✔ |
| `query_voucher_flow` | 憑單查詢 | FR-BP-08 | `query.rs:37` | ✔ |
| `query_tx_flow` | 交易查詢 | FR-BP-02, FR-BP-08 | `bankpay.rs:327`, `query.rs:37` | ✔ |
| `query_batch_flow` | 批次查詢 | FR-BP-08 | `query.rs:37` | ✔ |
| `query_rejects_unknown_kind` | 未知類型拒絕 | FR-BP-08 (邊界) | `query.rs:79–83` | ✔ |
| `build_twd_payment_request_format1` | 格式 1 請求 | FR-BP-01, FR-BP-03 | `bankpay.rs:404`, `media.rs:23` | ✔ |
| `build_twd_payment_request_format2` | 格式 2 請求 | FR-BP-01 | `bankpay.rs:404` | ✔ |
| `media_file_assembly` | 媒體檔組裝 | FR-BP-03 | `media.rs:23` | ✔ |
| `cents_to_yuan_str_precision` | 金額格式化 | FR-BP-01 (金額處理) | `bankpay.rs:277` | ✔ |
| `http_client_config_tls` | TLS 設定 | FR-BP-10 | `http.rs:42` | ✔ |
| `http_client_default_config` | 預設 TLS | FR-BP-10 | `http.rs:42` | ✔ |
| `end_to_end_export_import_query` | 端對端追溯 | FR-BP-01, FR-BP-02, FR-BP-08 | `bankpay.rs:510`, `bankpay.rs:659`, `query.rs:37` | ✔ |
| `test_export_vouchers_dedup_and_lock` | 去重與鎖定 | FR-BP-05, BR-3 | `bankpay.rs:559–566` | ✔ |
| `test_export_isolates_bad_amount` | 金額錯誤隔離 | FR-BP-09 | `bankpay.rs:576–589` | ✔ |
| `test_assemble_media_file_and_sha256` | 媒體檔與雜湊 | FR-BP-03 | `media.rs:23`, `bankpay.rs:267` | ✔ |
| `test_import_results_records_findings` | 匯入發現記錄 | FR-BP-09 | `bankpay.rs:709` | ✔ |
| `test_end_to_end_traceability_chain` | 追溯鏈 | FR-BP-01, FR-BP-03, FR-BP-08 | `bankpay.rs:510`, `bankpay.rs:659`, `query.rs:37` | ✔ |
| `test_encoding_round_trip_used_by_engine` | 編碼來回轉換 | FR-BP-04 | `encoding.rs:84`, `encoding.rs:99` | ✔ |
| `test_assemble_single_record` | 單筆媒體記錄 | FR-BP-03 | `media.rs:23` | ✔ |
| `test_empty_records_rejected` | 空記錄拒絕 | FR-BP-03 (邊界) | `media.rs:24–26` | ✔ |
| `test_base64_roundtrip` | Base64 來回轉換 | FR-BP-03 (傳輸) | `media.rs:134` | ✔ |
| `test_max_records_limit` | 最大記錄限制 | FR-BP-03 (邊界) | `media.rs:27–33` | ✔ |
| `test_config_default` | 預設設定 | FR-BP-10 | `http.rs:16` | ✔ |
| `test_client_creation` | 無憑證 TLS | FR-BP-10 | `http.rs:49` | ✔ |
| `test_query_voucher` | 憑單查詢 | FR-BP-08 | `query.rs:37` | ✔ |
| `test_query_tx` | 交易查詢 | FR-BP-08 | `query.rs:37` | ✔ |
| `test_query_batch` | 批次查詢 | FR-BP-08 | `query.rs:37` | ✔ |
| `test_query_rejects_unknown_kind` | 未知類型拒絕 | FR-BP-08 (邊界) | `query.rs:79–83` | ✔ |
| `test_verify_and_dispatch_flow` | 驗證與派送 | FR-BP-06 | `release.rs:16`, `release.rs:22` | ✔ |
| `test_uncertain_dispatch_reconciliation` | 不確定對帳 | FR-BP-07 | `release.rs:33` | ✔ |
| `test_reject_quarantines` | 拒絕隔離 | FR-BP-06 | `release.rs:16` | ✔ |
| `test_api_result_success` | API 結果成功 | 錯誤代碼 `910000` | `api.rs:378` | ✔ |
| `test_tx_status_success` | 交易狀態成功 | 狀態 `0000` | `api.rs:346` | ✔ |
| `test_tx_status_pending` | 交易狀態處理中 | 狀態 `1000–1008` | `api.rs:351` | ✔ |
| `test_tx_status_failed` | 交易狀態失敗 | 狀態 `4000` | `api.rs:359` | ✔ |
| `test_api_response_success_check` | ApiResponse 成功 | 錯誤代碼 `910000` | `api.rs:72` | ✔ |
| `test_api_response_failure_no_data` | ApiResponse 失敗 | 錯誤代碼 `910071` | `api.rs:72` | ✔ |
| `test_twd_payment_tx_serde_roundtrip` | Tx 序列化 | FR-BP-01 (資料模型) | `api.rs:84` | ✔ |
| `test_currency_codes_count` | 貨幣代碼 | `04-interface-specification.md` §10 | `api.rs:387` | ✔ |
| `test_detect_utf8_bom` | BOM 偵測 | FR-BP-04 | `encoding.rs:65` | ✔ |
| `test_detect_plain_utf8` | 純 UTF-8 偵測 | FR-BP-04 | `encoding.rs:65` | ✔ |
| `test_from_utf8_round_trip` | 編碼來回轉換 | FR-BP-04 | `encoding.rs:84`, `encoding.rs:99` | ✔ |
| `test_from_utf8_big5_ascii_no_errors` | Big5 ASCII 無錯誤 | FR-BP-04 | `encoding.rs:99` | ✔ |
| `test_cp950_label` | CP950 標籤 | FR-BP-04 | `encoding.rs:48` | ✔ |

---

## 7. 已知追溯缺口

| 缺口 ID | 說明 | 影響 | 優先級 |
|---------|------|------|--------|
| TG-01 | FR-BP-11 至 FR-BP-16 (FX 與其他 6 個 API 端點) 未實作 | 功能不完整，但本階段明確標記為外 | 中 |
| TG-02 | BR-2 (txDate ≤ 系統日期且非終端狀態需重新查詢) 未實作 | 狀態輪詢邏輯不完整 | 中 |
| TG-03 | 結果代碼 `910069` (per-tx 錯誤解析)、`910073` (超過查詢上限) 處理邏輯未實作 | 部分錯誤情境未處理 | 中 |
| TG-04 | 交易狀態 `9999` (查無資料) 未直接測試 | 邊界狀態驗證不足 | 低 |
| TG-05 | 憑證釘固未實作 | 資安控制缺口 | 高 |

---

## 8. 如何使用追溯矩陣

| 任務 | 操作方式 |
|------|---------|
| 確認某個需求是否有實作 | 查找需求 ID，查看「實作位置」欄位 |
| 確認某個測試驗證哪個需求 | 查找測試 ID，查看「驗證的需求/規則」欄位 |
| 修改某個欄位時評估影響 | 查找實作位置，查看對應的測試與需求 |
| 新增需求時 | 在本表新增一行，填寫需求 ID、描述、來源、設計、實作、測試、驗收、狀態 |
| 新增測試時 | 在「測試 → 需求逆向追溯表」新增一行，確保每個測試都有對應需求 |

---

## 9. 文件更新責任

| 觸發事件 | 責任角色 | 更新內容 |
|---------|---------|---------|
| 新增需求 | 產品經理 | 新增需求 ID 行 |
| 新增實作 | 軟體工程師 | 更新「實作位置」與「狀態」 |
| 新增測試 | QA 工程師 | 新增測試行，更新「測試位置」 |
| 變更狀態 | 專案經理 | 更新「狀態」欄位 (✔ 已實作 / ⚠ 待實作 / 🚫 本階段外) |
| 發現缺口 | 任務發現者 | 新增「已知追溯缺口」行 |

---

*本文件以 `docs/bankpay/` 設計文件與 `crates/wellfit_bankpay/` 實作原始碼為證據基礎。標記為⚠ 待實作或🚫 本階段外的項目，不得視為既有功能。*
