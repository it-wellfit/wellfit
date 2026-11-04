# 原廠回覆收件與整備手冊（OQ-F1/F2）

**文件編號：** VENDOR-RESP-INTAKE-001  
**版本：** 1.0（Draft）  
**日期：** 2026-11-04  
**適用：** FlapsErp（輔翼）對 OQ-F1/F2 正式需求提案之回覆，及其他原廠文件  
**前置文件：** `docs/research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v1.0.md`（Dev 工作區）、`Knowledge/adr/ADR-0001-source-hierarchy-and-conflict-arbitration.md`

---

## 1. 目的

原廠回覆到手時，以標準程序收件、登錄、驗證、落碼，使「外部輸入 → 知識庫 → 規格 → 程式碼 → 測試」鏈路一次到位，不重工、不推測。本手冊同時定義**回覆到手前即可先備妥**的接收面。

## 2. 收件與登錄（收到當日完成）

1. 原文（PDF/Excel/Email）存入知識庫來源區：`2026/Knowledge/sources/`（依 ADR-0001 來源層級；原廠文件為最高層級來源）。
2. 更新來源登錄：`Knowledge/sources/source-register.md`（檔名、日期、提供者、SHA-256）。
3. 建立逐項對照表（本手冊 §3），每項標記：已回覆／部分回覆／未回覆。
4. **未回覆項目不得以推測填補**；維持 ⚠ 標記並列入後續追蹤。

## 3. 逐項映射與落碼動作（回覆到手後的執行清單）

### OQ-F1（自動化政策與技術通道）

| # | 回覆項目 | 到手動作 | 受影響程式碼／文件 | TDD 順序 |
|---|---|---|---|---|
| F1-1 | 自動化政策書面聲明 | 政策允許 → 更新 ADR-0004 影子/取代階段條件；政策限制 → 記錄禁區 | `plans/adr/ADR-0004` | 文件先行 |
| F1-2 | RDA API 規格 | 建立 RDA 命令對照表；實作 adapter（feature-gated `rda`） | 新模組 `wellfit_core::rda`（規劃）＋ `spec.rs` VerifyStatus 更新 | 先寫契約測試（命令→回應 shape），再實作 |
| F1-3 | flapserp:// 協定規格 | 更新 A1 開頁腳本範本（URI 形狀、欄位、錯誤碼） | `scripts/a1-e2e/Open-FlapsErp.ps1`、`payload-template.json` | 先更新乾跑測試，再改腳本 |
| F1-4 | COM 物件清單 | 更新 COM 呼叫白名單與 ProgID 對照 | flaps_bridge 相關文件／程式碼 | 契約測試先行 |
| F1-5 | 64-bit 打包時程 | 記錄時程；若提供 x64 → 重跑效能基線對照 | `docs/SYNTHETIC_CAPACITY_BASELINE.md` 附註 | — |
| F1-6 | 憑證模式修正 | 若提供正式 TLS 配置 → 移除 `IGNORE_CERT_ERR` 依賴之記錄與 S-1 風險項 | `SECURITY_REVIEW` S 清單、bankpay 無關（FlapsErp 端） | — |
| F1-7 | 非同步 IPC | 記錄；影響批次自動化策略選擇 | roadmap Phase 2 註記 | — |
| F1-8 | 版本變更通知 | 建立 .ver 偵測 webhook/公告管道對照 | flaps_bridge 版本偵測邏輯 | 先寫偵測測試 |

### OQ-F2（資料契約與報表）

| # | 回覆項目 | 到手動作 | 受影響程式碼／文件 | TDD 順序 |
|---|---|---|---|---|
| F2-1 | ERP 匯入範本（CSV/Excel） | 存入 `spec/fixtures/fuyi-erp/`（附 PROVENANCE + SHA-256）；重寫 `shopee_order_contract` 對照 | `wellfit_shopee::shopee_engine` 契約欄位 | **先改契約測試（名稱解析、必填硬失敗），再改實作** |
| F2-2 | 欄位字典 | 建立欄位字典文件；更新驗證規則（長度/格式） | `wellfit_shopee`、`wellfit_reports` 驗證邏輯 | 逐欄測試先行 |
| F2-3 | 管理報表樣本 | 存入 fixtures；與現有 8 欄契約對照（差異清單） | `wellfit_reports::report_automator` | 對照測試先行 |
| F2-4 | 報表欄位定義 | VR-12 總額核對規則更新 | `wellfit_reports` | 核對測試先行 |
| F2-5 | 電商 API 串接文件 | 記錄；評估替代目前 RPA 路徑 | roadmap | — |

## 4. 驗證紀律（每次落碼必守）

1. **名稱解析、必填硬失敗、禁序位退讓**（ColumnContract 不變量）。
2. 範本/樣本進 fixtures 必附出處與 SHA-256（ADR-0003 精神）。
3. 合成資料不得呈現為真實資料；原廠樣本標記來源。
4. 差異（回覆 vs 逆向工程 vs 現行實作）一律列表，不靜默覆蓋。
5. 每項落碼完成 → 更新知識庫（`spec.rs` → 產生腳本 → 對應文件）→ 重跑工作區測試。

## 5. 到手前先備妥的接收面（本手冊隨附狀態）

| 接收面 | 狀態 |
|---|---|
| A1 開頁腳本（flapserp:// 乾跑模式，等 F1-3 校正） | ✔ 已備（`scripts/a1-e2e/`） |
| OQ-F1/F2 提案包 v1.0（8+5 請求項、回覆格式表） | ✔ 已備（Dev: `docs/research/proposals/`） |
| 逐項映射表（本文件 §3） | ✔ 已備 |
| fixtures 目錄（`spec/fixtures/fuyi-erp/`） | ⚠ 待建（收件當日建立並填 PROVENANCE） |
| RDA adapter 骨架 | 暫緩：規格未明前不建（避免投機實作） |

## 6. 責任與時程觸發

| 事件 | 負責 | SLA |
|---|---|---|
| 回覆到手 | PMT | 當日完成 §2 收件登錄 |
| 逐項落碼 | DEV | 每項 ≤ 2 工作日（含測試） |
| 政策/規格差異裁決 | ARCH | 3 工作日內出裁決紀錄 |
| 全案狀態更新 | PMT | 每項完成即更新 `status_ledger` |

**觸發日**：原廠回覆期限 2026-11-19；逾期未回覆 → 維持影子模式軌道，於月報揭露。

## 7. 檢討條件

原廠回覆改變通道可用性（如 RDA 停用）；或回覆內容與交付位元組出現不可調和差異；或 2026-12-31 提案有效期屆滿需重送時。
