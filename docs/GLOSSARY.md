# Wellfit BankPay 領域術語表

**文件編號**：WFLT-DOC-004  
**版本**：1.0  
**狀態**：Draft  
**文件語言**：繁體中文（技術名詞保留標準英文）

---

## 1. 使用說明

本術語表統一 Wellfit BankPay 整合模組的所有專業名稱。每個術語包含：

- **標準名稱（英文名稱）**：專案中採用的正式名稱
- **白話解釋**：用日常語言說明是什麼
- **在本專案中的具體用途**：該術語在程式碼、API 或業務流程中的角色
- **實際例子**：具體數值或程式碼片段
- **與容易混淆概念的差別**：避免名稱混用

**原則**：同一個概念在整個文件體系與程式碼中必須使用相同名稱。若程式碼名稱與業務用語不同，下表提供對照關係。

---

## 2. 核心領域術語

### 2.1 應付憑單（Accounts Payable Voucher / Voucher）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 應付憑單（Voucher） |
| 英文名稱 | Voucher |
| 白話解釋 | ERP 中記錄「要付給哪家廠商、多少錢、到期日、收款帳號」的付款指令單 |
| 本專案用途 | `Voucher` 結構體，代表一筆待匯出的應付款項，是匯出流程的輸入單位 |
| 所在檔案 | `crates/wellfit_bankpay/src/bankpay.rs` §30 |
| 關鍵屬性 | `voucher_no`（憑單號）、`vendor_code`（廠商代碼）、`amount_cents`（金額，整數分）、`bank_account`（收款帳號）、`due_date`（到期日）、`reference`（參考號） |
| 實際例子 | `voucher_no = "AP-001"`, `vendor_code = "V001"`, `amount_cents = 100000`（即 1,000.00 元） |
| 與混淆概念差別 | 與「銀行結果紀錄 (BankResult)」不同：Voucher 是 ERP 發出的付款指令；BankResult 是銀行回傳的付款結果 |

### 2.2 銀行結果紀錄（Bank Result）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 銀行結果紀錄（Bank Result） |
| 英文名稱 | BankResult |
| 白話解釋 | 銀行處理完付款後回傳的結果，包含交易參考號、狀態（成功/失敗/不確定）、結算金額 |
| 本專案用途 | `BankResult` 結構體，匯入流程的輸入單位，用於對帳與更新憑單狀態 |
| 所在檔案 | `crates/wellfit_bankpay/src/bankpay.rs` §52 |
| 關鍵屬性 | `bank_ref`（銀行交易參考號）、`voucher_no`（對應的 ERP 憑單號）、`status`（狀態：paid/failed/uncertain）、`amount_cents`（結算金額）、`processed_at`（銀行處理時間） |
| 實際例子 | `bank_ref = "ETRACE-9001"`, `status = "paid"`, `amount_cents = 12340` |
| 與混淆概念差別 | 與「應付憑單 (Voucher)」不同：BankResult 是銀行回傳，Voucher 是 ERP 發出 |

### 2.3 E.Sun Bank 安全元件（E.Sun Security Component）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | E.Sun Bank 企業銀企直連安控元件 |
| 英文名稱 | E.Sun Security Component |
| 白話解釋 | 銀行安裝在 ERP 伺服器或釋放管理員工作站上的本機 HTTPS 服務，負責憑證驗證、eToken 簽名，並將請求轉發至銀行後端 |
| 本專案用途 | `EsunHttpClient` 與本機安全元件（預設 `https://127.0.0.1:8443`）通訊，執行 2-Way SSL |
| 所在檔案 | `crates/wellfit_bankpay/src/http.rs` |
| 連接埠 | 內部 8443、外部 9443 |
| 實際例子 | ERP → `https://localhost:8443/payment/createTwdPaymentInstruction` → 安控元件 → 銀行後端 |
| 與混淆概念差別 | 不是銀行的公眾網銀入口網站，而是企業直連的本機安全中介軟體 |

### 2.4 統一編號（Enterprise Group Tax ID / Customer ID）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 企業集團統編（Customer ID / SourceSystemId 相關） |
| 英文名稱 | customerIxd / customer_id |
| 白話解釋 | 企業向稅務機關登記的統一編號，8–10 碼，用於識別付款發起企業 |
| 本專案用途 | `RqHeader.customer_ixd` 欄位，也是 `build_twd_payment_request` 的 `customer_id` 參數，用於組建 `msgId` |
| 所在檔案 | `crates/wellfit_bankpay/src/api.rs` §27；`crates/wellfit_bankpay/src/bankpay.rs` §404 |
| 實際例子 | `customer_ixd = "64000477"`（V2.8 規格範例）、`"12345678"`（測試範例） |
| 與混淆概念差別 | 與「SourceSystemId」不同：SourceSystemId 是銀行與 ERP 約定的系統識別碼；Customer ID 是企業稅籍編號 |

