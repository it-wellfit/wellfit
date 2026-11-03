# UI/UX Enhancement Plan — Toolkits, Toolchains & Templates

> 文件編號：KNB-UX-001｜狀態：Draft — 未核准、非執行計畫｜版本：v1.0｜日期：2026-10-24
> 撰寫：Wellfit 知識庫維護者｜審閱：未審閱｜核准：未核准
> 適用範圍：原草案記載 Dev/Wellfit（5-crate Rust 工作區）及 Knowledge 路徑；目前 crate 基線、文件所有權與需求均未重驗，請先依 `../PLAN.md` 拆分或裁定。
> 驗證標記：✔ 已對原始文件逐字查證｜△ 依標準條文或通行做法推導｜未驗證 本次無法確認

---

## 0. 目標（Done 標準）

全部五個領域同時達到「友善可用」標準：

| 領域 | 完成標準 |
|------|----------|
| **Rust CLI** | 色彩輸出（ANSI，尊重 NO_COLOR）、`--verbose`/`--quiet`/`--no-color`、`spec --format html` 產出自含 CSS 的零 JS 網頁、長操作有進度條、錯誤訊息帶補救建議、shell 装殼（powershell/bash/zsh） |
| **合規 PowerShell** | 色彩 console 輸出、`Write-Progress` 進度條、`-Verbose`/`-Quiet`/`-NoColor`、`-Interactive` 模式與確認提示、console 摘要表、HTML 報告選項 |
| **範本與表單** | 範本驗證器（BOM/編碼/欄數/主鍵/值域）有友善錯誤訊息與自動修復選項、CSV 檢查器可修正常見錯誤 |
| **build.ps1** | 色彩輸出、建置階段進度、更佳錯誤訊息 |

---

## 1. 設計決策（D-系列，不可重開）

| 編號 | 決策 | 內容 | 影響領域 |
|------|------|------|----------|
| D-1 | 顏色策略 | ANSI 色碼，預設開啟；`--no-color` / `NO_COLOR` 環境變數關閉；PowerShell 以 `Write-Host -ForegroundColor` 實作 | Rust CLI、PowerShell |
| D-2 | HTML 零 JS | `spec --format html` 產出自含內嵌 CSS 的單一 HTML 檔；搜尋／篩選使用 CSS `:target` 與 `<details>`/`<summary>`，**絕對不含 `<script>`**（符合 GEMINI.md §2.3 JavaScript-Free 不變量） | Rust CLI (`spec`) |
| D-3 | 混合模式 | 合規工具預設 batch 模式（CI 友善）；`-Interactive` 開啟確認提示與逐步導覽；`--verbose` / `--quiet` / `--nocolor` | PowerShell |
| D-4 | 相容性 | PowerShell 5.1，.ps1 主體 ASCII only，非 ASCII 內容置於 UTF-8 資料檔；Rust 零 `unsafe`、零 clippy 警告、零浮點金額 | 全部 |
| D-5 | 檔案邊界 | ADR-0008：工作流依檔案邊界分工，互不重疊；Rust crate 與 PowerShell 腳本不混修改同一檔案 | 全部 |
| D-6 | 向下相容 | 所有新增選項皆為可選，預設行為不變；現有腳本、Makefile、CI 指令無需改動 | 全部 |

---

## 2. 依賴關係圖

```
Rust CLI 彩色輸出 ──┐
                     ├──→ spec --format html (零 JS 網頁)
進度條 ──┐           │
         │           └──→ shell 裝殼 (powershell/bash/zsh)
PowerShell 彩色 ──┘
         │
         ├──→ -Verbose/-Quiet/-NoColor
         ├──→ -Interactive 模式
         └──→ HTML 報告選項
範本驗證器 ──┐
             ├──→ BOM/編碼自動修復
             └──→ 值域錯誤補救建議
build.ps1 ──→ 色彩 + 進度階段
```

---

## 3. 階段任務清單（順序不可交換，除非標註獨立）

### T1：Rust CLI 基礎 UX（獨立）

