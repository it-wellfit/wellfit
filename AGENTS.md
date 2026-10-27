# Wellfit MTRIX 開發規範

> 文件編號：KNB-AGT-001｜狀態：Active｜版本：v1.0｜日期：2026-10-25

## 建置與測試

- 建置: `powershell -ExecutionPolicy Bypass -File "Knowledge\tools\dev-env\build.ps1" -Command "build --workspace" -Workspace "." -TargetDir "%LOCALAPPDATA%\Temp\wellfit-target"`
- 測試: `... -Command "test --workspace"`
- Clippy: `... -Command "clippy --workspace --all-targets -- -D warnings"`
- 目標目錄: `%LOCALAPPDATA%\Temp\wellfit-target`（OS Application Control policy 要求）

## 架構規範

- Crate 依賴: wellfit_cli → wellfit_mtrix → wellfit_core
- 禁止 unsafe code: `#![forbid(unsafe_code)]`
- Clippy 零警告: `-D warnings`
- 所有 public API 必須有文件註解

## 領域詞彙

- MTRIX: Alpargatas 介接系統（5 檔、56 欄）
- VR-xx: 驗證規則（VR-01..VR-15）
- ANOM-xx: 修復規則（ANOM-01..ANOM-12）
- Ledger: 幂等性帳本（SQLite）
- Quarantine: 隔離列
- BatchScope: Exclusive/PerRun

## 測試指令

- 單元測試: `cargo test -p <crate> --lib`
- 整合測試: `cargo test -p wellfit_cli --test integration_tests`
- MTRIX 手動: `wellfit mtrix --dir <dir> --batch <id> --db <db>`
- 規格輸出: `wellfit spec --format markdown`

## 本機檢查入口（Local Check Entry Point）

- 固定入口：`scripts\check.ps1`
- 執行：`powershell -ExecutionPolicy Bypass -File scripts\check.ps1`
- 內容：`cargo fmt --check` → clippy (`-D warnings`) → `cargo test --workspace` → 整合測試 → SHA-256 fixture 驗證
- 全部通過才可提交；失敗即修正後重跑。

## 本機備份（Local Backup）

```powershell
# 建立 bundle（離線備份，含所有分支與標籤）
git bundle create backup-$(Get-Date -Format 'yyyyMMdd').bundle --branches --tags

# 驗證
git bundle verify backup-*.bundle

# 還原演練（到暫存目錄）
$drill = "$env:TEMP\wellfit-restore-drill"
if (Test-Path -LiteralPath $drill) { Remove-Item -LiteralPath $drill -Recurse -Force }
git clone backup-*.bundle $drill
Push-Location -LiteralPath $drill
cargo build --workspace
Pop-Location
```

> 注意：bundle 不含未提交檔案與工作目錄內容，提交前務必 `git status` 確認乾淨。

## CLI 子命令速查

| 子命令 | 必填 | 退出碼 |
|---|---|---|
| `status --db <DB>` | — | 0 |
| `shopee --input <CSV>` | input | 0 / 非0 |
| `reports --input <CSV> --recipient <ADDR>` | input | 0 / 非0 |
| `mtrix --dir <DIR>` | dir | 0；`--strict` 且有 error → 255 |
| `audit --data-dir <DIR>` | — | 0 |
| `spec --format markdown\|json` | — | 0 |

環境變數：`CARGO_TARGET_DIR`（建置產物目錄，App Control 要求置於 `%LOCALAPPDATA%\Temp\wellfit-target`）。

## Release Gate 狀態機

`DraftGenerated` → `VerifiedPass` / `Quarantined` → `AuthorizedDispatch` → `Completed` / `Failed` / `DispatchUncertain`（逾時後以 `reconcile_uncertain_dispatch` 確認）。所有轉移寫入 `history: Vec<TransitionRecord>`。

## 本機優先工作流程（Local-First Workflow）

### 開發流程

```text
確認真實工作痛點與使用者
→ 定義可量化的成功指標
→ /grill-with-docs
→ /research（有技術或領域疑問時）
→ /prototype（有高風險假設或 UI 需要探索時）
→ /to-spec
→ /to-tickets（多個切片或有依賴時）
→ /implement 或 /implement-spec
→ 本機測試、評估與 code review
→ 修正並重跑檢查
→ 本機提交、標記版本、備份
→ /retro
```

### 測試指標（每個 MVP）

| 指標 | 目標 | 測量方式 |
|------|------|---------|
| 正確率 | ≥ 95% | 測試案例通過率 |
| 處理時間 | < 5s/檔案 | `cargo bench` |
| 錯誤率 | < 1% | 隔離列比例 |
| 成本 | 最小化 | API 調用次數 |

### 本機版控與備份

- 在專案資料夾使用一般 Git repo
- 用分支隔離功能、用小型提交記錄可回復的進度
- 需求、規格、決策、測試案例和操作方式放在 repo 內
- `.env`、金鑰、個資和客戶原始資料不要放進版本庫
- 使用 `.env.example` 和合成測試資料
- 從第一個重要提交開始，備份到另一個實體位置
- Git bundle 可打包已提交的版本與分支（不包含未提交檔案）

### 備份指令

```powershell
# 建立 bundle（離線備份）
git bundle create backup-$(Get-Date -Format 'yyyyMMdd').bundle --branches --tags

# 還原驗證
git bundle verify backup-*.bundle
```

### 何時升級到遠端協作

當你需要異地工作、雲端備份、同事審查、跨部門協作或自動部署時，再把現有本機 repo 接到 GitHub 或 GitLab。屆時加上 PR/MR、必要 CI 檢查、權限與部署流程。

## 品質門檻

依功能風險設定合併前檢查：

- 格式、lint、型別檢查
- 單元與整合測試；關鍵使用流程加端對端測試
- API 或資料格式的契約測試
- 安全、依賴套件與機密資訊掃描
- 高風險變更的效能、無障礙或相容性檢查
- 發布後 smoke test、錯誤與效能監控

## 反覆錯誤防範

- 中文路徑: 使用 `-LiteralPath`
- Application Control policy: 使用 `%LOCALAPPDATA%\Temp\wellfit-target`
- CR-only 行尾: 使用 `split_lines`（支援 LF/CRLF/CR）
- 隔離列行號: 使用 row_lines 同步機制

## 技能參考：AI 時代嚴謹軟體工程技能包 v1.0.0

安裝位置：`.kilo/skills/rigorous-skill-pack/`（工作區層級，不覆蓋本檔）。

本技能包提供 15 個獨立技能與 12 個評估案例，總控層為 `00-orchestration`。按需載入對應技能，不直接覆寫本 `AGENTS.md` 或任何專案規則。所有完成判定均須有實際證據，不僅接受 AI 自稱。

參考檔案：
- `MASTER-WORKFLOW.md`：G0–G10 端到端工程總控流程
- `skills/00-orchestration/SKILL.md`：總控層與停止條件
- `evals/workflow-evals/cases.json`：12 個評估案例
- `scripts/validate_pack.py`：套件結構驗證
- `docs/source-registry.md`：來源與授權注意事項
