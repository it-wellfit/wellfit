# Wellfit — 大前實業輔翼 ERP 與三大產品自動化引擎

> 文件編號：WFLT-README-001｜狀態：Draft｜版本：v1.0｜日期：2026-10-20
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 變更摘要：v1.0 再對齊後重寫。撤回「58 欄全景清洗驗證」「27 項格式異常」「VR-01~08」三項無來源宣稱；
> 修正四個指向他機的 `file:///c:/Users/User/OneDrive/...` 連結、`contracts/federation/` 引用與
> `data/fuyi_inventory.csv` 範例；新增「規格來源」與「已知缺口」兩節。
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

---

## 1. 這是什麼

Wellfit 是一個 **7-crate Rust 工作區**，把大前實業（Dachen Industrial）輔翼 ERP 與外部產品之間的資料往來
變成可驗證、可追溯、可重送的管線：

| 產品 | 資料流 | 核心不變量 |
|---|---|---|
| **Product 1 — 蝦皮** | 蝦皮訂單匯出 → 輔翼 ERP 匯入批次 | EX-06 金額守恆（整數分，零浮點） |
| **Product 2 — 輔翼 ERP 報表** | 庫存報表 → 結構化解析 → 發布閘門 | 庫存恆等式 `期末 ≡ 期初 + 進貨 − 銷貨 + 調整` |
| **Product 3 — 哈瓦士 MTRIX** | 對接方五檔 CSV → 驗證／修復 → 下游 BI | 56 欄逐欄帶出處、15 條驗證規則、12 條編號化修復規則 |

工作區包含 `wellfit.exe`（`wellfit_cli`）與獨立的 `wellfit_sync.exe`。Sync 目前只有設備探索與狀態骨架，資料傳輸尚未實作。無前端、無 JavaScript。

## 2. Crate 地圖

| Crate | 路徑 | 職責 |
|---|---|---|
| `wellfit_core` | [`crates/wellfit_core/`](crates/wellfit_core/) | MDM 實體編碼清洗（條碼前導零不遺失、DIST_ID 去遮罩、廠商國碼前綴分離）、SQLite 冪等帳本、桌面互斥鎖、Release Gate、**聯邦契約模組** `wellfit_core::federation`（模組位於 `crates/wellfit_core/src/federation.rs`）、定點十進位 `scaled`、CR/LF/CRLF 皆可之行分割 `text`、名稱解析欄位契約 `contract` |
| `wellfit_shopee` | [`crates/wellfit_shopee/`](crates/wellfit_shopee/) | 蝦皮進貨訂單引擎：**8 欄名稱解析契約**（`Shopee inbound order export`）、EX-06 金額守恆、重複訂單去重、ERP 匯入 CSV 產生 |
| `wellfit_reports` | [`crates/wellfit_reports/`](crates/wellfit_reports/) | 輔翼 ERP 庫存報表解析（零 OCR）、**8 欄名稱解析契約**（`Fuyi ERP inventory report`）、庫存恆等式、異常列隔離、發布閘門 |
| `wellfit_mtrix` | [`crates/wellfit_mtrix/`](crates/wellfit_mtrix/) | MTRIX 五檔（ALTHCI／ALTHES／ALTHFV／ALTHPD／ALTHVD）**56 欄契約**（`spec.rs`）、**VR-01..VR-14** 驗證規則（`validators.rs`）、**ANOM-01..ANOM-12** 修復規則（`repairs.rs`）、跨檔參照與幣別分組彙總 |
| `wellfit_bankpay` | [`crates/wellfit_bankpay/`](crates/wellfit_bankpay/) | E.Sun BankPay 媒體匯出、回覆匯入及 API 查詢 |
| `wellfit_sync` | [`crates/wellfit_sync/`](crates/wellfit_sync/) | 設備探索、同步狀態及 daemon 骨架；資料傳輸尚未實作 |
| `wellfit_cli` | [`crates/wellfit_cli/`](crates/wellfit_cli/) | `wellfit.exe` 進入點，九個子命令：`status`／`shopee`／`reports`／`mtrix`／`audit`／`spec`／`access-control`／`jit`／`bankpay` |

> **重要更正**：舊版文件提到的 `contracts/federation/` **目錄不存在**。
> 聯邦契約是以 Rust 模組實作，位置為 `crates/wellfit_core/src/federation.rs`，對外介面為 `wellfit_core::federation`。

## 3. 快速開始

### 3.1 建置

本工作站的應用程式控制原則會阻擋「建置於桌面路徑下」的執行檔，
因此**必須**透過知識庫的建置腳本，它會設定 MSVC 工具鏈、Visual Studio 建置環境，
並把 target 目錄導向儲存庫之外：

