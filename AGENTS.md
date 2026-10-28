# Wellfit — AGENTS.md（代理人操作規範）

> 文件編號：WFLT-AGNT-001｜狀態：Draft｜版本：v1.0｜日期：2026-10-20
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 變更摘要：v1.0。建置入口改為知識庫的 `build.ps1`（因工作站應用程式控制原則阻擋桌面路徑下的產物）；
> 補上「無序位退讓」「不呈現合成資料為真實」兩條不變量；補上 Crate 地圖更新與「自知識庫接手」程序。
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

本檔供代理人（Agent）與開發者操作本專案之快速參考。**本檔只描述「如何做」，
「什麼是真的」一律以知識庫為準**（見 §6）。

---

## 1. 專案概況

- 大前實業（Dachen Industrial）輔翼 ERP 與三大商業產品自動化引擎，**5-crate Rust 工作區**。
- 單一原生二進位發行檔：`wellfit.exe`（`wellfit_cli`），無前端、無 JavaScript、無外部 DLL 依賴。
- 契約事實的唯一來源是跨專案知識庫 `2026/Knowledge/`，**不是**網路、**不是**既有文件的敘述。

## 2. Crate 地圖

| Crate | 路徑 | 職責 | 關鍵型別／模組 |
|---|---|---|---|
| `wellfit_core` | `crates/wellfit_core` | 共享核心：MDM 實體編碼清洗、SQLite 冪等帳本、桌面互斥鎖、Release Gate、聯邦契約、定點十進位、文字切分、欄位契約 | `MdmHub`／`MdmCatalog`、`IdempotencyLedger`、`ReleaseGate`、`wellfit_core::federation`（`src/federation.rs`）、`ScaledDecimal`（`src/scaled.rs`）、`split_lines`（`src/text.rs`）、`ColumnContract`／`ColumnMap`（`src/contract.rs`） |
| `wellfit_shopee` | `crates/wellfit_shopee` | 蝦皮進貨訂單 → 輔翼 ERP 匯入批次 | `shopee_order_contract()`（8 欄）、`ShopeeInboundEngine`、`ShopeeOrder::verify_line_sum`（EX-06） |
| `wellfit_reports` | `crates/wellfit_reports` | 輔翼 ERP 庫存報表解析與發布閘門 | `fuyi_inventory_report_contract()`（8 欄）、`ReportAutomator` |
| `wellfit_mtrix` | `crates/wellfit_mtrix` | MTRIX 五檔契約、驗證與修復 | `MTRIX_SPEC`（56 欄，`src/spec.rs`）、`VrRule`（15，`src/validators.rs`）、`REPAIR_RULES`（12，`src/repairs.rs`）、`MtrixEngine`（`src/mtrix_engine.rs`） |
| `wellfit_cli` | `crates/wellfit_cli` | `wellfit.exe` 進入點 | 六個子命令：`status`／`shopee`／`reports`／`mtrix`／`audit`／`spec` |

## 3. 建置入口（**必須**使用此入口）

本工作站的**應用程式控制原則會阻擋建置於桌面路徑下**的執行檔。
直接執行 `cargo build` 產出的 `.exe` 可能無法執行。因此所有建置／測試／執行
都必須透過知識庫的建置腳本：

