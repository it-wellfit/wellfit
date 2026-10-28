# Changelog

本專案之變更紀錄。格式參考 [Keep a Changelog](https://keepachangelog.com/zh-TW/1.1.0/)，
版本號遵循 [語意化版本](https://semver.org/lang/zh-TW/)。

> 對應之知識庫版本：`2026/Knowledge` v1.0（2026-10-20）。
> **所有數字皆可追溯**，追溯對照見 [`../Knowledge/traceability/claims-vs-evidence.md`](../../Knowledge/traceability/claims-vs-evidence.md)。

## [1.0.0] — 2026-10-20

### 規格驅動之 MTRIX 介接引擎

- **五檔 56 欄契約**：新增 `wellfit_mtrix::spec::MTRIX_SPEC`，涵蓋 ALTHCI（16）、ALTHES（6）、
  ALTHFV（10）、ALTHPD（12）、ALTHVD（12），共 **56** 欄。
  每欄的 `FieldSpec` 皆帶**來源層級**（`ManualL1`／`InternalL2`／`ObservedOnly`／`Unverified`）、
  **驗證狀態**（`confirmed`／`observed-only`／`deviation`／`pending`／`unverified`）、
  原文頁碼與對應議題編號；此四項為編譯期必填，無法省略。
- **欄位字典產生器**：新增 `wellfit spec --format markdown|json`，
  輸出即知識庫 `specs/mtrix-field-dictionary.md` 之內容（程式碼生成，不手改）。
- **56 欄逐欄對照表**：知識庫 `reconciliation/field-diff.md` 由
  `tools/reconciliation/build-field-diff.ps1` 自 `spec.rs` 與實際交付位元組產生；
  交付標題與 `spec.rs` 不符時腳本直接失敗。

### 驗證規則（VR-01..VR-14）

- 新增 `wellfit_mtrix::validators::VrRule`，共 **14** 條：
  - **VR-01..VR-12**：宣告自內部欄位字典（L2）。
  - **VR-13 水位對齊**（原始規格 p.18）、**VR-14 業務檔客戶存在性**（原始規格 p.34）：
    由 L1 原始規格衍生，L2 未宣告。
- 結果語意模型（`Outcome`）：`FileRejected`／`RowQuarantined`／`Repaired`／`Listed`／`NotEvaluable`。
- 嚴重度模型（`Severity`）：`Error`／`Warning`／`Info`。
- 批次狀態由結果彙總推導（`recompute_status`）：`accepted`／`partial-accepted`／`rejected`。

### 編號化修復規則（ANOM-01..ANOM-12）

- 新增 `wellfit_mtrix::repairs::REPAIR_RULES`，共 **12** 條，每條附**授權來源**與對應 VR 規則。
- 修復行為一律「針對性欄位」：條碼規則不作用於識別碼欄位、廠商前綴規則不作用於其他欄位。
- 修復明細（`before → after`）逐筆寫入帳本，可完整還原。

### 帳本與嚴重度模型

- `IdempotencyLedger` 新增 `severity` 與 `rule_code` 欄位，`validation_findings` 查詢，
  以及以 `Finding` 為輸入的 `log_quarantine_with_severity`。
- 批次去重維持「批次 ID + SHA-256」，修正後重送沿用同一批次 ID 即被辨識為重送。
- 隔離列保留**完整原值**。

### 欄位契約強制執行

- `wellfit_core::contract` 新增 `ColumnContract`／`ColumnMap`／`split_header`：
  欄位**以名稱解析**、大小寫不敏感、**無序位退讓**；必填欄缺漏即 `Err` 並列出所有缺漏欄名。
- 蝦皮與輔翼 ERP 各有 8 欄契約：`Shopee inbound order export`、`Fuyi ERP inventory report`。
- 契約於批次登記**之前**解析，被拒之檔案不消耗批次編號，發布閘門轉 `Quarantined`。

### 真實世界缺陷修正（CR-only 行尾）

- **對接方五個 MTRIX 檔案使用 CR（0x0D）行尾，LF 為 0**。
  Rust `str::lines()` 只認 LF／CRLF，會把整份檔案視為單一標題列，導致所有欄數檢查失敗
  （實測錯誤：`column count mismatch: expected 16, delivered 77`）。
- 新增 `wellfit_core::text::split_lines`，同時支援 LF／CRLF／CR 並移除 BOM；
  套用於 MTRIX、蝦皮、輔翼 ERP 三個引擎。
- 衍生 fixture 刻意保留 CR 行尾，避免缺陷復發時被 fixture 掩蓋。

### 真實世界缺陷修正（稽核可重跑、VR-05、報告與帳本一致）

- **`audit` 原本第二次執行必定失敗**：正式擷取的冪等規則是「同一份內容只能登錄一次」，
  而稽核是**驗證**、不是擷取，同一份檔案本來就該能被重複稽核。
  新增 `wellfit_core::ledger::BatchScope`：
  - `Exclusive`（正式擷取，行為不變）：內容雜湊即批次身分，同一內容第二次仍被拒。
  - `PerRun`（稽核）：批次身分含執行代次，同一內容可重複驗證。
  - `file_batches` 新增 `content_sha256` 欄位保存**真實內容雜湊**，
    因此被重複驗證的檔案仍留有可追溯的內容證據（舊資料庫以遷移回填）。
- **VR-05 從未產生任何 finding**：重複交易的彙總邏輯以 `grouped`（已去重）為基礎判斷重複，
  該集合內每個鍵都只出現一次，因此條件恆為 false。
  改為記錄**實際被併入既有交易**的鍵，每一筆彙總都產生一筆 `VR-05` Warning。
- **報告有、帳本沒有**：VR-01／VR-02／VR-04／VR-05／VR-10 的 finding 只寫進報告未寫進帳本，
  `status` 的警告數（1）與波次輸出的警告數（24）不一致。
  新增單一寫入點 `MtrixEngine::record_finding`，報告與帳本**同時寫入**；
  被隔離的列仍只寫隔離帳本，避免 `status` 重複計數。
  新增整合測試 `test_every_reported_finding_is_recorded_in_the_ledger` 守住此性質。
- **一行只報第一個違規**：列驗證遇到第一個違規即提前返回，
  同一列的「日期不合法」與「文件類型越界」只會看到其中一個，對接方需反覆重送才能發現其餘問題。
  改為**評估全部規則、隔離該列一次、回報每一個原因**。
- **`mutex` 逾時測試在 CPU 飽和時不穩**：第一把鎖僅給 1 秒取得預算，平行編譯時會取得失敗而非逾時；
  改為 30 秒預算，且測試用鎖檔改置於系統暫存目錄，避免失敗時在儲存庫留下鎖檔。
- 重複批次的錯誤訊息改為可行動的說明（提示改用新的 `--batch` 或新的 `--db`）。

### 協作工具修正（`build.ps1`）

- 暫存 `.bat` 的檔名帶有行程 ID，多個代理人同時呼叫建置腳本**不再互相覆寫對方的指令**。

### 定點十進位，零浮點

- 新增 `wellfit_core::scaled::ScaledDecimal`（`i64` + 4 位小數）。
- 蝦皮金額全程以 `i64` 分運算；匯入 CSV 的小數兩位以**整數除法 + 補零餘數**產生，不經浮點。
- 格式錯誤回傳 `None` 並隔離該列，**不當成 0**。

### CLI

- 六個子命令：`status`／`shopee`／`reports`／`mtrix`／`audit`／**`spec`**（新增）。
- `mtrix` 新增 `--output-dir`（修復輸出）、`--report`（JSON 報告）、`--strict`（有 error 時非零退出）。
- `reports` 新增 `--recipient`（可重複或以逗號分隔）。

### 測試

- **121 個測試全部通過**（2026-10-20 06:33 UTC 實測）：
  CLI 12、core 38 + 13（整合）、mtrix 37、reports 8、shopee 13。
- 新增 `crates/wellfit_core/tests/mdm_catalog.rs`（13 項），
  含「欄位以名稱解析、絕不依賴位置」與「條碼正規化保留前導零」之斷言。
- 新增 `crates/wellfit_cli/tests/integration_tests.rs`（11 項），
  以**真實交付位元組**（`spec/fixtures/mtrix/delivered/`）為基準。

---

## 破壞性變更（Breaking changes）

| # | 變更 | 影響 | 遷移方式 |
|---:|---|---|---|
| B-01 | **「58 欄全景清洗驗證」宣稱撤回** | 蝦皮介接事實由「58 欄已驗證」降級為「8 欄已驗證 + 50 欄具名缺口」 | 改用 `wellfit spec` 與 `specs/shopee-contract.md`；對外文件一併更新 |
| B-02 | **「27 項格式異常」宣稱撤回** | 改為 **12** 條編號化規則 ANOM-01..ANOM-12 | 引用 `specs/mtrix-anomaly-catalogue.md` |
| B-03 | **「VR-01~VR-08」規則集更正** | 實際為 **VR-01..VR-14** | 引用 `specs/mtrix-vr-rules.md` |
| B-04 | **`contracts/federation/` 目錄不存在** | 聯邦契約實作為模組 `crates/wellfit_core/src/federation.rs`（`wellfit_core::federation`） | 改引用模組路徑 |
| B-05 | **四個 `file:///c:/Users/User/OneDrive/桌面/WEB3/project/Wellfit/...` 連結失效** | 指向他機 | 改用倉庫相對路徑 `crates/<crate>/` |
| B-06 | **`reports --input data/fuyi_inventory.csv` 範例失效** | 檔名不存在 | 改為 `data/fuyi_inventory_report.csv` |
| B-07 | **契約缺欄改為回傳 `Err`** | `process_raw_orders`／`parse_native_csv_report` 在缺少必填欄時回傳錯誤字串（簽章不變） | 呼叫端既有 `map_err` 可直接顯示；需確認錯誤訊息之呈現 |
| B-08 | **金額格式錯誤不再當成 0** | 新增隔離碼 `ERR_MONEY_FORMAT`／`ERR_QTY_FORMAT`／`ERR_INV_NUMBER_FORMAT` | 依隔離碼分流處理 |
| B-09 | **`MdmHub::normalize_vendor_id` 移除** | 舊函式會強制加上來源未述之 `USA` 前綴 | 改用 `split_vendor_country_prefix`／`join_vendor_country_prefix` |
| B-10 | **`MdmCatalog::with_bootstrap_defaults` 改名** | 改為 `demo_bootstrap()`（無外部呼叫者） | 更新呼叫端 |
| B-11 | **`MdmHub::normalize_barcode` 改為無損** | 僅修剪空白與尾端 `;`，不再去除非數字字元 | 需嚴格檢查時改用 `validate_barcode`／`is_ean13`／`is_dun14` |
| B-12 | **`data/` 內早期合成樣本移除** | `data/raw_samples/*`、`data/ALTH*.csv` 已刪除；fixture 改置於 `spec/fixtures/mtrix/` | 引用新路徑；引用 `data/` 者須改標為**合成示範資料** |

---

## 已知缺口

| # | 缺口 | 狀態 |
|---|---|---|
| G-01 | 蝦皮真實匯出欄位表頭未取得（OI-003）；58 列候選 Mapping 中 47 欄 `OPEN`、11 欄 `REC` | `open` |
| G-02 | 輔翼 ERP 匯入範本與欄位字典未取得（OI-006）；來源文件已抽取但為技術研究方案，ERP 端未知項列為待填清單 | `open` |
| G-03 | MTRIX 逐欄必填清單不可得（MTRIX-003／ISS-013） | `pending-counterparty` |
| G-04 | 內部欄位字典 `.xlsx` 無法抽取，VR-01..VR-12 之原始條文不可引用 | `open` |
| G-05 | 售價合理性檢查未實作（無市場平均價基準） | `pending-counterparty` |
| G-06 | 計量單位一致性未驗證（五檔無單位欄） | `pending-counterparty` |
| G-07 | VR-12 僅部分實作：已產出幣別分組彙總，與管理報表之總額核對未實作 | `open` |
| G-08 | 傳輸協定、認證、重送策略未定義；現行為目錄投放 | `pending-counterparty` |
| G-09 | 幣別辨識方式未確認；金額依 `COUNTRY_ISO` 分組，不做換算 | `open` |
| G-10 | 8 處標題偏差已接受為現況（LOCAL-001..009） | `accepted-as-is` |
| G-11 | `VENDOR_ID` 國碼前綴長度不一致（原文 2 字母、交付 3 字母；廠商碼 16 位 vs 宣告 14） | `pending-counterparty` |
| G-12 | 正式 UAT 測試資料、允收標準與簽核人未提供（OI-018） | `open` |

完整清單（87 項）見 [`../Knowledge/issues/mtx-issue-register.md`](../../Knowledge/issues/mtx-issue-register.md)。

---

[1.0.0]: ../Knowledge/adr/ADR-0008-work-continuation-model.md
