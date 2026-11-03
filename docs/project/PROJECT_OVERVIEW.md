# Project Overview — Wellfit

> 文件編號：WFLT-DOC-PROJ-001｜狀態：Active｜版本：v1.0｜日期：2026-10-25

## 1. 這是什麼
Wellfit 是一個 7-crate Rust workspace，crate 清單以根目錄 `Cargo.toml` 為準。它處理大前實業（Dachen Industrial）輔翼 ERP 與外部產品之間的資料往來；具體支援範圍與未完成事項以專案 README 及經驗證的現況快照為準。此文件是產品概覽，不作為執行計畫或即時測試報告。

## 2. 產品範圍
| 產品 | 資料流 | 核心不變量 |
|---|---|---|
| Product 1 — 蝦皮 | 蝦皮訂單匯出 → 輔翼 ERP 匯入批次 | EX-06 金額守恆（整數分，零浮點） |
| Product 2 — 輔翼 ERP 報表 | 庫存報表 → 結構化解析 → 發布閘門 | 庫存恆等式 `期末 ≡ 期初 + 進貨 − 銷貨 + 調整` |
| Product 3 — 哈瓦士 MTRIX | 對接方五檔 CSV → 驗證/修復 → 下游 BI | 56 欄逐欄帶出處、15 條驗證規則、12 條編號化修復規則 |

BankPay is a separate integration workstream, not a fourth row in the three-product operational catalog: the checked-in E.Sun V2.8 specification supports API access through the bank security component, with JSON as the normal payload and a fixed-width payload only for the documented one-debit/one-credit case. Actual CLI network dispatch and bank end-to-end acceptance remain unverified; see [`docs/bankpay/08-open-questions-and-decisions.md`](../bankpay/08-open-questions-and-decisions.md).

## 3. Crate 地圖
| Crate | 路徑 | 職責 |
|---|---|---|
| `wellfit_core` | `crates/wellfit_core/` | 共用領域型別、帳本與管線基礎；實際 API 以程式碼為準 |
| `wellfit_shopee` | `crates/wellfit_shopee/` | 蝦皮資料處理 |
| `wellfit_reports` | `crates/wellfit_reports/` | 輔翼 ERP 報表處理 |
| `wellfit_mtrix` | `crates/wellfit_mtrix/` | MTRIX 五檔資料處理 |
| `wellfit_bankpay` | `crates/wellfit_bankpay/` | BankPay 模組；API-first 規格決策已由本機 E.Sun V2.8 文件支持，但 CLI 實際派送與銀行端端對端驗收仍未確認 |
| `wellfit_sync` | `crates/wellfit_sync/` | 同步模組；依現有專案說明，資料傳輸仍未完成 |
| `wellfit_cli` | `crates/wellfit_cli/` | `wellfit` CLI 進入點；子命令以 `docs/CLI.md` 與 `--help` 為準 |

## 4. 規格來源
跨專案知識庫位置與可用性須依開發環境確認；不要只依相對路徑或本文件宣稱來源文件存在。規格層級與裁決依據請以 `docs/knowledge-index.md` 及可檢查的原始來源為準；來源缺失或相互矛盾時，登錄為未決，不推測。

## 5. 已知限制
- 跨系統介接方式：現有條目未標明產品、來源或證據，不能視為 Wellfit 全域限制；BankPay 另依 E.Sun V2.8 採 API-first，且實際 CLI 派送仍待驗證
- 幣別不換算（依 COUNTRY_ISO 分組）
- VR-12 僅部分實作（缺管理報表來源）
- 蝦皮/ERP 端契約為內部宣告（對方未提供真實樣本）
- 價格合理性與單位一致性未實作（缺基準資料與單位欄）
- 逐欄 Mandatory 清單不可得（MTRIX-003）
- CP/郵遞區號格式未定義（MTRIX-001）
- 小數位數 2 位 vs 4 位未裁決（MTRIX-024）
- 廠商前綴長度 2 vs 3 字母未確認（G-11）

## 6. 快速開始
詳見 `README.md` §3 與 `docs/CLI.md`。建置必須透過 `Knowledge/tools/dev-env/build.ps1`，target 目錄導向 `%LOCALAPPDATA%\Temp\wellfit-target`（OS App Control 約束）。