| # | 任務 | 目標檔案 | 驗收 |
|---|------|----------|------|
| T1-01 | 新增全域 `--verbose` / `--quiet` / `--no-color` 旗標，並傳遞至各子命令 | `crates/wellfit_cli/src/main.rs` | `wellfit --help` 顯示新旗標；`-q` 只輸出錯誤；`-v` 輸出詳細步驟 |
| T1-02 | 彩色 console 輸出（ANSI 色碼），尊重 `NO_COLOR` 與 `--no-color` | `crates/wellfit_cli/src/main.rs` + 各子命令輸出函式 | 終端顯示色彩；`NO_COLOR=1` 或 `--no-color` 時無 ANSI 碼 |
| T1-03 | 錯誤訊息加上補救建議（例如：缺欄位時列出預期欄名與實際標題） | `crates/wellfit_cli/src/main.rs` | 錯誤訊息包含「建議動作」 |
| T1-04 | `mtrix`、`audit` 長操作顯示進度條（已處理檔／總檔） | `crates/wellfit_cli/src/main.rs` | 處理中顯示 `[████---] 60%` |
| T1-05 | `spec --format html` 產出自含內嵌 CSS 的單一 HTML 網頁，零 `<script>` | `crates/wellfit_cli/src/main.rs` | HTML 可在瀏覽器開啟；欄位可搜尋（CSS `:target`）；規則可展開（`<details>`） |
| T1-06 | 新增 shell 装殼腳本（powershell、`_wellfit.bash`、`_wellfit.zsh`） | `crates/wellfit_cli/completions/` | `wellfit` 指令可 Tab 補全 |

### T2：合規 PowerShell 工具鏈 UX（獨立）

| # | 任務 | 目標檔案 | 驗收 |
|---|------|----------|------|
| T2-01 | `verify-compliance.ps1` 加入 `-Verbose` / `-Quiet` / `-NoColor` 開關，console 輸出色彩化 | `tools/compliance/verify-compliance.ps1` | `-q` 只顯示 FAIL 摘要；`-v` 顯示每個檢查；`--NoColor` 無 ANSI |
| T2-02 | `verify-compliance.ps1` 加入 `-ReportFormat html`，產出 CSS 樣式的 HTML 報告 | `tools/compliance/verify-compliance.ps1` | HTML 報告可在瀏覽器開啟；FAIL 行以紅色標示 |
| T2-03 | `verify-compliance.ps1` 加入 `-Interactive` 開關，在發現 FAIL 時詢問是否繼續 | `tools/compliance/verify-compliance.ps1` | 互動模式下 FAIL 時暫停並等待 `Y/N` |
| T2-04 | `verify-compliance.ps1` 在 console 顯示摘要表（PASS/WARN/FAIL 計數、失敗清單） | `tools/compliance/verify-compliance.ps1` | 執行後 console 有色彩摘要表 |
| T2-05 | `new-isms-artifact.ps1` 加入 `-Interactive` 逐步導覽（詢問 Path/Number/Title/Category） | `tools/compliance/new-isms-artifact.ps1` | 無參數且 `-Interactive` 時逐項詢問 |
| T2-06 | 其他 7 支合規 `.ps1` 統一加入 `-Verbose` / `-Quiet` / `-NoColor` 與色彩輸出 | `tools/compliance/*.ps1` | 所有腳本 console 輸出一致風格 |

### T3：範本與表單驗證器 UX

| # | 任務 | 目標檔案 | 驗收 |
|---|------|----------|------|
| T3-01 | 建立 `tools/compliance/validate-templates.ps1`：檢查 BOM、編碼、欄數、主鍵唯一、值域在 vocab 內 | `tools/compliance/validate-templates.ps1` | 錯誤訊息指明檔案、列號、欄位名與建議修復 |
| T3-02 | `validate-templates.ps1` 加入 `-Fix` 自動修復常見錯誤（缺 BOM、錯誤編碼、空白標題） | `tools/compliance/validate-templates.ps1` | `-Fix` 後重跑驗證為綠燈 |
| T3-03 | 建立 `tools/compliance/lint-csv.ps1`：CSV 語法檢查，逗號／分號分隔符偵測，中文全形逗號警告 | `tools/compliance/lint-csv.ps1` | 輸出每個錯誤的檔案、行號、欄位與建議 |

### T4：build.ps1 UX

| # | 任務 | 目標檔案 | 驗收 |
|---|------|----------|------|
| T4-01 | `build.ps1` 加入色彩輸出與建置階段進度（編譯中／連結中／測試中） | `tools/dev-env/build.ps1` | console 顯示色彩化階段標示 |
| T4-02 | `build.ps1` 錯誤訊息加上補救建議（例如：vcvars64.bat 不存在時指引安裝 Visual Studio Build Tools） | `tools/dev-env/build.ps1` | 錯誤包含「建議動作」 |

