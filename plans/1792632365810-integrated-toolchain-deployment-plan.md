# Integrated Toolchain & System Deployment Plan

> 文件編號：KNB-DEP-001｜狀態：Draft｜版本：v1.0｜日期：2026-10-23
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 適用範圍：2026/ 全專案（哈瓦士串接、蝦皮平台資料處理與 ERP 資料匯入自動化、RPA 導入作業流程、國際標準稽核、Dev/Wellfit、Knowledge 合規子樹）
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

---

## 0. 目標與定義（Done）

全部五個領域同時達到「全部綠燈」狀態：

| 領域 | 定義完成（Done）的可驗證標準 |
|------|---------------------------|
| **Rust 工具鏈** | `build.ps1 -Command "build --workspace"` 綠燈；`wellfit.exe` 六個子命令（status/shopee/reports/mtrix/audit/spec）皆可執行並產出預期輸出；target 目錄導向 `%LOCALAPPDATA%\Temp\wellfit-target`（不寫入桌面路徑） |
| **合規工具鏈** | `verify-compliance.ps1 -Strict` → FAIL=0 / WARN=0 / PASS>0；全部 9 支 .ps1 腳本在 PowerShell 5.1 下執行無誤；CSV 全具 UTF-8 BOM、欄數一致、主鍵唯一；XLSX 匯出可由 Excel 開啟且中文正常 |
| **規格抽取管線** | `wellfit spec` 由 `sources/snapshots/` 快照決定性產出 `specs/mtrix-field-dictionary.md` 與 `reconciliation/field-diff.md`；56 欄全對齊、無漂移；產生器在資料不一致時直接失敗 |
| **ISMS 控制實作** | 93 項 Annex A 控制皆有 SoA 狀態（四維不跳級）與證據規劃；35 條流程皆關聯控制編號；權限登錄滿版（E-1=100%）；`verify-compliance.ps1` 對 `compliance/iso-27001/` 綠燈 |
| **整合部署** | `build.ps1` 佈建合規驗證關卡；CI 同時綠燈於 Rust 測試與合規驗證；規格抽取結果作為 ISMS 證據來源；無平行檔案、無雙軌狀態 |

---

## 1. 決策紀錄（D-系列，不可重開）

| 編號 | 決策 | 內容 | 影響領域 |
|------|------|------|----------|
| D-1 | 範圍 | 所有五個領域整合部署：Rust 工具鏈、合規工具鏈、規格抽取管線、ISMS 控制實作、整合部署 | 全部 |
| D-2 | 目標狀態 | 每個領域獨立「全部綠燈」標準，見 §0 | 全部 |
| D-3 | 滾動順序 | P0 Rust 工具鏈 → P1 合規工具鏈（可與 P0 並行）→ Dry-run 合規基線 → P2 規格抽取 → P3 ISMS 控制 → P4 整合部署 | 全部 |
| D-4 | 依賴關係 | P2 依賴 P0（`wellfit spec` 子命令）；P3 依賴 P1（合規驗證）與 P2（證據產出）；P4 依賴 P0–P3 全部 | 全部 |
| D-5 | 乾跑階段 | P1 後、P3 前，對現有 templates/ 執行 `verify-compliance.ps1`，建立 FAIL=0/WARN=0 基線，再填入 ISMS 控制內容 | 合規工具鏈、ISMS 控制 |
| D-6 | 權威分工 | ADR-0012：關卡/角色/WBS 以 IMS 套件為權威；控制編號/欄位/值域/證據以合規套件為權威；不得建立平行複本 | 合規工具鏈、ISMS 控制 |
| D-7 | 值域資料化 | ADR-0012 §7：值域只存在於 `tools/compliance/vocab/*.csv`；腳本主體 ASCII only；值域變更不修改腳本 | 合規工具鏈 |
| D-8 | 檔案邊界 | ADR-0008：工作流依檔案邊界分工，互不重疊；跨工作流依賴走「API 凍結＋產物介面」 | 全部 |
| D-9 | 建置入口 | AGENTS.md §3：所有建置/測試/執行必須透過 `tools/dev-env/build.ps1`；嚴禁直接 `cargo build` 於桌面路徑 | Rust 工具鏈 |
| D-10 | 不變量 | AGENTS.md §4：零 unsafe、零 clippy 警告、零浮點金額、欄位以名稱解析、合成資料不得呈現為真實、不可追溯之數字不得出現 | Rust 工具鏈 |

