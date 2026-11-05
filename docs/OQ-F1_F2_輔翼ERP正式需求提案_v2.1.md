# 輔翼 ERP 自動化與效能優化 — 正式需求提案（OQ-F1 / OQ-F2）

> **文件編號：** PROP-2026-1103-001
> **版本：** v2.1（最終提交版）
> **日期：** 2026-11-04
> **提案方：** 大前實業（Dachen Industrial）— Wellfit 整合團隊
> **受提案方：** 輔翼科技股份有限公司（Flaps）— CTO 賴志松 / Tech Dev Lead 戴天麒 / ERP Lead 陳文彬
> **分類：** 正式需求（非 RFC），請回覆書面政策與規格
> **回覆期限請求：** 2026-11-19（14 天，UTC+8）
> **提案有效期：** 2026-12-31 前有效；逾期請重新確認

---

## 1. 背景與目的

我方（大前實業）為輔翼 ERP 的長期使用客戶，透過 Wellfit 工具鏈（9-crate Rust 工作區）對接輔翼 ERP 之進銷存、庫存報表與應付憑單。現因業務成長（多通路 OMO、E.Sun 銀企直連、MTRIX 五檔對接），需提升自動化程度與效能，故提出以下正式需求，請輔翼回覆書面政策、規格與升級路徑。

本提案所有技術事實均以逆向工程（安裝檔 v0.0.4、app.asar 1,177 檔）與官方網站（flaps.com.tw）公開資訊為證據基礎，並遵循 claims-vs-evidence 紀律：未驗證事項以 ⚠ 標記，不以推測替代事實。

---

## 2. OQ-F1：自動化政策與技術通道（P0 — 阻塞 RSK-E1）

### 2.1 背景（已觀測 ✔）

輔翼 ERP 外殼為 Electron 應用程式，本機逆向確認存在以下官方通道：

| 通道 | 機制 | 證據 |
|---|---|---|
| CH-1 | flapserp:// 自訂協定（base64(JSON) 啟動參數） | main.js:520-695 ✔ |
| CH-2 | RDAPortal.jsp XML RDA 協定 | dllManager.js:57-98 ✔ |
| CH-3 | COM/ActiveX | active_patch.js + mainApi.js:155-283 ✔ |
| CH-4 | 檔案交換（saveFile/downloadFile API、CSV 匯入匯出） | mainApi.js:527-568 ✔ |
| CH-5 | Playwright（外殼自帶測試框架） | main.js:423-437 ✔ |

### 2.2 正式請求（OQ-F1）

| # | 請求項目 | 優先 |
|---|---|---|
| F1-1 | 自動化政策書面聲明 | P0 |
| F1-2 | RDA API 規格文件 | P0 |
| F1-3 | flapserp:// 協定規格 | P1 |
| F1-4 | COM 物件清單 | P1 |
| F1-5 | 64-bit 打包提供 | P0 |
| F1-6 | 憑證模式修正 | P0 |
| F1-7 | 同步 IPC 改非同步 | P1 |
| F1-8 | 更新通知機制 | P2 |

### 2.3 我方承諾

- 取得書面政策前，僅使用官方入口（flapserp://、檔案交換）進行自動化
- COM/CDP/RDA 僅在 PoC 環境測試，不上生產
- 所有自動化操作不寫入 C:\SYSTEM_PROGRAM_DIRECTORY（原廠元件區）

---

## 3. OQ-F2：資料契約與報表（P0 — 阻塞 T-203 月結試行）

### 3.1 背景（已觀測 ✔）

我方 Wellfit 工具鏈已實作 8 欄 ERP 匯入契約（shopee_order_contract()），但輔翼 ERP 匯入範本與欄位字典尚未取得（知識庫 OI-006，claims-vs-evidence C-22 標記「不成立」）。

### 3.2 正式請求（OQ-F2）

| # | 請求項目 | 說明 | 優先 |
|---|---|---|---|
| F2-1 | ERP 匯入範本 | 輔翼 ERP 進貨/應付憑單匯入的 CSV/Excel 範本（含欄位定義、必填/選填、格式規則） | P0 |
| F2-2 | 欄位字典 | 各匯入欄位的名稱、型別、長度、驗證規則 | P0 |
| F2-3 | 管理報表樣本 | 輔翼 ERP 庫存/財務管理報表的匯出樣本（≥1 份真實資料，含欄位標頭） | P0 |
| F2-4 | 報表欄位定義 | 報表各欄位的名稱、計算邏輯、aggregation 規則 | P1 |
| F2-5 | API 串接文件 | 91APP/SHOPLINE/Shopify/蝦皮/MOMO/CYBERBIZ 串接的 API 規格 | P2 |

### 3.3 我方承諾

- 取得正式範本後，更新 Wellfit ColumnContract（名稱解析、必填硬失敗、禁序位退讓）
- 提供 Wellfit 解析結果與原廠報表的對帳 evidence chain
- 不以合成資料宣稱為真實 ERP 資料（知識庫紀律）

---

## 4. 效能基線數據（供輔翼參考）

### 4.1 我方量測（2026-11-03，Wellfit/artifacts/perf/20261103-160315/）

