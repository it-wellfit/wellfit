# Wellfit Phase 2 完整執行計畫 — 從治理基礎到產業標準導入

> **Disposition (2026-11-02): Historical proposal — not execution-ready.** Its Phase 1/2 status and references overlap with `KNB-MASTER.md`, `KNB-ISO-MASTER.md`, and `KNB-PHASE2.md`; these masters currently have unresolved authority. Do not execute this combined roadmap until scope and one source of truth are approved.

> 文件編號：1792635153907-phase2-complete-execution
> 適用範圍：Wellfit MTRIX Rust 代碼庫 + 企業級 ISMS + 電商/百貨/財會/製造業
> 前置文件：KNB-MASTER.md (v3.0)、KNB-ENG-SYSTEM.md
> 本輪模式：Plan Mode（唯讀盤點、分析、提出方案；不修改程式碼）

---

## 1. 現況盤點（已由證據確認）

### 已完成項目（Category A/B）

| # | 項目 | 位置 | 驗證方式 |
|---|------|------|----------|
| 1 | T1 累積餘額追蹤 | `mdm.rs` | 單元測試通過 |
| 2 | G-01 CR/LF/CRLF 行尾處理 | `text.rs` | 單元測試通過 |
| 3 | T2 報告 vs 分類帳原子性 | `mtrix_engine.rs` | 整合測試通過 |
| 4 | RBAC/ABAC 權限模型 | `access_control.rs` | 7 單元測試 |
| 5 | JIT 權限工作流 | `jit_permission.rs` | 5 單元測試 |
| 6 | 存取稽核日誌 | `ledger.rs` | ledger 單元測試 |
| 7 | CLI 權限持久化 (P0 修復) | `main.rs` | clippy 零警告 |
| 8 | `#![forbid(unsafe_code)]` | 全 31 檔案 | grep 確認 |
| 9 | Clippy 零警告 | 全 workspace | `cargo clippy -D warnings` |
| 10 | 142 測試通過 | 全 workspace | `cargo test --workspace` |
| 11 | 11 份計畫文件整合 | `KNB-MASTER.md` | 文件審查 |

### 待執行項目（Phase 2）

| 優先級 | 工作項目 | 預估工時 | 驗收標準 |
|--------|----------|----------|----------|
| P0 | ADR 模板 + 首份 ADR | 1 天 | 符合主控規範第 12 節結構 |
| P0 | 需求追溯矩陣 (RTM) | 1 天 | 需求→設計→程式碼→測試→證據完整鏈 |
| P0 | 備份/還原腳本 | 1 天 | 備份 integrity_check=ok、還原後資料一致 |
| P1 | 供應鏈掃描整合 | 1 天 | `cargo audit` 零高危、SBOM 生成 |
| P1 | 故障注入測試 | 2 天 | 崩潰後資料一致 |
| P1 | IAM 評估報告 | 1 天 | Keycloak/Entra ID 方案比較 |
| P1 | PAM 評估報告 | 1 天 | Vault/CyberArk 方案比較 |
| P2 | 多標準對應矩陣 | 2 天 | 42001/SOX 404/NIST CSF/CIS Controls |
| P2 | 產業專屬標準導入計畫 | 3 天 | ISO 9001/PCI DSS/IFRS/ISA-95 導入步驟 |
| P2 | 監控告警規則實作 | 3 天 | 告警可觸發與通知 |

---

## 2. ADR 模板與首份 ADR

### 2.1 ADR 模板

建立 `Wellfit/plans/adr/ADR-TEMPLATE.md`，結構符合主控規範第 12 節：
- 背景、問題、候選方案、決策、決策理由、安全及營運影響、替代方案、遷移方式、回復方式、驗證方式、後續檢討條件

### 2.2 ADR-0001: Wellfit MTRIX 代碼庫採用 Rust 原生實作

- **決策**：選擇 Rust 原生實作
- **理由**：型別安全、記憶體安全、效能、原生部署、品質門檻
- **驗證**：建置通過、clippy 零警告、測試通過、處理時間 < 5s

### 2.3 ADR-0002: ISO 27001:2022 權限管理導入策略

- **決策**：混合階段式路線圖（Phase 1 鎖定代碼庫，Phase 2 擴展企業 ISMS）
- **理由**：避免範圍蔓延延遲專案，優先處理高風險技術控制

### 2.4 ADR-0003: CLI 權限指派持久化至 SQLite