```powershell
cd "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit"

powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "build -p wellfit_cli" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

執行檔位置：`%LOCALAPPDATA%\Temp\wellfit-target\debug\wellfit.exe`

### 3.2 執行

```powershell
$wf = "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"

# 契約面：5 檔 56 欄、15 條規則、12 條修復規則
& $wf spec --format markdown

# MTRIX 五檔投放（真實交付位元組）
& $wf mtrix --dir spec/fixtures/mtrix/delivered `
            --output-dir "$env:TEMP\wellfit-repaired" `
            --report "$env:TEMP\wellfit-report.json" `
            --batch DEMO-20261020 `
            --db "$env:TEMP\wellfit-ledger.sqlite"

# 蝦皮（合成示範資料）
& $wf shopee --input data/shopee_orders.csv --output "$env:TEMP\erp_shopee_import.csv"

# 輔翼 ERP 報表（合成示範資料）
& $wf reports --input data/fuyi_inventory_report.csv --recipient finance@dachen.com.tw

# 帳本狀態與稽核
& $wf status --db "$env:TEMP\wellfit-ledger.sqlite"
& $wf audit --data-dir data
```

> **更正**：舊版文件的 `reports --input data/fuyi_inventory.csv` 範例指向不存在的檔案。
> 實際檔名為 `data/fuyi_inventory_report.csv`。

## 4. 規格來源（Specification sources）

本專案的每一個契約事實都來自**跨專案知識庫**，不來自網路、不來自推測。

### 4.1 知識庫位置

[`../../Knowledge/`](../../Knowledge/) — 2026 年度跨專案知識庫（`2026/Knowledge`）。
索引見 [`docs/knowledge-index.md`](docs/knowledge-index.md)。

### 4.2 來源文件與層級

| 層級 | 文件 | 快照 |
|---|---|---|
| **L1 原始規格** | Alpargatas Programmer's Manual（DCA Revision, September 2024） | [`../../Knowledge/sources/snapshots/manual-alpargatas/text.md`](../../Knowledge/sources/snapshots/manual-alpargatas/text.md) |
| **L2 內部推薦版** | MTRIX BI 報表暨系統介接開發規格書 V2.0 | [`../../Knowledge/sources/snapshots/mtrix-bi-spec-v2/text.md`](../../Knowledge/sources/snapshots/mtrix-bi-spec-v2/text.md) |
| **L3 近期整合版** | ALPARGATAS 系統介接規格書 V6.0-Full | [`../../Knowledge/sources/snapshots/alpargatas-v6-docx/text.md`](../../Knowledge/sources/snapshots/alpargatas-v6-docx/text.md) |
| **L2 內部推薦版** | 蝦皮需求規格書 V1.0／V2.1 | [`../../Knowledge/sources/snapshots/shopee-requirements-v21/text.md`](../../Knowledge/sources/snapshots/shopee-requirements-v21/text.md) |
| **aux 輔助** | IMS 文件管制模板 v0.9 | [`../../Knowledge/sources/snapshots/ims-template-set/text.md`](../../Knowledge/sources/snapshots/ims-template-set/text.md) |

來源登錄（路徑、SHA-256、抽取完整性、驗證標記）：
[`../../Knowledge/sources/source-register.md`](../../Knowledge/sources/source-register.md) ／
[`source-register.json`](../../Knowledge/sources/source-register.json)

### 4.3 裁決原則

| 面向 | 依據 |
|---|---|
| 欄位**語意** | L1 原始規格優先；L2 不得覆寫 L1／L3；L3 自身不裁決 |
| 介接**驗收** | 以實際交付位元組為準 |
| 不一致處 | 登議題、保留原值、產 Warning，**不靜默改寫** |
| 不可驗證者 | 標 `unverified`，**絕不推測** |

詳見 [`../../Knowledge/adr/ADR-0001-source-hierarchy-and-conflict-arbitration.md`](../../Knowledge/adr/ADR-0001-source-hierarchy-and-conflict-arbitration.md)。

### 4.4 逐欄對照

[`../../Knowledge/reconciliation/field-diff.md`](../../Knowledge/reconciliation/field-diff.md) —
56 欄的「L1 原文／L2 推薦／L3 V6.0／交付觀察值／裁決／狀態／議題」對照表，
由 `tools/reconciliation/build-field-diff.ps1` 自 `spec.rs` 產生，不符即失敗。

## 5. 規則與數字（每一個都可追溯）

| 項目 | 數量 | 來源 |
|---|---:|---|
| MTRIX 檔案數 | 5 | `spec.rs::MTRIX_SPEC`；`test_five_files_declared` |
| MTRIX 欄位數 | 56 | `spec.rs::TOTAL_FIELDS`；`test_total_field_count_is_56`；`wellfit spec` 輸出 |
| 驗證規則 | 14（欄位字典 12 + 原始規格衍生 2） | `validators.rs::VrRule::all()`、`L2_DECLARED`、`MANUAL_DERIVED` |
| 修復規則 | 12 | `repairs.rs::REPAIR_RULES`；`test_rule_catalogue_is_numbered_and_unique` |
| 標題偏差 | 8 | `spec.rs::test_deviations_are_flagged_and_never_dropped` |
| 蝦皮契約必填欄 | 8 | `shopee_engine.rs::shopee_order_contract()` |
| 輔翼 ERP 契約必填欄 | 8 | `report_automator.rs::fuyi_inventory_report_contract()` |
| 測試數 | 121 通過／0 失敗 | `build.ps1 -Command "test --workspace --no-fail-fast"`，2026-10-20 06:33 UTC 實測 |

規則目錄：
[`../../Knowledge/specs/mtrix-vr-rules.md`](../../Knowledge/specs/mtrix-vr-rules.md)、
[`../../Knowledge/specs/mtrix-anomaly-catalogue.md`](../../Knowledge/specs/mtrix-anomaly-catalogue.md)、
[`../../Knowledge/specs/mtrix-field-dictionary.md`](../../Knowledge/specs/mtrix-field-dictionary.md)（程式碼生成）。

### 5.1 已撤回的舊宣稱

| 舊宣稱 | 現況 |
|---|---|
| 「蝦皮 **58 欄**全景清洗驗證」 | **撤回**。58 是內部需求文件的**候選 Mapping 列數**（47 欄 `OPEN`／11 欄 `REC`），不是實際欄位。實作契約為 **8 欄**。詳見 [`../../Knowledge/specs/shopee-contract.md`](../../Knowledge/specs/shopee-contract.md) |
| 「MTRIX 修復 **27 項**格式異常」 | **撤回**。無來源。實際為 **12 條編號化修復規則 ANOM-01..ANOM-12**，每條附授權來源 |
| 「**VR-01~08** 跨檔外鍵勾稽」 | **撤回並更正**。實際規則集為 **VR-01..VR-14** |
| 「`contracts/federation/` 乾淨室資料交換協定」 | **更正**。實作為模組 `crates/wellfit_core/src/federation.rs`（`wellfit_core::federation`），非目錄 |
| 四個 `file:///c:/Users/User/OneDrive/桌面/WEB3/project/Wellfit/...` 連結 | **更正**為倉庫相對路徑（指向 `crates/<crate>/`） |
| `reports --input data/fuyi_inventory.csv` | **更正**為 `data/fuyi_inventory_report.csv` |

