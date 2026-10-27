# Wellfit 工程基線與治理差距盤點 — 主控規範執行計畫

> 文件編號：KNB-WELL-000｜狀態：Draft｜版本：v1.0｜日期：2026-10-25
> 依據規範：WELLFIT 全生命週期工程治理、零缺陷演進與自動化品質保證主控規範（16 章）
> 適用範圍：Wellfit Rust 工作區（C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit）
> 本輪模式：Plan Mode（唯讀盤點、分析、提出方案；不修改程式碼）

---

## 1. 執行摘要

本計畫依據 WELLFIT 工程治理主控規範第 16 節，對 Wellfit 專案進行基線盤點與治理差距分析。經唯讀檢查，已驗證的技術控制項與未完成的治理項目如下：

- **已實作且由程式碼/測試確認**：T1 累積餘額、G-01 行尾處理、T2 原子性寫入、RBAC/ABAC、JIT 權限、存取稽核日誌、`#![forbid(unsafe_code)]`、CLI 權限子命令、5 份 ISO 27001 規劃文件、142 測試通過、clippy 零警告。
- **尚未實作**：IAM/SSO、PAM、ELK/Wazuh、Drata/Vanta、ISO 42001/27701/SOX 404、ADR 文件、需求追溯矩陣、備份/還原程序、SBOM、模糊測試、故障注入測試。
- **設計缺陷**：CLI 的 `assign`/`revoke`/`register-role` 僅在記憶體中操作，退出後遺失；未持久化至分類帳。

最重要的五項風險：1) 權限變更未持久化；2) 缺乏 ADR 與需求追溯；3) 無備份/還原程序；4) 缺乏供應鏈掃描（SBOM）；5) 無故障注入與崩潰復原測試。

---

## 2. 基線盤點（唯讀確認）

### 2.1 已由程式碼確認（Category A）

| 項目 | 位置 | 證據 |
|------|------|------|
| `#![forbid(unsafe_code)]` | 全 31 個 crate 檔案 | `grep` 確認 31 處 |
| T1 累積餘額 | `crates/wellfit_core/src/mdm.rs:184-582` | `cumulative_balance` 表、`register_cumulative`、`get_cumulative_balance`、測試 `test_cumulative_balance_accumulates_across_batches` |
| G-01 CR/LF/CRLF | `crates/wellfit_core/src/text.rs:8-143` | `MAX_INPUT_BYTES`、`read_capped`、`split_lines_keeping_blanks`、`split_records` |
| T2 原子性寫入 | `crates/wellfit_mtrix/src/mtrix_engine.rs:326-1417` | `record_finding` 先寫分類帳再寫報告 |
| RBAC/ABAC | `crates/wellfit_core/src/access_control.rs` | `PermissionLevel`、`Classification`、`Permission`、`Role`、`SodConstraint`、`AccessController`、7 單元測試 |
| JIT 權限 | `crates/wellfit_core/src/jit_permission.rs` | `JitRequestStatus`、`JitAccessRequestParams`、`JitRequest`、`JitGrant`、`JitPermissionManager`、5 單元測試 |
| 存取稽核日誌 | `crates/wellfit_core/src/ledger.rs:130-582` | `AccessAuditParams`、`access_audit_log` 表、`log_access`、`get_access_audit_log` |
| CLI 權限子命令 | `crates/wellfit_cli/src/main.rs:164-1252` | `access-control {check,assign,revoke,audit-log,register-role}`、`jit {request,approve,reject,revoke,list-pending,list-grants}` |

### 2.2 已由測試確認（Category B）

| 指標 | 結果 | 命令 |
|------|------|------|
| 建置 | 通過 | `cargo build --workspace` |
| Clippy | 零警告 | `cargo clippy --workspace --all-targets -- -D warnings` |
| 單元測試 | 142/142 通過 | `cargo test --workspace` |

### 2.3 僅有文件描述（Category C）

| 項目 | 文件位置 | 備註 |
|------|----------|------|
| ISO/IEC 27001:2022 主計畫 | `plans/001-iso27001-access-control-implementation-plan.md` | 24 週階段式時程 |
| Annex A 控制項對應 | `plans/002-annex-a-control-mapping-evidence.md` | 93 項控制項對應表 |
| 工具包參考 | `plans/003-toolkit-resource-reference.md` | IAM/PAM/MFA/審計/合規工具清單 |
| 驗證與測試計畫 | `plans/004-verification-testing-plan.md` | 測試案例、合規檢查清單、品質門檻 |
| 風險與持續監控 | `plans/005-risk-continuous-monitoring.md` | 風險登記冊、PDCA、告警規則 |
| 開發規範 | `AGENTS.md` | 建置/測試/粗糙檢查指令 |

### 2.4 尚未驗證（Category D）