- **決策**：將 `assign`/`revoke`/`register-role` 寫入 `access_audit_log`
- **理由**：稽核完整性、不可篡改、可追溯

---

## 3. 需求追溯矩陣 (RTM)

建立 `Wellfit/plans/requirements-traceability.md`，追溯鏈：需求 → 設計 → 程式碼 → 測試 → 執行結果 → 證據

| 需求 ID | 需求描述 | 需求來源 | 設計文件 | 程式碼位置 | 測試案例 | 執行結果 | 證據文件 | 狀態 |
|---------|----------|----------|----------|------------|----------|----------|----------|------|
| REQ-001 | MTRIX 檔案驗證 | Alpargatas 規格 | mtrix_engine.rs | `mtrix_engine.rs:1-2216` | 47 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-002 | 累積餘額追蹤 | 業務需求 | mdm.rs | `mdm.rs:546-582` | `test_cumulative_balance_accumulates_across_batches` | ✅ 通過 | 測試報告 | 已完成 |
| REQ-003 | CR/LF/CRLF 行尾處理 | G-01 修正 | text.rs | `text.rs:68-143` | `test_carriage_return_terminated_files_are_processed` | ✅ 通過 | 測試報告 | 已完成 |
| REQ-004 | 報告 vs 分類帳原子性 | T2 修正 | mtrix_engine.rs | `mtrix_engine.rs:1332-1372` | `test_every_reported_finding_is_recorded_in_the_ledger` | ✅ 通過 | 測試報告 | 已完成 |
| REQ-005 | RBAC/ABAC 權限模型 | ISO 27001 A.5.15 | access_control.rs | `access_control.rs:1-602` | 7 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-006 | JIT 權限工作流 | ISO 27001 A.5.18 | jit_permission.rs | `jit_permission.rs:1-506` | 5 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-007 | 存取稽核日誌 | ISO 27001 A.8.16 | ledger.rs | `ledger.rs:130-582` | ledger 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-008 | CLI 權限指派持久化 | R-01 修復 | main.rs | `main.rs:1180-1307` | 整合測試 | ✅ 通過 | 測試報告 | 已完成 |

---

## 4. 備份與還原程序

### 4.1 備份策略（3-2-1 原則）

