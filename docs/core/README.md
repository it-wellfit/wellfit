# Wellfit — 核心知識模型

**文件編號**: WFLT-CORE-000  
**版本**: v1.0  
**狀態**: Draft  
**依據**: `Dev/Wellfit/crates/wellfit_bankpay/src/` 實際原始碼、`Dev/Wellfit/crates/wellfit_core/src/` 共享核心、E.Sun Bank 技術文件 V2.8  
**讀者**: 架構師、資深工程師、測試、維運

---

## 1. 系統邊界

```
┌─────────────────────────────────────────────────────────────────┐
│                     Wellfit BankPay 系統邊界                      │
│                                                                 │
│  ┌──────────────┐    HTTPS/TLS 1.2+ 2-Way SSL    ┌──────────┐  │
│  │ FlapsErp 輔翼 │───────────────────────────────►│ 安控元件 │  │
│  │ ERP (應付憑單 │  port 8443 (內部)              │ (本地)   │  │
│  │  CSV/JSON 匯出 │                                │ eToken   │  │
│  └──────────────┘                                │ 簽章     │  │
│                                                  │ port 9443│  │
│   ┌──────────────────────────────────────────────┘          │  │
│   │    Wellfit BankPay Engine (wellfit_bankpay crate)      │  │
│   │  - Voucher model / ColumnContract                     │  │
│   │  - IdempotencyLedger (SQLite)                         │  │
│   │  - ReleaseGate state machine                          │  │
│   │  - API models (api.rs) / HTTP client (http.rs)        │  │
│   │  - Media file assembler (media.rs)                    │  │
│   │  - Traceable query (query.rs)                         │  │
│   └──────────────────────────────────────────────────────┘  │
│                                                  │ port 9443 │  │
│                                                  └──────────┘  │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │              E.Sun Bank 後端 (eospt/eospu/eosp)          │  │
│   │   - TWD 付款放行 / 狀態查詢 / 明細查詢 / FX 等 8 個 API  │  │
│   │   - 結果代碼 910000=成功；交易狀態 0000=成功等           │  │
│   └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

**系統不包含**：
- 銀行後端實作（由 E.Sun 提供與維運）。
- 安控元件安裝與設定（由銀行交付、ERP 伺服器與放行主管電腦安裝）。
- eToken 硬體（由銀行簽發）。
- 前端 UI（Wellfit 僅為原生 CLI 二進位，無 JavaScript、無網頁介面）。

---

## 2. 領域概念與術語表

| 術語（中文） | 英文名稱 | 定義 | 本專案具體用途 |
|--------------|----------|------|----------------|
| 應付憑單 | Accounts Payable Voucher | 企業應支付給供應商的單據，包含金額、收款帳戶、到期日等 | `Voucher` 結構（`crates/wellfit_bankpay/src/bankpay.rs`），來源於 FlapsErp 匯出的 CSV |
| 批次識別碼 | Batch ID | 一組憑單的唯一標識 | `batch_id` 參數（`--batch`），寫入 `file_batches` 表 |
| 冪等鍵 | Idempotency Key | 防止重複處理的唯一鍵 | `VOUCHER_NO:AMOUNT:BATCH_DATE`，寫入 `transaction_keys` 表 |
| 媒體檔 | Media File | 銀行接受的固定長度付款檔 | 422 字元/記錄、BIG-5 編碼（`media.rs` `assemble_twd_media_file`） |
| 內容雜湊 | Content Hash | 檔案完整性驗證 | SHA-256（`media_file_sha256`），寫入 ledger |
| 發行閘門 | Release Gate | 付款發送前的狀態機 | `DraftGenerated → VerifiedPass/Quarantined → AuthorizedDispatch → Completed/Failed/DispatchUncertain` |
| 不確定發送 | Dispatch Uncertain | 發送後未收到確認，狀態不明 | 需透過狀態查詢 API 復原（`reconcile_uncertain`） |
| 結果代碼 | Result Code | E.Sun API 回應狀態碼 | `910000`=成功，其他為失敗（`api.rs` `api_result`） |
| 交易狀態 | Transaction Status | 付款在銀行端的處理狀態 | `0000`=成功、`1000-1008`=處理中、`4000`=失敗、`9999`=無資料 |
| 定點十進位 | Scaled Decimal | 零浮點的金額運算 | `ScaledDecimal`（`wellfit_core::scaled`，10^4 縮放） |
| 欄位契約 | Column Contract | CSV 欄位名稱→索引解析 | `ColumnContract` / `ColumnMap`（`wellfit_core::contract`） |
| 隔離紀錄 | Quarantine Record | 無法解析的資料列 | `log_quarantine`，寫入 `quarantine_log` 表 |
| 稽核發現 | Finding | 每一筆異常或查詢的稽核證據 | `log_finding`，寫入 `validation_findings` 表 |

---

## 3. Crate 與模組責任

Wellfit 工作區共 5 個 crate。BankPay 相關如下：

| Crate | 路徑 | 職責 | 關鍵模組 |
|-------|------|------|----------|
| `wellfit_core` | `crates/wellfit_core` | 共享核心：SQLite 冪等帳本、Release Gate、欄位契約、定點十進位、文字切分 | `IdempotencyLedger`、`ReleaseGate`、`ColumnContract`、`ScaledDecimal`、`text::split_records` |
| `wellfit_bankpay` | `crates/wellfit_bankpay` | E.Sun BankPay 整合：API 模型、HTTP 客戶端、媒體檔、引擎、查詢 | `api.rs`、`bankpay.rs`、`http.rs`、`media.rs`, `encoding.rs`、`query.rs`、`release.rs` |
| `wellfit_cli` | `crates/wellfit_cli` | `wellfit.exe` 進入點，`bankpay {export\|import\|query}` 子命令 | `main.rs` `BankpayAction::Export/Import/Query` |
| `wellfit_shopee` | `crates/wellfit_shopee` | Wave 1：蝦皮訂單 → 輔翼 ERP 匯入 | （與 BankPay 無直接關聯） |
| `wellfit_reports` | `crates/wellfit_reports` | Wave 2：庫存報表解析與發布閘門 | （與 BankPay 無直接關聯） |
| `wellfit_mtrix` | `crates/wellfit_mtrix` | Wave 3：MTRIX 五檔驗證與修復 | （與 BankPay 無直接關聯） |

### 3.1 wellfit_bankpay 模組詳解

| 模組 | 檔案 | 責任 | 公開型別與函式 |
|------|------|------|---------------|
| 公共匯出 | `lib.rs` | 重出所有模組 | `pub mod api/bankpay/encoding/http/media/query/release` |
| API 模型 | `api.rs` | E.Sun V2.8 請求/回應 serde 結構 | `TwdPaymentTx`, `TwdPaymentRequest`, `TwdPaymentResponseData`, `TwdStatusQueryRequest`, `RqHeader`, `RsStatus`, `api_result`, `tx_status`, `CURRENCY_CODES` |
| 引擎 | `bankpay.rs` | 憑單讀取、API 轉換、冪等鎖、ReleaseGate 整合 | `BankpayEngine`, `voucher_to_twd_tx`, `build_twd_payment_request`, `query_twd_status`, `is_api_success`, `is_tx_success`, `export_vouchers`, `import_results` |
| HTTP 客戶端 | `http.rs` | ureq + rustls，2-Way SSL，POST JSON | `EsunHttpClient`, `EsunHttpClientConfig`, `post_json`, `create_twd_payment`, `retrieve_twd_status` |
| 媒體檔 | `media.rs` | 422-char 固定長度 BIG-5 組裝、base64 | `assemble_twd_media_file`, `TwdMediaRecord`, `base64_encode_media`, `base64_decode_media` |
| 編碼轉換 | `encoding.rs` | UTF-8 ↔ CP950/Big5/Big5-HKSCS | `BankEncoding`, `to_utf8`, `from_utf8`, `detect_encoding` |
| 追溯查詢 | `query.rs` | voucher/tx/batch 三向查詢，每次查詢記錄 Finding | `QueryResult`, `query` |
| 發布閘門 | `release.rs` | ReleaseGate 封裝 | `verify_payload`, `dispatch_payload`, `reconcile_uncertain`, `is_terminal`, `is_uncertain` |

---

## 4. 核心資料模型

### 4.1 Voucher（應付憑單）

| 欄位 | 型別 | 說明 |
|------|------|------|
| `voucher_no` | `String` | 憑單編號（如 `AP-00123`） |
| `vendor_code` | `String` | 供應商代碼 |
| `amount_cents` | `i64` | 應付金額（整數分） |
| `due_date` | `String` | 到期日（ISO-8601） |
| `bank_account` | `String` | 受益人銀行帳戶 |
| `reference` | `String` | 自由文字備註 |
| `line` | `usize` | 來源列號（稽核用） |
| `raw` | `String` | 原始來源列（隔離複原用） |

### 4.2 TWDPaymentTx（銀行付款交易）

對應 E.Sun API `twdTxList` 欄位（`api.rs`）：

| 欄位 | 必填 | 長度 | 說明 |
|------|------|------|------|
| `payerAccountNo` | Y | 16 | 玉山帳號 (≥13 碼) |
| `payerId` | Y | 15 | 歸戶統編 (≥10 碼) |
| `payeeBankId` | Y | 3 | 收款銀行代碼 |
| `payeeBranchId` | Y | 4 | 收款銀行分行 |
| `payeeAccountNo` | Y | 16 | 收款帳號 |
| `payeeName` | Y | 80 | 收款戶名 |
| `txDate` | Y | 8 | YYYYMMDD |
| `txAmt` | Y | 13 | 付款金額 (>0) |
| `txSeq` | Y | 22 | ERP 交易編號（不可重複） |
| `chargeFeeType` | Y | 1 | 0=外收, 1=內扣 |
| 其他欄位 | N | — | memo, notifyMethod, notifyEmail, notifyFax, writeOffNo, remark, payeeId, detailList |

### 4.3 IdempotencyLedger（SQLite 冪等帳本）

| 表 | 用途 |
|----|------|
| `file_batches` | 批次登記（`register_file_batch`，重複內容拒絕） |
| `transaction_keys` | 冪等鍵鎖定（`check_and_lock_order`，`VOUCHER_NO:AMOUNT:BATCH_DATE`） |
| `validation_findings` | 稽核發現（`log_finding`，每一筆異常或查詢） |
| `quarantine_log` | 隔離紀錄（`log_quarantine`，無法解析的列） |

---

## 5. 核心執行流程

### 5.1 匯出（ERP → 銀行）

```
讀取 FlapsErp 憑單 CSV
    → ColumnContract 欄位解析 → ScaledDecimal 金額
    → 冪等鍵鎖定 (VOUCHER_NO:AMOUNT:BATCH_DATE)
    → register_file_batch (登記批次)
    → 建構 TwdPaymentRequest (JSON, formatType=2)
    → ReleaseGate.verify_payload (two-way door 驗證)
    → EsunHttpClient.create_twd_payment (POST /payment/createTwdPaymentInstruction)
    → 解析 rsStatus.code (910000=成功)
    → 更新 ledger (transaction_keys, validation_findings)
    → 若非終端狀態 → DispatchUncertain → query_twd_status 復原
