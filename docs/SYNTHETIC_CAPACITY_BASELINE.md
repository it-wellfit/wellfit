# Wellfit 合成容量基線

**版本：** 1.3  
**測試日期：** 2026-11-02  
**狀態：** 本機合成工程量測；不是生產驗收、容量保證或 SLA 承諾  
**執行工具：** [`scripts/benchmark_synthetic_ingestion.ps1`](../scripts/benchmark_synthetic_ingestion.ps1)、[`scripts/benchmark_synthetic_test.ps1`](../scripts/benchmark_synthetic_test.ps1)  
**操作方式：** [`CLI.md`](CLI.md)「本機合成容量量測」

## 1. 目的與邊界

本基線記錄 Wellfit 四條記錄處理流程（Shopee、Reports、MTRIX、BankPay）的合成工程量測與資源觀測。Sync 尚無資料傳輸，不能以 records/sec 計入。此文件不證明企業每日真實工作量、四流程合計吞吐、併發表現、資料分布代表性或正式營運 SLA。

本輪新增 MTRIX、BankPay 工程量測；Sync 尚無資料傳輸：

- **MTRIX：** 五檔 drop 流程以規格衍生的虛構資料完成三次容量量測，並另以相同 250,000 列 profile 復核一次（各檔 50,000 列）；均全數接受、無隔離、0 errors／warnings。這不是百萬筆、實際業務分布或 SLA 驗收。
- **BankPay：** 匯出引擎以合成 250,000 筆資料及注入的 transport mock 完成三次工程量測，並另復核一次；未呼叫任何真實外部銀行端點。這些觀測不是容量驗收。匯出與匯入現均在讀取階段套用 100 MiB 單檔 byte cap，但完整資料仍會在記憶體中保留及轉換；這不是串流或容量保證。
- **Sync：** `run_sync_cycle` 尚無資料傳輸實作；沒有 records/sec 可報告。
- `wellfit_core`／`wellfit_cli` 是共用層，不另計資料流程筆數。

## 2. 重現方式

依專案核准建置入口建置 CLI，再由 repository 根目錄執行：

```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "build -p wellfit_cli" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

powershell -ExecutionPolicy Bypass -File "scripts\benchmark_synthetic_ingestion.ps1" `
  -RecordsPerPipeline 1000000 `
  -WellfitExe "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"
```

基線使用 `debug` build、逐流程各執行一次。工具在 repository 外 `%TEMP%` 建立唯一資料目錄及 SQLite 帳本；Shopee 每列使用獨立合成訂單編號與固定測試 SKU，Reports 每列使用合成品號及可核對庫存恆等式。腳本自行驗證 CLI 回報筆數、輸入實體行數及 Shopee 輸出行數，並記錄 SHA-256。兩個輸入均低於目前 100 MiB 單檔上限。

## 3. 實測環境與結果

下表為借用冪等鍵改動前的一百萬筆歷史量測，保留作工程比較，不代表目前版本的性能結果。現行版復測見 §3.3。

| 指標 | Shopee | Reports |
|---|---:|---:|
| 合成輸入筆數 | 1,000,000 | 1,000,000 |
| 接受筆數 | 1,000,000 | 1,000,000 |
| 隔離筆數 | 0 | 0 |
| 實測處理時間 | 207.475 秒 | 9.024 秒 |
| 實測吞吐 | 4,819.87 筆／秒 | 110,814.75 筆／秒 |
| 程序 CPU 時間 | 163.734 秒 | 8.719 秒 |
| 峰值程序工作集 | 1,236,541,440 bytes | 322,383,872 bytes |
| 工作集取樣次數 | 1,784 | 80 |
| 輸入大小 | 80,000,084 bytes | 76,000,074 bytes |
| SQLite 帳本大小 | 173,809,664 bytes | 57,344 bytes |
| 輸出列數（含標題） | 1,000,001 | 不適用 |

測試機為 AMD Ryzen 7 4800HS、16 個邏輯處理器、約 15.42 GiB 實體記憶體。峰值工作集以程序啟動後及每 100 ms 採樣；採樣法可能漏掉短暫尖峰。上表數據是各自獨立程序的結果，不是同時併發執行的總系統資源數。

### 3.1 改動前輸入／輸出指紋