| 項目 | 狀態 | 原因 |
|------|------|------|
| IAM/SSO 整合（Keycloak/Entra ID） | 未實作 | 僅在計畫文件中提及 |
| PAM 部署（Vault/CyberArk） | 未實作 | 僅在計畫文件中提及 |
| ELK/Wazuh/Splunk 監控 | 未實作 | 僅在計畫文件中提及 |
| Drata/Vanta 合規自動化 | 未實作 | 僅在計畫文件中提及 |
| ISO 42001 / 27701 / SOX 404 | 未開始 | 規劃中，無程式碼 |
| ADR 文件 | 未創建 | 主控規範第 12 節要求 |
| 需求追溯矩陣 | 未創建 | 主控規範第 1 節要求 |
| 備份/還原程序 | 未實作 | 主控規範第 5.2、11 節要求 |
| SBOM/供應鏈掃描 | 未實作 | 主控規範第 10 節要求 |
| 模糊測試（cargo-fuzz） | 未實作 | 主控規範第 4 節要求 |
| 故障注入測試 | 未實作 | 主控規範第 4 節要求 |

### 2.5 已知設計缺陷（Category E）

| 項目 | 位置 | 問題 |
|------|------|------|
| 權限指派未持久化 | `crates/wellfit_cli/src/main.rs:1180-1196` | `assign`/`revoke` 建立新的 `AccessController` 實例，操作僅在記憶體中，CLI 退出後遺失；未寫入 `access_audit_log` |
| 角色註冊未持久化 | `crates/wellfit_cli/src/main.rs:1226-1250` | `register-role` 同上，僅記憶體操作 |

### 2.6 已淘汰（Category F）

| 檔案 | 原因 |
|------|------|
| `crates/wellfit_core/src/mdcatalog.rs` | 功能併入 `mdm.rs`，標記為 DEPRECATED |
| `crates/wellfit_core/src/mdhub.rs` | 功能併入 `mdm.rs`，標記為 DEPRECATED |
| `crates/wellfit_mtrix/src/engine.rs` | 功能併入 `mtrix_engine.rs`，標記為 DEPRECATED |
| `crates/shopee_engine.rs` | 錯誤位置，正確在 `crates/wellfit_shopee/src/shopee_engine.rs`，標記為 DEPRECATED |

---

## 3. 問題與風險清單

### 3.1 高風險（須優先處理）

| ID | 風險 | 影響 | 目前狀態 | 建議緩減 |
|----|------|------|----------|----------|
| R-01 | 權限指派/撤銷未持久化 | 權限變更在 CLI 退出後遺失，稽核斷鏈 | 設計缺陷（Category E） | 將 `assign`/`revoke`/`register-role` 持久化至 `access_audit_log` 或新增 `role_assignments` 表 |
| R-02 | 缺乏 ADR 與需求追溯矩陣 | 設計決策無法追溯，變更影響無法評估 | 未創建（Category D） | 建立 ADR 模板與需求追溯矩陣文件 |
| R-03 | 無備份/還原程序 | 資料遺失無法復原 | 未實作（Category D） | 定義 SQLite 備份策略與還原測試 |
| R-04 | 無供應鏈掃描（SBOM） | 相依套件漏洞無法偵測 | 未實作（Category D） | 引入 `cargo-audit` / `cargo-deny`，生成 SBOM |
| R-05 | 無故障注入與崩潰復原測試 | 分類帧原子性在異常下未驗證 | 未實作（Category D） | 增加故障注入測試案例 |

### 3.2 中風險

| ID | 風險 | 影響 | 目前狀態 | 建議緩減 |
|----|------|------|----------|----------|
| R-06 | ISO 42001 / 27701 / SOX 404 未對齊 | 多標準合規缺口 | 未開始（Category D） | 制定多標準對應矩陣 |
| R-07 | IAM/SSO 未整合 | 特權帳戶缺乏集中身分管理 | 未實作（Category D） | 評估 Keycloak / Entra ID 整合 |
| R-08 | PAM 未部署 | 特權帳號無輪替機制 | 未實作（Category D） | 評估 HashiCorp Vault / CyberArk |

---

## 4. 不變量定義（強制工作流程第 3.2 節）

| 模組 | 前置條件 | 後置條件 | 不變量 | 禁止的狀態轉換 |
|------|----------|----------|--------|--------------|
| 權限指派 | 使用者存在、角色存在 | 指派記錄持久化 | 指派記錄必須可在 `access_audit_log` 查詢 | 指派後不得在未記錄的情況下生效 |
| JIT 權限 | 請求狀態為 Pending | 授權有效期限內有效 | 過期授權必須自動失效 | 已過期的授權不得可用 |
| 分類帧寫入 | 交易鎖定已取得 | 兩端（報告+分類帧）均寫入或皆失敗 | 報告與分類帧計數必須一致 | 單端寫入成功另一端失敗 |
| SQLite Schema | 遷移腳本可重複執行 | Schema 版本遞增 | 既有資料不得被遷移破壞 | 破壞性變更不得在無備份情況下執行 |

---

## 5. 威脅分析（強制工作流程第 2 節）