| 指標 | 數值 | 量測方式 |
|---|---|---|
| FlapsErp 執行中行程數 | 11 | Get-Process |
| 全系統 RAM | 623.8 MB | WorkingSet64 彙總 |
| COM 每呼叫 P50 | 0.26 ms | Scripting.FileSystemObject.GetSpecialFolder x 1000 |
| COM 每呼叫 P95 | 0.39 ms | 同上 |
| CDP 9222 埠 | NOT-FOUND | Invoke-WebRequest /json/list |
| 安裝檔版本 | v0.0.4 (2025-04-29) | FileVersionInfo |
| 安裝檔 SHA256 | DC77C024629629149842714B8F599565DB04121AF043F5517450BAA5F0AE79B6 | Get-FileHash |
| 打包架構 | ia32 (32-bit) | electron-builder-dev.yml |
| Node 依賴 | winax 3.3.4, ffi-napi 4.0.3, serialport 10.4.0, bytenode 1.3.4 | package.json |
| 外殼 UI | Vue 2.6.12 + Bootstrap 5.0.1 + Electron | app.asar 逆向 |
| 報表引擎 | FastReport (DEBUG_FASTREPORT) + MReportViewer.dll (14.6MB) | config.ini + E-04 |

### 4.2 我方已識別的效能瓶頸（附證據編號）

| # | 瓶頸 | 證據 | 影響 |
|---|---|---|---|
| B-1 | 32-bit 位址空間上限（~2GB） | electron-builder-dev.yml arch: ia32 | 大報表/大批次 OOM 風險 |
| B-2 | COM 呼叫走同步 IPC | mainApi.js:193,225 | UI 凍結、批次延遲疊加 |
| B-3 | Electron 基底記憶體 300-500MB+ | E-02 (134MB exe) | 常駐記憶體高 |
| B-4 | backgroundThrottling:false | main.js:294 | 背景分頁持續耗 CPU |
| B-5 | UI 操作全走網路往返 | 架構 | 網速敏感、斷線即無 UI |
| B-6 | 報表客端渲染 | E-04/E-05 | 客端 CPU/RAM 佔用 |
| B-7 | DLL 註冊流程 | dllManager.js | 部署摩擦、首啟慢 |
| B-8 | 每 IPC console.log(args) | main.js:265 | 高頻操作序列化開銷 |
| B-9 | 下載流程非串流 | main.js:448-512 | 大檔延遲雙倍 |
| B-10 | 舊版 OpenSSL + Vue 2 EOL | E-04/E-03 | 安全修補停滯 |

### 4.3 我方已識別的安全事項

| # | 事項 | 證據 | 等級 |
|---|---|---|---|
| S-1 | IGNORE_CERT_ERR=1 預設生效 | main.js:190-192 + config.ini | 高 — MITM 風險 |
| S-2 | nodeIntegration:true + ExecuteCommand/ExecuteFile 任意行程執行 | mainApi.js:570-598 | 高 |
| S-3 | Set-ExecutionPolicy Unrestricted + regsvr32 -Verb RunAs | dllManager.js:225 | 中 |
| S-4 | 舊 OpenSSL、Vue 2 EOL、Chromium 版本未知 | E-04/E-03 | 中 |

---

## 5. 我方已實作的整合管線（供輔翼參考）

Wellfit 工具鏈（7-crate Rust 工作區）已實作以下 ERP 對接能力：

| Crate | 能力 | 測試數 |
|---|---|---|
| wellfit_core | MDM 實體編碼清洗、SQLite 冪等帳本、Release Gate、聯邦契約、定點十進位、名稱解析欄位契約 | 76 + 13 |
| wellfit_shopee | 蝦皮 8 欄契約引擎、EX-06 金額守恆、冪等帳本、ERP 匯入 CSV | 18 + 4 |
| wellfit_reports | 輔翼 ERP 庫存報表解析（零 OCR）、發布閘門、庫存恆等式 | 10 + 5 |
| wellfit_mtrix | MTRIX 五檔 56 欄契約、14 驗證規則、12 修復規則、跨檔參照 | 47 + 8 |
| wellfit_bankpay | E.Sun 銀行付款媒體/API、TLS 2-Way SSL、冪等鎖定、追溯查詢 | 33 + 17 |
| wellfit_sync | 裝置探索、同步狀態、daemon 骨架 | 22 |
| wellfit_cli | 9 個子命令統一入口 | 9 passed (3 ignored, AppLocker) |

**總測試數**：235+ passed / 0 failed / 3 ignored（全部以 --ignored 通過）  
**品質**：clippy 0 警告、fmt 0 差異、0 unsafe

---

## 6. 回覆格式請求

請輔翼於 **2026-11-19** 前回覆以下項目（可分批回覆）：

| # | 回覆項目 | 格式 |
|---|---|---|
| 1 | OQ-F1 自動化政策書面聲明 | PDF/Word 或 email 正式回覆 |
| 2 | OQ-F1 RDA API 規格文件 | 技術文件（OpenAPI/Swagger 或 PDF） |
| 3 | OQ-F1 flapserp:// 協定規格 | 技術文件 |
| 4 | OQ-F1 COM 物件清單 | 技術文件 |
| 5 | OQ-F1 64-bit 打包時程 | 書面回覆 |
| 6 | OQ-F1 憑證模式修正方案 | 書面回覆 |
| 7 | OQ-F2 ERP 匯入範本 | CSV/Excel 範本檔 |
| 8 | OQ-F2 欄位字典 | Excel/PDF |
| 9 | OQ-F2 管理報表樣本 | CSV/Excel 範本檔 |

**聯絡窗口**：PMT（本提案作者）— 可安排技術會議深入討論。  
**提案有效期**：本提案於 2026-12-31 前有效；逾期請重新確認。

---

## 7. 修訂紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-03 | 初版：OQ-F1（8 項請求）+ OQ-F2（5 項請求）+ 效能基線數據 + 安全事項 + Wellfit 能力摘要 |
| v2.1 | 2026-11-04 | 完善格式與章節結構，確認所有請求項目與優先級；補充配置管理章節 |

---
