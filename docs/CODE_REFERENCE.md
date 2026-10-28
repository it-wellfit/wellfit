# Wellfit BankPay 程式碼參考文件

**文件編號**：WFLT-DOC-005  
**版本**：1.0  
**狀態**：Draft  
**適用範圍**：`Wellfit/crates/wellfit_bankpay/` 及與其互動的 CLI 程式碼  
**文件語言**：繁體中文（技術名詞保留標準英文）  
**證據來源**：實際原始碼檔案（以檔案路徑、行號定位）

---

## 1. 使用說明

本文件是 Wellfit BankPay 模組的程式碼參考手冊。每個重要物件、屬性、方法、函式、介面都列出：

1. **名稱與正式定義**：程式碼中的型別或函式名稱
2. **存在目的**：解決什麼問題、扮演什麼角色
3. **所在檔案與行號**：可直接定位到原始碼
4. **關鍵屬性與方法**：名稱、型別、必要性、約束
5. **與其他物件的關係**：依賴方向、呼叫關係
6. **錯誤情境與邊界條件**：可能失敗的方式
7. **測試方式**：對應的測試函式

**證據標記**：✔ 已確認（有原始碼支持）| △ 合理推論 | ● 設計建議 | ⚠ 待確認 | 🚫 已知限制

---

## 2. Crate 目錄結構

```
Wellfit/crates/wellfit_bankpay/
├── Cargo.toml              # crate 定義與依賴 (✔ 已確認)
├── src/
│   ├── lib.rs              # 公開重匯出 (✔ 已確認，32 行)
│   ├── api.rs              # V2.8 API 請求/回應模型 (✔ 已確認，494 行)
│   ├── bankpay.rs          # 引擎、領域模型、媒體檔組裝 (✔ 已確認，958 行)
│   ├── encoding.rs         # CP950/Big5/Big5-HKSCS/UTF-8 轉換 (✔ 已確認，148 行)
│   ├── http.rs             # EsunHttpClient (ureq + rustls, 2-Way SSL) (✔ 已確認，207 行)
│   ├── media.rs            # 422 字元固定長度 BIG-5 媒體檔組裝 (✔ 已確認，201 行)
│   ├── query.rs            # 追溯查詢 (voucher↔tx↔batch↔sha256) (✔ 已確認，162 行)
│   └── release.rs          # ReleaseGate 包裝層 (✔ 已確認，88 行)
└── tests/
    └── integration_tests.rs # 整合測試，17 個測試函式 (✔ 已確認，317 行)
```

---

## 3. 公開重匯出（lib.rs）

| 項目 | 定義 | 所在 | 說明 |
|------|------|------|------|
| `api` | `pub mod api` | `lib.rs:18` | API 請求/回應模型 |
| `bankpay` | `pub mod bankpay` | `lib.rs:19` | 引擎與領域模型 |
| `encoding` | `pub mod encoding` | `lib.rs:20` | 編碼轉換 |
| `http` | `pub mod http` | `lib.rs:21` | HTTP 客戶端 |
| `media` | `pub mod media` | `lib.rs:22` | 媒體檔組裝 |
| `query` | `pub mod query` | `lib.rs:23` | 追溯查詢 |
| `release` | `pub mod release` | `lib.rs:24` | 發布閘門包裝 |
| `pub use api::*` | 重匯出 | `lib.rs:26` | 讓 API 類型可直接使用 `wellfit_bankpay::` |
| `pub use bankpay::*` | 重匯出 | `lib.rs:27` | 讓 Voucher、BankpayEngine 等可直接使用 |
| `pub use encoding::*` | 重匯出 | `lib.rs:28` | 讓 BankEncoding 等可直接使用 |
| `pub use http::*` | 重匯出 | `lib.rs:29` | 讓 EsunHttpClient 等可直接使用 |
| `pub use media::*` | 重匯出 | `lib.rs:30` | 讓 TwdMediaRecord 等可直接使用 |
| `pub use query::*` | 重匯出 | `lib.rs:31` | 讓 query 等可直接使用 |
| `pub use release::*` | 重匯出 | `lib.rs:32` | 讓 verify_payload 等可直接使用 |

**不變量**：每個 `.rs` 第一行必須是 `#![forbid(unsafe_code)]`（✔ 已確認，所有 8 個原始碼檔案皆有此行）。

---

## 4. 模組 4.1：api.rs — API 請求/回應模型

### 4.1.1 常用 envelope 類型