| 威脅類別 | 具體場景 | 風險等級 | 現有防護 | 建議補強 |
|----------|----------|----------|----------|----------|
| 權限擴大 | 使用者透過 CLI 指派自己更高權限 | 高 | 無（設計缺陷 R-01） | 持久化指派記錄 + SoD 檢查 |
| 資料遺失 | SQLite 檔案損壞或誤刪 | 高 | 無（R-03） | 備份策略 + 還原測試 |
| 供應鏈攻擊 | 惡意相依套件注入 | 中 | `#![forbid(unsafe_code)]` | `cargo-audit` + SBOM |
| 稽核斷鏈 | 權限變更未記錄 | 中 | `access_audit_log` 存在但 CLI 未寫入 | CLI 持久化修復 |
| 崩潰不一致 | 分類帧寫入中斷 | 中 | `record_finding` 原子性 | 故障注入測試 |

---

## 6. 最小安全改善路線圖（建議順序）

| 優先級 | 工作項目 | 預期產出 | 驗收標準 |
|--------|----------|----------|----------|
| P0 | 修復 CLI 權限指派持久化 | `assign`/`revoke`/`register-role` 寫入 `access_audit_log` | 退出後重新查詢仍可看到指派記錄 |
| P0 | 建立 ADR 模板與首份 ADR | `plans/adr-0001-...md` | 符合主控規範第 12 節 ADR 結構 |
| P0 | 建立需求追溯矩陣 | `plans/requirements-traceability.md` | 需求→設計→程式碼→測試→證據完整鏈 |
| P1 | 定義備份/還原程序 | `plans/backup-restore.md` | 備份腳本 + 還原驗證測試 |
| P1 | 引入供應鏈掃描 | `cargo-audit` / `cargo-deny` 整合 | 建置時掃描漏洞，生成 SBOM |
| P2 | 故障注入測試 | 分類帧原子性故障注入案例 | 異常下報告與分類帧計數一致 |
| P2 | IAM/SSO 評估報告 | `plans/iam-evaluation.md` | Keycloak / Entra ID 整合方案比較 |

---

## 7. 驗證與測試矩陣

| 測試類型 | 範圍 | 工具 | 狀態 |
|----------|------|------|------|
| 單元測試 | 權限模型、JIT、MDM、Ledger | `cargo test -p <crate> --lib` | ✅ 通過 |
| 整合測試 | CLI 權限子命令、MTRIX 流程 | `cargo test -p wellfit_cli --test integration_tests` | ✅ 通過 |
| Clippy 靜態分析 | 全 workspace | `cargo clippy --all-targets -- -D warnings` | ✅ 零警告 |
| 建置檢查 | 全 workspace | `cargo build --workspace` | ✅ 通過 |
| 供應鏈掃描 | 相依套件漏洞 | `cargo-audit` | ❌ 未實作 |
| 故障注入 | 分類帧原子性 | 自定義測試 | ❌ 未實作 |
| 備份/還原 | SQLite 資料庫 | 自定義腳本 | ❌ 未實作 |

---

## 8. 回復與復原方案

| 場景 | 回復方式 | 驗證方法 |
|------|----------|----------|
| CLI 權限指派錯誤 | 重新執行 `revoke` 撤銷錯誤指派 | 查詢 `access_audit_log` 確認 |
| SQLite 資料庫損壞 | 從備份檔案還原 | 執行還原後驗證資料完整性 |
| 相依套件漏洞 | 升級套件版本 | `cargo audit` 確認無高危漏洞 |
| 分類帧不一致 | 執行 `release_file_batch` 反向操作 | 對帳報告與分類帧計數 |

---

## 9. 未解決事項與殘餘風險

| 項目 | 說明 | 殘餘風險等級 | 處理方式 |
|------|------|--------------|----------|
| 權限指派持久化 | CLI 指派不持久化 | 高 | P0 修復 |
| ADR 缺失 | 設計決策無追溯 | 高 | P0 建立 |
| 備份缺失 | 資料遺失無法復原 | 高 | P1 定義程序 |
| 供應鏈掃描缺失 | 漏洞未偵測 | 中 | P1 引入工具 |
| 故障注入缺失 | 崩潰復原未驗證 | 中 | P2 增加測試 |

---

## 9. 下一個最小且高價值的工作

**建議：P0 修復 CLI 權限指派持久化（R-01）**

這是唯一一個已證實的設計缺陷，直接影響稽核完整性。修復方式：將 `assign`/`revoke`/`register-role` 的操作寫入 `access_audit_log` 或新增 `role_assignments` 表，確保退出後指派記錄仍可查詢。

---

## 9. 給使用者的單一問題（必須回答才能繼續）

**問題：** 在進入 P0 修復前，您希望我優先處理哪一項？

建議答案：**A. P0 修復 CLI 權限指派持久化（assign/revoke/register-role 寫入 access_audit_log）** — 這是唯一已證實的設計缺陷，直接影響稽核完整性，修復後即可標記為「零已知設計缺陷」。

請確認或選擇其他優先順序（B. 建立 ADR 與需求追溯矩陣 / C. 定義備份還原程序 / D. 引入供應鏈掃描 / E. 其他）。

---

**當前狀態：** 盤點完成，5 個已驗證控制項 + 5 個設計缺陷/缺口已識別。等待使用者確認下一個優先級後，可由具備實作權限的代理進入 Phase 3 實作。
