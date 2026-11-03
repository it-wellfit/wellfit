# Wellfit BankPay 資料模型與生命週期

**文件編號**：WFLT-DOC-006  
**版本**：1.0  
**狀態**：Draft  
**文件語言**：繁體中文（技術名詞保留標準英文）  
**證據來源**：`crates/wellfit_bankpay/src/` 原始碼、`crates/wellfit_core/src/` 共享核心

---

## 1. 使用說明

本文件說明 Wellfit BankPay 模組的所有資料實體、欄位、型別、關係與生命週期。區分三個資料層次：

1. **記憶體中的程式型別**（Rust 結構體）
2. **傳輸資料格式**（JSON API 請求/回應、CSV 檔案）
3. **持久化結構**（SQLite Idempotency Ledger）

必須明確區分這三層：同一個欄位在三層中的名稱、型別、長度可能不同。

---

## 2. 記憶體中的領域實體（Rust 結構體）

### 2.1 Voucher — ERP 應付憑單

| 屬性 | 型別 | 可空 | 初始值 | 必要性 | 約束與驗證 | 來源與去向 | 生命週期 | 業務意義 |
|------|------|------|--------|--------|-----------|-----------|---------|---------|
| `voucher_no` | `String` | 否 | — | 必填 | 非空字串 | 從 CSV 標題欄 `VOUCHER_NO` 解析 | 建立於 CSV 解析，不變 | ERP 憑單編號，例如 `AP-001` |
| `vendor_code` | `String` | 否 | — | 必填 | — | 從 CSV 欄 `VENDOR_CODE` | 建立於 CSV 解析，不變 | 廠商/供應商代碼 |
| `amount_cents` | `i64` | 否 | — | 必填 | 透過 `ScaledDecimal::parse` 解析，除以 10,000 得分 | 從 CSV 欄 `AMOUNT`，例如 `"1000.0000"` → `1000` | 建立於 CSV 解析，不變 | 應付金額，以整數分儲存，避免浮點數 |
| `due_date` | `String` | 是 (可空字串) | `""` | 選填 | ISO-8601 格式 | 從 CSV 欄 `DUE_DATE`，選填 | 建立於 CSV 解析，不變 | 到期日，空字串表示使用當日 |
| `bank_account` | `String` | 否 | — | 必填 | 非空字串 | 從 CSV 欄 `BANK_ACCOUNT` | 建立於 CSV 解析，不變 | 收款人銀行帳號 |
| `reference` | `String` | 是 (可空字串) | `""` | 選填 | — | 從 CSV 欄 `REFERENCE`，選填 | 建立於 CSV 解析，不變 | 自由文字參考/備註 |
| `line` | `usize` | 否 | — | 必填 | 一-based 行號 | 從 CSV 解析時的行計數器 | 建立於 CSV 解析，不變 | 稽核用的來源行號 |
| `raw` | `String` | 否 | — | 必填 | — | CSV 原始行，以 `,` 連接 | 建立於 CSV 解析，不變 | 隔離重現用的原始行內容 |

**所在檔案**：`crates/wellfit_bankpay/src/bankpay.rs:32–49`  
**建立方式**：`parse_voucher_row()` (行 169–201)  
**錯誤情境**：`VOUCHER_NO` 空值 → `Err`；`BANK_ACCOUNT` 空值 → `Err`；`AMOUNT` 無法解析 → `Err`  
**測試**：`test_export_vouchers_dedup_and_lock`、`test_export_isolates_bad_amount`、`test_end_to_end_traceability_chain`

---

### 2.2 BankResult — 銀行付款結果