| 類型 | 行號 | 目的 | 關鍵屬性 |
|------|------|------|---------|
| `RqHeader` | §18 | 請求標頭 envelope | `msg_id` (23–25), `request_time` (YYYY/MM/DD HH:MM:SS), `customer_ixd` (8–10), `source_system` |
| `RsHeader` | §34 | 回應標頭 | `msg_id`, `log_id` (40), `response_time` |
| `RsStatus` | §49 | 回應狀態 | `code` (6, 910000=成功), `message` (60), `more_info` |
| `ApiResponse<T>` | §61 | 通用回應包裝 | `rs_header`, `rs_status`, `rs_data` (Option) |
| `is_api_success(code)` | §378 | 檢查是否成功 | `code == "910000"` |

### 4.1.2 TWD 付款發放 API 模型

| 類型 | 行號 | 目的 | 關鍵屬性 |
|------|------|------|---------|
| `TwdPaymentTx` | §84 | 單筆 TWD 付款交易 | `payer_account_no` (16, Y), `payer_id` (15, Y), `payee_bank_id` (3, Y), `payee_branch_id` (4, Y), `payee_account_no` (16, Y), `payee_name` (80, Y), `payee_id` (15, N), `tx_date` (8, Y), `tx_amt` (13, Y), `charge_fee_type` (1, Y), `memo` (80, N), `notify_method` (1, N), `notify_email` (400, N), `notify_fax` (20, N), `write_off_no` (40, N), `remark` (38, N), `tx_seq` (22, Y), `detail_list` (Vec) |
| `TwdPaymentDetail` | §142 | 發票明細 | `invoice_date` (8), `invoice_no` (20), `invoice_amt` (18.2), `invoice_return` (18.2), `invoice_actual_amt` (18.2), `invoice_remark` (40) |
| `FlowInfo` | §165 | 流程資訊 (verificationType=2 時需要) | `operator_name` (32), `flow_name` (64), `flow_key` (Option<i64>) |
| `TwdPaymentRequestData` | §179 | 請求資料 (rqData) | `format_type` (1=檔案, 2=JSON), `payment_type` (1/2), `verification_type` (1/2), `flow_info`, `content` (Base64) |
| `TwdPaymentRequest` | §198 | 完整 TWD 付款請求 | `rq_header`, `rq_data` |
| `TwdPaymentResponseData` | §207 | 付款回應資料 | `data_count`, `tx_record_list` (Vec<TwdPaymentRecord>) |
| `TwdPaymentRecord` | §218 | 付款回應記錄 | `tx_seq`, `auto_delay`, `payer_account_no`, `payer_id`, `payee_bank_id`, `payee_branch_id`, `payee_account_no`, `payee_name`, `payee_id`, `tx_date`, `tx_amt`, `charge_fee_type`, `memo`, `notify_method`, `notify_email`, `notify_fax`, `write_off_no`, `remark`, `detail_list` |

### 4.1.3 TWD 交易狀態查詢 API 模型

| 類型 | 行號 | 目的 | 關鍵屬性 |
|------|------|------|---------|
| `TwdStatusQueryRequestData` | §286 | 狀態查詢請求資料 | `payment_no`, `tx_seq` |
| `TwdStatusQueryRequest` | §296 | 完整狀態查詢請求 | `rq_header`, `rq_data` |
| `TwdStatusResponseData` | §305 | 狀態查詢回應 | `tx_status` (0000=成功), `bank_ref`, `tx_amt`, `tx_date`, `error_message` |

### 4.1.4 交易狀態常數

| 常數 | 行號 | 值 | 說明 |
|------|------|-----|------|
| `tx_status::SUCCESS` | §331 | `"0000"` | 交易成功 |
| `tx_status::PENDING_RANGE` | §333 | `(1000, 1008)` | 處理中範圍 |
| `tx_status::FX_RELATED` | §335 | `["1030","1031","1032"]` | 外幣相關狀態 |
| `tx_status::FX_PRECHECK` | §337 | `["3000","3001"]` | FX 預檢核狀態 |
| `tx_status::FAILED` | §339 | `"4000"` | 交易失敗 |
| `tx_status::NO_DATA` | §341 | `"9999"` | 查無資料 |
| `is_tx_success(code)` | §345 | — | `code == "0000"` |
| `is_tx_pending(code)` | §350 | — | 1000–1008 |
| `is_tx_failed(code)` | §359 | — | `code == "4000"` |

### 4.1.5 API 結果代碼常數

| 常數 | 行號 | 值 | 說明 |
|------|------|-----|------|
| `api_result::SUCCESS` | §370 | `"910000"` | 訊息回覆成功 |
| `api_result::FIELD_ERROR` | §372 | `"910071"` | 欄位驗證錯誤 |
| `api_result::UNKNOWN` | §374 | `"999999"` | 未知錯誤 |
| `is_api_success(code)` | §378 | — | `code == "910000"` |