| 管線 | 輸入 SHA-256 | 輸出 SHA-256 | SQLite SHA-256 |
|---|---|---|---|
| Shopee | `4D8B32ADA834170C0536BE0F409DBBCE1231731E2FA798F652D9A8FAF63511A1` | `00DE1EA4861481EA53820E1D6FD39156832617EC73FDC80E91F598F5CCBA0E5C` | `B7FC44B714C588521514EE8E6E0F46242765F24093322B3E3571AD6B3E8D2E5A` |
| Reports | `38C920EFEDE2BAC4C9D0F5457C0E04C84EE810849BBCED28942E6640E3C6B666` | 不產生檔案輸出 | `9E2C6C1088A62116B50111F0ECDA50E7A983251764A6AB9F7DB3CC8E2FA50D55` |

### 3.2 Shopee 冪等鎖批次化後的 500,000 筆復測

為減少每個訂單各自提交 SQLite 寫入所造成的交易開銷，現行 Shopee 流程將訂單冪等鎖放在單一 transaction 中批次寫入。相同測試機、合成資料腳本及 debug CLI 下，每條流程一次執行 500,000 筆；Shopee 變更後重跑兩次：

| 管線／版本 | 筆數 | 耗時 | 吞吐 | CPU 時間 | 採樣峰值工作集 |
|---|---:|---:|---:|---:|---:|
| Shopee，批次化前單次 | 500,000 | 81.305 秒 | 6,149.67 筆／秒 | 63.406 秒 | 650,735,616 bytes |
| Shopee，批次化後 run 1 | 500,000 | 20.576 秒 | 24,299.62 筆／秒 | 20.281 秒 | 638,861,312 bytes |
| Shopee，批次化後 run 2 | 500,000 | 19.871 秒 | 25,162.13 筆／秒 | 19.578 秒 | 621,797,376 bytes |
| Reports，參考 run 1 | 500,000 | 2.714 秒 | 184,225.11 筆／秒 | 2.672 秒 | 187,273,216 bytes |
| Reports，參考 run 2 | 500,000 | 4.163 秒 | 120,110.02 筆／秒 | 4.141 秒 | 178,393,088 bytes |
| Reports，參考 run 3 | 500,000 | 4.250 秒 | 117,633.64 筆／秒 | 4.156 秒 | 176,644,096 bytes |

前後比較是工程觀測，不是統計保證；Shopee 批次化後兩次約為先前單次吞吐的四倍，但 500,000 筆時仍採樣到約 593–609 MiB 工作集，未解決整檔與訂單聚合所需記憶體。Reports 多次結果也有明顯波動。後續百萬筆復測見 §3.3，該單次結果僅為工程量測，不構成容量驗收。

### 3.3 借用冪等鍵後的百萬筆復測

為避免在 Shopee 組合所有訂單後再複製一份百萬筆 `(String, amount)` 冪等請求，`wellfit_core::ledger::check_and_lock_order_refs` 現接受借用鍵的 iterator，仍在單一 SQLite transaction 中完成鎖定。以相同的百萬筆合成輸入和同一主機／debug CLI 前後各執行一次：

| 管線 | 接受筆數 | 耗時 | 吞吐 | CPU 時間 | 採樣峰值工作集 | SQLite 帳本大小 |
|---|---:|---:|---:|---:|---:|---:|
| Shopee（借用鍵前） | 1,000,000 | 207.475 秒 | 4,819.87 筆／秒 | 163.734 秒 | 1,236,541,440 bytes | 173,809,664 bytes |
| Shopee（借用鍵後） | 1,000,000 | 43.386 秒 | 23,048.92 筆／秒 | 42.922 秒 | 1,221,337,088 bytes | 173,854,720 bytes |
| Reports（借用鍵前） | 1,000,000 | 9.024 秒 | 110,814.75 筆／秒 | 8.719 秒 | 322,383,872 bytes | 57,344 bytes |
| Reports（借用鍵後） | 1,000,000 | 8.326 秒 | 120,100.11 筆／秒 | 8.297 秒 | 321,716,224 bytes | 57,344 bytes |
| Shopee（本輪重建復測） | 1,000,000 | 60.888 秒 | 16,423.48 筆／秒 | 59.328 秒 | 1,238,110,208 bytes | 173,842,432 bytes |
| Reports（本輪重建復測） | 1,000,000 | 12.188 秒 | 82,045.98 筆／秒 | 11.938 秒 | 320,061,440 bytes | 57,344 bytes |

以上各次皆為單次工程觀測，不是受控性能研究或未來性能保證。本輪由目前工作樹重新建置 debug CLI 後，各流程再跑一百萬筆一次；輸入 SHA-256 與前次相同，但 Shopee／Reports 耗時分別為 60.888／12.188 秒，與先前單次 43.386／8.326 秒有差異。此變異表示不可把任一單次吞吐視為穩定容量承諾，需先控制主機負載、快取及建置版本，再進行多輪比較。改動後另一次 200,000 筆 spot check：Shopee 6.451 秒（31,002.74 筆／秒；226,037,760 bytes 工作集），Reports 1.669 秒（119,863.43 筆／秒；65,728,512 bytes 工作集）。此較小測試是在百萬筆執行後進行，可能受快取影響，不可與百萬筆結果拼成線性擴展曲線。

