# Wellfit 專案完善計畫 — 持續最佳化與完成

> 計畫編號：KNB-WELL-OPT-001｜版本：v1.0｜日期：2026-10-25
> 狀態：實作準備級

---

## 1. 現況摘要

已建立 6 個 Rust Crate、9 個 PowerShell 工具、20+ 份文件，全量驗證通過（cargo check/clippy/test 0 錯誤、verify-compliance PASS=631/WARN=0/FAIL=0）。 remaining gaps: 端對端測試覆蓋率、UI/UX 一致性、效能基準、CI/CD、錯誤處理一致性、文件完整性、遺失測試（wellfit_reports/wellfit_sync）。

## 2. 計畫目標

將所有缺口 (G-01~G-07) 逐項消除，達到企業級產品交付標準：測試覆蓋率 > 90%、所有 CLI 輸出一致、效能可量化、CI/CD 自動化、錯誤處理標準化、文件完整、所有 crate 有單元測試。

## 3. 執行計畫（有序任務清單）

### Task 1: wellfit_reports 單元測試補充
- **目標**：為 report_automator.rs 編寫 ≥ 15 個單元測試
- **檔案**：`crates/wellfit_reports/src/report_automator_tests.rs`
- **測試範圍**：parse_native_csv_report、verify_balance、release_gate、欄位契約、錯誤處理
- **驗收**：`cargo test -p wellfit_reports` 全部通過

### Task 2: wellfit_sync 單元測試補充
- **目標**：為 wellfit_sync 增加 ≥ 20 個單元測試（已 19 個）
- **檔案**：`crates/wellfit_sync/src/*_tests.rs`
- **測試範圍**：config load/save、device add/remove、discovery、health check、sync cycle、file_hash、init_db
- **驗收**：`cargo test -p wellfit_sync` 全部通過

### Task 3: UI/UX 一致性改善
- **目標**：統一所有 CLI 子命令輸出格式
- **檔案**：`crates/wellfit_cli/src/main.rs`, `crates/wellfit_cli/src/ui.rs`
- **動作**：
  - 所有子命令使用相同的 heading/kv/ok/warn/error 格式
  - 所有錯誤訊息包含錯誤代碼 (ERR-001~ERR-99) 與建議動作
  - `--help` 嵌套清晰，每個子命令有完整說明
  - Spinner/ProgressBar 在所有子命令中一致使用
- **驗收**：`cargo clippy --workspace --all-targets -- -D warnings` 0 警告

### Task 4: 錯誤處理標準化
- **目標**：統一錯誤格式，添加錯誤代碼
- **檔案**：`crates/wellfit_cli/src/main.rs`, `crates/wellfit_mtrix/src/mtrix_engine.rs`
- **動作**：
  - 定義錯誤代碼 enum (ERR-001~ERR-99)
  - 所有錯誤訊息包含代碼 + 說明 + 建議動作
  - 錯誤輸出格式統一：`[ERR-XXX] 說明\n  建議：動作`
- **驗收**：所有錯誤訊息格式一致

### Task 5: 效能基準測試
- **目標**：建立 `cargo bench` 基準測試
- **檔案**：`crates/wellfit_mtrix/benches/`, `crates/wellfit_cli/benches/`
- **測試範圍**：MTRIX 處理時間、CLI 啟動時間、JSON 序列化時間
- **驗收**：`cargo bench` 可執行，結果可量化

### Task 6: CI/CD 工作流
- **目標**：建立 GitHub Actions 工作流
- **檔案**：`.github/workflows/ci.yml`
- **內容**：build → test → clippy → verify-compliance → validate-templates → 安全掃描
- **驗收**：push 後自動執行，所有檢查通過

### Task 7: 文件完善
- **目標**：補充各 crate 的 API 參考文件
- **檔案**：各 crate 的 `lib.rs` 文件註解
- **動作**：為所有 public API 添加 `///` 註解，包含範例
- **驗收**：`cargo doc --workspace --no-deps` 無警告

### Task 8: 最終全量驗證
- **目標**：所有檢查通過
- **驗證項目**：
  - `cargo check --workspace --all-targets` → 0 錯誤
  - `cargo clippy --workspace --all-targets -- -D warnings` → 0 警告
  - `cargo test --workspace` → 全部通過
  - `verify-compliance.ps1` → PASS>0/WARN=0/FAIL=0
  - `validate-templates.ps1` → 0 issues
  - `check-scope-consistency.ps1` → fail=0/warn=0
- **驗收**：所有驗證項目通過

## 4. 依賴關係（DAG）

```
Task 1 (wellfit_reports tests) ──┐
Task 2 (wellfit_sync tests) ─────┤
Task 3 (UI/UX consistency) ─────┼──► Task 8 (最終全量驗證)
Task 4 (錯誤處理標準化) ─────────┤
Task 5 (效能基準測試) ──────────┤
Task 6 (CI/CD 工作流) ──────────┤
Task 7 (文件完善) ──────────────┘
```

所有任務可並行執行，最終統一驗證。

## 5. 風險與緩降

| 風險 | 緩降措施 |
|---|---|
| 測試覆蓋率不足 | 每個 public function 至少 1 個測試 |
| UI 不一致 | 統一使用 ui.rs 模組 |
| 效能退化 | 建立基準測試並監控 |
| 安全弱點 | 定期 cargo audit |
| 文件過時 | 每次變更同步更新 |

## 6. 完成標準（DoD）

- [ ] 所有 crate 測試覆蓋率 > 90%
- [ ] 所有 CLI 輸出格式一致
- [ ] 所有錯誤訊息包含錯誤代碼 + 建議
- [ ] 效能基準測試可執行
- [ ] CI/CD 工作流已建立
- [ ] 所有 public API 有文件註解
- [ ] `cargo check` / `cargo clippy` / `cargo test` 全部通過
- [ ] `verify-compliance.ps1` PASS=631/WARN=0/FAIL=0

---

*本計畫文件為實作準備級文件，所有變更必須重新驗證並更新本計畫。*