### 4.1.6 貨幣代碼

| 常數 | 行號 | 值 | 說明 |
|------|------|-----|------|
| `CURRENCY_CODES` | §387 | 15 種貨幣 | USD, CNY, HKD, JPY, EUR, AUD, CAD, GBP, ZAR, NZD, CHF, SEK, SGD, MXN, THB（不含 TWD） |

---

## 5. 模組 4.2：bankpay.rs — 引擎與領域模型

### 5.1 領域模型結構體

| 結構體 | 行號 | 目的 | 關鍵屬性 |
|--------|------|------|---------|
| `Voucher` | §32 | ERP 應付憑單 | `voucher_no` (String), `vendor_code` (String), `amount_cents` (i64, 整數分), `due_date` (String, ISO-8601), `bank_account` (String), `reference` (String), `line` (usize), `raw` (String) |
| `BankResult` | §52 | 銀行付款結果 | `bank_ref` (String), `voucher_no` (String), `status` (String), `amount_cents` (i64), `processed_at` (String), `line` (usize), `raw` (String) |
| `ExportResult` | §72 | 匯出結果 | `batch_id` (String), `media_file` (String), `file_sha256` (String), `vouchers_accepted` (usize), `vouchers_isolated` (usize), `dispatched` (bool), `release_state` (String) |
| `ImportResult` | §84 | 匯入結果 | `batch_id` (String), `results_accepted` (usize), `results_isolated` (usize), `matched_vouchers` (usize), `unmatched` (usize) |

### 5.2 欄位契約（ColumnContract）

| 函式 | 行號 | 目的 | 必填欄位 | 選填欄位 |
|------|------|------|---------|---------|
| `voucher_contract()` | §112 | ERP 憑單匯出 CSV 契約 | `VOUCHER_NO`, `VENDOR_CODE`, `AMOUNT`, `BANK_ACCOUNT` | `DUE_DATE`, `REFERENCE` |
| `bank_result_contract()` | §121 | 銀行結果匯入 CSV 契約 | `BANK_REF`, `VOUCHER_NO`, `STATUS`, `AMOUNT` | `PROCESSED_AT` |

### 5.3 媒體檔組裝（bankpay.rs 中的 assemble_media_file）

| 函式 | 行號 | 目的 | 參數 | 回傳值 |
|------|------|------|------|---------|
| `assemble_media_file()` | §246 | 組裝 CSV 銀行媒體檔內容 | `vouchers: &[Voucher]`, `batch_id: &str`, `encoding: BankEncoding` | `(Vec<u8>, bool)` — (位元組, 是否有編碼錯誤) |
| `media_file_sha256()` | §267 | 計算媒體檔 SHA-256 雜湊 | `bytes: &[u8]` | `String` (64 碼十六進位) |
| `cents_to_yuan_str()` | §277 | 將分金額格式化為元字串 | `cents: i64` | `String` (例如 "10.00") |

### 5.4 BankpayEngine 引擎

| 項目 | 行號 | 類型 | 說明 |
|------|------|------|------|
| `BankpayEngine` 結構體 | §292 | 結構體 | 持有 ledger、gate、選擇性 http_client |
| `ledger` 屬性 | §293 | `Arc<IdempotencyLedger>` | 冪等帳本 |
| `gate` 屬性 | §294 | `Arc<Mutex<ReleaseGate>>` | 發布閘門 |
| `http_client` 屬性 | §295 | `Option<EsunHttpClient>` | HTTP 客戶端（選擇性） |
| `new()` | §300 | 靜態方法 | 建立生產環境引擎（獨佔批次主張） |
| `new_scoped()` | §309 | 靜態方法 | 建立每次執行主張的引擎（用於驗證/稽核） |
| `with_http_client()` | §318 | 方法 | 附加 HTTP 客戶端 |
| `query_twd_status()` | §327 | 方法 | 查詢 TWD 交易狀態，透過安全元件 API |
| `voucher_to_twd_tx()` | §366 | 靜態方法 | 將 Voucher 轉換為 TwdPaymentTx |
| `build_twd_payment_request()` | §404 | 方法 | 建立 TwdPaymentRequest，包含 RqHeader 與媒體內容 |
| `is_api_success()` | §482 | 方法 | 檢查 API 結果代碼 |
| `is_tx_success()` | §487 | 方法 | 檢查交易狀態是否成功 |
| `is_tx_pending()` | §492 | 方法 | 檢查交易狀態是否處理中 |
| `is_tx_failed()` | §497 | 方法 | 檢查交易狀態是否失敗 |
| `export_vouchers()` | §510 | 方法 | 匯出應付憑單：讀取、驗證、鎖定、組裝媒體檔、發布閘門驗證與派送 |
| `import_results()` | §659 | 方法 | 匯入銀行結果：讀取、驗證、記錄發現、對帳 |

