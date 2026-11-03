# Plan: 輔翼銷貨退貨單 + 退貨源分析 + 財務自動化 (wellfit_returns)

> **Disposition (2026-11-02): Proposal only — not approved for implementation.** No accountable business/finance owner or source-file contract is recorded. Confirm return-reason taxonomy, accounting treatment, authorization boundaries, and the actual FlapsErp export format with their owners before treating any decision or acceptance criterion below as binding.

## Goal
建立 Wellfit Rust 工作區新增 `wellfit_returns` Crate，自動化處理「輔翼銷貨退貨單」，並透過 FlapsErp Electron 桌面客戶端匯入退貨單資料，分析退貨源（退貨原因歸類），將會計分錄寫入 SQLite Idempotency Ledger 供財務對账，並提供快速智能對账功能。整體遵循現有 Wellfit `wellfit_shopee` / `wellfit_reports` 架構模式。

## Context & Source Evidence
- Wellfit 是「大前實業 輔翼 ERP 整合與三大產品自動化引擎」（5+ Crate Rust 工作區）：`wellfit_core` → `wellfit_shopee` / `wellfit_reports` / `wellfit_mtrix` → `wellfit_cli`。
- FlapsErp 是位於 `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp` 的 Electron 桌面應用程式（`resources/app.asar`），為輔翼 ERP 桌面客戶端；退貨單資料源自該客戶端匯出。
- 現有模式：欄位契約以名稱解析（無序位退讓）、金額全程以整數分 (`i64`) 禁止浮點、SQLite IdempotencyLedger 幂等去重、Release Gate 狀態機（DraftGenerated→VerifiedPass/Quarantined→AuthorizedDispatch→Completed/Failed/DispatchUncertain）、CLI 子命令、隔離列保留完整原值、所有規則必須有來源（不可推測）。
- Shared Standards: `#![forbid(unsafe_code)]`、clippy 零警告、public API 必須有文件註解、貨幣欄位用整數、依賴版本鎖定。

## Proposed decisions (owner approval required)
1. **新增 Crate `wellfit_returns`** 在 `crates/wellfit_returns/`，依賴 `wellfit_core`，由 `wellfit_cli` 組裝，新增 CLI 子命令 `returns`。
2. **資料來源**：FlapsErp Electron 桌面客戶端匯出的銷貨退貨單 CSV（以檔案投放方式攝取，不臆設 SFTP/API；端點與認證標記為 pending-counterparty，與 MTRIX G-08 一致）。
3. **財務目標**：會計分錄（貸項通知單、應收帳款沖銷、銷貨收入退回、庫存退回、銷貨成本調整）以整數分寫入 `wellfit_core::ledger::IdempotencyLedger`（SQLite），供財務系統匯入對账；不寫入外部會計軟體 API。
4. **退貨源分析**：以退貨原因代碼（原因碼：瑕疵品／出錯貨／客戶改主意／運輸損壞／過期／其他）、商品、客戶、業務員、期間做歸類彙整，輸出 JSON 報告與退貨源分析 CSV。
5. **智能對账**：退貨單號與會計分錄之自動比對（一對一/一對多），偵測未對账分錄與金額差異，以 `reconcile_uncertain_dispatch` 模式處理逾時/不明狀態。
6. **不破壞現有功能**：新 Crate 與新 CLI 子命令，不修改現有 `shopee`/`reports`/`mtrix` 行為。

## Affected Boundaries
- `crates/wellfit_returns/`（新建）：`contract.rs`（退貨單欄位契約）、`engine.rs`（RetEngine）、`returns_source.rs`（退貨源分析）、`finance.rs`（會計分錄產生）、`reconcile.rs`（智能對账）、`lib.rs`。
- `crates/wellfit_cli/src/main.rs`：新增 `returns` 子命令（`--input`、`--output`、`--reconcile`、`--batch`、`--db`、`--report`）。
- `crates/wellfit_core/src/ledger.rs`：可能新增 `finance_entry` / `reconciliation` 查詢（唯讀，不破壞既有表）。
- `Cargo.toml`（Workspace）：加入 `wellfit_returns` member。
- FlapsErp：僅作為 CSV 匯出來源，不修改 FlapsErp 程式碼（Electron asar 為二進位，逆向工程不在本計畫範圍；若需直接讀取 FlapsErp 內部資料庫，另立 ADR）。

