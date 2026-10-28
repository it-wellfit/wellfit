# Wellfit BankPay 文件體系 — 入口與導覽

**文件編號**：WFLT-DOC-001  
**版本**：1.0  
**狀態**：Draft  
**適用產品**：Wellfit / Fuyi ERP (FlapsErp) 應付憑單匯款波 — E.Sun Bank 銀企直連整合  
**文件語言**：繁體中文（技術名詞保留標準英文）

---

## 1. 這個文件體系是什麼？

本套文件完整描述 **Wellfit BankPay 整合模組** (`crates/wellfit_bankpay`) 的設計、實作、測試、部署與維護資訊。它讓第一次接觸專案的人，能在不依賴原作者口頭說明的情況下，理解系統目的、找到程式碼位置、完成建置與測試、並安全地進行修改與維護。

本文件體系遵循「證據優先」原則：所有關於程式行為、欄位規格、API 契約與測試結果的陳述，皆以實際原始碼、測試檔與建置設定為依據。未經驗證的假設、規劃中的功能，皆以明確標記識別，不得誤認為既有功能。

---

## 2. 閱讀路徑（循序 vs 目的式查詢）

| 你的身份 | 建議閱讀順序 | 目的 |
|---------|------------|------|
| **新進工程師** | README → DOCUMENTATION_MAP → PROJECT_OVERVIEW → GLOSSARY → CODE_REFERENCE → ENVIRONMENT_BUILD_DEPLOY → TESTING_QUALITY | 先建立全貌，再進入實作 |
| **資深工程師（修改功能）** | CODE_REFERENCE → DATA_MODEL → TRACEABILITY_MATRIX → MAINTENANCE_EVOLUTION | 快速定位要修改的物件與影響範圍 |
| **測試工程師** | TESTING_QUALITY → TRACEABILITY_MATRIX → 06-test-and-acceptance.md | 確認測試覆蓋與驗收標準 |
| **維運 / SRE** | ENVIRONMENT_BUILD_DEPLOY → 07-deployment-and-operations.md → 05-security-and-audit.md | 部署、監控、故障處理 |
| **架構師 / 審查者** | DOCUMENTATION_MAP → PROJECT_OVERVIEW → 03-technical-design.md → QUALITY_REVIEW_REPORT.md | 驗證設計一致性與架構決策 |

**目的式查詢**：遇到特定錯誤碼、API 端點、欄位名稱或設定值時，直接使用 `DOCUMENTATION_MAP.md` 的索引表定位到對應章節。

---

## 3. 文件地圖（DOCUMENTATION_MAP）

完整文件清單與關係請見 [`DOCUMENTATION_MAP.md`](./DOCUMENTATION_MAP.md)。

### 已存在的 BankPay 領域文件（`Wellfit/docs/bankpay/`）

這 8 份文件由 BankPay 領域專用，由領域專家維護：

| 檔案 | 目的 | 主要讀者 |
|------|------|---------|
| [`01-problem-and-scope.md`](bankpay/01-problem-and-scope.md) | 問題陳述、範圍、假設、成功標準 | 產品、架構、開發 |
| [`02-business-workflow.md`](bankpay/02-business-workflow.md) | 端對端業務流程、角色、狀態碼 | 業務、開發、測試 |
| [`03-technical-design.md`](bankpay/03-technical-design.md) | 架構概覽、資料流、媒體檔格式、錯誤策略 | 架構、開發 |
| [`04-interface-specification.md`](bankpay/04-interface-specification.md) | 8 個 API 端點、請求/回應欄位、結果代碼 | 開發、整合測試 |
| [`05-security-and-audit.md`](bankpay/05-security-and-audit.md) | 傳輸安全、認證流程、結果代碼表、資料保護 | 資安、維運 |
| [`06-test-and-acceptance.md`](bankpay/06-test-and-acceptance.md) | 測試策略、測試案例、驗收標準 | QA、開發 |
| [`07-deployment-and-operations.md`](bankpay/07-deployment-and-operations.md) | 部署前置作業、網路拓撲、運營手冊 | 維運、SRE |
| [`08-open-questions-and-decisions.md`](bankpay/08-open-questions-and-decisions.md) | 未決議題、ADR 決策、風險與緩降 | 架構、產品、管理 |

### 本體系新增的橫向文件（`Wellfit/docs/`）