| 屬性 | 型別 | 可空 | 初始值 | 必要性 | 約束與驗證 | 來源與去向 | 生命週期 | 業務意義 |
|------|------|------|--------|--------|-----------|-----------|---------|---------|
| `bank_ref` | `String` | 否 | — | 必填 | 非空字串 | 從 CSV 欄 `BANK_REF` | 建立於 CSV 解析，不變 | 銀行交易參考號，用於對帳 |
| `voucher_no` | `String` | 否 | — | 必填 | 非空字串 | 從 CSV 欄 `VOUCHER_NO` | 建立於 CSV 解析，不變 | 對應的 ERP 憑單號 |
| `status` | `String` | 否 | — | 必填 | `paid`/`failed`/`uncertain` | 從 CSV 欄 `STATUS` | 建立於 CSV 解析，不變 | 結算狀態 |
| `amount_cents` | `i64` | 否 | — | 必填 | 透過 `ScaledDecimal::parse` 解析 | 從 CSV 欄 `AMOUNT` | 建立於 CSV 解析，不變 | 結算金額，整數分 |
| `processed_at` | `String` | 是 (可空字串) | `""` | 選填 | ISO-8601 時間戳記 | 從 CSV 欄 `PROCESSED_AT`，選填 | 建立於 CSV 解析，不變 | 銀行處理時間 |
| `line` | `usize` | 否 | — | 必填 | 一-based 行號 | 從 CSV 解析時的行計數器 | 建立於 CSV 解析，不變 | 稽核用的來源行號 |
| `raw` | `String` | 否 | — | 必填 | — | CSV 原始行 | 建立於 CSV 解析，不變 | 隔離重現用的原始行內容 |

**所在檔案**：`crates/wellfit_bankpay/src/bankpay.rs:52–68`  
**建立方式**：`parse_bank_result_row()` (行 204–234)  
**錯誤情境**：`BANK_REF` 空值 → `Err`；`VOUCHER_NO` 空值 → `Err`；`AMOUNT` 無法解析 → `Err`  
**測試**：`test_import_results_records_findings`、`test_end_to_end_traceability_chain`

---

### 2.3 ExportResult — 匯出結果

| 屬性 | 型別 | 說明 |
|------|------|------|
| `batch_id` | `String` | 匯出批次識別碼 |
| `media_file` | `String` | 生成的媒體檔案路徑 (例如 `BATCH-1.bankpay.media`) |
| `file_sha256` | `String` | 媒體檔 SHA-256 雜湊 (64 碼十六進位) |
| `vouchers_accepted` | `usize` | 通過驗證的憑單數量 |
| `vouchers_isolated` | `usize` | 被隔離的錯誤憑單數量 |
| `dispatched` | `bool` | 是否成功派送至銀行 |
| `release_state` | `String` | 發布閘門最終狀態的 Debug 字串 |

**所在檔案**：`crates/wellfit_bankpay/src/bankpay.rs:72–80`  
**建立方式**：`export_vouchers()` 方法結尾 (行 640–648)

---

### 2.4 ImportResult — 匯入結果

| 屬性 | 型別 | 說明 |
|------|------|------|
| `batch_id` | `String` | 匯入批次識別碼 |
| `results_accepted` | `usize` | 接受的銀行結果數量 |
| `results_isolated` | `usize` | 被隔離的結果數量 |
| `matched_vouchers` | `usize` | 成功對帳的憑單數量 |
| `unmatched` | `usize` | 無法對帳的結果數量 |

**所在檔案**：`crates/wellfit_bankpay/src/bankpay.rs:84–90`  
**建立方式**：`import_results()` 方法結尾 (行 742–748)

---

### 2.5 TwdMediaRecord — E.Sun V2.8 媒體檔記錄

| 屬性 | 型別 | 寬度 | 必要性 | 說明 |
|------|------|------|--------|------|
| `payer_account_no` | `String` | 16 | Y | 付款人帳號 |
| `payer_id` | `String` | 15 | Y | 付款人統編 |
| `payee_bank_id` | `String` | 3 | Y | 收款銀行代碼 |
| `payee_branch_id` | `String` | 4 | Y | 收款銀行分行 |
| `payee_account_no` | `String` | 16 | Y | 收款帳號 |
| `payee_name` | `String` | 80 | Y | 收款戶名 |
| `payee_id` | `String` | 15 | N | 收款人統編 |
| `tx_date` | `String` | 8 | Y | 付款日期 YYYYMMDD |
| `tx_amt` | `String` | 13 | Y | 付款金額 |
| `charge_fee_type` | `String` | 1 | Y | 手續費負擔 (0/1) |
| `memo` | `String` | 80 | N | 附言 |
| `notify_method` | `String` | 1 | N | 通知方式 |
| `notify_email` | `String` | 40 | N | 通知 Email |
| `notify_fax` | `String` | 20 | N | 通知傳真 |
| `write_off_no` | `String` | 40 | N | 銷帳編號 |
| `remark` | `String` | 38 | N | 備註 |
| `tx_seq` | `String` | 22 | Y | 交易序號 |