| 改動後指紋 | Shopee | Reports |
|---|---|---|
| 輸入 SHA-256 | `4D8B32ADA834170C0536BE0F409DBBCE1231731E2FA798F652D9A8FAF63511A1` | `38C920EFEDE2BAC4C9D0F5457C0E04C84EE810849BBCED28942E6640E3C6B666` |
| SQLite SHA-256 | `B8A3F4A1D51231C6F2D6346D663120E704C29844184BF56B5446873946D42F7D` | `71CEC4B0253F1A69EE727CB6A84ED0573D55E07BDB65072CE9339C54388F77D3` |
| 輸出 SHA-256 | `F7349F1067987F98FED7DEB9FD79E30D5F2252669AD8405E854BC15B9FF07045` | 不產生檔案輸出 |

輸出 SHA-256 是該次執行的指紋，不是固定 golden hash；目前訂單以 HashMap 彙總，輸出列順序不保證跨次相同。

本輪重建復測的輸入 SHA-256 與上表相同；SQLite SHA-256 分別為 Shopee `6281833553D4A70313E67A70793CB9FE0973A0E76BECA8577B8C6C19578BE7D1`、Reports `CEA90D52679809232F75F3065CAE662BF5D1C7A796BC29931CC0A6141755F80D`；Shopee 輸出 SHA-256 為 `23D613E2BEC5CF5DAC757D7FDA21B2CB1E25F7854078B36E865FE7C4269A4C16`。帳本雜湊會隨執行產生的批次識別值而變化；這些值是該次執行指紋，不是 golden hash。

### 3.4 現行版 250,000 筆合成觀測

以本基線所列工具、目前 debug CLI 及同一主機，每條已支援流程各執行一次 250,000 筆；輸入 SHA-256 可重現。每條流程均接受 250,000 筆、隔離 0 筆。這只是現行版合成樣本觀測；沒有冷／暖快取控制、多次試驗或真實業務分布，不能外推至更大規模或生產 SLA。

| 管線 | 耗時 | 吞吐 | CPU 時間 | 採樣峰值工作集 | 輸入大小 | SQLite 大小 |
|---|---:|---:|---:|---:|---:|---:|
| Shopee | 11.685 秒 | 21,395.26 筆／秒 | 11.469 秒 | 303,349,760 bytes | 20,000,084 bytes | 42,352,640 bytes |
| Reports | 2.510 秒 | 99,620.21 筆／秒 | 2.453 秒 | 92,524,544 bytes | 19,000,074 bytes | 57,344 bytes |

| 管線 | 輸入 SHA-256 | SQLite SHA-256 | 輸出 SHA-256 |
|---|---|---|---|
| Shopee | `0004941B5A9FF1E80F2E0A274ED5A2E27C288ED3CA430C8806F8BA6C16705C17` | `22F810700A6E4966A1F6B08834D6895DE21D0E571C5089B0B5256C8BF9E86813` | `F9CE5BA583B2CB317E5AD41FF6DC9E345CC448C4C426E0EA836A81205ED9FD36` |
| Reports | `7E4C4D72DE4A347DA9D3C90D32DDA27179FD93EA7E4D51AE316F5612AB5D0A0A` | `E3B8ED969AFB589F1D34E83E67694DDE4D5860D84AE2A1E953DB5CEDBA464219` | 不產生檔案輸出 |

### 3.5 BankPay 250,000 筆合成 mock 基準（四次觀測）

新增的忽略式整合基準直接呼叫 `BankpayEngine::export_vouchers`，使用唯一合成 voucher、記憶體 SQLite ledger 及只記錄 payload 的 transport closure。`export_config` 為 `None`，不建立 HTTP client；測試斷言所有列接受、mock transport 被呼叫且 `api_dispatch_attempted=false`。輸入及媒體輸出均存放於 `%TEMP%`，測試結束後清理。