撤回理由與逐條證據見 [`../../Knowledge/traceability/claims-vs-evidence.md`](../../Knowledge/traceability/claims-vs-evidence.md)。

## 6. 已知缺口（Known gaps）

> 這裡列出的每一項都是**已知的、已登錄的**缺口，不是待辦雜項。
> 完整清單見 [`../../Knowledge/issues/mtx-issue-register.md`](../../Knowledge/issues/mtx-issue-register.md)（87 項）。

| # | 缺口 | 影響 | 狀態 |
|---|---|---|---|
| G-01 | **蝦皮真實匯出欄位表頭未取得**（OI-003）；58 列候選 Mapping 僅 47 欄待確認、11 欄推薦 | 蝦皮端介接事實為 `unverified`；不得宣稱已對接真實蝦皮資料 | `open` |
| G-02 | **輔翼 ERP 匯入範本與欄位字典未取得**（OI-006）；來源 `輔翼ERP自動化.docx` 已抽取，但內容為**技術研究方案**而非介接契約，其 §5 將 ERP 端未知項列為待填清單 | ERP 端欄位對應為 `unverified`；查詢條件、排序分頁、單位與來源一致性層亦未定義 | `open` |
| G-03 | **MTRIX 逐欄必填清單不可得**（MTRIX-003／ISS-013／LOCAL-013） | 空值一律記 Warning、不隔離 | `pending-counterparty` |
| G-04 | **內部欄位字典 `.xlsx` 無法抽取**（LOCAL-020） | VR-01..VR-12 的**原始條文**目前不可引用，只能由程式行為與 L2／L3 交叉推得 | `open` |
| G-05 | **售價合理性檢查未實作**（LOCAL-014，原文 p.13 要求對照市場平均，但無基準資料） | 該項驗證不存在 | `pending-counterparty` |
| G-06 | **計量單位一致性未驗證**（LOCAL-015，五檔無單位欄） | 跨商品加總可能混合單位 | `pending-counterparty` |
| G-07 | **VR-12 僅部分實作**：已產出依 `COUNTRY_ISO` 分組之數量／金額；與管理報表之**總額核對未實作**（缺管理報表來源） | 不可宣稱已完成管理報表核對 | `open` |
| G-08 | **傳輸協定、認證、重送策略未定義**（MTRIX-015／MTRIX-026／ISS-011／ISS-012／ISS-031） | 現行為目錄投放；不臆設 SFTP | `pending-counterparty` |
| G-09 | **幣別辨識方式未確認**（MTRIX-023／ISS-014） | 金額依 `COUNTRY_ISO` 分組，**不做幣別換算** | `open` |
| G-10 | **8 處標題偏差已接受為現況**（LOCAL-001..009）：交付 `ADRESS`／`ZIP_CODE`／`CUSTOMER_ID`／`MANAGER_CODE`／`SALESPERSON_CODE`／`COUNTRY`／`COUNTRY_ISO` 與原文寫法不同 | 輸出使用正規化名，偏差記於報告 | `accepted-as-is` |
| G-11 | **VENDOR_ID 國碼前綴長度不一致**：原文為 2 字母，交付為 3 字母且廠商碼 16 位（宣告 14） | 前綴獨立保存、**不截斷**；需對接方確認 | `pending-counterparty` |
| G-12 | **正式 UAT 測試資料、允收標準與簽核人未提供**（OI-018） | 不可宣稱「UAT 完成」 | `open` |