**總寬度**：422 字元（包含 10 字元 trailing 空白）  
**所在檔案**：`crates/wellfit_bankpay/src/media.rs:59–77`  
**組裝方式**：`assemble_twd_media_file()` (行 23–55)  
**驗證**：每筆記錄必須正好 422 字元，否則返回 `Err`  
**編碼**：CP950 (Big5-HKSCS)，若有無法表示的字元則返回 `Err`

---

## 3. 持久化資料模型（SQLite Idempotency Ledger）

Idempotency Ledger 位於 `crates/wellfit_core/src/ledger.rs`，由 `BankpayEngine` 透過 `Arc<IdempotencyLedger>` 使用。以下為 BankPay 相關的資料表與欄位（以實際原始碼行為準，部分細節可能在 wellfit_core crate 中）：

### 3.1 檔案批次表 (file_batches)

| 欄位 | 型別 | 說明 |
|------|------|------|
| `batch_id` | `String` | 批次識別碼 |
| `batch_type` | `String` | 批次類型 (例如 `"E_SUN_PAY"`、 `"E_SUN_RESULT"`、 `"BANKPAY_QUERY"`) |
| `filename` | `String` | 來源檔案名稱 |
| `content_bytes` | `Vec<u8>` | 檔案內容位元組 |
| `content_hash` | `String` | SHA-256 雜湊，用於重複檢查 |
| `registered_at` | `DateTime` | 註冊時間 |

**方法**：`register_file_batch(batch_id, type, filename, bytes, 0)` — 若內容雜湊已存在，返回 `false` (重複拒絕)  
**查詢方法**：`count_file_batches()`、 `content_hash_of(batch_id)`

### 3.2 訂單鎖表 (order_locks)

| 欄位 | 型別 | 說明 |
|------|------|------|
| `key` | `String` | 鎖定鍵 (匯出: `VOUCHER_NO:AMOUNT:BATCH_DATE`；匯入: `BANK_REF:BATCH_ID`) |
| `batch_id` | `String` | 關聯的批次識別碼 |
| `amount` | `i64` | 金額 (分) |
| `locked_at` | `DateTime` | 鎖定時間 |

**方法**：`check_and_lock_order(key, batch_id, amount)` — 若鍵已存在，返回 `false` (鎖定失敗)

### 3.3 驗證發現表 (validation_findings)

| 欄位 | 型別 | 說明 |
|------|------|------|
| `batch_id` | `String` | 關聯批次 |
| `rule_code` | `String` | 規則代碼 (例如 `VR-BANKPAY-01`、`VR-BANKPAY-QUERY`) |
| `severity` | `String` | 嚴重度 (`Error`/`Warning`/`Info`) |
| `filename` | `String` | 來源檔案名稱 |
| `line_number` | `i32` | 行號 |
| `key_value` | `String` | 關鍵值 (例如憑單號、銀行參考號) |
| `reason` | `String` | 錯誤/資訊說明 |

**方法**：`log_finding(Finding{...})`、 `get_findings(Some(batch_id))`  
**BankPay 規則代碼**：
- `VR-BANKPAY-01`：匯出驗證錯誤 (bankpay.rs:580)
- `VR-BANKPAY-02`：匯入結果資訊 (bankpay.rs:711)
- `VR-BANKPAY-03`：隔離列錯誤 (bankpay.rs:732)
- `VR-BANKPAY-QUERY`：追溯查詢 (query.rs:90)

### 3.4 隔離檔案表 (quarantine)

| 欄位 | 型別 | 說明 |
|------|------|------|
| `batch_id` | `String` | 關聯批次 |
| `filename` | `String` | 來源檔案名稱 |
| `line_number` | `i32` | 行號 |
| `raw_line` | `String` | 原始行內容 |
| `rule_code` | `String` | 規則代碼 |
| `reason` | `String` | 錯誤說明 |

