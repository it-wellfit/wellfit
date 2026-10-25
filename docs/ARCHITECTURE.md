# Wellfit 架構說明（ARCHITECTURE）

> 文件編號：WFLT-DOC-ARC-001｜狀態：Draft｜版本：v1.0｜日期：2026-10-20
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 變更摘要：v1.0。對齊 v1.0.0 之實作：補上 `contract`／`text`／`scaled` 三個新模組、
> 六個 CLI 子命令、真實交付位元組之 fixture 佈局，以及 CR-only 行尾這個關鍵約束。
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

## 1. 全景

```
              ┌──────────────────────────── 2026/Knowledge（跨專案知識庫）────────────────────────────┐
              │  sources/snapshots/  adr/  specs/  reconciliation/  issues/  traceability/  uat/  ops/  │
              └───────────────▲──────────────────────────────────────────────────────────┬───────────────┘
                      契約事實 │                                                  產出文件 │
                              │                                                          │
  ┌───────────────────────────┴──────────────────────────────────────────────────────────┴───────────┐
  │                                   Dev/Wellfit（5-crate Rust 工作區）                            │
  │                                                                                                │
  │  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐  │
  │  │ wellfit_core │◀──│wellfit_shopee│   │wellfit_reports│  │ wellfit_mtrix │   │ wellfit_cli  │  │
  │  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘  │
  │        ▲                  │                   │                  ▲                │          │
  └────────┼──────────────────┴───────────────────┴──────────────────┼────────────────┼──────────┘
           │                                                     契約（56 欄）      │
   ┌───────┴────────┐  ┌────────────────┐  ┌─────────────────┐  ┌────┴─────┐  ┌──────┴───────┐
   │ contract（名稱 │  │ text（LF/CRLF/ │  │ scaled（定點    │  │ spec.rs  │  │ validators.rs │
   │ 解析，無序位） │  │  CR 行分割）   │  │ 十進位 i64）   │  │ repairs.rs│  │ mtrix_engine │
   └────────────────┘  └────────────────┘  └─────────────────┘  └──────────┘  └──────────────┘
```

**分層原則**：

| 層 | 職責 | 不得做的事 |
|---|---|---|
| `wellfit_core` | 與業務無關的共用機制：欄位契約、文字切分、定點數、帳本、閘門、MDM、互斥鎖 | 不得知道 MTRIX／蝦皮／ERP 的欄位語意 |
| `wellfit_mtrix`／`wellfit_shopee`／`wellfit_reports` | 各產品之契約與規則 | 不得自行發明規則；每條規則都必須有來源 |
| `wellfit_cli` | 組裝與輸出 | 不得包含業務邏輯 |

## 2. 資料流

### 2.1 Product 3 — MTRIX 五檔（最具代表性）

```
  對接方樣本資料夾
  ALTHCI…csv ALTHES…csv ALTHFV…csv ALTHPD…csv ALTHVD…csv   （CR-only 行尾、UTF-8 無 BOM、分號分隔）
        │
        ▼
  MtrixEngine::discover_files()            ── 五檔缺一即中止（test_discovery_requires_all_five_files）
        │
        ▼
  wellfit_core::text::split_lines()         ── LF / CRLF / CR 皆可；移除 BOM；去除空行
        │
        ▼
  repairs::repair_header()                  ── ANOM-01/02/03/12：尾端空欄、正規化名
        │
        ▼
  validators::compare_header()              ── VR-02：欄數 + 三種寫法（交付／原文／技術別名）
        │                                            ✘ → 僅拒絕該檔，其餘四檔照常（ADR-0002）
        ▼
  MtrixEngine::process_drop()               ── 逐檔：VR-01 → VR-03/04/07 → 逐列驗證
        │                                     ✘ → 該列隔離（row-quarantined），原值入帳本
        │                                     負庫存 → ANOM-08 歸零 + Warning
        ▼
  aggregate_duplicates()                    ── VR-05／ANOM-10：同交易彙總（不隔離）
        │
        ▼
  apply_cross_file_rules()                  ── VR-08/09/10/11/13/14：列出鍵值、保留列
        │
        ▼
  currency_groups()                         ── 依 COUNTRY_ISO 分組，i128 checked sum，10^4 / 10^8 縮放，不換算幣別
        │
        ▼
  ValidationReport（JSON） + 修復輸出檔 + SQLite 帳本
```

