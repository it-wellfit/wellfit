# Wellfit CLI 命令參考

> 文件編號：WFLT-DOC-CLI-001｜狀態：Draft｜版本：v1.0｜日期：2026-10-20
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 變更摘要：v1.0。依 `wellfit --help` 與各子命令 `--help` 之**實際輸出**重寫；新增 `spec` 子命令；
> 修正 `--input data/fuyi_inventory.csv` 與 `--dir data` 兩個錯誤範例。
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

> 本檔是專案層的精簡參考；知識庫層的完整版（含語意說明）見
> [`../../Knowledge/ops/CLI.md`](../../../Knowledge/ops/CLI.md)。

## 0. 總覽

```
Usage: wellfit.exe <COMMAND>

Commands:
  status   Show the idempotency ledger state and the isolated records
  shopee   Wave 1 - Shopee inbound orders to the Fuyi ERP import batch
  reports  Wave 2 - Fuyi ERP inventory report with the release gate
  mtrix    Wave 3 - Alpargatas MTRIX five file drop
  audit    Full physical audit over the real files in the data folder
  spec     Print the MTRIX contract and rule catalogue
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

**執行檔位置**：`%LOCALAPPDATA%\Temp\wellfit-target\debug\wellfit.exe`
（由 `build.ps1` 建置，target 在儲存庫之外——本機應用程式控制原則會阻擋桌面路徑下的產物）。

**工作目錄**：建議於 `Dev/Wellfit` 下執行；`--dir`、`--input`、`--data-dir` 皆相對工作目錄解析。

## 1. `status`

```
Usage: wellfit.exe status [OPTIONS]

Options:
  -d, --db <DB>  Ledger database path [default: data/wellfit_ledger.sqlite]
```

```powershell
& $wf status --db "$env:TEMP\wellfit-ledger.sqlite"
```

```
Wellfit system status
  ledger            : <帳本路徑>
  isolated records  : 0
  validation errors : 22
  validation warnings: 24