### 5.5 export_vouchers 方法詳細流程（✔ 已確認，行 510–649）

| 步驟 | 行號 | 動作 | 失敗處理 |
|------|------|------|---------|
| 1 | 518–526 | 讀取來源檔案，檢查位元組大小上限 (`MAX_INPUT_BYTES`) | 超過上限返回 `Err` |
| 2 | 527 | 編碼轉換：位元組 → UTF-8（自動偵測 BOM） | — |
| 3 | 534–542 | 註冊檔案批次到 IdempotencyLedger | 重複批次返回 `Err` |
| 4 | 544–549 | 解析標題行，解析欄位索引 | 契約不符返回 `Err` |
| 5 | 554–591 | 逐行解析憑單，驗證金額、帳號，鎖定冪等鍵 | 錯誤行隔離，記錄 `VR-BANKPAY-01` Finding |
| 6 | 593–594 | 組裝媒體檔，計算 SHA-256 | 編碼錯誤設 `had_errors=true` |
| 7 | 596–598 | 寫入媒體檔案到磁碟 | 寫入失敗返回 `Err` |
| 8 | 601–609 | ReleaseGate.verify_payload 檢查 | 隔離行或編碼錯誤 → `valid=false` |
| 9 | 611–636 | 若 valid，呼叫 transport 閉包派送 | 失敗返回 `Err` |
| 10 | 638–648 | 回傳 ExportResult | — |

### 5.6 import_results 方法詳細流程（✔ 已確認，行 659–749）

| 步驟 | 行號 | 動作 | 失敗處理 |
|------|------|------|---------|
| 1 | 665–667 | 讀取銀行結果檔案，轉換為 UTF-8 | — |
| 2 | 674–677 | 註冊結果批次到 IdempotencyLedger | — |
| 3 | 679–684 | 解析標題行，解析欄位索引 | 契約不符返回 `Err` |
| 4 | 691–739 | 逐行解析銀行結果，檢查是否已結算 | 重複 → 隔離；無法解析 → `log_quarantine` (`VR-BANKPAY-03`) |
| 5 | 708–721 | 記錄 `VR-BANKPAY-02` Finding | — |
| 6 | 742–748 | 回傳 ImportResult | — |

---

## 6. 模組 4.3：http.rs — EsunHttpClient

### 6.1 設定結構體

| 欄位 | 行號 | 型別 | 必要性 | 預設值 | 說明 |
|------|------|------|--------|--------|------|
| `base_url` | §18 | `String` | Y | `"https://127.0.0.1:8443"` | 安全元件基礎 URL |
| `client_cert_path` | §20 | `Option<String>` | N | `None` | 客戶端憑證路徑 (PEM) |
| `client_key_path` | §22 | `Option<String>` | N | `None` | 客戶端私鑰路徑 (PEM) |
| `ca_cert_path` | §24 | `Option<String>` | N | `None` | CA 憑證路徑 (PEM) |
| `timeout_secs` | §26 | `u64` | Y | `30` | 請求逾時秒數 |

### 6.2 EsunHttpClient 結構體

| 屬性 | 行號 | 型別 | 說明 |
|------|------|------|------|
| `config` | §43 | `EsunHttpClientConfig` | 設定 |
| `tls_config` | §44 | `Option<Arc<rustls::ClientConfig>>` | TLS 設定（無憑證時為 None，使用 ureq 預設） |

### 6.3 方法

| 方法 | 行號 | 參數 | 回傳值 | 說明 |
|------|------|------|--------|------|
| `new(config)` | §49 | `EsunHttpClientConfig` | `Self` | 建立客戶端，同時建置 TLS 設定 |
| `build_tls_config(config)` | §58 | `&EsunHttpClientConfig` | `Option<Arc<rustls::ClientConfig>>` | 從憑證路徑建置 rustls ClientConfig；無憑證路徑時回傳 None |
| `post_json<Req, Resp>()` | §116 | `path`, `source_system`, `&Req` | `Result<ApiResponse<Resp>, String>` | POST JSON 請求，強制 HTTPS，設定 Source-System 標頭 |
| `create_twd_payment()` | §162 | `source_system`, `&TwdPaymentRequest` | `Result<ApiResponse<TwdPaymentResponseData>, String>` | 呼叫 `/payment/createTwdPaymentInstruction` |
| `retrieve_twd_status()` | §175 | `source_system`, `&TwdStatusQueryRequest` | `Result<ApiResponse<TwdStatusResponseData>, String>` | 呼叫 `/payment/retrieveTwdPaymentInstructionResult` |

