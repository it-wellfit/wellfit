# ADR-0003: CLI 權限指派持久化至 SQLite

> 狀態：Accepted
> 日期：2026-10-25
> 決策者：Wellfit 架構團隊
> 相關文件：KNB-WELL-006, access_control.rs, ledger.rs, main.rs

---

## 背景

Wellfit CLI 的 `access-control assign`、`revoke`、`register-role` 子命令原本建立新的 `AccessController` 實例，操作僅在記憶體中，CLI 退出後遺失。這導致稽核斷鏈，違反 ISO 27001:2022 A.8.16（監控行為）的要求。

## 問題

如何確保 CLI 權限指派/撤銷/註冊操作的持久化與可追溯性？

## 候選方案

### 方案 A: 寫入 access_audit_log 表
- **優點**：利用現有 SQLite ledger 基礎設施，無需新增表格，與 `log_access()` 方法一致
- **缺點**：`access_audit_log` 表設計為稽核日誌，非角色指派專用表
- **風險**：查詢角色指派需從稽核日誌篩選，非直接查詢

### 方案 B: 新增 role_assignments 表
- **優點**：專用表格，查詢效率高，語意清晰
- **缺點**：需要 schema 遷移，增加維護成本
- **風險**：遷移失敗可能影響既有資料

### 方案 C: 檔案式 JSON 儲存
- **優點**：簡單，無需資料庫操作
- **缺點**：缺乏交易保證，無法與 SQLite ledger 整合，稽核斷鏈
- **風險**：資料一致性無法保證

## 決策

選擇方案 A: 寫入 `access_audit_log` 表。

## 決策理由

1. **利用現有基礎設施**：`access_audit_log` 表已建立，`log_access()` 方法已實作
2. **交易保證**：SQLite 交易確保寫入原子性
3. **稽核完整性**：所有權限變更皆記錄在不可篡改的稽核日誌中
4. **最小變更**：無需 schema 遷移，降低風險

## 安全及營運影響

- **正面**：稽核斷鏈修復，ISO 27001 A.8.16 合規
- **負面**：查詢角色指派需從稽核日誌篩選
- **營運**：CLI 退出後指派記錄仍可查詢

## 替代方案

- 方案 B：需 schema 遷移，增加風險
- 方案 C：缺乏交易保證，稽核斷鏈

## 遷移方式

不適用（無舊系統遷移）。

## 回復方式

若 `access_audit_log` 不適合角色指派記錄，可後續新增 `role_assignments` 表並遷移資料。

## 驗證方式

- `cargo build --workspace` 成功
- `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- `cargo test --workspace` 全部通過
- CLI `access-control audit-log` 可查詢到指派記錄

## 後續檢討條件

- 若 `access_audit_log` 查詢效能不足（需新增專用表）
- 若角色指派查詢需求增加（需新增 `role_assignments` 表）
- 若需要更細緻的角色生命週期追蹤