```powershell
cd "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit"

powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "<cargo 子命令>" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

| `-Command` 對應 | 用途 |
|---|---|
| `build -p wellfit_cli` | 建置 CLI |
| `build --workspace` | 建置全部 |
| `check --workspace --all-targets` | 型別檢查 |
| `clippy --workspace --all-targets -- -D warnings` | 零警告檢查 |
| `test --workspace` | 全部測試 |
| `test --workspace --no-fail-fast` | 全部測試（不因單一失敗中止） |
| `run -p wellfit_cli -- <子命令> <參數>` | 執行 CLI |

`build.ps1` 會做三件事：

1. **設定 MSVC 工具鏈**（`stable-x86_64-pc-windows-msvc`）。
2. **載入 Visual Studio 建置環境**（`vcvars64.bat`），使 `link.exe` 可用。
3. **將 target 目錄導向儲存庫之外**（預設 `%LOCALAPPDATA%\Temp\wellfit-target`），繞開桌面路徑的應用程式控制原則。

> **協作注意**：`build.ps1` 會把 cargo 指令寫到暫存 `.bat`；檔名帶有行程 ID，
> 因此多個代理人同時呼叫**不會互相覆寫**對方的指令。
> 若仍遇到指令被換掉的情形，請各自使用**不同的 `-TargetDir`**。

## 4. 不變量（Invariants）

以下各條為硬性要求，違反即視為缺陷：

| # | 不變量 | 落地機制 |
|---:|---|---|
| 1 | **零 `unsafe`** | 每個 `.rs` 第一行 `#![forbid(unsafe_code)]` |
| 2 | **Clippy 零警告** | `clippy --workspace --all-targets -- -D warnings` |
| 3 | **金額／數量不用浮點** | `wellfit_core::scaled`（`ScaledDecimal`，10^4 縮放）、蝦皮以 `i64` 分運算；格式化用整數除法 + 補零餘數 |
| 4 | **欄位以名稱解析，禁止序位退讓** | `wellfit_core::contract` 的 `ColumnContract`／`ColumnMap`；必填欄缺漏 → `Err`，**不得**退回硬編碼序位 |
| 5 | **合成資料不得呈現為真實資料** | fixture 必須附出處與 SHA-256（`spec/fixtures/mtrix/PROVENANCE.md`）；`data/` 內示範檔不得以「交付樣本」名義引用 |
| 6 | **不可驗證來源標記，不推測** | `spec.rs` 的 `VerifyStatus`（`confirmed`／`observed-only`／`deviation`／`pending`／`unverified`）；無來源之數字不得寫入文件 |
| 7 | **不靜默改寫資料** | 修復必須有編號規則與授權來源（ANOM-01..12）；跨檔問題只「列出鍵、保留列」 |
| 8 | **不可追溯之數字不得出現** | 每一個數量都對應到測試 ID、命令或來源快照（見知識庫 `traceability/claims-vs-evidence.md`） |

### 4.1 Windows 執行規範

- 多指令串接用分號 `;`，**嚴禁 `&&`**（Windows PowerShell 5.1 不支援）。
- 建置前先終止殘留的 `wellfit.exe`（OS 會鎖住 `.exe`）。
- `.ps1` 腳本在 Windows PowerShell 5.1 下以系統 ANSI 碼頁解析：
  **無 BOM 的 `.ps1` 不得含非 ASCII 字面量**；非 ASCII 內容一律置於 UTF-8 資料／模板檔，
  由腳本以 `Get-Content -Encoding UTF8` 讀取（見知識庫 `tools/` 內既有腳本的頭註）。

## 5. 執行與 DEMO

完整步驟見 [`docs/DEMO.md`](docs/DEMO.md)；命令參考見 [`docs/CLI.md`](docs/CLI.md)。

```powershell
$wf = "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"

& $wf spec --format markdown
& $wf mtrix --dir spec/fixtures/mtrix/delivered --report "$env:TEMP\r.json" --db "$env:TEMP\l.sqlite"
& $wf shopee --input data/shopee_orders.csv --output "$env:TEMP\erp.csv"
& $wf reports --input data/fuyi_inventory_report.csv --recipient finance@dachen.com.tw
& $wf status --db "$env:TEMP\l.sqlite"
& $wf audit --data-dir data
```

> `data/shopee_orders.csv` 與 `data/fuyi_inventory_report.csv` 是**合成示範資料**。
> MTRIX 的 `spec/fixtures/mtrix/delivered/` 才是**真實交付位元組**。

## 6. 自知識庫接手（Resume procedure）

**新 session 或新代理人的第一件事不是讀程式碼，是讀知識庫。** 順序如下：