### 6.4 TLS 建置邏輯（✔ 已確認，行 58–109）

| 情況 | 處理方式 |
|------|---------|
| 無憑證路徑 | 回傳 `None`，使用 ureq 預設 TLS |
| 有 `ca_cert_path` | 讀取 PEM，建置 `RootCertStore`，加入 CA 憑證 |
| 無 `ca_cert_path` | 使用 `webpki_roots::TLS_SERVER_ROOTS` 作為預設根憑證 |
| 有 `client_cert_path` + `client_key_path` | 讀取客戶端憑證與私鑰，`with_client_auth_cert()` |
| 僅有一個客戶端憑證/鑰匙 | 回傳 `None`（`client_auth` 為 `None`，使用 `with_no_client_auth()`） |

**⚠ 待確認**：目前未實作憑證釘固（certificate pinning），`05-security-and-audit.md` §2 列為 gap。

---

## 7. 模組 4.4：encoding.rs — 編碼轉換

### 7.1 BankEncoding 列舉

| 變體 | 行號 | 預設 | encoding_rs 標籤 | 說明 |
|------|------|------|-----------------|------|
| `Utf8` | §21 | ✔ 預設 | `"UTF-8"` | 內部標準編碼 |
| `Cp950` | §23 | — | `"Big5-HKSCS"` | Microsoft 代碼頁 950，對應 Big5-HKSCS |
| `Big5` | §25 | — | `"Big5"` | Big5 編碼 |
| `Big5Hkscs` | §27 | — | `"Big5-HKSCS"` | Big5-HKSCS |

### 7.2 方法

| 方法 | 行號 | 參數 | 回傳值 | 說明 |
|------|------|------|--------|------|
| `encoding()` | §32 | — | `&'static Encoding` | 根據變體回傳 encoding_rs 編碼物件 |
| `label()` | §48 | — | `&'static str` | MIME/IANA 標籤 |
| `detect_encoding(bytes)` | §65 | `&[u8]` | `(BankEncoding, usize)` | 偵測 BOM，否則預設 UTF-8 |
| `to_utf8(bytes, source)` | §84 | `&[u8]`, `Option<BankEncoding>` | `(String, BankEncoding)` | 位元組 → UTF-8 字串，無效序列替換為 replacement character |
| `from_utf8(text, target)` | §99 | `&str`, `BankEncoding` | `(Vec<u8>, bool)` | UTF-8 字串 → 目標編碼位元組，`had_errors` 表示是否有字元遺失 |

---

## 8. 模組 4.5：media.rs — 422 字元媒體檔組裝

### 8.1 常數

| 常數 | 行號 | 值 | 說明 |
|------|------|-----|------|
| `MAX_RECORDS` | §13 | `1000` | 每個媒體檔最大記錄數 |
| `RECORD_LENGTH` | §16 | `422` | 單筆記錄固定字元長度 |

### 8.2 TwdMediaRecord 結構體

| 欄位 | 行號 | 型別 | 寬度 (字元) | 必要性 | 填充方式 | E.Sun V2.8 說明 |
|------|------|------|------------|--------|---------|----------------|
| `payer_account_no` | §60 | `String` | 16 | Y | 左對齊，空白 | 付款人帳號 (≥13 碼) |
| `payer_id` | §61 | `String` | 15 | Y | 左對齊，空白 | 付款人統編 (≥10 碼) |
| `payee_bank_id` | §62 | `String` | 3 | Y | — | 收款銀行代碼 (固定 3 碼) |
| `payee_branch_id` | §63 | `String` | 4 | Y | 左對齊，空白 | 收款銀行分行 |
| `payee_account_no` | §64 | `String` | 16 | Y | 右對齊，零填充 | 收款帳號 |
| `payee_name` | §65 | `String` | 80 | Y | 左對齊，空白 | 收款戶名 (全形 ≤39) |
| `payee_id` | §66 | `String` | 15 | N | 左對齊，空白 | 收款人統編 |
| `tx_date` | §67 | `String` | 8 | Y | YYYYMMDD | 付款日期 |
| `tx_amt` | §68 | `String` | 13 | Y | 右對齊，零填充 (>0) | 付款金額 |
| `charge_fee_type` | §69 | `String` | 1 | Y | `0`=外收, `1`=內扣 | 手續費負擔 |
| `memo` | §70 | `String` | 80 | N | 左對齊，空白 | 附言 (全形 ≤39) |
| `notify_method` | §71 | `String` | 1 | N | `1`=不通知, `2`=Email, `3`=Fax, `9`=全部 | 通知方式 |
| `notify_email` | §72 | `String` | 40 | N | 左對齊，空白 | 通知 Email (JSON 規格為 400) |
| `notify_fax` | §73 | `String` | 20 | N | 左對齊，空白 | 通知傳真 (≥10 碼含區碼) |
| `write_off_no` | §74 | `String` | 40 | N | 左對齊，空白 | 銷帳編號 |
| `remark` | §75 | `String` | 38 | N | 左對齊，空白 | 備註 (全形 ≤18) |
| `tx_seq` | §76 | `String` | 22 | Y | 左對齊，空白 | ERP 交易序號 (唯一) |
| trailing | §108 | — | 10 | — | 空白 | 填充至 422 |