```

> **數字口徑說明（LOCAL-053）**：`status` 顯示**帳本累計值**
> （`get_findings(None)`／`get_quarantine_items(None)`，跨所有波次），
> 而波次輸出的 `errors`／`warnings` 僅為**本波次**新產生的 finding 數。
> 兩者在單次稽核中可能不一致：`status` 是生命週期累計，波次輸出是單次波次。

## 2. `shopee`

```
Usage: wellfit.exe shopee [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>    Source order export
  -o, --output <OUTPUT>  Destination ERP import CSV
  -b, --batch <BATCH>    Batch identifier [default: BATCH-CLI-SHOPEE]
      --db <DB>          Ledger database path [default: data/wellfit_ledger.sqlite]
```

```powershell
& $wf shopee --input data/shopee_orders.csv --output "$env:TEMP\erp_shopee_import.csv"
```

```
wave 1 shopee: 2 order(s) accepted, 0 record(s) isolated
  ERP import batch   : <輸出路徑>
```

- **契約**：`Shopee inbound order export`，8 個必填欄位
  （`ORDER_NO`、`CUSTOMER_REF`、`ORDER_DATETIME`、`ORDER_TOTAL`、`ITEM_NO`、`QTY`、`UNIT_PRICE`、`LINE_AMOUNT`）。
- 缺任一必填欄 → 整批拒收，訊息格式：
  `Shopee column contract violation: Shopee inbound order export: required column(s) missing: <COL> (header: ...)`
- 金額守恆（EX-06）不符 → 該列隔離（`ERR_SUM_MISMATCH_EX06`）。
- 金額／數量格式錯誤 → 該列隔離（`ERR_MONEY_FORMAT`／`ERR_QTY_FORMAT`），**不當成 0**。

> `data/shopee_orders.csv` 為**合成示範資料**，非蝦皮實際匯出檔。

## 3. `reports`

```
Usage: wellfit.exe reports [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>          Source inventory report
  -r, --recipient <RECIPIENT>  Recipient address, repeatable or comma separated
  -b, --batch <BATCH>          Batch identifier [default: BATCH-CLI-REPORT]
      --db <DB>                Ledger database path [default: data/wellfit_ledger.sqlite]
```

```powershell
& $wf reports --input data/fuyi_inventory_report.csv --recipient finance@dachen.com.tw
```

```
wave 2 reports: 4 record(s) accepted, 0 record(s) isolated
  recipients        : finance@dachen.com.tw
```

- **契約**：`Fuyi ERP inventory report`，8 個必填欄位
  （`ITEM_NO`、`ITEM_NAME`、`WAREHOUSE_CODE`、`BEGIN_INV`、`IN_QTY`、`OUT_QTY`、`ADJ_QTY`、`END_INV`）。
- 庫存恆等式不符 → 該列隔離（`ERR_INV_BALANCE_MISMATCH`）。
- 數量不可解析 → 該列隔離（`ERR_INV_NUMBER_FORMAT`），**不當成 0**。

> **更正**：舊版文件寫的是 `--input data/fuyi_inventory.csv`，該檔不存在。
> 正確檔名為 `data/fuyi_inventory_report.csv`。
> 該檔為**合成示範資料**，非輔翼 ERP 實際匯出檔。

## 4. `mtrix`

```
Usage: wellfit.exe mtrix [OPTIONS] --dir <DIR>

Options:
  -d, --dir <DIR>                Folder holding the five delivered files
  -o, --output-dir <OUTPUT_DIR>  Folder for the repaired files
      --report <REPORT>          Validation report destination
      --strict                   Exit with a non zero code when a validation error is recorded
  -b, --batch <BATCH>            Batch identifier [default: BATCH-CLI-MTRIX]
      --db <DB>                  Ledger database path [default: data/wellfit_ledger.sqlite]
```

```powershell
& $wf mtrix --dir spec/fixtures/mtrix/delivered `
            --output-dir "$env:TEMP\wellfit-repaired" `
            --report "$env:TEMP\wellfit-report.json" `
            --batch DEMO-20261020 `
            --db "$env:TEMP\wellfit-ledger.sqlite"
```

```
wave 3 mtrix: status=partial-accepted files=5 errors=22 warnings=24 repairs=38
  - ALTHCI rows 4/4 accepted, 0 isolated, 5 repairs
  - ALTHES rows 4/4 accepted, 0 isolated, 5 repairs
  - ALTHFV rows 4/4 accepted, 0 isolated, 12 repairs
  - ALTHPD rows 4/4 accepted, 0 isolated, 5 repairs
  - ALTHVD rows 9/9 accepted, 0 isolated, 11 repairs
  - VR-01 warning ALTHCI line 0 ALTHCI30062020153005130 2 (3).csv: file name carries trailing residue ' 2 (3).csv' after the timestamp
  - VR-08 error ALTHVD30062020153005130 5 (4).csv line 2 7AC: customer '7AC' does not exist in the customer master (ALTHCI)
  - VR-09 error ALTHVD30062020153005130 5 (4).csv line 2 7893249911649: item '7893249911649' is not present in both the inventory file (false) and the product master (false)
  - VR-10 warning ALTHVD30062020153005130 5 (4).csv line 2 : SALESPERSON_CODE is empty; mandatory-ness is undefined by the manual (MTRIX-003)
  - VR-04 warning ALTHVD30062020153005130 5 (4).csv line 2 : DOCUMENT_TYPE is empty; mandatory-ness is undefined by the manual (MTRIX-003)
  specification     : 5 files, 56 fields, 14 rules (12 from the field dictionary, 2 from the original manual)
  report            : <報告路徑>
```

> 終端只列出前若干筆 finding；**完整明細以 `--report` 的 JSON 為準**。

| 退出碼 | 意義 |
|---:|---|
| 0 | 執行完成（**不代表批次被接受**，須看 `status`） |
| 非 0（`--strict` 且有 error，實測 -1／Windows 上為 255） | 有驗證錯誤 |
| 非 0 | 執行期錯誤（缺檔、資料夾不存在、輸出無法寫入） |

> **更正**：舊版文件的 `--dir data` 範例不正確——`data/` 並不含五個 MTRIX 檔。
> 正確用法是 `--dir spec/fixtures/mtrix/delivered`（真實交付位元組）。

## 5. `audit`

```
Usage: wellfit.exe audit [OPTIONS]

Options:
  -d, --data-dir <DATA_DIR>  Folder holding the audit inputs [default: data]
      --report <REPORT>      Audit report destination
      --db <DB>              Ledger database path [default: data/wellfit_ledger.sqlite]
```

```powershell
& $wf audit --data-dir data --report "$env:TEMP\audit.json"
```

```
audit wave1-shopee: accepted (2 order(s) accepted, 0 isolated)
audit wave2-reports: accepted (4 record(s) accepted, 0 isolated)
audit wave3-mtrix: partial-accepted (5 file(s), 22 error(s), 24 warning(s))
audit specification: 5 files, 56 fields, 14 rules
audit report        : <稽核報告路徑>
```

> `audit` 走 `data/` 的**合成示範資料**（蝦皮、輔翼 ERP）與
> `spec/fixtures/mtrix/delivered/` 的**真實交付位元組**（MTRIX）。
> 稽核結果**不可**當作對接驗收證據。

## 6. `spec`

```
Usage: wellfit.exe spec [OPTIONS]

Options:
  -f, --format <FORMAT>  Output format [default: markdown] [possible values: markdown, json]
```

```powershell
& $wf spec --format markdown
& $wf spec --format json
```

```
檔案 5 個、欄位 56 個、驗證規則 15 條（欄位字典 13 條、原始規格 2 條）、修復規則 12 條

## ALTHCI - Customer / PoS（16 欄）

| 序 | 交付欄名 | 原文欄名 | 技術欄名 | 型別 | 長度 | 允許值 | 鍵角色 | 來源 | 狀態 | 原文頁 | 議題 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | VENDOR_ID | VENDOR_ID | VENDOR_ID | text | 14 | - | vendor | ManualL1 | confirmed | 19 | - |
| 5 | ADRESS | ADDRESS | ADDRESS | text | 40 | - | none | ManualL1 | deviation | 19 | - |
...
```

此輸出即知識庫 [`../../Knowledge/specs/mtrix-field-dictionary.md`](../../../Knowledge/specs/mtrix-field-dictionary.md) 之內容。

## 7. 共通約定

| 項目 | 說明 |
|---|---|
| 預設帳本 | `data/wellfit_ledger.sqlite`（相對工作目錄） |
| 預設批次識別碼 | `shopee`：`BATCH-CLI-SHOPEE`；`reports`：`BATCH-CLI-REPORT`；`mtrix`：`BATCH-CLI-MTRIX` |
| 重送 | 以**同一 `--batch` 識別碼**重送同一批資料；帳本以批次 ID + SHA-256 判定 |
| 互動性 | 全為非互動命令，適合排程與 CI |
| 退出碼 | 0 = 執行完成；非 0 = 錯誤或（`--strict` 時）驗證失敗 |

| **報告與帳本計數差異** | `audit_report.json` 與 `wellfit status` 的計數可能不同：`audit_report.json` 的 `errors`/`violations` 為**單次審計**的筆數；`wellfit status` 的 `isolated records` 為**帳本累計**的所有歷次審計隔離項目總數。兩者皆正確，語義不同：前者為本次審計結果，後者為帳本歷史總計。 |

