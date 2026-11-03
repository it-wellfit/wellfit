# Current State — Wellfit

> 文件編號：WFLT-DOC-PROJ-002｜狀態：Active｜版本：v1.0｜日期：2026-10-25

> **時效性：**以下建置、測試、Git 與功能資訊是截至 2026-10-25 的記錄，不是 2026-11-02 的現況驗證。工作區檢視時發現程式碼與文件已有大量未提交變更；本次未重跑建置、測試或 Clippy。更新本快照前，須先在目標 commit 上重新執行並記錄驗證命令與結果。不得將此處測試數字當作目前工作樹的通過證據。

## 1. 版本
- 目前版本：v1.0.6（tag `v1.0.6`，commit `f17fecc`）
- Git 分支：`main`，工作樹乾淨（僅代表 2026-10-25 快照）

## 2. 建置與測試狀態（最後驗證 2026-10-25）
| 檢查 | 結果 |
|---|---|
| `cargo build --workspace` | 0 errors |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| `cargo test --workspace --no-fail-fast` | 162 passed, 0 failed |
| `cargo test -p wellfit_cli --test integration_tests` | 12 passed, 0 failed |
| `wellfit mtrix`（delivered） | 5 files, 22 errors, 50 warnings, 37 repairs |
| `wellfit spec` | 5 files, 56 fields, 15 rules, 12 repairs |
| `scripts/check.ps1` | green（含 golden parity） |
| `scripts/mutation-check.ps1` | green（無 tracked-file 變動） |

## 3. 已完成的主要功能
- MTRIX 五檔 56 欄契約與 VR-01..VR-15 / ANOM-01..ANOM-12
- 蝦皮 8 欄契約與 EX-06 金額守恆
- 輔翼 ERP 報表 8 欄契約與庫存恆等式
- SQLite 幂等帳本（批次去重、隔離記錄、finding 寫入、累計餘額）
- Release Gate 狀態機
- 桌面互斥鎖
- CR-only 行尾支援
- 名稱解析欄位契約（無序位退讓）
- 定點十進位（零浮點）
- 可配置驗證（ValidationConfig）
- 管線基礎架構（pipeline.rs，非接線）
- 故障注入測試（ledger 失敗傳播、崩潰恢復）
- 安全掃描 CI（security.yml、deny.toml、cargo audit、trufflehog）

## 4. 未完成 / 受阻項目
詳見 `../../Knowledge/issues/mtx-issue-register.md` 與 `README.md` §6（已知缺口 G-01..G-12）。主要受阻於對接方提供規格（CP 格式、Mandatory 清單、小數位數、廠商前綴長度、管理報表、傳輸協定）。

## 5. 本機開發環境
- 建置腳本：`Knowledge/tools/dev-env/build.ps1`
- Target 目錄：`%LOCALAPPDATA%\Temp\wellfit-target`（OS App Control 要求）
- 檢查入口：`scripts/check.ps1`
- 備份：`git bundle` → `Documents\wellfit-backups\`
- 還原演練：已驗證（clone + build 成功）

## 6. 下一步
1. Phase 2：管線接線（pipeline.rs → process_drop），以 golden parity 守住行為等價
2. Phase 3：規則/欄位/修復外置化（TOML + JSON）
3. Phase 5：對接方規格到齊後實作 C-01..C-05
4. Phase 7：retro + A+ 評級