### 2.2 Product 1 — 蝦皮

```
  蝦皮匯出 CSV（8 欄）
        │  resolve_columns() → ColumnContract::resolve()   ✘ 缺必填欄 → Err（不消耗批次編號）
        ▼
  ShopeeOrder { order_total_cents: i64 }    ── 整數分，全程零浮點
        │  verify_line_sum()：ΣLINE_AMOUNT == ORDER_TOTAL  ✘ → ERR_SUM_MISMATCH_EX06，列隔離
        │  parse_money_cents() → Option<i64>               ✘ → ERR_MONEY_FORMAT，列隔離（不當成 0）
        ▼
  IdempotencyLedger（批次 ID + SHA-256 去重）
        ▼
  generate_fuyi_erp_import_csv()            ── 小數兩位以整數除法 + 補零餘數產生
```

### 2.3 Product 2 — 輔翼 ERP 庫存報表

```
  庫存報表 CSV（8 欄）
        │  parse_native_csv_report() → ColumnContract::resolve()  ✘ 缺必填欄 → Err
        ▼
  逐列檢查
        │  END_INV == BEGIN_INV + IN_QTY - OUT_QTY + ADJ_QTY   ✘ → ERR_INV_BALANCE_MISMATCH，列隔離
        │  數量不可解析 → ERR_INV_NUMBER_FORMAT，列隔離（不當成 0）
        ▼
  ReleaseGate                              ── 僅白名單網域（dachen.com.tw / wellfit.tw / alpargatas.com）
        ▼
  發布（recipients）
```

### 2.4 可插拔管線（`pipeline.rs`）

`crates/wellfit_mtrix/src/pipeline.rs` 提供 `PipelineStage` trait 與 `Pipeline` 容器，
支援 Parse / Repair / Validate / Aggregate / CrossFile / Report / Output 七階段可插拔、可跳過、可重排，
共享 `PipelineContext`。目前為基礎架構（階段實作逐步遷入，行為等價目標）。

### 3.1 `contract` — 名稱解析的欄位契約

```rust
pub struct ColumnContract { name: &'static str, required: Vec<&'static str>, optional: Vec<&'static str> }
pub struct ColumnMap { /* 欄名 → 索引 */ }
```

- `resolve(&[&str]) -> Result<ColumnMap, String>`：**只接受名稱**。
- 缺任一必填欄 → `Err`，錯誤訊息列出**所有**缺漏欄名與實際標題。
- `ColumnMap::index` 對未宣告欄位回傳 `Err`；**不存在序位退讓的程式路徑**。

> 這是 ADR-0005 的實作落點。舊實作以 `header.iter().position(...)` 解析並在失敗時退回硬編碼序位，
> 會把資料靜默寫到錯欄；現在沒有這條路。

### 3.2 `text` — 行分割

```rust
pub fn split_lines(raw: &str) -> Vec<&str>   // LF / CRLF / CR 皆可，移除 BOM，去除空行
pub fn split_fields(line: &str, delim: char) -> Vec<String>
```

> **為什麼需要它**：Rust `str::lines()` 只認 LF 與 CRLF，**單獨的 CR 不換行**。
> 對接方的五個 MTRIX 檔案正是 CR-only，會被整份視為單一標題列。
> 衍生 fixture 刻意保留 CR 行尾，確保此約束不被 fixture 掩蓋。

### 3.3 `scaled` — 定點十進位

```rust
pub struct ScaledDecimal { units: i64 }   // fixed 10^4 scale
```