### 8.3 函式

| 函式 | 行號 | 參數 | 回傳值 | 說明 |
|------|------|------|--------|------|
| `assemble_twd_media_file()` | §23 | `&[TwdMediaRecord]` | `Result<Vec<u8>, String>` | 組裝 422 字元 BIG-5 媒體檔，驗證長度，CP950 編碼 |
| `format_twd_record()` | §80 | `&TwdMediaRecord`, `usize` | `Result<String, String>` | 格式化單筆 422 字元記錄 |
| `pad()` | §120 | `&str`, `usize` | `String` | 補齊或截斷字串至指定寬度（空白填充） |
| `base64_encode_media()` | §134 | `&[u8]` | `String` | Base64 標準編碼 |
| `base64_decode_media()` | §139 | `&str` | `Result<Vec<u8>, String>` | Base64 解碼 |

---

## 9. 模組 4.6：query.rs — 追溯查詢

### 9.1 QueryResult 結構體

| 欄位 | 行號 | 型別 | 說明 |
|------|------|------|------|
| `lookup_key` | §16 | `String` | 查詢關鍵字 |
| `lookup_kind` | §17 | `String` | 查詢類型：`voucher` / `tx` / `batch` |
| `batch_id` | §19 | `Option<String>` | 匹配的批次 ID |
| `bank_ref` | §20 | `Option<String>` | 匹配的銀行參考號 |
| `voucher_no` | §21 | `Option<String>` | 匹配的憑單號 |
| `file_sha256` | §22 | `Option<String>` | 媒體檔 SHA-256（目前未填入，固定為 None） |
| `release_state` | §23 | `Option<String>` | 發布狀態（目前未填入，固定為 None） |
| `findings` | §24 | `Vec<String>` | 稽軌跡發現 |

### 9.2 查詢函式

| 函式 | 行號 | 參數 | 回傳值 | 說明 |
|------|------|------|--------|------|
| `query()` | §37 | `engine: &mut BankpayEngine`, `lookup_kind: &str`, `key: &str` | `Result<QueryResult, String>` | 執行追溯查詢，記錄 `VR-BANKPAY-QUERY` Finding |

**查詢類型行為**（✔ 已確認，行 58–84）：

| 查詢類型 | 行號 | 行為 |
|---------|------|------|
| `voucher` | §59–64 | 檢查 `ledger.is_order_processed(key)`，回傳 `voucher_no=Some(key)` |
| `tx` | §66–70 | 回傳 `bank_ref=Some(key)`，記錄交叉參照發現 |
| `batch` | §72–77 | 回傳 `batch_id=Some(key)`，記錄已註冊批次數量 |
| 其他 | §79–83 | 回傳 `Err`，錯誤訊息說明使用 `voucher`, `tx`, 或 `batch` |

---

## 10. 模組 4.7：release.rs — 發布閘門包裝層

| 函式 | 行號 | 參數 | 回傳值 | 說明 |
|------|------|------|--------|------|
| `verify_payload()` | §16 | `gate: &mut ReleaseGate`, `is_valid: bool`, `reason: &str` | `ReleaseState` | 二向門驗證 |
| `dispatch_payload()` | §22 | `gate: &mut ReleaseGate`, `transport: impl FnOnce() -> Result<bool, String>`, `recipients: Option<&[String]>` | `Result<bool, String>` | 單向門派送 |
| `reconcile_uncertain()` | §33 | `gate: &mut ReleaseGate`, `status_check: impl FnOnce() -> bool` | `ReleaseState` | 解決 DispatchUncertain |
| `is_terminal()` | §41 | `state: ReleaseState` | `bool` | `Completed | Failed` |
| `is_uncertain()` | §46 | `state: ReleaseState` | `bool` | `DispatchUncertain` |