| 步驟 | 動作 | 檔案 |
|---:|---|---|
| 1 | 讀治理規則、目錄地圖、命名規則、驗證標記 | [`../Knowledge/README.md`](../../Knowledge/README.md) |
| 2 | 讀**工作延續模型**（本次工作的目標與邊界） | [`../Knowledge/adr/ADR-0008-work-continuation-model.md`](../../Knowledge/adr/ADR-0008-work-continuation-model.md) |
| 3 | 讀**研究規約**（什麼可以查網路、什麼不可以） | [`../Knowledge/sources/research-protocol.md`](../../Knowledge/sources/research-protocol.md) |
| 4 | 查**來源登錄**：哪九份文件、哪些抽不到、雜湊為何 | [`../Knowledge/sources/source-register.md`](../../Knowledge/sources/source-register.md) |
| 5 | 查**議題登錄**：87 項已登議題與狀態 | [`../Knowledge/issues/mtx-issue-register.md`](../../Knowledge/issues/mtx-issue-register.md) |
| 6 | 查**裁決紀錄**：已定案的決策 | [`../Knowledge/adr/`](../../Knowledge/adr/) |
| 7 | 查**追溯表**：每個能力宣稱對應到哪個測試／命令／快照 | [`../Knowledge/traceability/claims-vs-evidence.md`](../../Knowledge/traceability/claims-vs-evidence.md) |
| 8 | 執行 `build.ps1 -Command "test --workspace --no-fail-fast"`，確認當下基線 | 本文件 §3 |
| 9 | 更新受影響文件並**重跑**測試 | — |

### 6.1 改動前必讀的三份規則

| 規則 | 會影響你的情境 |
|---|---|
| [`../Knowledge/adr/ADR-0001-source-hierarchy-and-conflict-arbitration.md`](../../Knowledge/adr/ADR-0001-source-hierarchy-and-conflict-arbitration.md) | 你要新增或修改任何欄位規格 |
| [`../Knowledge/adr/ADR-0003-fixture-provenance-strategy.md`](../../Knowledge/adr/ADR-0003-fixture-provenance-strategy.md) | 你要新增任何測試資料 |
| [`../Knowledge/adr/ADR-0005-mandatory-columns-fail-hard.md`](../../Knowledge/adr/ADR-0005-mandatory-columns-fail-hard.md) | 你要修改任何 CSV 解析程式 |

### 6.2 產生與更新衍生文件

| 文件 | 產生方式 |
|---|---|
| `../Knowledge/reconciliation/field-diff.md` | `powershell -ExecutionPolicy Bypass -File ../Knowledge/tools/reconciliation/build-field-diff.ps1` |
| `../Knowledge/specs/mtrix-field-dictionary.md` | `powershell -ExecutionPolicy Bypass -File ../Knowledge/tools/spec-extract/generate-field-dictionary.ps1` |

兩支腳本都會**在資料不一致時直接失敗**，不會產生靜默錯誤的對照表。

## 7. 目錄約定

| 路徑 | 內容 | 規則 |
|---|---|---|
| `crates/` | Rust 原始碼 | 第一行 `#![forbid(unsafe_code)]` |
| `spec/fixtures/mtrix/delivered/` | **真實**交付位元組 | 與來源 SHA-256 相同；不得修改 |
| `spec/fixtures/mtrix/derived/` | **衍生**負向案例 | 由 `generate-derived.ps1` 決定性產生 |
| `spec/fixtures/mtrix/PROVENANCE.md` | fixture 溯源 | 新增 fixture 必須同步更新 |
| `data/` | 示範輸入與執行產出 | 示範輸入為**合成資料**；產出可被覆寫 |
| `docs/` | 專案文件 | 對外文件與知識庫保持一致 |
| `target/`、`*.sqlite`、`*.lock`、`*.exe` | 建置產物 | 已在 `.gitignore` 忽略，不得提交 |

## 8. 提交與變更紀錄

- 變更功能時同步更新 [`CHANGELOG.md`](CHANGELOG.md)。
- 變更任何契約（欄位、規則、修復）時，**先**更新知識庫（`spec.rs` → 重跑產生腳本 → 更新對應文件），**再**改程式碼。
- 不提交機敏資料：對接樣本含客戶代碼、產品碼與價格，僅限內部版控。