**方法**：`log_quarantine(batch_id, filename, line_no, raw_line, rule_code, reason)` (bankpay.rs:727)

---

## 4. 傳輸資料格式

### 4.1 JSON API 請求（rqHeader + rqData）

根據 `api.rs` 與 `04-interface-specification.md`，TWD 付款發放請求的 JSON 結構：

```json
{
  "rqHeader": {
    "msgId": "12345678202611020000001",
    "requestTime": "2026/11/02 10:00:00",
    "customerIxd": "12345678",
    "sourceSystem": "S001"
  },
  "rqData": {
    "formatType": "2",
    "paymentType": "1",
    "verificationType": "1",
    "flowInfo": {
      "operatorName": "",
      "flowName": "",
      "flowKey": null
    },
    "content": "Base64EncodedJSON..."
  }
}
```

其中 `content` 為 Base64 編碼的 JSON，解碼後為：

```json
[
  {
    "payerAccountNo": "ACCT-123",
    "payerId": "1234567890",
    "payeeBankId": "000",
    "payeeBranchId": "0000",
    "payeeAccountNo": "ACCT-1",
    "payeeName": "V001",
    "payeeId": "",
    "txDate": "20261103",
    "txAmt": "10.00",
    "chargeFeeType": "0",
    "memo": "INV-100",
    "notifyMethod": "1",
    "notifyEmail": "",
    "notifyFax": "",
    "writeOffNo": "AP-001",
    "remark": "",
    "txSeq": "IOB20261102AP-001",
    "detailList": []
  }
]
```

**欄位寬度對照（JSON vs 媒體檔）**：

| 欄位 | JSON 長度 (api.rs) | 媒體檔長度 (media.rs) | 差異說明 |
|------|-------------------|----------------------|---------|
| `payeeName` | 80 | 80 | 一致 |
| `memo` | 80 | 80 | 一致 |
| `notifyEmail` | 400 | 40 | ⚠ JSON 允許 400，媒體檔僅 40 |
| `payeeAccountNo` | 16 | 16 | 一致 |
| `txSeq` | 22 | 22 | 一致 |
| `remark` | 38 | 38 | 一致 |

---

### 4.2 CSV 匯出檔案格式（ERP 憑單輸入）

標題行：
```
VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT,DUE_DATE,REFERENCE
```

資料行：
```
AP-001,V001,1000.0000,ACCT-1,2026-11-03,INV-100
```

**分隔符號自動偵測**：`;` 優先，其次 `,`，最後 `\t` (bankpay.rs:753–761)  
**金額格式**：透過 `ScaledDecimal::parse` 解析，支援 `1000.0000` 等格式，結果為整數分  
**必要欄位**：`VOUCHER_NO`、`VENDOR_CODE`、`AMOUNT`、`BANK_ACCOUNT`  
**選填欄位**：`DUE_DATE`、`REFERENCE`

---

### 4.3 CSV 匯入檔案格式（銀行結果輸入）

標題行：
```
BANK_REF,VOUCHER_NO,STATUS,AMOUNT,PROCESSED_AT
```

資料行：
```
EREF-999,AP-001,paid,1000.0000,2026-11-03T09:00:00Z
```

**必要欄位**：`BANK_REF`、`VOUCHER_NO`、`STATUS`、`AMOUNT`  
**選填欄位**：`PROCESSED_AT`

---

### 4.4 422 字元固定長度媒體檔 (formatType=1)

媒體檔組裝為固定長度 BIG-5 編碼記錄，每筆正好 422 字元：

| 欄位 | 起始位置 | 長度 | 填充 |
|------|---------|------|------|
| payer_account_no | 1 | 16 | 左對齊，空白 |
| payer_id | 17 | 15 | 左對齊，空白 |
| payee_bank_id | 32 | 3 | — |
| payee_branch_id | 35 | 4 | 左對齊，空白 |
| payee_account_no | 39 | 16 | 右對齊，零填充 |
| payee_name | 55 | 80 | 左對齊，空白 |
| payee_id | 135 | 15 | 左對齊，空白 |
| tx_date | 150 | 8 | YYYYMMDD |
| tx_amt | 158 | 13 | 右對齊，零填充 |
| charge_fee_type | 171 | 1 | `0`/`1` |
| memo | 172 | 80 | 左對齊，空白 |
| notify_method | 252 | 1 | `1`/`2`/`3`/`9` |
| notify_email | 253 | 40 | 左對齊，空白 |
| notify_fax | 303 | 20 | 左對齊，空白 |
| write_off_no | 323 | 40 | 左對齊，空白 |
| remark | 363 | 38 | 左對齊，空白 |
| tx_seq | 401 | 22 | 左對齊，空白 |
| trailing | 423 | 10 | 空白 |