**類型別名**：`BankPayReleaseState = ReleaseState`（行 11）

---

## 11. CLI 整合點（wellfit_cli/src/main.rs）

### 11.1 BankpayAction 列舉（行 343–418）

| 子命令 | 行號 | 參數 | 說明 |
|--------|------|------|------|
| `Export` | §347 | `input`, `batch`, `batch_date`, `encoding`, `esun_url`, `source_system`, `customer_id`, `payer_account`, `payer_id`, `format_type`, `client_cert_path`, `client_key_path`, `ca_cert_path`, `db` | 匯出應付憑單 |
| `Import` | §392 | `input`, `batch`, `encoding`, `db` | 匯入銀行結果 |
| `Query` | §407 | `kind` (voucher/tx/batch), `key`, `db` | 追溯查詢 |

### 11.2 run_bankpay 函式（✔ 已確認，行 2002–2140）

| 分支 | 行號 | 關鍵邏輯 | 問題 |
|------|------|---------|------|
| `Export` | §2004–2072 | 建立 `EsunHttpClientConfig`（含 TLS 憑證路徑），建立 `BankpayEngine`，呼叫 `export_vouchers()`，transport 閉包回傳 `Ok(true)`（不實際發送網路請求） | `source_system`、`customer_id`、`payer_account`、`payer_id`、`format_type` 參數被標記為 `_`（⚠ **未使用**，行 2010–2014） |
| `Import` | §2073–2104 | 建立引擎，呼叫 `import_results()` | 正常 |
| `Query` | §2105–2139 | 建立引擎，呼叫 `query()`，顯示結果 | 正常 |

**⚠ 待確認**：CLI Export 分支中，`source_system`、`customer_id`、`payer_account`、`payer_id`、`format_type` 雖在 `BankpayAction::Export` 中定義為參數，但在 `run_bankpay` 中被 `_` 忽略，未傳遞至 `build_twd_payment_request` 或 `export_vouchers`。這意味著 CLI 匯出目前不使用這些參數。

---

## 12. 測試對照表

### 12.1 wellfit_bankpay 單元測試（bankpay.rs 行 763–957）

| 測試函式 | 行號 | 測試內容 | 狀態 |
|----------|------|---------|------|
| `test_export_vouchers_dedup_and_lock` | §784 | 匯出去重與冪等鎖定 | ✔ 通過 |
| `test_export_isolates_bad_amount` | §810 | 錯誤金額隔離 | ✔ 通過 |
| `test_assemble_media_file_and_sha256` | §829 | 媒體檔組裝與 SHA-256 | ✔ 通過 |
| `test_import_results_records_findings` | §849 | 匯入結果記錄發現 | ✔ 通過 |
| `test_end_to_end_traceability_chain` | §867 | 端對端追溯鏈（憑單→媒體檔→SHA-256→銀行結果→匯入→查詢） | ✔ 通過 |
| `test_encoding_round_trip_used_by_engine` | §949 | 編碼來回轉換 | ✔ 通過 |

### 12.2 wellfit_bankpay 單元測試（media.rs 行 145–200）

| 測試函式 | 行號 | 測試內容 | 狀態 |
|----------|------|---------|------|
| `test_assemble_single_record` | §172 | 組裝單筆記錄 | ✔ 通過 |
| `test_empty_records_rejected` | §182 | 空記錄被拒絕 | ✔ 通過 |
| `test_base64_roundtrip` | §188 | Base64 來回轉換 | ✔ 通過 |
| `test_max_records_limit` | §196 | 超過 1000 記錄被拒絕 | ✔ 通過 |

### 12.3 wellfit_bankpay 單元測試（http.rs 行 188–206）

| 測試函式 | 行號 | 測試內容 | 狀態 |
|----------|------|---------|------|
| `test_config_default` | §193 | 預設設定驗證 | ✔ 通過 |
| `test_client_creation` | §200 | 無憑證時 tls_config 為 None | ✔ 通過 |

### 12.4 wellfit_bankpay 單元測試（query.rs 行 111–161）

| 測試函式 | 行號 | 測試內容 | 狀態 |
|----------|------|---------|------|
| `test_query_voucher` | §123 | 憑單查詢 | ✔ 通過 |
| `test_query_tx` | §134 | 交易查詢 | ✔ 通過 |
| `test_query_batch` | §145 | 批次查詢 | ✔ 通過 |
| `test_query_rejects_unknown_kind` | §155 | 未知類型被拒絕 | ✔ 通過 |