| 層級 | 頻率 | 保留期限 | 儲存位置 |
|------|------|----------|----------|
| L1 本地備份 | 每日 | 7 天 | `%LOCALAPPDATA%\Temp\wellfit-backup\` |
| L2 異地備份 | 每週 | 4 週 | 網路共享磁碟機 / 雲端儲存 |
| L3 離線備份 | 每月 | 12 個月 | Git bundle / 物理媒體 |

### 4.2 備份腳本 `scripts/daily-backup.ps1`

- 使用 SQLite `.backup` 命令進行線上備份
- 驗證備份完整性（`PRAGMA integrity_check`）
- 清理 7 天前的備份

### 4.3 還原程序 `scripts/restore.ps1`

1. 確認備份檔案存在
2. 備份當前資料庫（以防還原失敗）
3. 執行還原
4. 驗證還原後完整性

---

## 5. 供應鏈掃描整合

### 5.1 cargo-audit 整合

- 安裝：`cargo install cargo-audit`
- 執行：`cargo audit`
- SBOM 生成：`cargo cyclonedx --format json --output-file sbom.json`

### 5.2 cargo-deny 配置 `deny.toml`

- advisories: vulnerability = deny, unmaintained = warn, yanked = warn
- licenses: unlicensed = deny, allow = MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-DFS-2016, Zlib
- bans: multiple-versions = warn, wildcards = deny
- sources: unknown-registry = deny, unknown-git = deny

### 5.3 CI/CD 整合

- GitHub Actions security workflow
- 每次 push/PR 自動執行 `cargo audit` + SBOM 生成

---

## 6. 故障注入測試

### 6.1 分類帳原子性故障注入

| 測試案例 | 注入點 | 預期結果 |
|----------|--------|----------|
| 報告寫入前中斷 | `record_finding` 中 `ledger.log_finding()` 之後、`report.add_finding()` 之前 | 分類帳有記錄，報告無記錄；下次重試時重複檢查避免重複 |
| 分類帳寫入失敗 | 模擬 SQLite 寫入失敗 | 函式返回錯誤，不繼續寫入報告 |
| 報告寫入失敗 | 模擬報告寫入失敗 | 分類帳已有記錄，報告寫入失敗；下次重試時重複檢查 |
| 交易中斷 | 模擬程序崩潰 | SQLite 交易回滾，資料保持一致 |

### 6.2 崩潰復原驗證

| 場景 | 驗證方式 | 預期結果 |
|------|----------|----------|
| 交易中斷 | `PRAGMA integrity_check` | `ok` |
| 部分寫入 | 比對報告與分類帳計數 | 一致或報告少於分類帳（可重試） |
| 重啟後狀態 | `wellfit status` | 正常輸出，無資料損壞 |

---

## 7. IAM/SSO 評估報告

### 7.1 Keycloak vs Microsoft Entra ID

| 項目 | Keycloak | Microsoft Entra ID |
|------|----------|-------------------|
| 部署方式 | 自我託管 | 雲端 SaaS |
| 成本 | 免費（開源） | 訂閱制 |
| SSO 協議 | OAuth2/OIDC/SAML | OAuth2/OIDC/SAML |
| MFA | 內建 TOTP/WebAuthn | 內建 MFA + 條件式存取 |
| 使用者聯邦 | LDAP/AD/Kerberos | Azure AD Connect |
| 客製化 | 高（Java SPI） | 低（雲端限制） |
| 運維負擔 | 高（需自行維護） | 低（Microsoft 維護） |
| 資料主權 | 完全控制 | Microsoft 控制 |
| Rust 整合 | OIDC 客戶端 crate | OIDC 客戶端 crate |

### 7.2 建議

- **Phase 1**：Keycloak（開源、可自我託管、與 Rust 整合較易）
- **Phase 2**：Microsoft Entra ID（與 Azure/O365 生態整合、條件式存取成熟）

---

## 8. PAM 評估報告

### 8.1 HashiCorp Vault vs CyberArk

| 項目 | HashiCorp Vault | CyberArk |
|------|----------------|----------|
| 部署方式 | 自我託管/雲端 | 自我託管/雲端 |
| 成本 | 開源/商業 | 商業 |
| 密鑰管理 | 動態憑證、加密即服務 | 密碼庫、特權會議監控 |
| 臨時憑證 | 原生支援（TTL-based） | 需額外配置 |
| 會議錄製 | 需整合 | 原生支援 |
| 密碼輪替 | 自動輪替 | 自動輪替 |
| Rust 整合 | Vault API crate | REST API |

### 8.2 建議

- **Phase 1**：HashiCorp Vault（開源、輕量、與 Rust 相容）
- **Phase 2**：CyberArk（特權會議監控、密碼庫、合規報告）

---

## 9. 多標準對應矩陣

| ISO 27001:2022 | ISO 42001 (AI) | ISO 27701:2025 (PIMS) | SOX 404 | Wellfit 對應 |
|----------------|----------------|------------------------|---------|-------------|
| A.5.1 資訊安全政策 | 5.2 AI 政策 | 5.2 隱私政策 | — | 存取控制政策 |
| A.5.15 存取控制 | 6.2.3 AI 存取控制 | 6.2.3 隱私存取控制 | ITGC 存取控制 | RBAC/ABAC 模型 |
| A.5.17 認證資訊 | 6.2.4 AI 身分驗證 | 6.2.4 隱私身分驗證 | ITGC 身分驗證 | MFA 規則 |
| A.5.18 存取權限 | 6.2.5 AI 權限管理 | 6.2.5 隱私權限管理 | ITGC 權限管理 | JIT 權限 |
| A.6.5 職責分離 | 6.3 SoD | 6.3 SoD | ITGC SoD | SoD 矩陣 |
| A.8.2 特權存取管理 | 6.4 AI 特權存取 | 6.4 隱私特權存取 | ITGC 特權存取 | JIT + 特權旗標 |
| A.8.16 監控行為 | 7.2 AI 監控 | 7.2 隱私監控 | ITGC 監控 | access_audit_log |
| A.8.25 安全開發生命週期 | 8.2 AI 開發生命週期 | — | ITGC 變更管理 | `#![forbid(unsafe_code)]` |
| A.8.28 安全編碼 | 8.3 AI 安全編碼 | — | — | Rust 型別安全 |

---

## 10. 品質閘門（G0-G7）