**最大記錄數**：1,000 筆  
**編碼**：CP950 (Big5-HKSCS)  
**傳輸**：Base64 編碼後放入 API `content` 欄位

---

## 5. 資料生命週期

### 5.1 匯出流程（ERP → 銀行）

```
1. 讀取 CSV 憑單檔案
   ↓ (bankpay.rs:518–526)
2. 編碼轉換：位元組 → UTF-8 (自動偵測 BOM)
   ↓ (bankpay.rs:527)
3. 註冊檔案批次到 IdempotencyLedger (檢查重複)
   ↓ (bankpay.rs:534–542)
4. 解析標題行，驗證欄位契約
   ↓ (bankpay.rs:544–549)
5. 逐行解析憑單：驗證金額、帳號，鎖定冪等鍵 VOUCHER_NO:AMOUNT:BATCH_DATE
   ↓ (bankpay.rs:554–591)
   - 錯誤行：隔離，記錄 VR-BANKPAY-01 Finding
   - 重複鍵：隔離 (已處理)
6. 組裝媒體檔 (CSV 或 422 字元 BIG-5)，計算 SHA-256
   ↓ (bankpay.rs:593–594)
7. 寫入媒體檔案到磁碟
   ↓ (bankpay.rs:596–598)
8. ReleaseGate.verify_payload() 檢查 (無隔離行、無編碼錯誤)
   ↓ (bankpay.rs:601–609)
9. 若 valid，呼叫 transport 閉包派送至安全元件
   ↓ (bankpay.rs:611–636)
10. 回傳 ExportResult
    ↓ (bankpay.rs:640–648)
```

**終止狀態**：`Completed` (成功派送) / `Failed` (派送失敗) / `DispatchUncertain` (逾時，需對帳)  
**失敗路徑**：任何步驟返回 `Err` 即中止，已註冊的批次不會自動回滾（需手動處理）

---

### 5.2 匯入流程（銀行 → ERP）

```
1. 讀取銀行結果 CSV 檔案
   ↓ (bankpay.rs:665–667)
2. 註冊結果批次到 IdempotencyLedger
   ↓ (bankpay.rs:674–677)
3. 解析標題行，驗證欄位契約
   ↓ (bankpay.rs:679–684)
4. 逐行解析銀行結果：
   ↓ (bankpay.rs:691–739)
   - BANK_REF 已結算：隔離 (重複)
   - 無法解析：log_quarantine (VR-BANKPAY-03)，unmatched++
   - 正常：log_finding (VR-BANKPAY-02)，matched++
5. 回傳 ImportResult
   ↓ (bankpay.rs:742–748)
```

**終止狀態**：所有行處理完畢  
**對帳關鍵**：透過 `bank_ref` 與 `voucher_no` 建立銀行結果與 ERP 憑單的關聯

---

### 5.3 追溯查詢流程

```
1. 呼叫 query(engine, kind, key)
   ↓ (query.rs:37)
2. 註冊查詢批次到 IdempotencyLedger (類型 "BANKPAY_QUERY")
   ↓ (query.rs:48–54)
3. 根據 kind 執行查詢：
   - voucher: is_order_processed(key)
   - tx: 回傳 bank_ref
   - batch: 回傳已註冊批次數量
   ↓ (query.rs:58–84)
4. 記錄 VR-BANKPAY-QUERY Finding (查詢本身亦受稽軌跡)
   ↓ (query.rs:87–97)
5. 回傳 QueryResult (包含 findings)
   ↓ (query.rs:99–108)
```

**終止狀態**：查詢完成，所有查詢行為皆留下稽軌跡  
**稽軌跡價值**：查詢行為本身可被稽核，防止未記錄的資料查看

