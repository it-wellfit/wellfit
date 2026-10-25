# Project Overview — Wellfit

> 文件編號：WFLT-DOC-PROJ-001｜狀態：Active｜版本：v1.0｜日期：2026-10-25

## 1. 這是什麼
Wellfit 是一個 5-crate Rust 工作區，把大前實業（Dachen Industrial）輔翼 ERP 與三個外部產品之間的資料往來變成可驗證、可追溯、可重送的管線。發行物是單一原生二進位 `wellfit.exe`（`wellfit_cli`），無前端、無 JavaScript、無外部 DLL 依賴。

## 2. 產品範圍
| 產品 | 資料流 | 核心不變量 |
|---|---|---|
| Product 1 — 蝦皮 | 蝦皮訂單匯出 → 輔翼 ERP 匯入批次 | EX-06 金額守恆（整數分，零浮點） |
| Product 2 — 輔翼 ERP 報表 | 庫存報表 → 結構化解析 → 發布閘門 | 庫存恆等式 `期末 ≡ 期初 + 進貨 − 銷貨 + 調整` |
| Product 3 — 哈瓦士 MTRIX | 對接方五檔 CSV → 驗證/修復 → 下游 BI | 56 欄逐欄帶出處、15 條驗證規則、12 條編號化修復規則 |

## 3. Crate 地圖
| Crate | 路徑 | 職責 |
|---|---|---|
| `wellfit_core` | `crates/wellfit_core/` | MDM 實體編碼清洗、SQLite 冪等帳本、桌面互斥鎖、Release Gate、聯邦契約、定點十進位、CR/LF/CRLF 行分割、名稱解析欄位契約 |
| `wellfit_shopee` | `crates/wellfit_shopee/` | 蝦皮進貨訂單引擎（8 欄契約、EX-06 金額守恆、重複訂單去重、ERP 匯入 CSV 產生） |
| `wellfit_reports` | `crates/wellfit_reports/` | 輔翼 ERP 庫存報表解析（零 OCR、8 欄契約、庫存恆等式、異常列隔離、發布閘門） |
| `wellfit_mtrix` | `crates/wellfit_mtrix/` | MTRIX 五檔 56 欄契約、VR-01..VR-15 驗證規則、ANOM-01..ANOM-12 修復規則、跨檔參照與幣別分組彙總 |
| `wellfit_cli` | `crates/wellfit_cli/` | `wellfit.exe` 進入點，六個子命令：`status`/`shopee`/`reports`/`mtrix`/`audit`/`spec` |
| `wellfit_sync` | `crates/wellfit_sync/` | 多平台同步守護程式（裝置探索、健康監控） |

## 4. 規格來源
跨專案知識庫位於 `../../Knowledge/`。來源層級：L1 原始規格（Alpargatas Programmer's Manual）> L2 內部推薦版（MTRIX BI V2）> L3 近期整合版（ALPARGATAS V6.0-Full）。裁決原則：欄位語意以 L1 優先；介接驗收以實際交付位元組為準；不一致處登錄議題、保留原值、產 Warning。

## 5. 已知限制
- 介接為目錄投放，非主動傳輸（傳輸協定未定）
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