### 2.5 來源系統識別碼（Source System ID）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 來源系統識別碼（SourceSystemId） |
| 英文名稱 | sourceSystem / source_system |
| 白話解釋 | 銀行與 ERP 系統之間約定的識別碼，標識付款請求來自哪個 ERP 系統 |
| 本專案用途 | `RqHeader.source_system` 欄位，CLI `--source-system` 參數，`msgId` 組建的一部分 |
| 所在檔案 | `crates/wellfit_bankpay/src/api.rs` §30；`crates/wellfit_cli/src/main.rs` §364 |
| 實際例子 | `source_system = "S001"`（測試範例）、V2.8 規格中為「約定之 SourceSystemId」 |
| 與混淆概念差別 | 與「Customer ID」不同：SourceSystemId 識別系統，Customer ID 識別企業 |

### 2.6 訊息識別碼（Message ID / msgId）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 訊息識別碼（Message ID） |
| 英文名稱 | msgId |
| 白話解釋 | 每一筆付款請求的唯一識別碼，由企業統編 + 日期 + 7 碼流水組成，長度 23–25 碼 |
| 本專案用途 | `RqHeader.msg_id`，由 `build_twd_payment_request` 根據 `customer_id` 與當前時間組建 |
| 業務規則 | BR-1: `msgId` = 企業集團統編 (8–10 碼) + YYYYMMDD + 7 碼流水序號 |
| 所在檔案 | `crates/wellfit_bankpay/src/bankpay.rs` §413 |
| 實際例子 | `"12345678202611020000001"`（統編 8 碼 + 日期 8 碼 + 流水 7 碼） |
| 與混淆概念差別 | 與「交易序號 (txSeq)」不同：msgId 是整筆請求的識別碼；txSeq 是單筆交易的識別碼 |

### 2.7 交易序號（Transaction Sequence / txSeq）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 交易序號（Transaction Sequence） |
| 英文名稱 | txSeq |
| 白話解釋 | ERP 為每筆付款交易生成的唯一編號，用於在銀行回傳結果時對帳 |
| 本專案用途 | `TwdPaymentTx.tx_seq`，由 `voucher_to_twd_tx` 組建為 `IOB` + 日期 + 憑單號；媒體檔記錄欄位 |
| 業務規則 | BR-3: `txSeq` 必須每個 ERP 付款草稿唯一 |
| 所在檔案 | `crates/wellfit_bankpay/src/bankpay.rs` §392；`crates/wellfit_bankpay/src/api.rs` §134 |
| 實際例子 | `"IOB20261102AP-001"` |
| 與混淆概念差別 | 與「訊息識別碼 (msgId)」不同：txSeq 是單筆交易識別碼，msgId 是整筆請求識別碼 |

### 2.8 媒體檔記錄（TWD Media Record）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | TWD 付款媒體檔記錄 |
| 英文名稱 | TwdMediaRecord |
| 白話解釋 | E.Sun V2.8 規格定義的 422 字元固定長度 BIG-5 編碼記錄，用於格式 1（檔案傳輸）的付款資料 |
| 本專案用途 | `TwdMediaRecord` 結構體，`assemble_twd_media_file` 將多筆記錄組裝為媒體檔，再 Base64 編碼放入 API `content` 欄位 |
| 所在檔案 | `crates/wellfit_bankpay/src/media.rs` §59 |
| 記錄長度 | 422 字元（固定） |
| 最大記錄數 | 1,000 筆 |
| 實際例子 | payer_account_no(16) + payer_id(15) + ... + trailing(10) = 422 字元 |
| 與混淆概念差別 | 與 JSON 格式的 `TwdPaymentTx` 不同：媒體記錄是固定長度 BIG-5 字串，TwdPaymentTx 是 JSON 序列化物件 |

