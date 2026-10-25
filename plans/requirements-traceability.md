# 需求追溯矩陣 (Requirements Traceability Matrix)

> 文件編號：KNB-RTM-001｜狀態：Active｜版本：v1.0｜日期：2026-10-25
> 適用範圍：Wellfit MTRIX Rust 代碼庫
> 追溯鏈：需求 → 設計 → 程式碼 → 測試 → 執行結果 → 證據

---

## 1. 追溯矩陣

| 需求 ID | 需求描述 | 需求來源 | 設計文件 | 程式碼位置 | 測試案例 | 執行結果 | 證據文件 | 狀態 |
|---------|----------|----------|----------|------------|----------|----------|----------|------|
| REQ-001 | MTRIX 檔案驗證（5 檔、56 欄） | Alpargatas 規格 | mtrix_engine.rs | `mtrix_engine.rs:1-2216` | 47 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-002 | 累積餘額追蹤（跨批次累積） | 業務需求 | mdm.rs | `mdm.rs:546-582` | `test_cumulative_balance_accumulates_across_batches` | ✅ 通過 | 測試報告 | 已完成 |
| REQ-003 | CR/LF/CRLF 行尾處理 | G-01 修正 | text.rs | `text.rs:68-143` | `test_carriage_return_terminated_files_are_processed` | ✅ 通過 | 測試報告 | 已完成 |
| REQ-004 | 報告 vs 分類帳原子性寫入 | T2 修正 | mtrix_engine.rs | `mtrix_engine.rs:1332-1372` | `test_every_reported_finding_is_recorded_in_the_ledger` | ✅ 通過 | 測試報告 | 已完成 |
| REQ-005 | RBAC/ABAC 權限模型 | ISO 27001 A.5.15 | access_control.rs | `access_control.rs:1-602` | 7 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-006 | JIT 權限工作流 | ISO 27001 A.5.18 | jit_permission.rs | `jit_permission.rs:1-506` | 5 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-007 | 存取稽核日誌 | ISO 27001 A.8.16 | ledger.rs | `ledger.rs:130-582` | ledger 單元測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-008 | CLI 權限指派持久化 | R-01 修復 | main.rs | `main.rs:1180-1307` | 整合測試 | ✅ 通過 | 測試報告 | 已完成 |
| REQ-009 | `#![forbid(unsafe_code)]` | 安全政策 | 全 Crate | 31 檔案 | grep 驗證 | ✅ 通過 | grep 輸出 | 已完成 |
| REQ-010 | Clippy 零警告 | 品質門檻 | 全 Crate | 全 workspace | `cargo clippy -D warnings` | ✅ 通過 | clippy 輸出 | 已完成 |

---

## 2. 追溯矩陣統計

| 狀態 | 數量 | 百分比 |
|------|------|--------|
| 已完成 | 10 | 100% |
| 部分完成 | 0 | 0% |
| 未開始 | 0 | 0% |
| **合計** | **10** | **100%** |

---

## 3. 追溯鏈定義

```
需求 (Requirement)
  ↓
設計 (Design) — ADR、架構文件、規格
  ↓
程式碼 (Code) — Rust 原始碼、配置檔
  ↓
測試 (Test) — 單元測試、整合測試、滲透測試
  ↓
執行結果 (Execution) — 建置輸出、測試報告、clippy 報告
  ↓
證據 (Evidence) — 測試報告、稽核日誌、文件簽核
```

---

## 4. 追溯完整性檢查

| 檢查項目 | 狀態 | 說明 |
|----------|------|------|
| 每個需求都有設計文件 | ✅ | 10/10 |
| 每個需求都有程式碼位置 | ✅ | 10/10 |
| 每個需求都有測試案例 | ✅ | 10/10 |
| 每個需求都有執行結果 | ✅ | 10/10 |
| 每個需求都有證據文件 | ✅ | 10/10 |
| **追溯完整率** | **100%** | 所有需求可追溯 |

---

**本文件為 Wellfit 需求追溯矩陣，涵蓋 10 項核心需求的完整追溯鏈（需求→設計→程式碼→測試→執行結果→證據），追溯完整率 100%。**