---

## 2. 依賴關係圖

```
P0: Rust 工具鏈 ──┐
                  ├──→ P2: 規格抽取管線 ──┐
P1: 合規工具鏈 ──┘                        │
                   Dry-run: 合規基線       │
                  ──┐                     │
P3: ISMS 控制實作 ──┘→ P4: 整合部署 ◄──────┘
                    (依賴 P1 驗證 + P2 證據)
```

- **P0 → P2**：`wellfit spec` 子命令位於 `crates/wellfit_cli`，必須先能建置並執行。
- **P1 → P3**：ISMS 控制 CSV（`templates/*.csv`）必須經 `verify-compliance.ps1 -Strict` 通過才算完成。
- **P2 → P3**：規格抽取產出（field-dictionary、field-diff）作為 ISMS 證據層（E3）的來源。
- **P4**：所有領域綠燈後，將合規驗證併入 `build.ps1` CI 關卡。

---

## 3. 階段任務清單（順序不可交換，除非標註獨立）

### P0：Rust 工具鏈（獨立，可與 P1 並行）

| # | 任務 | 目標檔案/產物 | 驗收 |
|---|------|--------------|------|
| P0-01 | 確認 `build.ps1` 可在 PowerShell 5.1 下設定 MSVC 工具鏈並載入 `vcvars64.bat` | `Knowledge/tools/dev-env/build.ps1` | `build --workspace` 綠燈 |
| P0-02 | 建置全部 5 crate（`wellfit_core`, `wellfit_shopee`, `wellfit_reports`, `wellfit_mtrix`, `wellfit_cli`） | `crates/` | `wellfit.exe` 產出於 `%LOCALAPPDATA%\Temp\wellfit-target` |
| P0-03 | 執行 `clippy --workspace --all-targets -- -D warnings` 零警告 | `crates/*.rs` | 0 警告 |
| P0-04 | 執行 `test --workspace --no-fail-fast` 全綠（含真實 fixture 期望值） | `crates/*/tests/` | 0 失敗 |
| P0-05 | 驗證 `wellfit.exe` 六個子命令：status / shopee / reports / mtrix / audit / spec | `wellfit.exe` | 各子命令可執行並產出預期輸出 |
| P0-06 | 確認 target 目錄不寫入桌面路徑（繞開應用程式控制原則） | `%LOCALAPPDATA%\Temp\wellfit-target` | 建置產物不在 `Dev/` 下 |

### P1：合規工具鏈（獨立，可與 P0 並行）

| # | 任務 | 目標檔案/產物 | 驗收 |
|---|------|--------------|------|
| P1-01 | 確認全部 9 支 `.ps1` 在 PowerShell 5.1 下執行，腳本主體 ASCII only | `tools/compliance/*.ps1` | 0 執行錯誤 |
| P1-02 | 確認全部 11 個 vocab CSV 具 UTF-8 BOM、欄數一致 | `tools/compliance/vocab/*.csv` | `Import-Csv` 無誤 |
| P1-03 | 確認全部 23 份 template CSV + 23 份 schema md 欄位契約一致 | `compliance/iso-27001/templates/` | 欄數一致、主鍵唯一 |
| P1-04 | 確認全部 19 份 form CSV | `compliance/iso-27001/forms/f-01~f-19.csv` | 欄數一致、主鍵唯一 |
| P1-05 | `verify-compliance.ps1 -Strict` 綠燈（現有 templates/ 基線） | `compliance/iso-27001/` | FAIL=0 / WARN=0 / PASS>0 |
| P1-06 | `build-compliance-xlsx.ps1` 產出可由 Excel 開啟且中文正常的 XLSX | `tools/compliance/build-compliance-xlsx.ps1` | XLSX 存在且可開啟 |
| P1-07 | 新文件骨架：`new-isms-artifact.ps1 -Path -Number -Title` 正確產出文件管制標頭 | `tools/compliance/new-isms-artifact.ps1` | 檔頭含文件編號/狀態/版本/日期 |

### Dry-run：合規基線（P1 後、P3 前）