---

### 5.4 發布閘門狀態機生命週期

```
DraftGenerated
    │ verify_payload(true, "")
    ▼
VerifiedPass ──┐
               │ dispatch(transport)
               ▼
        ┌──────────────┐
        │  transport   │
        │  (成功/失敗)   │
        └──────┬───────┘
               │
      ┌────────┴────────┐
      ▼                 ▼
  Completed       DispatchUncertain  ← 逾時或不確定
      │                 │
      │ reconcile_uncertain()
      ▼                 ▼
  Completed       Failed (若對帳失敗)
```

**所在檔案**：`crates/wellfit_core/src/release_gate.rs` (wellfit_core)，包裝層 `crates/wellfit_bankpay/src/release.rs`  
**歷程記錄**：所有狀態轉換寫入 `history: Vec<TransitionRecord>` (03-technical-design.md §6)

---

## 6. 資料擁有權與存取控制

| 資料 | 擁有模組 | 讀取權限 | 修改權限 | 介面 |
|------|---------|---------|---------|------|
| `Voucher` | `wellfit_bankpay` | 匯出流程 | 匯出流程 (建立後不變) | `parse_voucher_row()` |
| `BankResult` | `wellfit_bankpay` | 匯入流程 | 匯入流程 (建立後不變) | `parse_bank_result_row()` |
| `IdempotencyLedger` | `wellfit_core` (共享) | BankPay、Shopee、Reports、MTRIX | 各模組透過方法 | `register_file_batch()`、`check_and_lock_order()`、`log_finding()` |
| `ReleaseGate` | `wellfit_core` (共享) | BankPay | BankPay (透過 `export_vouchers`) | `verify_payload()`、`dispatch()` |
| 媒體檔案 | 檔案系統 | 匯出流程 | 匯出流程 (寫入 `.bankpay.media`) | `fs::write()` |
| 驗證發現 (Finding) | `wellfit_core` (共享) | 所有波 | 各模組 | `log_finding()` |

---

## 7. 已知資料模型缺口

| 項目 | 位置 | 說明 | 優先級 |
|------|------|------|--------|
| `QueryResult.file_sha256` / `release_state` 固定為 `None` | `query.rs:105–106` | 追溯查詢未填入實際的媒體檔雜湊與發布狀態 | 低 |
| `notify_email` 寬度不一致 | `media.rs` (40) vs `api.rs` (400) | JSON 規格允許 400 字元，媒體檔僅 40 字元 | 低 (規格差異) |
| SQLite 實體定義未直接可見 | `wellfit_core/src/ledger.rs` | Ledger 在 wellfit_core crate，非 wellfit_bankpay | 中 (需交叉檢查) |

---

## 8. 如何修改資料模型

| 修改任務 | 起點 | 需同步更新 |
|---------|------|---------|
| 新增 Voucher 欄位 | `bankpay.rs:32` (結構體定義) | `parse_voucher_row()` (行 169)、`voucher_contract()` (行 112)、`04-interface-specification.md`、測試 |
| 新增 BankResult 欄位 | `bankpay.rs:52` (結構體定義) | `parse_bank_result_row()` (行 204)、`bank_result_contract()` (行 121)、測試 |
| 新增媒體檔欄位 | `media.rs:59` (TwdMediaRecord) | `format_twd_record()` (行 80)、`RECORD_LENGTH` 驗證 (行 16)、測試 `media_file_assembly` |
| 新增 Ledger 規則代碼 | `bankpay.rs` (log_finding 呼叫) | `05-security-and-audit.md` §6、測試 |
| 修改金額處理 | `bankpay.rs:277` (cents_to_yuan_str) | `ScaledDecimal` (wellfit_core)、所有金額欄位 |

**重要原則**：修改任何資料欄位前，必須先更新 `CODE_REFERENCE.md` 與 `DATA_MODEL.md`，再修改程式碼，最後更新測試與文件。

---

*本文件以 `crates/wellfit_bankpay/src/` 與 `crates/wellfit_core/src/` 的實際原始碼為證據基礎。未直接可見的 SQLite 實體定義標記為需交叉檢查。*