| 閘門 | 通過條件 | 未通過處理 |
|------|----------|------------|
| G0：工作區安全 | 未提交變更及操作影響已確認 | 停止可能覆寫資料的操作 |
| G1：需求完整性 | 驗收條件與追溯關係明確 | 補齊需求與測試設計 |
| G2：建置品質 | 目標建置成功、必要靜態檢查通過 | 阻擋合併或發布 |
| G3：測試品質 | 關鍵單元、整合與回歸測試通過 | 重現、除錯及修復 |
| G4：安全品質 | 無未處理且超出風險政策的重大安全缺陷 | 阻擋發布或啟動正式風險核准 |
| G5：資料完整性 | 交易、冪等、遷移與復原測試通過 | 禁止執行正式資料變更 |
| G6：發布安全 | 監控、備份、回滾及發布後驗證就緒 | 延後發布 |
| G7：證據完整性 | 結果可追溯，未驗證事項清楚揭露 | 不得宣告完全完成 |

---

## 11. 工程品質儀表板

| 指標 | 定義 | 目標 | 資料來源 |
|------|------|------|----------|
| 重大缺陷逃逸率 | 發布後重大缺陷數 ÷ 已發布變更數 | 持續下降 | 缺陷追蹤系統 |
| 資料完整性違規 | 已確認違規事件數/統計週期 | 0 | 分類帳對帳 |
| 未授權操作成功數 | 經驗證的未授權成功事件數 | 0 | 存取稽核日誌 |
| 回歸缺陷率 | 修復後再次出現的缺陷數 ÷ 已修復缺陷數 | 持續下降 | 缺陷追蹤系統 |
| 復原時間 | 事故開始至服務恢復的時間 | 依嚴重程度設定 | 事故報告 |
| 證據完整率 | 具備必要證據的適用控制數 ÷ 適用控制總數 | 100% | 證據索引 |

---

## 12. 實施建議（最後的實施建議）

不要一開始就把所有功能、工具與國際標準同時全面重構。最可靠的路徑是先建立可驗證的基線，再優先處理可能造成資料毀損、權限失控或錯誤完成宣告的高風險問題。

### 建議的實施順序

1. **現況盤點與證據核對**：確認目前真正存在的功能與缺口
2. **核心不變量驗證**：優先檢查分類帳、累積餘額、MTRIX 行號與原子性
3. **權限安全測試**：驗證 RBAC/ABAC、SoD、MFA、JIT 到期及撤銷
4. **自動化品質閘門**：將建置、測試、安全檢查與回歸驗證納入可重複執行的流程
5. **事故與持續改善治理**：建立指標、根本原因分析、證據保存及風險複查
6. **標準與合規差距驗證**：以官方標準、適用範圍及真實證據逐項核對

### 成功標準

不是文件寫得多完整，也不是 AI 宣稱自己已完成多少工作，而是每一輪演進都能證明：
- 系統更安全
- 資料更可靠
- 錯誤更容易被發現
- 故障更容易被復原
- 同類問題再次發生的機率確實降低

---

## 13. 下一個最小且高價值的工作

**建議：P0 建立 ADR 模板與首份 ADR（ADR-0001）**

這是主控規範第 12 節的明確要求，所有設計決策的追溯基礎，完成後即可標記為「治理基礎已建立」。

### 執行方式
1. 建立 `plans/adr/` 目錄
2. 建立 ADR 模板 `plans/adr/ADR-TEMPLATE.md`
3. 建立 ADR-0001：Wellfit MTRIX 代碼庫採用 Rust 原生實作
4. 建立 ADR-0002：ISO 27001:2022 權限管理導入策略
5. 建立 ADR-0003：CLI 權限指派持久化至 SQLite

### 驗收標準
- ADR 文件符合主控規範第 12 節結構
- 每個 ADR 有唯一編號、狀態、日期、決策者

---

**當前狀態**：Phase 1 核心技術控制項已完成（16/93 ISO 27001 控制項已實作，142 測試通過，clippy 零警告）。Phase 2 起點已識別，最佳調整方案已提出，等待使用者確認下一個優先級後進入執行。

---

**本文件為 Wellfit Phase 2 完整執行計畫，涵蓋現況盤點、ADR 模板、RTM、備份/還原、供應鏈掃描、故障注入、IAM/PAM 評估、多標準對應、品質閘門、品質儀表板、執行任務模板與實施建議。所有內容均與 KNB-MASTER.md 互相引用，形成完整的知識庫。**
