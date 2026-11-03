# Wellfit BankPay 維護、擴充與交接指南

**文件編號**：WFLT-DOC-009  
**版本**：1.0  
**狀態**：Draft  
**文件語言**：繁體中文（技術名詞保留標準英文）  
**證據來源**：`crates/wellfit_bankpay/src/` 原始碼、`Wellfit/AGENTS.md`、`docs/bankpay/08-open-questions-and-decisions.md`

---

## 1. 使用說明

本文件提供後續工程師安全地修改、擴充、重構及接手 Wellfit BankPay 模組的操作指引。每項建議皆說明原因、操作步驟、可能影響及驗證方法，而非僅提出原則性口號。

---

## 2. 新增功能時應修改的層次與檔案

### 2.1 新增一個欄位（例如在 Voucher 中新增欄位）

| 層次 | 檔案 | 行號範圍 | 動作 |
|------|------|---------|------|
| 領域模型 | `crates/wellfit_bankpay/src/bankpay.rs` | §32–49 (`Voucher` 結構體) | 新增欄位定義，說明型別、可空性、預設值 |
| CSV 解析 | `crates/wellfit_bankpay/src/bankpay.rs` | §130–145 (`resolve_voucher_columns`) | 若欄位來自 CSV，新增欄位索引解析 |
| 欄位契約 | `crates/wellfit_bankpay/src/bankpay.rs` | §112–118 (`voucher_contract`) | 若為必要欄位，加入 `required` 欄位清單 |
| 資料解析 | `crates/wellfit_bankpay/src/bankpay.rs` | §169–201 (`parse_voucher_row`) | 新增欄位解析邏輯與驗證 |
| 媒體檔 (若適用) | `crates/wellfit_bankpay/src/media.rs` | §59–77 (`TwdMediaRecord`) | 新增欄位與 `format_twd_record` 中的 `pad()` 呼叫 |
| API 模型 (若適用) | `crates/wellfit_bankpay/src/api.rs` | §84–139 (`TwdPaymentTx`) | 新增 JSON 序列化欄位 |
| 測試 | `crates/wellfit_bankpay/tests/integration_tests.rs` | — | 新增測試案例驗證新欄位 |
| 文件 | `docs/CODE_REFERENCE.md`、`docs/DATA_MODEL.md` | — | 更新欄位說明 |
| 文件 | `docs/bankpay/04-interface-specification.md` | — | 更新 API 欄位規格表 |

**驗證方法**：執行 `cargo test -p wellfit_bankpay` 與 `cargo clippy --workspace --all-targets -- -D warnings`，確認零警告與所有測試通過。

---

### 2.2 新增一個 API 端點

| 層次 | 檔案 | 行號範圍 | 動作 |
|------|------|---------|------|
| 請求/回應模型 | `crates/wellfit_bankpay/src/api.rs` | — | 新增 `Serde` 結構體 (RqData/RsData) |
| HTTP 客戶端方法 | `crates/wellfit_bankpay/src/http.rs` | §116–159 (`post_json` 範例) | 新增 `post_json` 包裝方法，指定 endpoint 路徑 |
| 引擎方法 | `crates/wellfit_bankpay/src/bankpay.rs` | — | 新增 `BankpayEngine` 方法，呼叫 HTTP 客戶端 |
| CLI 子命令 | `crates/wellfit_cli/src/main.rs` | §343–418 (`BankpayAction`) | 新增 CLI 子命令與參數 |
| 測試 | `crates/wellfit_bankpay/tests/integration_tests.rs` | — | 新增整合測試 |
| 介面規格 | `docs/bankpay/04-interface-specification.md` | — | 新增 API 端點、欄位、結果代碼表 |
| 資安文件 | `docs/bankpay/05-security-and-audit.md` | — | 若涉及新端點，更新結果代碼與安全考量 |

**驗證方法**：使用模擬安全元件 (mock security component) 驗證請求/回應契約符合 V2.8 規格。

---

### 2.3 新增一個驗證規則

| 層次 | 檔案 | 行號範圍 | 動作 |
|------|------|---------|------|
| 規則定義 | `crates/wellfit_core/src/validators.rs` | — | 新增 `VrRule` 實例 |
| 規則代碼 | `crates/wellfit_bankpay/src/bankpay.rs` | — | 新增 `VR-BANKPAY-XX` 規則代碼 |
| 發現記錄 | `crates/wellfit_bankpay/src/bankpay.rs` | §578, §709, §727 | 在對應處理流程中呼叫 `log_finding()` |
| 測試 | `crates/wellfit_bankpay/tests/integration_tests.rs` | — | 新增驗證規則觸發的測試 |
| 文件 | `docs/bankpay/05-security-and-audit.md` | §6 | 新增規則代碼與處理方式 |