| 執行 | 接受／隔離 | 引擎耗時 | 引擎吞吐 | 程序 CPU | 程序峰值工作集 | 輸入大小 | 媒體輸出大小 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | 250,000 / 0 | 7.683 秒 | 32,538.07 筆／秒 | 8.500 秒 | 267,431,936 bytes | 19,000,062 bytes | 17,750,174 bytes |
| 2 | 250,000 / 0 | 7.732 秒 | 32,329.45 筆／秒 | 8.469 秒 | 285,216,768 bytes | 19,000,062 bytes | 17,750,174 bytes |
| 3 | 250,000 / 0 | 7.660 秒 | 32,633.57 筆／秒 | 8.406 秒 | 273,879,040 bytes | 19,000,062 bytes | 17,750,174 bytes |
| 復核 | 250,000 / 0 | 7.479 秒 | 33,424.57 筆／秒 | 8.219 秒 | 280,969,216 bytes | 19,000,062 bytes | 17,750,174 bytes |

四次使用相同輸入，SHA-256 為 `82B9CDE0F4A4DDEC2BEB4CBD7982940889D2BB5539A932209091047BF84B4B76`。測試 batch id 含程序識別值，媒體 trailer 因而令輸出雜湊每次不同；不能將媒體雜湊當穩定 golden hash。程序 CPU 與峰值工作集涵蓋測試啟動、合成輸入產生及匯出；引擎耗時只計匯出。測量腳本讀取 Windows process peak working set 並以 50 ms 間隔抽樣。這些 debug test 沒有併發、超限、失敗恢復或銀行端到端測試；結果不構成容量驗收、付款格式核准或正式銀行介接證據。

重跑（repository 根目錄；透過核准 build.ps1 建置並量測）：  

```powershell
powershell -ExecutionPolicy Bypass -File 'scripts\benchmark_synthetic_test.ps1' `
  -Pipeline BankPay -Records 250000
