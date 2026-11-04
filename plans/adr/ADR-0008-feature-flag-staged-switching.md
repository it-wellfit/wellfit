# ADR-0008: 功能旗標階段式切換與回退

> 狀態：Accepted
> 日期：2026-11-04
> 決策者：架構設計團隊
> 相關文件：`crates/wellfit_integration/src/switching.rs`、`switching_drill.rs`、FLAPS-ARCH-OPT-003 §18

---

## 背景

雙軌對帳達標後，切換本身仍需可控：需能逐步放大新系統佔比、隨時回退、且每次切換留下稽核記錄。直接「開關切換」無法承載風險。

## 問題

如何在保證可回退與可稽核的前提下完成四階段過渡（雙軌 → 影子舊系統 → 舊系統唯讀 → 舊系統退役）？

## 候選方案

### 方案 A：環境變數一次性切換
- **優點**：簡單。
- **缺點**：無階段性、無稽核、無回退程序、需重啟。

### 方案 B：功能旗標 + 狀態機 + 稽核記錄
- **優點**：四階段狀態機限制非法跳轉（如雙軌直接跳退役被拒）；每步記錄（含 SHA-256）；百分比階段式推出；回退為明確操作。
- **缺點**：需維護旗標狀態與歷史。
- **風險**：低。

## 決策

採用 **方案 B**：

| 要素 | 實作 |
|---|---|
| 狀態機 | `CoordinationMode`：DualTrack → ShadowLegacy → LegacyReadOnly → Decommissioned |
| 非法跳轉 | 拒絕（如 DualTrack → Decommissioned 必須經中間態） |
| 階段式推出 | `FeatureFlag::enable(percentage)` 0–100% |
| 回退 | `rollback()`（Decommissioned 前皆可）；`disable_flag()`；`rollback_all()` |
| 稽核 | 每次切換產生 `SwitchingRecord`（前後狀態、原因、SHA-256） |
| 演練 | `SwitchingDrill::run_drill` 全生命週期 E2E；`test_recovery` 回退測試 |

## 決策理由

1. 狀態機使「不可逆動作」被結構性阻擋，而非靠紀律。
2. 稽核記錄讓每次切換可查證（誰、何時、為何、從哪到哪）。
3. 演練模組讓切換程序可離線彩排，M3 出口條件可直接以測試證明。
4. 回退路徑明確：任何階段（除退役）可拉回雙軌。

## 安全及營運影響

- 退役（Decommissioned）為終態，`rollback()` 明確拒絕，防止誤操作。
- 回退全部旗標（`rollback_all`）為事故應急一鍵操作，仍在稽核鏈內。
- 階段式推出降低單點切換的營運衝擊。

## 替代方案

方案 A 無階段、無稽核、無防呆，否決。

## 遷移方式

過渡期以 `SwitchingController` 管理旗標；正式切換依演練報告逐步推進各階段，每階段之間重跑對帳。

## 回復方式

`rollback()` 或 `rollback_all()` 回雙軌；演練已涵蓋回退測試（`test_recovery_rollback`）。

## 驗證方式

- `cargo test -p wellfit_integration`（36 測試含：狀態機轉換、非法跳轉拒絕、階段式推出、回退、演練全生命週期、恢復回退、稽核雜湊）✔
- 演練效能：1,000 筆 × 4 階段 1.25 ms（release）✔

## 後續檢討條件

首次真實切換演練後；或需要更細粒度（per-flow）切換控制時；或退役程序定義變更時。