```

### 5.2 匯入（銀行 → ERP）

```
讀取銀行結果檔/回應
    → 驗證欄位契約
    → 比對 bank_ref 與 ledger transaction_keys
    → log_finding (VR-BANKPAY-02，每一筆結果)
    → 無法比對 → quarantine_log (VR-BANKPAY-03)
    → 更新憑單狀態 (已付款/失敗)
```

### 5.3 追溯查詢（Traceable Query）

```
query(kind="voucher", key="AP-001")
    → is_order_processed("AP-001") → processed flag
    → log_finding (VR-BANKPAY-QUERY)
    → 返回 batch_id, findings

query(kind="tx", key="EREF-999")
    → 跨參考 bank_ref 與 transaction_keys
    → log_finding (VR-BANKPAY-QUERY)

query(kind="batch", key="BATCH-20261102")
    → count_file_batches → registered count
    → log_finding (VR-BANKPAY-QUERY)
```

---

## 6. 與實際原始碼的對應表

| 知識模型概念 | 程式碼位置 | 函式/型別 |
|--------------|------------|-----------|
| Voucher | `crates/wellfit_bankpay/src/bankpay.rs` | `struct Voucher` |
| TWDPaymentTx | `crates/wellfit_bankpay/src/api.rs` | `struct TwdPaymentTx` |
| IdempotencyLock | `crates/wellfit_core/src/ledger.rs` | `check_and_lock_order(key, batch_id, amount)` |
| ReleaseGate | `crates/wellfit_core/src/release_gate.rs` | `ReleaseGate`, `ReleaseState` |
| Media assembler | `crates/wellfit_bankpay/src/media.rs` | `assemble_twd_media_file`, `TwdMediaRecord` |
| HTTP client | `crates/wellfit_bankpay/src/http.rs` | `EsunHttpClient`, `post_json` |
| Encoding | `crates/wellfit_bankpay/src/encoding.rs` | `BankEncoding`, `to_utf8`, `from_utf8` |
| Traceable query | `crates/wellfit_bankpay/src/query.rs` | `QueryResult`, `query` |
| CLI entry | `crates/wellfit_cli/src/main.rs` | `BankpayAction::Export/Import/Query` |

---

## 7. 已知限制與未決事項

| 項目 | 說明 | 狀態 |
|------|------|------|
| HTTP 客戶端 TLS 憑證 | `EsunHttpClientConfig` 有 `client_cert_path`/`client_key_path`/`ca_cert_path`，但 `post_json` 尚未實作 rustls 憑證載入（需待銀行憑證可用後實作） | 待確認 |
| FlapsErp COM 介面 | `OQ-E3`：需確認 ERP 提供的 COM ProgID/方法，或改用 HTTP API | 待銀行/ERP 廠商確認 |
| 銀行憑證與認證 | SourceSystemId、ClientId、啟用碼、2-Way SSL 憑證皆需銀行提供 | 待銀行交付 |
| FX 波次 | TWD 已完整模型；FX（外幣）API 模型已定義但尚未整合進引擎 | P3 後續 |

---

> **事實來源**：以上所有模組名稱、函式簽名、欄位定義，均可在 `Dev/Wellfit/crates/wellfit_bankpay/src/` 與 `Dev/Wellfit/crates/wellfit_core/src/` 中直接查證。銀行規格（欄位長度、必填、錯誤碼、狀態代碼）來源於 `Dev/Wellfit/docs/bankpay/references/` 的 E.Sun Bank 技術文件 V2.8。