## Data Flow
```
FlapsErp Electron 客戶端
  │ 匯出銷貨退貨單 CSV
  ▼
wellfit.exe returns --input <退貨單CSV> --output <財務分錄CSV> --report <分析報告> --batch <ID> --db <ledger.sqlite>
  │
  ├─► contract.resolve()  (名稱解析，缺欄整批拒收，不消耗 batch ID)
  ├─► IdempotencyLedger.register_file_batch()  (幂等去重)
  ├─► 驗證規則：退貨單號唯一、金額格式（整數分）、LINE_AMOUNT = QTY × UNIT_PRICE、退貨原因碼在允許集、參照原銷貨單存在性
  ├─► 隔離列：保留完整原值，寫入 ledger（code=ERR_*），不當成 0
  ├─► 會計分錄產生：貸項通知單 / AR 沖銷 / 銷售退回 / 庫存退回 / COGS 調整（i64 分）
  ├─► Release Gate：VerifiedPass → 發布；失敗 → Quarantined
  ├─► 輸出：財務分錄 CSV + 退貨源分析報告 JSON/CSV
  └─► 智能對账：退貨單號 vs 會計分錄比對，標示未對账/差異
```

## Failure Modes & Handling
| 失敗模式 | 處理方式 |
|---|---|
| 缺必填欄 | 整批拒收，錯誤訊息列出所有缺漏欄名，batch ID 保留可重送 |
| 金額格式錯誤 | 該列隔離 (`ERR_MONEY_FORMAT`)，不當成 0，保留原值 |
| LINE_AMOUNT ≠ QTY × UNIT_PRICE | 該列隔離 (`ERR_AMOUNT_MISMATCH`) |
| 重複退貨單號 | Ledger 拒絕 (`ERR_DUPLICATE_RETURN_NO`)，不重複入帳 |
| 原銷貨單不存在 | 該列隔離 (`ERR_ORIGINAL_SALES_ORDER_NOT_FOUND`)，作 Warning 或 Error 視契約而定 |
| 資料庫錯誤 | 傳回明確錯誤，不回報看似成功 |
| 對账不符 | 標示差異，供財務人工覆核；逾時用 `reconcile_uncertain_dispatch` |

## Rollout / Migration
1. 加入 `wellfit_returns` 至 workspace `Cargo.toml` members。
2. 實作 `wellfit_returns` Crate（contract + engine + source analysis + finance + reconcile + lib）。
3. 在 `wellfit_cli` 新增 `returns` 子命令並註冊。
4. 撰寫單元測試（每個 public 函數）與整合測試（以合成退貨單 CSV 為基準，模擬 FlapsErp 匯出格式）。
5. 執行 `cargo fmt -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace --no-fail-fast` 全綠。
6. 合成測試資料不得標示為 FlapsErp 真實匯出（反模擬原則）。
7. 文件：新增 `docs/ARCHITECTURE.md` 節、`docs/API_SPEC.md` `returns` 子命令、`ADR-XXXX` 記錄本決策。

## Validation Plan
- `cargo build --workspace --locked`：零錯誤
- `cargo clippy --workspace --all-targets -- -D warnings`：零警告
- `cargo test --workspace --no-fail-fast`：全部通過（含新增的 `wellfit_returns` 測試）
- `wellfit returns --input <合成退貨單CSV> --output <財務分錄CSV> --report <報告> --batch <ID> --db <ledger.sqlite>`：正確產生會計分錄與退貨源分析，智能對账標示正確
- 隔離列保留完整原值，每筆 finding 有規則編號、嚴重度、行號、鍵值、原因
- 幂等：同一 `--batch` 重送不重複入帳；異 batch 重複退貨單號被拒

## Open Questions / Out of Scope
- FlapsErp 客戶端 CSV 匯出的精確欄位名稱與格式尚未取得（標記為 `unverified`，合成測試資料先行，等對接方提供樣本後再簽核契約）。
- FlapsErp 內部資料庫直接讀取（非 CSV 匯出）不在本計畫範圍；如需，另立 ADR 並進行逆向工程評估。
- 財務系統端的匯入契約（會計軟體名稱、欄位對應）尚未確認，以 SQLite ledger 為中繼，供財務人工或系統匯入。
- 退貨原因代碼允許集（瑕疵品／出錯貨／客戶改主意／運輸損壞／過期／其他）需與業務方確認。