### 6.1 已解決但值得知道的兩個真實世界陷阱

| 陷阱 | 說明 |
|---|---|
| **CR-only 行尾** | 對接方五個 MTRIX 檔案使用 **CR（0x0D）行尾，LF 為 0**。Rust `str::lines()` 只認 LF／CRLF，會把整份檔案當成單一標題列，導致所有欄數檢查失敗。已改用 `wellfit_core::text::split_lines`（LF／CRLF／CR 皆可），並在衍生 fixture 中刻意保留 CR 行尾以免缺陷復發（`LOCAL-019`） |
| **下載殘留在檔名** | 交付檔名在時間戳之後帶有 ` 2 (3)`、` 5 (4)` 等殘留。`VR-01` 以 **Warning** 回報，不拒收（`LOCAL-022`） |

## 7. 資料來源與測試資料

| 類別 | 內容 | 說明 |
|---|---|---|
| **真實交付樣本** | [`spec/fixtures/mtrix/delivered/`](spec/fixtures/mtrix/delivered/) | 與對接方 `API_Integration_Samples/` **SHA-256 逐一相同**，未修改 |
| **衍生案例** | [`spec/fixtures/mtrix/derived/`](spec/fixtures/mtrix/derived/) | 四組負向案例，由 [`spec/fixtures/mtrix/generate-derived.ps1`](spec/fixtures/mtrix/generate-derived.ps1) 決定性產生，每組只注入一個缺陷 |
| **合成示範資料** | `data/shopee_orders.csv`、`data/fuyi_inventory_report.csv` | **合成資料，非實體來源**，不得以「交付樣本」名義引用 |

溯源細節（含逐檔雜湊與衍生規則）：[`../../Knowledge/data/provenance.md`](../../Knowledge/data/provenance.md)。

## 8. 專案文件

| 文件 | 內容 |
|---|---|
| [`AGENTS.md`](AGENTS.md) | 代理人操作規範、建置入口、不變量、接手程序 |
| [`CHANGELOG.md`](CHANGELOG.md) | 版本變更紀錄與破壞性變更 |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | 架構與資料流 |
| [`docs/CLI.md`](docs/CLI.md) | 命令參考（實測輸出） |
| [`docs/DEMO.md`](docs/DEMO.md) | 示場流程與問答 |
| [`docs/knowledge-index.md`](docs/knowledge-index.md) | 知識庫索引 |

## 9. 不變量摘要

1. 每個 `.rs` 第一行 `#![forbid(unsafe_code)]`，零 `unsafe`。
2. `cargo clippy --workspace --all-targets -- -D warnings` 必須零警告。
3. 金額與數量一律定點十進位／整數分，**禁止浮點**。
4. 欄位一律**以名稱解析**，**禁止序位退讓**；必填欄缺漏即硬性失敗。
5. **絕對反模擬**：合成資料不得呈現為實體資料；每份 fixture 皆附出處與 SHA-256。
6. 不可驗證之來源標 `unverified`，**絕不推測**；不可追溯之數字不得寫入文件。

完整版見 [`AGENTS.md`](AGENTS.md) 與知識庫 [`adr/`](../../Knowledge/adr/)。
