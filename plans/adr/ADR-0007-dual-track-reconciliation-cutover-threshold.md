# ADR-0007: 雙軌對帳與 99.5% 切換門檻

> 狀態：Accepted
> 日期：2026-11-04
> 決策者：架構設計團隊
> 相關文件：`crates/wellfit_integration/src/coordination.rs`、`reconciliation.rs`、FLAPS-ARCH-OPT-003 §18

---

## 背景

從 FlapsErp 過渡到 Wellfit 自主系統需經雙軌並行期：兩系統同時處理相同輸入，逐筆比對輸出。缺乏明確的切換門檻將無法客觀判斷「新系統是否可信任」。

## 問題

雙軌對帳應比較什麼、以什麼門檻判定可切換？

## 候選方案

### 方案 A：人工抽樣比對
- **優點**：彈性。
- **缺點**：不可重現、無法規模化、無量化門檻。

### 方案 B：全量雜湊比對 + 99.5% 匹配率門檻
- **優點**：全量、確定性、可重現；門檻量化；差異自動列出（含 diff 類型）。
- **缺點**：需兩邊輸出可雜湊化；相同語義不同序列化會誤判（以欄位級比對緩解）。
- **風險**：低。

### 方案 C：無門檻、憑感覺切換
- **優點**：速度。
- **缺點**：不可接受 — 違反可查證原則。

## 決策

採用 **方案 B**：

| 項目 | 值 |
|---|---|
| 比對單位 | 記錄鍵 → (new_hash, legacy_hash) |
| 匹配判定 | 雜湊相等 |
| 差異分類 | VALUE_MISMATCH / MISSING_IN_NEW / MISSING_IN_LEGACY |
| 切換門檻 | 匹配率 ≥ 99.5% 且差異筆數 ≤ 5（可配置） |
| 欄位級對帳 | `ReconciliationEngine::compare_records`（逐欄位 diff） |

## 決策理由

1. 99.5% 與 FLAPS-ARCH-OPT-003 §19 的「精準 ≥ 99.5%」品質指標一致。
2. 差異不靜默：每筆差異入 `diffs` 清單（鍵 + 雙邊雜湊 + 類型），可逐筆人工裁決。
3. 門檻可配置（`ReconciliationConfig`），不同流程可採不同嚴格度。
4. 對帳歷史保留（`reconciliation_history`），趨勢可觀察。

## 安全及營運影響

- 雙軌期兩系統並行，Wellfit 側為唯讀觀察者，直到門檻達成。
- 未達門檻時 `meets_cutover_threshold` 回報 false，切換流程拒絕推進。
- 差異清單支撐根因分析（配合 JEV/AI 階段後續使用）。

## 替代方案

方案 A 不可量化；方案 C 違反可查證原則。

## 遷移方式

Phase 3 起以 `CoordinationEngine` 執行雙軌；門檻由預設 99.5% 起步，逐流程經實測後決定是否調整。

## 回復方式

未達門檻即維持 DualTrack，無需回復動作；已達門檻但發現問題時，`rollback()` 回 DualTrack。

## 驗證方式

- `cargo test -p wellfit_integration`（對帳全匹配/差異/缺失、門檻判定）✔
- 切換演練在門檻未達時中止並回退（`test_drill_fails_below_threshold`）✔
- 效能：10,000 筆對帳 579.1 µs（17.2M 筆/秒，release）✔

## 後續檢討條件

首次真實雙軌對帳後（差異分布未知）；或單一流程需要更嚴格門檻（如應付 100%）時。