---

## 3. 修改既有行為時的檢查清單

修改下列任何項目前，必須執行完整檢查清單：

| 檢查項目 | 檢查方式 | 預期結果 |
|---------|---------|---------|
| 編譯無錯誤 | `cargo check --workspace --all-targets` | 無錯誤 |
| Clippy 零警告 | `cargo clippy --workspace --all-targets -- -D warnings` | 零警告 |
| 所有單元測試通過 | `cargo test -p wellfit_bankpay` | 32/32 通過 |
| 整合測試編譯 | `cargo test -p wellfit_bankpay --test integration_tests --no-run` | 編譯成功 |
| 文件同步更新 | 檢查 `CODE_REFERENCE.md`、`DATA_MODEL.md`、`04-interface-specification.md` | 名稱與行號一致 |
| 追溯矩陣更新 | 檢查 `TRACEABILITY_MATRIX.md` | 需求與測試對應 |
| CHANGELOG 更新 | 編輯 `Wellfit/CHANGELOG.md` | 記錄變更內容 |

---

## 4. 避免破壞既有功能與外部相容性

### 4.1 向後相容性規則

| 變更類型 | 是否允許 | 條件 |
|---------|---------|------|
| 新增選填欄位 (default 值) | ✔ 允許 | 欄位必須有 `#[serde(default)]` 或合理預設值 |
| 新增必要欄位 | ⚠ 需評估 | 所有呼叫方必須提供新欄位，否則舊呼叫方會失敗 |
| 修改欄位型別 | 🚫 禁止 | 會破壞序列化/反序列化相容性 |
| 修改欄位名稱 | 🚫 禁止 | 會破壞 JSON 鍵名與 CSV 標題 |
| 刪除欄位 | 🚫 禁止 | 會破壞現有資料讀取 |
| 修改媒體檔欄位寬度 | ⚠ 需評估 | 必須保持總長度 422 字元，且下游系統需同意 |
| 修改 API 端點路徑 | 🚫 禁止 | 外部系統依賴固定端點 |
| 修改 `msgId` 格式 | ⚠ 需評估 | 銀行可能驗證格式，變更需與銀行確認 |
| 修改 `txSeq` 格式 | ⚠ 需評估 | 必須保持唯一性，銀行對帳依賴此欄位 |

### 4.2 媒體檔格式變更的特別限制

媒體檔採用 422 字元固定長度，任何欄位寬度變更必須：

1. 保持 `RECORD_LENGTH = 422` 不變 (`media.rs:16`)
2. 调整其他欄位的填充以彌補寬度差異
3. 更新 `format_twd_record()` 中的 `pad()` 呼叫順序與寬度
4. 更新測試 `test_assemble_single_record` (驗證 422 字元)
5. 與銀行確認新格式可接受 (V2.8 規格可能不允許變更)

---

## 5. 錯誤調查與問題診斷

### 5.1 調查錯誤的步驟

| 步驟 | 動作 | 工具/檔案 |
|------|------|---------|
| 1 | 查看 `wellfit status` 輸出的隔離記錄與驗證錯誤 | `wellfit status --db <path>` |
| 2 | 閱讀 Idempotency Ledger 的 `validation_findings` 表 | SQLite 資料庫，或透過 CLI status |
| 3 | 對照規則代碼含義 | `docs/bankpay/05-security-and-audit.md` §6 |
| 4 | 檢查原始 CSV 檔案的對應行號 | 開啟來源檔案，定位到 line_number |
| 5 | 檢查媒體檔 SHA-256 是否與導出結果一致 | `media_file_sha256()` 與 `ExportResult.file_sha256` 比對 |
| 6 | 若涉及 API 呼叫失敗，檢查 `rsStatus.code` 與 `moreInfo` | 對照 `05-security-and-audit.md` §4 結果代碼表 |
| 7 | 若涉及交易狀態非終端，輪詢查詢 | `bankpay query --kind tx --key <txSeq>` |

### 5.2 常見錯誤與對應規則代碼