| 檔案 | 目的 | 主要讀者 |
|------|------|---------|
| [`README.md`](./README.md) | 文件入口與導覽（本檔） | 所有讀者 |
| [`DOCUMENTATION_MAP.md`](./DOCUMENTATION_MAP.md) | 文件地圖、閱讀動線、交叉連結索引 | 所有讀者 |
| [`PROJECT_OVERVIEW.md`](./PROJECT_OVERVIEW.md) | 專案全貌、快速入門、環境需求 | 新進工程師 |
| [`GLOSSARY.md`](./GLOSSARY.md) | 領域術語表、英文對照、本專案定義 | 所有讀者 |
| [`CODE_REFERENCE.md`](./CODE_REFERENCE.md) | 物件、屬性、方法、介面、資料結構參考 | 開發、維護 |
| [`DATA_MODEL.md`](./DATA_MODEL.md) | 資料實體、欄位、關係、生命週期 | 開發、測試 |
| [`ENVIRONMENT_BUILD_DEPLOY.md`](./ENVIRONMENT_BUILD_DEPLOY.md) | 建置、執行、部署、監控、除錯 | 維運、開發 |
| [`TESTING_QUALITY.md`](./TESTING_QUALITY.md) | 測試策略、品質關卡、覆蓋率、缺口 | QA、開發 |
| [`MAINTENANCE_EVOLUTION.md`](./MAINTENANCE_EVOLUTION.md) | 擴充、修改、重構、退役指南 | 維護工程師 |
| [`TRACEABILITY_MATRIX.md`](./TRACEABILITY_MATRIX.md) | 需求→設計→實作→測試→驗收追溯 | 架構、QA、管理 |
| [`QUALITY_REVIEW_REPORT.md`](./QUALITY_REVIEW_REPORT.md) | 反向審查結果、已修正缺陷、未決風險 | 審查者、管理 |

---

## 4. 證據標記規範

本文件體系對每個重要陳述使用以下標記：

| 標記 | 意義 | 來源依據 |
|------|------|---------|
| **✔ 已確認** | 有原始碼、測試或設定檔直接支持 | 程式碼路徑、函式名、測試 ID |
| **△ 合理推論** | 由現有證據推導，尚未直接驗證 | 邏輯推導，標註推論依據 |
| **● 設計建議** | 尚未實作，屬於改善方案 | 標記為建議，非既有功能 |
| **⚠ 待確認** | 缺少必要資訊，無法可靠判定 | 需補充證據 |
| **🚫 已知限制** | 系統目前無法做到或不支援 | 程式碼行為或明確設計限制 |

---

## 5. 如何從這裡開始

### 步驟 1：讀自知識庫（若適用）
若你是在 Wellfit 知識庫環境中工作，請先依 `Wellfit/AGENTS.md` §6 的「自知識庫接手」程序，依序讀取治理規則、工作延續模型、研究規約、來源登錄、議題登錄、裁決紀錄與追溯表。

### 步驟 2：閱讀專案概覽
前往 [`PROJECT_OVERVIEW.md`](./PROJECT_OVERVIEW.md) 了解系統是什麼、要解決什麼問題、如何建置與執行。

### 步驟 3：定位程式碼
前往 [`CODE_REFERENCE.md`](./CODE_REFERENCE.md) 查找特定類別、函式或欄位的定義位置與行為說明。

### 步驟 4：建置與測試
前往 [`ENVIRONMENT_BUILD_DEPLOY.md`](./ENVIRONMENT_BUILD_DEPLOY.md) 依照指令建置與執行測試。

---

## 6. 已知限制與未決事項

| 項目 | 說明 | 狀態 |
|------|------|------|
| 沙箱憑證不可用 | E.Sun 沙箱（OQ-E3）憑證尚未取得，無法對真實安全元件進行端對端測試 | ⚠ 待確認（阻塞 P1 驗收） |
| TLS 憑證釘固 | `http.rs` 的 `EsunHttpClient` 尚未實作憑證釘固，僅支援基本 2-Way SSL | ● 設計建議 |
| 其他 6 個 API 端點 | 僅實作 `createTwdPaymentInstruction` 與 `retrieveTwdPaymentInstructionResult`；其餘 6 個端點（對帳單、FX 等）未實作 | 🚫 已知限制（本階段外） |
| Windows AppLocker | 本工作站應用程式控制原則阻擋桌面路徑下執行檔，必須透過 `build.ps1` 導向 temp 目錄 | 🚫 已知限制（環境限制） |

---

## 7. 文件維護責任

| 文件 | 責任人 | 更新時機 |
|------|--------|---------|
| `DOCUMENTATION_MAP.md` | 文件架構師 | 新增/移除文件時 |
| `CODE_REFERENCE.md` | 資深軟體工程師 | 新增/修改類別、函式、欄位時 |
| `DATA_MODEL.md` | 資深軟體工程師 | 新增/修改資料欄位、關係時 |
| `TESTING_QUALITY.md` | QA 工程師 | 新增/修改測試案例時 |
| `ENVIRONMENT_BUILD_DEPLOY.md` | SRE 工程師 | 部署流程或設定變更時 |
| `QUALITY_REVIEW_REPORT.md` | 獨立技術審查者 | 每次反向審查後 |

所有文件變更必須同步更新 `CHANGELOG.md`，並重新執行 `cargo clippy --workspace --all-targets -- -D warnings` 與 `cargo test --workspace --no-fail-fast` 確保程式碼與文件一致。

---

## 8. 反饋與問題回報

若發現文件與程式碼不一致、缺少重要資訊或標記錯誤，請：

1. 在 `QUALITY_REVIEW_REPORT.md` 中記錄發現
2. 修正可靠修正的內容
3. 對無法確認的內容，加入 `⚠ 待確認` 標記
4. 重新驗證受影響的章節、連結與追溯關係

---

*本文件體系遵循 Wellfit AGENTS.md (WFLT-AGNT-001) 治理規則。契約事實的唯一來源是跨專案知識庫與實際程式碼，不是網路或既有文件敘述。*
