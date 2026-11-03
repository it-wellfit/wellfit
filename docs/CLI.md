# Wellfit CLI 命令參考

> 文件編號：WFLT-DOC-CLI-001｜狀態：Draft｜版本：v1.3｜日期：2026-11-02
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 變更摘要：v1.3。補記一百萬筆復測的效能變異，並更新 MTRIX／BankPay 復核量測次數。
> v1.1 記錄 Shopee／Reports 標準 CSV 引號解析、欄內換行、重複欄名與格式錯誤行為，並揭露目前的記憶體／檔案大小限制。
> v1.0 依 `wellfit --help` 與各子命令 `--help` 之**實際輸出**重寫；新增 `spec` 子命令；
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
  access-control  Evaluate access-control policies
  jit      Manage just-in-time permission requests
  bankpay  E.Sun Bank payment integration
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

The `bankpay`, `access-control`, and `jit` command groups are also available; use
`wellfit.exe <COMMAND> --help` for their current arguments. `wellfit_sync.exe` is
a separate binary. Its current sync cycle only discovers peers; when a peer is
available it exits with an explicit "data transfer is not implemented" error
and does not claim that records were synchronized. Generated CSV, JSON,
BankPay media, Sync configuration, and repaired MTRIX files are staged beside
their destinations and published by rename after the complete file has been
flushed; each destination is atomic individually, not as a multi-file
transaction.

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
- 使用 RFC 4180 相容解析：以雙引號包住含逗號／換行的欄位，並以 `""` 表示欄位內的雙引號；重複欄名或不完整／不合法引號會在帳本登記前拒收整份輸入。

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
- 使用 RFC 4180 相容解析：含逗號、雙引號或換行的品名需正確引用；重複欄名或不合法引號會在帳本登記前拒收整份輸入。

> **容量限制（目前實作）**：輸入檔有 100 MiB 單檔上限，且以最多上限加一個 byte 的有界讀取辨識超限檔案；BankPay 匯出與匯入也在讀取階段套用此上限。這不等於常數記憶體：Shopee 需保留訂單彙總，Reports API／CLI 仍收集有效與隔離結果，BankPay 仍保留輸入、憑證及媒體檔資料。CSV 逐記錄解析不代表整體流程已具常數記憶體。Shopee 與 Reports 的百萬筆合成量測是各一次；峰值記憶體、重複執行、併發、背壓與故障恢復尚未完成驗收；不得將量測描述為生產容量保證或百萬筆 SLA。詳見 [`SYNTHETIC_CAPACITY_BASELINE.md`](SYNTHETIC_CAPACITY_BASELINE.md)。

### 本機合成容量量測

先依 `AGENTS.md` 的核准入口建置 CLI，再執行：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/benchmark_synthetic_ingestion.ps1 `
  -RecordsPerPipeline 200000
```

此腳本只生成合成 Shopee／Reports 資料，預設存放於 `%TEMP%` 唯一目錄，輸出筆數、耗時、CPU、採樣工作集、輸入雜湊及 SQLite 大小／雜湊。`-OutputDirectory` 可指定 repository 外的空白目錄，`-WellfitExe` 可指定其他建置輸出。MTRIX／BankPay 的 contract-derived mock benchmark 另由 [`benchmark_synthetic_test.ps1`](../scripts/benchmark_synthetic_test.ps1) 執行；它採核准 build.ps1 建置測試，再收集 engine 結果、程序 CPU 與 peak working set。Sync 無傳輸實作，兩個腳本均不測量 Sync。Shopee／Reports 的 100 ms 工作集採樣可能漏掉短暫峰值；MTRIX／BankPay runner 同時讀取 Windows process peak working set 並以 50 ms 間隔抽樣，記錄值包含測試啟動／資料產生，不等於純引擎記憶體需求。所有合成結果均不等於生產 SLA。

```powershell
powershell -ExecutionPolicy Bypass -File scripts/benchmark_synthetic_test.ps1 `
  -Pipeline MTRIX -Records 50000
powershell -ExecutionPolicy Bypass -File scripts/benchmark_synthetic_test.ps1 `
  -Pipeline BankPay -Records 250000
```

#### 已量測的合成基線（2026-11-02）

在 Windows、16 個邏輯處理器、約 15.4 GiB 可見實體記憶體的本機，依序執行每條管線一次。兩條管線各自接受全部有效列、隔離 0 列；結果可由上述腳本重跑。此為合成輸入的單次工程觀測，不是生產流量預測、SLA 或信賴區間。最新百萬筆資料與雜湊見容量基線文件；它明確區分此次量測及更早的工程觀測。

| 管線 | 輸入筆數 | 輸入大小 | 耗時 | 吞吐 | CPU 時間 | 採樣峰值工作集 |
|---|---:|---:|---:|---:|---:|---:|
| Shopee | 250,000 | 20,000,084 B | 37.858 s | 6,603.58 records/s | 33.141 s | 316 MiB |
| Shopee, before batched ledger locks | 500,000 | 40,000,084 B | 81.305 s | 6,149.67 records/s | 63.406 s | 620 MiB |
| Shopee, after batched ledger locks (run 1) | 500,000 | 40,000,084 B | 20.576 s | 24,299.62 records/s | 20.281 s | 609 MiB |
| Shopee, after batched ledger locks (run 2) | 500,000 | 40,000,084 B | 19.871 s | 25,162.13 records/s | 19.578 s | 593 MiB |
| Reports | 250,000 | 19,000,074 B | 1.528 s | 163,648.15 records/s | 1.453 s | 89 MiB |
| Reports, run 1 | 500,000 | 38,000,074 B | 2.714 s | 184,225.11 records/s | 2.672 s | 179 MiB |
| Reports, run 2 | 500,000 | 38,000,074 B | 4.163 s | 120,110.02 records/s | 4.141 s | 170 MiB |
| Reports, run 3 | 500,000 | 38,000,074 B | 4.250 s | 117,633.64 records/s | 4.156 s | 169 MiB |

「after」執行改為在單一 SQLite transaction 中批次取得 Shopee 訂單冪等鎖，兩次 500,000 列觀測約為原單次觀測的 4 倍吞吐；這只是同一主機的前後工程比較，不能視為普遍效能保證。借用鍵避免另建大型冪等鍵複本後，Shopee／Reports 百萬筆初次復測各為 43.386／8.326 秒；由目前工作樹重建 CLI 後的另一次單次復測為 60.888／12.188 秒。輸入雜湊相同但耗時不同，顯示目前單次測量不能作穩定容量承諾；完整指標與雜湊見 [`SYNTHETIC_CAPACITY_BASELINE.md`](SYNTHETIC_CAPACITY_BASELINE.md)。工作集於啟動時及每 100 ms 採樣，可能低估短暫峰值；Shopee 仍將整批輸入、訂單聚合及輸出保留於記憶體，尚未解決有界記憶體或故障恢復問題。CLI 單檔輸入上限仍為 100 MiB。上述 Shopee／Reports 數據不包含 MTRIX、BankPay 或 Sync 的混合執行，因此不代表四流程混合量測或每日百萬筆驗收。

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