### T5：整合驗證（依賴 T1–T4 全部）

| # | 任務 | 目標檔案 | 驗收 |
|---|------|----------|------|
| T5-01 | `build.ps1 -Command "test --workspace --no-fail-fast"` 全綠，零 clippy 警告 | `crates/*.rs` | 121 個測試通過，0 警告 |
| T5-02 | `verify-compliance.ps1 -Strict` FAIL=0 / WARN=0 / PASS>0 | `compliance/iso-27001/` | 驗證綠燈 |
| T5-03 | `validate-templates.ps1` 對全部 templates/ 與 forms/ 綠燈 | `templates/*.csv`、`forms/*.csv` | 0 錯誤 |
| T5-04 | `wellfit spec --format html` 產出可在瀏覽器開啟的 HTML，零 `<script>` | `wellfit spec` | HTML 檔存在，不含 `<script>` |
| T5-05 | shell 裝殼在 powershell / bash / zsh 下可補全 `wellfit` 子命令與旗標 | `completions/` | Tab 補全正常運作 |

---

## 4. 驗證關卡（G-系列）

| 關卡 | 名稱 | 放行判準 |
|------|------|----------|
| G-UX-1 | Rust CLI 彩色與進度 | `wellfit --help` 顯示新旗標；`--no-color` 無 ANSI；`mtrix` 有進度條 |
| G-UX-2 | spec HTML 零 JS | `spec --format html` 產出 HTML；檢查不含 `<script>`；CSS 內嵌 |
| G-UX-3 | PowerShell 色彩與互動 | `verify-compliance.ps1 -Verbose` 有色彩；`-Interactive` 詢問確認；`-NoColor` 無色碼 |
| G-UX-4 | 範本驗證器 | `validate-templates.ps1` 對所有 CSV 綠燈；`-Fix` 能修復缺 BOM |
| G-UX-5 | build.ps1 色彩 | `build.ps1` console 顯示色彩化階段 |
| G-UX-6 | 零退化 | `cargo test --workspace --no-fail-fast` 全綠；`verify-compliance.ps1 -Strict` FAIL=0 |

---

## 5. 失敗模式與回滾

| 失敗模式 | 徵兆 | 回滾方式 |
|----------|------|----------|
| ANSI 色碼在非支援終端顯示雜亂 | console 輸出含 `ESC[` 碼 | 預設關閉色彩，僅在偵測到支援終端時開啟；`--no-color` 強制關閉 |
| HTML 報告 CSS 損毀 | 瀏覽器顯示純文字 | CSS 內嵌於 `<style>` 區塊；檢查 HTML 可開啟 |
| PowerShell 色彩在舊主控台失效 | `Write-Host -ForegroundColor` 無效 | 偵測 `$host.Name`，非支援時退化為純文字 |
| 進度條干擾管線輸出 | 管線重導向時進度條混雜 | 偵測 `[Console]::IsOutputRedirected`，重導向時關閉進度條 |
| shell 裝殼腳本路徑錯誤 | Tab 補全不運作 | 裝殼腳本置於 `completions/`，使用者自行 source |

---

## 6. 開放決策（需專案負責人裁示）

| # | 決策事項 | 建議答案 | 責任角色 |
|---|----------|----------|----------|
| O-01 | Rust 彩色輸出使用外部 crate（`console`、`console-term`）還是手寫 ANSI | 手寫 ANSI（避免新增依賴，保持零外部 DLL 原則） | 架構師 |
| O-02 | HTML 報告內嵌 CSS 還是外部 .css 檔 | 內嵌 CSS（單一檔案，零外部依賴） | 架構師 |
| O-03 | shell 裝殼腳本由 crates/wellfit_cli/completions/ 產出還是手寫 | 手寫（穩定，不需建置步驟） | 開發者 |

---

## 7. 下一動作

此計畫已達 implementation-ready。執行代理應：

1. 先讀 `Dev/Wellfit/AGENTS.md` §4 不變量與 §3 建置入口。
2. 從 T1-01 開始依序完成；T1 與 T2 可並行（不同檔案邊界，符合 ADR-0008）。
3. 在 T5 前執行全部 G-UX 關卡驗證。
4. 在 T5-06 全部綠燈後，向使用者回報各領域驗收結果。