### 2.9 發布閘門（Release Gate）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 發布閘門（Release Gate） |
| 英文名稱 | ReleaseGate |
| 白話解釋 | 狀態機，控制付款載荷是否允許派送：先經「二向門驗證」（verify_payload），再經「單向門派送」（dispatch），遇到逾時進入「不確定派送」狀態需對帳 |
| 本專案用途 | `BankpayEngine` 包裝 `ReleaseGate`，匯出流程中驗證媒體檔是否有隔離行或編碼錯誤後才派送 |
| 所在檔案 | `crates/wellfit_core/src/release_gate.rs`（wellfit_core）；`crates/wellfit_bankpay/src/release.rs`（包裝層） |
| 狀態流程 | DraftGenerated → VerifiedPass / Quarantined → AuthorizedDispatch → Completed / Failed / DispatchUncertain |
| 實際例子 | `verify_payload(true, "")` → `VerifiedPass` → `dispatch(|| Ok(true), None)` → `Completed` |
| 與混淆概念差別 | 與「媒體檔組裝」不同：Release Gate 控制派送流程，不負責組裝媒體檔內容 |

### 2.10 冪等帳本（Idempotency Ledger）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 冪等帳本（Idempotency Ledger） |
| 英文名稱 | IdempotencyLedger |
| 白話解釋 | SQLite 資料庫，記錄已處理的檔案批次與訂單鎖，防止同一批次或同一筆憑單被重複處理 |
| 本專案用途 | `BankpayEngine.ledger`，匯出前鎖定憑單鍵 `VOUCHER_NO:AMOUNT:BATCH_DATE`，匯入前檢查銀行參考號是否已結算 |
| 所在檔案 | `crates/wellfit_core/src/ledger.rs`（wellfit_core）；`crates/wellfit_bankpay/src/bankpay.rs` §510、§659 |
| 鎖定鍵格式 | `VOUCHER_NO:AMOUNT:BATCH_DATE`（匯出）、`BANK_REF:BATCH_ID`（匯入） |
| 實際例子 | `"AP-001:1000:2026-11-02"` |
| 與混淆概念差別 | 與「稽軌跡紀錄 (Finding)」不同：帳本記錄處理狀態，Finding 記錄驗證發現與稽核事件 |

### 2.11 驗證發現（Finding）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 驗證發現（Finding） |
| 英文名稱 | Finding |
| 白話解釋 | 驗證過程中發現的錯誤、警告或資訊事件，記錄規則代碼、嚴重度、檔案、行號與原因 |
| 本專案用途 | 匯出錯誤用 `VR-BANKPAY-01`，匯入結果用 `VR-BANKPAY-02`，隔離列用 `VR-BANKPAY-03`，追溯查詢用 `VR-BANKPAY-QUERY` |
| 所在檔案 | `crates/wellfit_core/src/ledger.rs`（Finding 定義）；`crates/wellfit_bankpay/src/bankpay.rs` §578、§709、§727、§941 |
| 規則代碼 | `VR-BANKPAY-01`（匯出）、`VR-BANKPAY-02`（匯入）、`VR-BANKPAY-03`（隔離）、`VR-BANKPAY-QUERY`（查詢） |
| 嚴重度 | Error、Warning、Info |
| 與混淆概念差別 | 與「隔離記錄 (Quarantine)」不同：Finding 是驗證事件記錄，Quarantine 是無法解析的資料列 |

---

## 3. API 與介面術語

### 3.1 請求標頭（Request Header / RqHeader）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 請求標頭（Request Header） |
| 英文名稱 | RqHeader |
| 白話解釋 | E.Sun API 請求 envelope 的標頭部分，包含 msgId、requestTime、customerIxd、sourceSystem |
| 本專案用途 | `RqHeader` 結構體，`build_twd_payment_request` 與 `query_twd_status` 皆組建此標頭 |
| 所在檔案 | `crates/wellfit_bankpay/src/api.rs` §18 |
| 欄位 | `msg_id`（23–25 碼）、`request_time`（YYYY/MM/DD HH:MM:SS）、`customer_ixd`（8–10 碼）、`source_system`（約定識別碼） |

### 3.2 回應狀態（Response Status / RsStatus）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 回應狀態（Response Status） |
| 英文名稱 | RsStatus |
| 白話解釋 | E.Sun API 回傳的結果代碼，`910000` 表示成功，其他皆為失敗 |
| 本專案用途 | `ApiResponse<T>.rs_status`，`is_api_success()` 檢查 code 是否等於 `"910000"` |
| 所在檔案 | `crates/wellfit_bankpay/src/api.rs` §49 |
| 重要代碼 | `910000`=成功、`910069`=交易資料檢核有誤、`910073`=超過查詢上限 500 筆、`999999`=未知錯誤 |