- 解析、加總、輸出皆為整數運算；加總使用 `checked_add`，溢位回傳 `None`。
- 輸入超過四位小數或格式不合法時明確拒絕，不隱式捨入或截斷。
- 國別數量總計以 `10^4` 縮放、數量乘價格總計以 `10^8` 縮放；使用 `i128` checked arithmetic，無法精確表示或溢位時明確失敗。
- 蝦皮金額另以 `i64` 分為單位；兩者之間不做浮點換算。

## 4. MTRIX 契約面（`wellfit_mtrix::spec`）

```rust
pub struct FieldSpec {
    seq: u8,
    source_name: &'static str,   // 交付標題（介接驗收基準）
    spec_name: &'static str,     // 原文欄名（語意基準）
    canonical_name: &'static str,// 技術別名（下游輸出）
    kind: FieldKind,             // Text / Number / Date / Code
    length: u16,
    allowed: Option<&'static [&'static str]>,
    key_role: KeyRole,
    tier: SourceTier,            // ManualL1 / InternalL2 / ObservedOnly / Unverified
    status: VerifyStatus,        // Confirmed / ObservedOnly / Deviation / Pending / Unverified
    manual_page: Option<u16>,    // 原 PDF 印刷頁碼
    issue_id: Option<&'static str>,
}
```

| 欄位 | 為何存在 |
|---|---|
| `source_name`／`spec_name`／`canonical_name` 三者並存 | 交付寫法、原文寫法、技術別名三者皆須被接受（VR-02），但不可混為一談 |
| `tier` | 記錄這條規格來自 L1／L2 還是觀察值 |
| `status` | 記錄這條規格目前**可不可以被當成事實** |
| `manual_page` | 原文印刷頁碼；與快照的 `[pN]`（Word 重排頁碼）**不同**，引用時必須註明 |
| `issue_id` | 把欄位直接連到議題登錄 |

`is_deviation()`（`source_name != spec_name`）之結果為 **8**，即 8 處標題偏差。

## 5. 帳本與發布閘門

| 元件 | 職責 |
|---|---|
| `IdempotencyLedger`（SQLite） | 批次去重（批次 ID + SHA-256）、隔離記錄（原值完整保留）、`validation_findings` 查詢、`severity`＋`rule_code`；`cumulative_balance` 表（客戶層累計，`MdmCatalog::new` 初始化） |
| `MdmCatalog::count_file_batches` | 提供已登記批次數（`IdempotencyLedger` 的唯讀查詢 API） |
| `Mutex`（桌面互斥鎖） | 跨工作流互斥；自訂波次與逾時鎖回收 |
| `ReleaseGate` | 驗證通過才可發布；白名單網域；失敗轉 `Quarantined` |

### 5.1 Release Gate 狀態機

`wellfit_core::release_gate::ReleaseGate`（來源檔 `src/release_gate.rs`）實現 Two-Way Door vs One-Way Door 邊界：

| 狀態 | 說明 |
|---|---|
| `DraftGenerated` | 初始狀態；尚未驗證 |
| `VerifiedPass` | 驗證通過（Two-Way Door）；允許發布 |
| `Quarantined` | 驗證失敗；不發布，保留原值 |
| `AuthorizedDispatch` | 發送已啟動（One-Way Door）；中途狀態 |
| `Completed` | 發送成功或 `DispatchUncertain` 之狀態確認後為已送達 |
| `Failed` | 發送明確失敗 |
| `DispatchUncertain` | 發送逾時／不明，須執行 `reconcile_uncertain_dispatch` 確認 |

**轉移邏輯**：
- `verify_payload(is_valid, reason)` → Two-Way Door：`true` 至 `VerifiedPass`，`false` 至 `Quarantined`。
- `validate_recipients(recipients)` → 驗證收件人地址格式與白名單網域。
- `dispatch(transport_fn, recipients)` → One-Way Door：僅 `VerifiedPass`／`AuthorizedDispatch` 可呼叫；`Ok(true)` → `Completed`，`Ok(false)` → `Failed`，逾時 → `DispatchUncertain`。
- `reconcile_uncertain_dispatch(status_check_fn)` → 逾時後的狀態確認：對端收到 → `Completed`；否則回 `VerifiedPass` 允許重送。
- 所有轉移均附加到 `history: Vec<TransitionRecord>`，提供稽核線索。