### 12.5 wellfit_bankpay 單元測試（release.rs 行 50–87）

| 測試函式 | 行號 | 測試內容 | 狀態 |
|----------|------|---------|------|
| `test_verify_and_dispatch_flow` | §55 | 驗證與派送流程 | ✔ 通過 |
| `test_uncertain_dispatch_reconciliation` | §68 | 不確定派送對帳 | ✔ 通過 |
| `test_reject_quarantines` | §79 | 拒絕導致隔離 | ✔ 通過 |

### 12.6 wellfit_bankpay 整合測試（tests/integration_tests.rs 行 30–317）

| 測試函式 | 行號 | 測試內容 | 狀態 |
|----------|------|---------|------|
| `export_vouchers_success` | §30 | 匯出成功 | ✔ 定義 |
| `export_vouchers_isolates_bad_rows` | §55 | 錯誤列隔離 | ✔ 定義 |
| `import_results_success` | §76 | 匯入成功 | ✔ 定義 |
| `query_voucher_flow` | §96 | 憑單查詢流程 | ✔ 定義 |
| `query_tx_flow` | §109 | 交易查詢流程 | ✔ 定義 |
| `query_batch_flow` | §121 | 批次查詢流程 | ✔ 定義 |
| `query_rejects_unknown_kind` | §133 | 未知類型拒絕 | ✔ 定義 |
| `build_twd_payment_request_format1` | §144 | 格式 1 (媒體檔) 請求組建 | ✔ 定義 |
| `build_twd_payment_request_format2` | §176 | 格式 2 (JSON) 請求組建 | ✔ 定義 |
| `media_file_assembly` | §211 | 媒體檔組裝驗證 | ✔ 定義 |
| `cents_to_yuan_str_precision` | §247 | 金額格式化精度 | ✔ 定義 |
| `http_client_config_tls` | §256 | TLS 客戶端設定 | ✔ 定義 |
| `http_client_default_config` | §272 | 預設設定 | ✔ 定義 |
| `end_to_end_export_import_query` | §280 | 匯出→匯入→查詢端對端 | ✔ 定義 |

**總計**：32 個單元測試 + 17 個整合測試 = 49 個測試函式（✔ 已確認）

---

## 13. 已知程式碼缺口與待辦事項

| 項目 | 位置 | 說明 | 優先級 |
|------|------|------|--------|
| CLI Export 未使用 source_system/customer_id/payer_account/payer_id/format_type | `main.rs:2010–2014` | 參數被 `_` 忽略，未傳遞至引擎 | 高 |
| TLS 憑證釘固未實作 | `http.rs:39` (gap 註記) | `05-security-and-audit.md` §2 列為 gap | 高 |
| 其他 6 個 API 端點未實作 | `api.rs` 定義存在，但 `http.rs` 無對應方法 | FX、對帳單、餘額、列印等 | 中（本階段外） |
| `QueryResult.file_sha256` / `release_state` 固定為 None | `query.rs:105–106` | 未填入實際值 | 低 |
| `notify_email` 寬度不一致 | `media.rs` (40) vs `api.rs` (400) | 媒體檔與 JSON 規格寬度不同 | 低（規格差異） |

---

## 14. 如何使用本文件進行維護

| 維護任務 | 起點 | 後續檢查 |
|---------|------|---------|
| 修改媒體檔欄位寬度 | `media.rs` §59–77 (§8.2) | 更新 `RECORD_LENGTH` 驗證（§16），更新測試 `media_file_assembly`（integration_tests.rs §211） |
| 新增 API 端點 | `http.rs` §162（範例） | 新增 `api.rs` 模型，更新 `04-interface-specification.md`，新增測試 |
| 修改 TLS 憑證邏輯 | `http.rs` §58–109 | 更新 `05-security-and-audit.md` §2，新增憑證釘固測試 |
| 修改匯出流程 | `bankpay.rs` §510–649 | 更新 `02-business-workflow.md` §2，檢查 ReleaseGate 整合 |
| 新增測試 | `tests/integration_tests.rs` | 確保 `cargo test -p wellfit_bankpay` 全部通過，clippy 零警告 |

---

*本文件以實際原始碼為證據基礎。所有行號參考 `Wellfit/crates/wellfit_bankpay/` 下的檔案。若程式碼變更導致行號不符，請以函式名稱與結構體名稱為準進行定位。*