### 3.3 TWD 付款交易（TWD Payment Transaction）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | TWD 付款交易（TWD Payment Transaction） |
| 英文名稱 | TwdPaymentTx |
| 白話解釋 | E.Sun API 中單筆台幣付款的資料結構，包含付款人、收款人、金額、日期、備註等欄位 |
| 本專案用途 | `TwdPaymentTx` 結構體，`voucher_to_twd_tx` 將 `Voucher` 轉換為此格式；`twdTxList` 是陣列 |
| 所在檔案 | `crates/wellfit_bankpay/src/api.rs` §84 |
| 必填欄位 | `payerAccountNo`、`payerId`、`payeeBankId`、`payeeBranchId`、`payeeAccountNo`、`payeeName`、`txDate`、`txAmt`、`chargeFeeType`、`txSeq` |

### 3.4 格式類型（Format Type）

| 說明項目 | 內容 |
|---------|------|
| 標準名稱 | 格式類型（Format Type） |
| 英文名稱 | formatType |
| 白話解釋 | 付款資料的傳輸格式：`1` = 檔案（固定長度媒體檔，Base64 編碼），`2` = JSON |
| 本專案用途 | `TwdPaymentRequestData.format_type`，CLI `--format-type` 參數預設為 `"2"` |
| 所在檔案 | `crates/wellfit_bankpay/src/api.rs` §183；`crates/wellfit_cli/src/main.rs` §376 |
| 設計決策 | ADR-BP-001: 以 `formatType=2` (JSON) 為主要傳輸格式，`formatType=1` 僅用於傳統/銀行要求情況 |

---

## 4. 程式碼術語與對照

| 業務用語 | 程式碼名稱 | 所在檔案 |
|---------|-----------|---------|
| 應付憑單 | `Voucher` | `bankpay.rs:32` |
| 銀行結果 | `BankResult` | `bankpay.rs:53` |
| 匯出結果 | `ExportResult` | `bankpay.rs:72` |
| 匯入結果 | `ImportResult` | `bankpay.rs:84` |
| TWD 媒體記錄 | `TwdMediaRecord` | `media.rs:59` |
| 請求標頭 | `RqHeader` | `api.rs:18` |
| 回應狀態 | `RsStatus` | `api.rs:49` |
| TWD 付款交易 | `TwdPaymentTx` | `api.rs:84` |
| TWD 付款請求 | `TwdPaymentRequest` | `api.rs:199` |
| 安全元件 HTTP 客戶端 | `EsunHttpClient` | `http.rs:42` |
| HTTP 客戶端設定 | `EsunHttpClientConfig` | `http.rs:16` |
| BankPay 引擎 | `BankpayEngine` | `bankpay.rs:292` |
| 追溯查詢結果 | `QueryResult` | `query.rs:14` |
| 發布閘門包裝 | `verify_payload` / `dispatch_payload` | `release.rs` |
| 金額轉換 | `cents_to_yuan_str` | `bankpay.rs:277` |
| 媒體檔組裝 | `assemble_twd_media_file` | `media.rs:23` |
| 追溯查詢 | `query` | `query.rs:37` |

---

## 5. 狀態碼術語（參考 V2.8）

### 5.1 API 結果代碼（rsStatus.code）

| 代碼 | 意義 | 處理方式 |
|------|------|---------|
| `910000` | 訊息回覆成功 | 解析 rsData |
| `910069` | 交易資料檢核有誤 | 檢查 moreInfo 中的 per-tx 錯誤 |
| `910073` | 超過查詢上限（500 筆） | 分批查詢 |
| `999999` | 未知錯誤 | 記錄完整 payload，升級 |

完整表格請見 `docs/bankpay/05-security-and-audit.md` §4。

### 5.2 交易狀態代碼（txStatus）

| 代碼 | 意義 | 是否終端 |
|------|------|---------|
| `0000` | 交易成功 | 是 |
| `1000–1008` | 處理中 / 待放行 等 | 否（需輪詢） |
| `4000` | 交易失敗 | 是 |
| `9999` | 查無資料 | 是 |

完整表格請見 `docs/bankpay/02-business-workflow.md` §5。

---

## 6. 名稱一致性規則

1. 文件與程式碼中的名稱必須完全一致。例如 `EsunHttpClientConfig` 不得在文件中寫成 `EsunClientConfig`。
2. 業務用語與程式碼名稱不同時，必須在本表中建立對照（如「應付憑單」→ `Voucher`）。
3. 新增術語時，必須同時更新 `CODE_REFERENCE.md` 與實際程式碼中的註解。
4. 中英文對照以本表為準，不得自行創造新的英文譯名。

---

*本術語表是 Wellfit BankPay 文件體系的語言基準。所有文件引用術語時，必須以本表定義為準。*