```

使用 `-Records` 可在 1 至 1,000,000 列間調整負載；100 MiB 單檔讀取上限仍會拒絕超大輸入。量測腳本輸出 JSON，包含 engine 結果、程序 CPU 與 peak working set。

### 3.6 MTRIX 已交付 fixture smoke test（非容量測試）

使用 `spec/fixtures/mtrix/delivered/` 的交付 fixture 執行一次 CLI smoke test，程序牆鐘時間 0.174 秒。5 個檔案共 25 列，25 列接受、0 列隔離；狀態為 `partial-accepted`，另有 22 errors、29 warnings、37 repairs。這些 errors／warnings 是規則 findings，不等於 22 個失敗列或 29 個不同問題。本文件保存本次核對的摘要；原始 JSON 報告寫在 `%TEMP%` 暫存目錄，非本文件交付物，執行上方命令可重建。此 smoke test 不是大檔量測或介接驗收。

重跑範例（由 repository 根目錄執行；使用新的空白輸出目錄與 ledger）：

```powershell
$wf = "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"
$out = Join-Path $env:TEMP ("wellfit-mtrix-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $out -Force | Out-Null
& $wf mtrix --dir "spec\fixtures\mtrix\delivered" `
  --output-dir (Join-Path $out "repaired") `
  --report (Join-Path $out "report.json") `
  --batch ("MTRIX-SMOKE-" + [guid]::NewGuid().ToString("N")) `
  --db (Join-Path $out "ledger.sqlite")
```

### 3.7 MTRIX 規格衍生 250,000 列合成基準（四次觀測）

新增的忽略式整合基準直接從 `MTRIX_SPEC` 的五檔欄位定義生成虛構記錄，並用固定 synthetic key mapping 建立 Customer、SalesForce、Inventory、Product、Sales 間的引用關係。未讀取、複製或轉換 delivered fixture。引擎驗證五檔所有標頭與列；測試要求每檔 50,000 列接受、0 列隔離，且 error findings 為 0。

| 執行 | 每檔列數 | 總列數接受／隔離 | Errors／warnings | 引擎耗時 | 引擎吞吐 | 程序 CPU | 程序峰值工作集 | 輸入大小 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 50,000 | 250,000 / 0 | 0 / 0 | 12.788 秒 | 19,549.14 列／秒 | 13.891 秒 | 255,373,312 bytes | 28,100,647 bytes |
| 2 | 50,000 | 250,000 / 0 | 0 / 0 | 12.593 秒 | 19,851.02 列／秒 | 13.859 秒 | 254,595,072 bytes | 28,100,647 bytes |
| 3 | 50,000 | 250,000 / 0 | 0 / 0 | 12.806 秒 | 19,521.84 列／秒 | 13.938 秒 | 255,533,056 bytes | 28,100,647 bytes |
| 復核 | 50,000 | 250,000 / 0 | 0 / 0 | 13.160 秒 | 18,996.79 列／秒 | 14.234 秒 | 255,086,592 bytes | 28,100,647 bytes |

| 檔案 | 輸入 SHA-256 |
|---|---|
| ALTHCI | `94E1CABAA84C8790E2D8E07D139F8198DF88E2C8BA13F03A38B0350EED870DB3` |
| ALTHES | `611FB5F299DFE97479C7B0BB9E15FD66AA53572F6570A4290DFD7F993E697416` |
| ALTHFV | `5E5D8352D189C7AA0295AEE0B240ABBCC87B1801EE8CC417370FC4D41020A16C` |
| ALTHPD | `2A5D9D950AC419A22906E9CD25D145B5C394012042880387D5DC50C3F95AB406` |
| ALTHVD | `39A7DC4B762D683D8EBE3E85ECD19A365FEA6E0B5890633E016BF5D805EFAF5A` |

四次為同一台機器上的 debug test，輸入檔雜湊相同。程序 CPU 與峰值工作集涵蓋測試啟動、合成輸入產生及處理；引擎耗時只計 drop 驗證。工作集以 Windows process peak working set 及 50 ms 間隔抽樣量測。此生成 profile 只證明目前宣告契約及指定跨檔 key 關係下的工程路徑；未涵蓋 ledger／輸出磁碟成長、併發、故障恢復或不同欄位分布，不代表 MTRIX 企業資料分布或生產容量。

重跑（repository 根目錄；透過核准 build.ps1 執行）：

```powershell
powershell -ExecutionPolicy Bypass -File 'scripts\benchmark_synthetic_test.ps1' `
  -Pipeline MTRIX -Records 50000
```

`-Records` 可在 1 至 250,000 列間調整每個輸入檔的合成量。量測腳本輸出 JSON，包含 engine 結果、程序 CPU 與 peak working set。

## 4. 解讀與容量限制

1. 現行版 Shopee 與 Reports 各自完成一百萬筆合成負載，另有 250,000 筆觀測；這不是四流程合計一百萬筆、混合併發或端到端驗收。
2. 現行版 Shopee 百萬筆峰值工作集約 1.14 GiB。這不證明記憶體有界；Shopee 仍須保存訂單與明細彙總，峰值可能隨不同訂單／明細分布增加。
3. Reports 百萬筆峰值工作集約 307 MiB；目前 API／CLI 仍收集有效與隔離結果，不能宣稱常數記憶體串流。
4. MTRIX 與 BankPay 已量測程序 CPU 及峰值工作集，但尚無核准資源上限；各流程的少量執行也未覆蓋冷／暖快取差異、併發、排隊／背壓、外部下游節流、磁碟滿、資料庫鎖定、中斷重啟、隔離比例或實際最大欄位長度。
5. 合成資料只驗證各基準所生成的固定欄位分布；MTRIX 的跨檔 key 參照已在指定虛構 profile 下驗證，但不能替代企業實際資料剖析、財務／數量業務規則核准或生產環境負載測試。
6. CLI 仍受 100 MiB 單檔限制並將輸入讀入記憶體；MTRIX／BankPay 已測得的工作集只代表指定合成 profile，不是記憶體上限，不得僅因本次輸入成功而提高單檔上限。

## 5. 後續驗收門檻

若為工程比較而選擇等分合成 profile，可將 Shopee、Reports、MTRIX、BankPay 各 250,000 筆作為一百萬筆測試案例；這只是測試設計選項，不是已核准的目標流量、業務需求或 SLA。現行程式有 Shopee／Reports 合成容量數據、MTRIX 四次 250,000 列量測及 BankPay 四次 mock 量測；這些結果均非生產容量驗收。Sync 尚無資料傳輸，排除於 records/sec 計數。

要宣稱達成此四流程每日百萬筆測試目標，仍需：

- 由流程負責人確認容量需求、四流程真實筆數分布、有效／隔離比例、欄位長度、檔案大小與業務計數口徑；無核准需求時僅能報告標明合成分布的工程測量。
- MTRIX 已有四次規格衍生合成 drop 與跨檔關聯量測；仍需更大及多種資料分布、隔離比例、輸出與 SQLite 空間測試。
- BankPay 僅以測試替身／合成固定回應量測；不得呼叫真實銀行端點或使用正式憑證。
- Sync 在 peer 協定、資料單位、冪等、失敗與恢復語義核准並完成傳輸實作前，維持 `Not implemented / Not measured`。
- 各適用流程加入重複執行、並行／背壓、故障注入與恢復測試；以多次實測和逐流程結果核定容量，而不以平均掩蓋慢路徑。

本報告僅為可重跑的本機工程基線，不是企業負載核准或生產容量承諾。