| # | 任務 | 目標檔案/產物 | 驗收 |
|---|------|--------------|------|
| Dry-01 | 對現有 `templates/`（未填入 ISMS 控制內容前）執行 `verify-compliance.ps1 -Strict` | `compliance/iso-27001/templates/` | FAIL=0 / WARN=0 |
| Dry-02 | 記錄乾跑報告至 `verification/` | `verification/verify-report-*.md` | 報告存在，FAIL=0 |
| Dry-03 | 任何模板缺陷（欄位不一致、主鍵重複、值域不在 vocab）先行修復 | `templates/*-schema.md` | V-02 全過 |

### P2：規格抽取管線（依賴 P0）

| # | 任務 | 目標檔案/產物 | 驗收 |
|---|------|--------------|------|
| P2-01 | `wellfit spec` 由 `sources/snapshots/<doc-id>/text.md` 產出 `specs/mtrix-field-dictionary.md` | `specs/mtrix-field-dictionary.md` | 56 欄全對齊 |
| P2-02 | `tools/reconciliation/build-field-diff.ps1` 產出 `reconciliation/field-diff.md` | `reconciliation/field-diff.md` | 56 欄 × 四來源對齊，無衝突 |
| P2-03 | 規格抽取在資料不一致時直接失敗（不產靜默錯誤） | `tools/spec-extract/` | 不一致 → 腳本失敗 |
| P2-04 | 產出含方法、雜湊、時間戳（可重現） | `specs/*.md` | 每份檔含產生方式與時間戳 |

### P3：ISMS 控制實作（依賴 P1 驗證、P2 證據）

| # | 任務 | 目標檔案/產物 | 驗收 |
|---|------|--------------|------|
| P3-01 | 93 項 Annex A 控制 SoA 四維狀態（設計/實施/運作/證據，不跳級） | `templates/soa-93-controls.csv` | V-04（93/93）、四維值域合法 |
| P3-02 | 35 條流程 × 控制 × 簽核矩陣，既有 18 個 process_id 逐字保留 | `templates/process-control-signoff-matrix.csv` | V-18（35 條、既有 18 ID 保留） |
| P3-03 | 權限登錄滿版（申請→核准→授予→複審→撤銷五階段） | `templates/access-register.csv` | E-1 = 100% |
| P3-04 | 自動化身分四元關係（執行人↔排程↔目標系統↔憑證） | `templates/automation-access-log.csv` | V-24/V-31 |
| P3-05 | 三波時序權限快照（08:30 蝦皮／10:00 庫存／17:00 MTRIX） | `templates/access-snapshot-log.csv` | V-32 |
| P3-06 | 風險評估與 SoA 判定（適用/不適用/待確認，排除項具四要素） | `templates/risk-register.csv` | V-09、V-06/V-07 |
| P3-07 | 證據索引（SHA-256 雜湊、唯讀、不回寫來源） | `templates/evidence-index.csv` | V-10 |
| P3-08 | 內部稽核計畫與不符合項登錄 | `templates/internal-audit-program.csv`、`templates/nonconformity-register.csv` | V-17–V-20 |
| P3-09 | `verify-compliance.ps1 -Strict` 對 `compliance/iso-27001/` 綠燈 | `compliance/iso-27001/` | FAIL=0 / WARN=0 / PASS>0 |
| P3-10 | 規格抽取結果作為 ISMS 證據層（E3）來源，寫入證據索引 | `evidence-index.csv` 引用 `specs/`、`reconciliation/` | 證據 ID 可解析 |

### P4：整合部署（依賴 P0–P3 全部）

| # | 任務 | 目標檔案/產物 | 驗收 |
|---|------|--------------|------|
| P4-01 | `build.ps1` 佈建合規驗證關卡：Rust 測試綠燈 ＋ 合規驗證綠燈 | `tools/dev-env/build.ps1` | 雙關卡皆綠燈才放行 |
| P4-02 | CI 同時驗證 Rust 工作區與合規套件 | `build.ps1` + `verify-compliance.ps1` | 兩者退出碼 0 |
| P4-03 | 規格抽取結果納入 ISMS 證據索引，作為控制實作證據 | `evidence-index.csv` | `evidence_id` 指向 `specs/` 與 `reconciliation/` |
| P4-04 | 全量回歸：執行 P0–P3 全部驗收，無衝突 | 全部產物 | 全部綠燈 |
| P4-05 | 文件管制清單更新（KNB-CMP-001～010、ADR-0010～012、IMS 受控文件識別碼） | `document-control-list.csv` | 編號無重複、狀態可追溯 |