| 錯誤現象 | 規則代碼 | 可能原因 | 處理方式 |
|---------|---------|---------|---------|
| 匯出行被隔離，金額無法解析 | `VR-BANKPAY-01` | `AMOUNT` 欄位格式錯誤 | 修正 CSV 檔案金額格式 |
| 匯出行被隔離，帳號空白 | `VR-BANKPAY-01` | `BANK_ACCOUNT` 欄位為空 | 補充銀行帳號 |
| 重複批次 ID 被拒絕 | — (ledger) | `batch_id` 已註冊 | 使用新批次 ID |
| 銀行結果重複結算 | `VR-BANKPAY-02` / isolation | `BANK_REF` 已存在 | 檢查是否重複匯入 |
| 銀行結果無法解析 | `VR-BANKPAY-03` | CSV 格式錯誤或欄位缺漏 | 修正銀行結果檔案 |
| API 結果代碼非 910000 | — | 銀行業務錯誤 | 對照 `05-security-and-audit.md` §4 |
| 媒體檔編碼錯誤 | `had_errors=true` | 包含無法在 CP950 表示的字元 | 移除或替換不支援的字元 |

### 5.3 縮小問題範圍

```
問題發現
    ↓
檢查 wellfit status → 是否有隔離記錄？
    ├─ 是 → 查看規則代碼，對照對應處理流程
    ├─ 否 → 檢查導出/導入結果的 dispatched / accepted 計數
    ↓
若涉及 API → 檢查 rsStatus.code 與 moreInfo
    ├─ 910000 → 成功，檢查 rsData
    ├─ 910069 → per-tx 錯誤，檢查 moreInfo 明細
    └─ 其他 → 對照 05-security-and-audit.md §4
    ↓
若涉及媒體檔 → 驗證 SHA-256 與 422 字元長度
    └─ media_file_sha256() 與 ExportResult.file_sha256 比對
```

---

## 6. 判斷模組是否適合擴充、替換或重構

### 6.1 擴充適合性評估

| 評估維度 | 問題 | 適合擴充的跡象 | 不適合的跡象 |
|---------|------|-------------|-----------|
| 耦合度 | 該模組是否依賴其他模組的內部實作？ | 僅透過公開介面依賴 | 直接存取其他模組的私有欄位 |
| 職責單一性 | 該模組是否只負責一個明確領域？ | 職責邊界清晰 | 職責重疊，難以說明 |
| 測試覆蓋 | 該模組是否有足夠測試？ | 測試覆蓋主要流程與邊界 | 測試稀少，變更風險高 |
| 文件完整度 | 該模組是否有完整文件？ | CODE_REFERENCE、DATA_MODEL 已更新 | 缺少關鍵文件 |

**BankPay 模組擴充適合性**：✔ 高 — 職責清晰（應付憑單匯出/匯入/查詢）、測試覆蓋良好（49 個測試函式）、文件完整（本文件體系）。

### 6.2 重構建議

| 建議 | 原因 | 優先級 |
|------|------|--------|
| CLI Export 參數未使用 | `source_system`、`customer_id` 等參數被 `_` 忽略，未傳遞至引擎 | 高 |
| TLS 憑證釘固未實作 | `http.rs` 未實作憑證釘固，安全性有缺口 | 高 |
| 其他 6 個 API 端點未實作 | FX、對帳單、餘額、列印等端點僅定義未實作 | 中 (本階段外) |
| `QueryResult.file_sha256` / `release_state` 固定為 None | 追溯查詢未填入實際值 | 低 |

---

## 7. 棄用功能與舊版介面處理

### 7.1 媒體檔格式 (formatType=1)

媒體檔格式 (422 字元 BIG-5) 是 E.Sun V2.8 規格定義的可選傳輸格式，目前以 `formatType=2` (JSON) 為主要路徑。

**棄用建議**：
- 不立即棄用 formatType=1，因為銀行可能仍要求媒體檔傳輸
- 若未來銀行廢止媒體檔支援，可將 `media.rs` 標記為 `#[deprecated]`
- 保留 `assemble_twd_media_file` 與 `TwdMediaRecord`，以備舊系統相容

### 7.2 舊版 API 欄位

若 E.Sun 發布 V2.9 規格，導致欄位長度或名稱變更：

1. 在 `api.rs` 中新增新欄位，舊欄位保留 `#[serde(default)]`
2. 在 `bankpay.rs` 中同時支援新舊格式，透過版本參數區分
3. 在 `04-interface-specification.md` 中標記舊欄位為棄用
4. 提供遷移期間雙軌運行

---

## 8. 資料遷移與系統退役

### 8.1 SQLite Ledger 遷移

Idempotency Ledger 使用 SQLite，結構由程式碼動態管理。若需遷移：

1. 備份現有 `.sqlite` 檔案
2. 執行新版本程式碼，Ledger 會自動建立所需表格
3. 驗證 `cargo test -p wellfit_bankpay` 通過
4. 若結構有重大變更，編寫遷移腳本（SQL ALTER TABLE）

### 8.2 系統退役

若 Wellfit BankPay 模組將被退役：