## 6. CLI

| 子命令 | 輸入 | 產出 |
|---|---|---|
| `status` | 帳本 | 隔離筆數、驗證 error／warning 計數 |
| `shopee` | 蝦皮匯出 CSV | 輔翼 ERP 匯入批次 CSV |
| `reports` | 庫存報表 CSV | 收件人清單、發布閘門結果 |
| `mtrix` | 五檔資料夾 | 驗證報告 JSON、修復輸出檔 |
| `audit` | `data/` + fixtures | 全量稽核報告 |
| `spec` | — | 56 欄契約與規則目錄（markdown／json） |

命令細節與實測輸出見 [`CLI.md`](CLI.md)。

## 7. 測試資料佈局

```
spec/fixtures/mtrix/
├── PROVENANCE.md          ── 逐檔出處與 SHA-256
├── generate-derived.ps1   ── 決定性產生四組負向案例
├── delivered/             ── 真實交付位元組（SHA-256 與來源相同，CR 行尾保留）
└── derived/
    ├── case-a-value-violations/          ── 負庫存、型別越界、不存在的日期、文件類型越界
    ├── case-b-duplicate-transaction/      ── 同交易重複（測試彙總）
    ├── case-c-header-rejected/           ── 標題不符（測試逐檔拒絕）
    └── case-d-salesforce-orphan-customer/ ── 業務檔孤島客戶（測試 VR-14）
```

每組衍生案例都是**完整五檔**，且**只有一個檔案帶一個缺陷**，使單一規則可被單獨驗證。

## 8. 已知架構限制

| # | 限制 | 說明 |
|---:|---|---|
| 1 | 介接為**目錄投放**，非主動傳輸 | 傳輸協定、認證與重送策略皆未定義（`MTRIX-015`／`ISS-011`／`ISS-031`）。L3 建議「暫定 SFTP」但端點不明，本專案不臆設 |
| 2 | 幣別**不換算** | 幣別辨識方式未確認（`MTRIX-023`），故以 `COUNTRY_ISO` 分組呈現 |
| 3 | 與管理報表之總額核對未實作 | VR-12 僅完成幣別分組彙總；管理報表來源未取得 |
| 4 | 蝦皮／ERP 端契約為**內部宣告** | 對方未提供真實樣本；8 欄契約為專案自我宣告，ERP 側欄位語意 `unverified` |
| 5 | 價格合理性與單位一致性**未實作** | 缺基準資料與單位欄（`LOCAL-014`、`LOCAL-015`） |

## 9. 相關文件

| 文件 | 內容 |
|---|---|
| [`../README.md`](../README.md) | 專案總覽、規格來源、已知缺口 |
| [`../AGENTS.md`](../AGENTS.md) | 建置入口、不變量、接手程序 |
| [`CLI.md`](CLI.md) | 命令參考（實測輸出） |
| [`DEMO.md`](DEMO.md) | 示場流程與問答 |
| [`knowledge-index.md`](knowledge-index.md) | 知識庫索引 |
| [`../CHANGELOG.md`](../CHANGELOG.md) | 版本變更與破壞性變更 |
| [`../../Knowledge/reconciliation/field-diff.md`](../../../Knowledge/reconciliation/field-diff.md) | 56 欄對齊與裁決表 |
| [`../../Knowledge/specs/mtrix-vr-rules.md`](../../../Knowledge/specs/mtrix-vr-rules.md) | VR-01..VR-14 目錄 |
| [`../../Knowledge/specs/mtrix-anomaly-catalogue.md`](../../../Knowledge/specs/mtrix-anomaly-catalogue.md) | ANOM-01..ANOM-12 目錄 |