---

## 4. 驗證關卡（G-系列，與 IMS 對齊）

| 關卡 | 名稱 | 放行判準 | 對應階段 |
|------|------|----------|----------|
| G0 | 動員 | 章程與決策權核准；核心角色任命；文件/證據/存取/分類路徑建立 | P0–P1 開始前 |
| G1 | 適用性與範圍決定 | 每候選項有狀態、權威來源、事實需求與判定理由；範圍有業務擁有人驗證 | P0–P1 完成後 |
| G2 | 治理與基線 | 範圍與風險方法核准；每實質要求有狀態與來源；Current Profile 含證據或標記未評估 | P3-06 風險評估完成後 |
| G3 | 風險處置與設計基線 | SoA 與授權標準一致；重大風險已處置或授權接受；優先控制有擁有人、運作、證據、例外、測試準則 | P3-01〜P3-08 完成後 |
| G4 | 優先控制實施 | 選定優先控制於既定範圍已實施；操作者可示範流程；例外有權限與到期；控制狀態不跳級 | P3-09 驗證通過後 |
| G5 | 第一波測試 | 至少完成一個文件化測試週期；目標／準則／期間／母體／抽樣理由／結果／覆核者記錄 | P4-04 回歸測試 |
| G6 | 內部稽核與管理評審 | 稽核以證據為基礎且獨立；發現可追溯至準則；管理評審形成具名決議 | P4 整合後 |
| G7 | 維運移交與收尾 | 維運擁有人接受控制、監控、證據覆核、例外、事件、更新與資源；無高風險被隱藏 | P4-05 文件管制清單完成後 |

---

## 5. 失敗模式與回滾

| 失敗模式 | 徵兆 | 回滾方式 |
|----------|------|----------|
| Rust 建置失敗（clippy 警告、測試紅燈） | `build.ps1` 退出碼非 0 | 回復至上一個綠燈 commit；target 目錄可安全刪除重建 |
| 合規驗證 FAIL>0 | `verify-compliance.ps1` 報告 FAIL>0 | 修復 templates/ 或 vocab/；不修改腳本主體（值域只動 vocab） |
| 規格抽取漂移 | field-diff 與 source-register 不一致 | 重跑抽取管線；快照不可變（只增不改） |
| ISMS 控制與實際系統不一致 | 權限登錄與系統匯出不符 | 每月系統匯出比對登錄（E-1）；差異列為不符合項 |
| 平行檔案衝突 | 同一檔案由兩工作流修改 | 依 ADR-0008：同一檔案不得由兩個工作流同時修改；先 `ls` 比對目錄 |

---

## 6. 開放決策（需專案負責人裁示）

| # | 決策事項 | 建議答案 | 責任角色 |
|---|----------|----------|----------|
| O-01 | Q1–Q5 待確認事項（組織法人、據點數、系統清單、AI 使用、客戶/監管要求）何時填入實值 | Phase 1 結束前填入，標示來源 | 執行贊助人／最高管理層 |
| O-02 | 認證目標時程（Stage 1／Stage 2） | 先不對外承諾；G-3 確定後再對外 | 執行贊助人 |
| O-03 | PIPL 適用性（蝦皮訂單收件人個資） | 法遵/外部顧問意見後再判定 | 法務／法遵／隱私職能 |
| O-04 | 汽車客戶（Alpargatas）TISAX/ISA 評估時程 | ISA2027 適用於 2027-01-01 起訂購之評估；舊版首次評估最後期限為 3 月 | 法務／採購 |

---

## 7. 下一動作

此計畫已達 implementation-ready。執行代理應：

1. 先讀 `Knowledge/adr/ADR-0008-work-continuation-model.md` 了解工作延續模型。
2. 從 P0-01 開始依序完成，P0 與 P1 可並行。
3. 在 P3 前執行 Dry-01 乾跑，建立合規基線。
4. 在 P4-04 全量回歸綠燈後，向使用者回報各領域驗收結果。