1. 停用 `bankpay export` 功能，透過功能旗標關閉
2. 財務團隊恢復至手動銀行入口網站操作（見 `07-deployment-and-operations.md` §8）
3. 保留 Ledger 資料庫為唯讀，供歷史對帳與稽核查詢
4. 保留 `media.rs` 與 `bankpay.rs` 原始碼作為參考，不刪除
5. 更新 `CHANGELOG.md` 記錄退役決定與日期

---

## 9. 新進工程師接手程序

| 步驟 | 動作 | 檔案 |
|------|------|------|
| 1 | 讀治理規則與目錄地圖 | `Wellfit/AGENTS.md` §6 |
| 2 | 讀工作延續模型 | `Knowledge/adr/ADR-0008-work-continuation-model.md` |
| 3 | 讀研究規約 | `Knowledge/sources/research-protocol.md` |
| 4 | 查來源登錄 | `Knowledge/sources/source-register.md` |
| 5 | 查議題登錄 | `Knowledge/issues/mtx-issue-register.md` |
| 6 | 查裁決紀錄 | `Knowledge/adr/` |
| 7 | 查追溯表 | `Knowledge/traceability/claims-vs-evidence.md` |
| 8 | 執行建置與測試基線 | `build.ps1 -Command "test --workspace --no-fail-fast"` |
| 9 | 閱讀 BankPay 文件體系 | `docs/README.md` → `docs/CODE_REFERENCE.md` → `docs/DATA_MODEL.md` |
| 10 | 進行小規模修改並驗證 | 修改一個欄位，執行測試與 clippy |

---

## 10. 變更影響矩陣

| 修改元件 | 可能影響的模組 | 需重新測試 |
|---------|-------------|---------|
| `media.rs` (TwdMediaRecord 欄位) | `bankpay.rs` (build_twd_payment_request), CLI Export | integration_tests: `media_file_assembly`, `build_twd_payment_request_format1` |
| `bankpay.rs` (Voucher 欄位) | `export_vouchers()`, `assemble_media_file()` | integration_tests: `export_vouchers_success`, `export_vouchers_isolates_bad_rows` |
| `http.rs` (TLS 設定) | CLI Export (EsunHttpClient) | `http_client_config_tls`, `http_client_default_config` |
| `api.rs` (TwdPaymentTx 欄位) | `build_twd_payment_request()`, `query_twd_status()` | `build_twd_payment_request_format1/2`, `test_twd_payment_tx_serde_roundtrip` |
| `query.rs` (查詢邏輯) | CLI Query | `query_voucher_flow`, `query_tx_flow`, `query_batch_flow` |
| `release.rs` (發布閘門) | `export_vouchers()` | `test_verify_and_dispatch_flow`, `test_reject_quarantines` |

---

## 11. 文件更新責任

| 文件 | 責任角色 | 更新時機 |
|------|---------|---------|
| `CODE_REFERENCE.md` | 資深軟體工程師 | 新增/修改類別、函式、欄位時 |
| `DATA_MODEL.md` | 資深軟體工程師 | 新增/修改資料欄位、關係時 |
| `MAINTENANCE_EVOLUTION.md` | 架構師 | 變更影響矩陣、重構建議時 |
| `TRACEABILITY_MATRIX.md` | QA 工程師 | 新增測試案例時 |
| `QUALITY_REVIEW_REPORT.md` | 獨立技術審查者 | 每次反向審查後 |
| `CHANGELOG.md` | 變更提出者 | 每次功能變更時 |

---

## 12. 已知維護風險

| 風險 ID | 說明 | 影響 | 緩降措施 |
|---------|------|------|---------|
| MR-01 | CLI Export 參數未使用 (source_system/customer_id 等) | 匯出無法使用 API-based 流程，僅產生媒體檔 | 修補 `run_bankpay` 將參數傳遞至引擎 |
| MR-02 | TLS 憑證釘固未實作 | 中間人攻擊風險 | 實作 `build_tls_config()` 的憑證釘固 |
| MR-03 | 整合測試無法本機執行 (AppLocker) | 變更後無法快速驗證 | CI/CD 執行完整測試 |
| MR-04 | 監控與警報未實作 | 問題發現依賴人工檢查 | 整合現有監控系統或實作簡易監控 |
| MR-05 | 沙箱憑證不可用 (OQ-E3) | 無法對真實安全元件測試 | 建置模擬安全元件從 V2.8 規格 |

---

*本文件以 `crates/wellfit_bankpay/src/` 原始碼與 `Wellfit/AGENTS.md` 為證據基礎。維護建議中的「未實作」項目皆標記為需後續補充，不得視為既有功能。*
