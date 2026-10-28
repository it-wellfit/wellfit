# Wellfit BankPay 文件地圖

**文件編號**：WFLT-DOC-002  
**版本**：1.0  
**狀態**：Draft  
**適用範圍**：`Wellfit/crates/wellfit_bankpay` 整合模組及其相關 CLI、測試、設定  
**文件語言**：繁體中文（技術名詞保留標準英文）

---

## 1. 文件體系總覽

本文件地圖列出所有與 BankPay 整合相關的文件，說明每份文件的目的、主要讀者、內容範圍，以及與其他文件的關聯。文件分為 **BankPay 領域文件**（已存在，`docs/bankpay/`）與 **橫向工程文件**（新增，`docs/`）。

### 1.1 文件領域分類

| 領域 | 檔案位置 | 文件數 | 說明 |
|------|---------|--------|------|
| 專案入口與導覽 | `docs/` | 2 | README、DOCUMENTATION_MAP |
| 專案概覽與快速入門 | `docs/` | 1 | PROJECT_OVERVIEW |
| 領域術語 | `docs/` | 1 | GLOSSARY |
| 問題與範圍 | `docs/bankpay/` | 1 | 01-problem-and-scope.md |
| 業務流程 | `docs/bankpay/` | 1 | 02-business-workflow.md |
| 技術設計 | `docs/bankpay/` | 1 | 03-technical-design.md |
| 介面規格 | `docs/bankpay/` | 1 | 04-interface-specification.md |
| 資安與稽核 | `docs/bankpay/` | 1 | 05-security-and-audit.md |
| 測試與驗收 | `docs/bankpay/` | 1 | 06-test-and-acceptance.md |
| 部署與營運 | `docs/bankpay/` | 1 | 07-deployment-and-operations.md |
| 未決議題與決策 | `docs/bankpay/` | 1 | 08-open-questions-and-decisions.md |
| 程式碼參考 | `docs/` | 1 | CODE_REFERENCE |
| 資料模型 | `docs/` | 1 | DATA_MODEL |
| 環境與建置 | `docs/` | 1 | ENVIRONMENT_BUILD_DEPLOY |
| 測試與品質 | `docs/` | 1 | TESTING_QUALITY |
| 維護與擴充 | `docs/` | 1 | MAINTENANCE_EVOLUTION |
| 追溯矩陣 | `docs/` | 1 | TRACEABILITY_MATRIX |
| 品質審查報告 | `docs/` | 1 | QUALITY_REVIEW_REPORT |

**總計**：20 份文件（8 份領域 + 12 份橫向）

---

## 2. 縱向知識鏈（目標 → 實作）

縱向知識鏈讓讀者從高層目標逐步追溯到實際程式碼。每個節點都有對應的文件：

```
專案目標
    ↓ (WFLT-DOC-003 PROJECT_OVERVIEW.md)
需求與範圍
    ↓ (docs/bankpay/01-problem-and-scope.md)
業務流程
    ↓ (docs/bankpay/02-business-workflow.md)
系統架構與設計
    ↓ (docs/bankpay/03-technical-design.md)
API 介面契約
    ↓ (docs/bankpay/04-interface-specification.md)
程式碼實作（物件、方法、屬性）
    ↓ (docs/CODE_REFERENCE.md)
資料模型與生命週期
    ↓ (docs/DATA_MODEL.md)
環境、建置、部署
    ↓ (docs/ENVIRONMENT_BUILD_DEPLOY.md)
測試與品質保證
    ↓ (docs/TESTING_QUALITY.md)
維護、擴充、交接
    ↓ (docs/MAINTENANCE_EVOLUTION.md)
```

---

## 3. 橫向依賴關係索引

| 依賴方 | 被依賴方 | 關係類型 | 文件位置 |
|--------|---------|---------|---------|
| `wellfit_bankpay` | `wellfit_core` | 型別依賴（ColumnContract, IdempotencyLedger, ReleaseGate, ScaledDecimal） | CODE_REFERENCE §4.1 |
| `wellfit_cli` | `wellfit_bankpay` | CLI 呼叫 BankPay Engine | CODE_REFERENCE §4.2 |
| `wellfit_bankpay` | `ureq` + `rustls` | HTTP/TLS 傳輸 | CODE_REFERENCE §4.3 |
| `wellfit_bankpay` | `encoding_rs` | 編碼轉換（Big5/CP950） | CODE_REFERENCE §4.4 |
| `bankpay` Export | `ReleaseGate` | 發布閘門狀態機 | CODE_REFERENCE §5.3 |
| `bankpay` 查詢 | `IdempotencyLedger` | 追溯查詢資料來源 | CODE_REFERENCE §5.4 |
| 所有 API 呼叫 | `Finding` 記錄 | 稽軌跡跡 | DATA_MODEL §3 |
| TLS 憑證 | `EsunHttpClientConfig` | 憑證路徑設定 | CODE_REFERENCE §4.3 |
| 媒體檔組裝 | `TwdMediaRecord` | 422 字元固定長度 | CODE_REFERENCE §5.1 |

---

## 4. 文件清單與交叉連結表

| # | 檔案 | 目的 | 主要讀者 | 交叉連結到 |
|---|------|------|---------|-----------|
| 1 | `docs/README.md` | 文件入口與導覽 | 所有讀者 | 所有文件 |
| 2 | `docs/DOCUMENTATION_MAP.md` | 文件地圖（本檔） | 所有讀者 | 所有文件 |
| 3 | `docs/PROJECT_OVERVIEW.md` | 專案全貌、快速入門 | 新進工程師 | 1, 4, 10, 11 |
| 4 | `docs/GLOSSARY.md` | 領域術語表 | 所有讀者 | 3, 5, 6 |
| 5 | `docs/CODE_REFERENCE.md` | 程式碼物件、方法、屬性參考 | 開發、維護 | 4, 6, 7 |
| 6 | `docs/DATA_MODEL.md` | 資料實體、欄位、生命週期 | 開發、測試 | 4, 5, 7 |
| 7 | `docs/ENVIRONMENT_BUILD_DEPLOY.md` | 建置、執行、部署、監控 | 維運、開發 | 3, 11 |
| 8 | `docs/TESTING_QUALITY.md` | 測試策略、品質關卡 | QA、開發 | 5, 6, 14 |
| 9 | `docs/MAINTENANCE_EVOLUTION.md` | 擴充、修改、退役指南 | 維護工程師 | 5, 6, 14 |
| 10 | `docs/TRACEABILITY_MATRIX.md` | 需求→設計→實作→測試追溯 | 架構、QA | 11, 12, 13 |
| 11 | `docs/QUALITY_REVIEW_REPORT.md` | 反向審查結果與缺陷 | 審查者、管理 | 10 |
| 12 | `docs/bankpay/01-problem-and-scope.md` | 問題陳述、範圍、假設 | 產品、架構 | 2, 3, 10 |
| 13 | `docs/bankpay/02-business-workflow.md` | 業務流程、角色、狀態碼 | 業務、開發 | 2, 4, 12 |
| 14 | `docs/bankpay/03-technical-design.md` | 架構、資料流、媒體格式 | 架構、開發 | 2, 5, 13 |
| 15 | `docs/bankpay/04-interface-specification.md` | API 端點、欄位、結果代碼 | 開發、整合測試 | 2, 5, 13 |
| 16 | `docs/bankpay/05-security-and-audit.md` | 傳輸安全、認證、稽核 | 資安、維運 | 2, 7, 13 |
| 17 | `docs/bankpay/06-test-and-acceptance.md` | 測試策略、測試案例、驗收 | QA、開發 | 8, 13, 14 |
| 18 | `docs/bankpay/07-deployment-and-operations.md` | 部署、網路、運營手冊 | 維運、SRE | 7, 16 |
| 19 | `docs/bankpay/08-open-questions-and-decisions.md` | 未決議題、ADR、風險 | 架構、產品 | 10, 11 |

---

## 5. 讀者路徑速查

| 你的問題 | 建議先讀的文件 | 接著讀 |
|---------|--------------|--------|
| 這個系統是什麼？為什麼需要它？ | `PROJECT_OVERVIEW.md` §1 | `01-problem-and-scope.md` |
| 如何建置並執行測試？ | `ENVIRONMENT_BUILD_DEPLOY.md` §3 | `TESTING_QUALITY.md` |
| 特定函式/類別在哪裡定義？ | `CODE_REFERENCE.md` | 實際原始碼 |
| 欄位 X 的長度與格式是什麼？ | `04-interface-specification.md` §2 | `CODE_REFERENCE.md` §5 |
| 部署需要什麼憑證與埠號？ | `07-deployment-and-operations.md` | `05-security-and-audit.md` |
| 測試覆蓋率與品質關卡？ | `TESTING_QUALITY.md` | `06-test-and-acceptance.md` |
| 修改某個欄位會影響什麼？ | `MAINTENANCE_EVOLUTION.md` | `TRACEABILITY_MATRIX.md` |
| API 呼叫失敗如何診斷？ | `05-security-and-audit.md` §4 | `07-deployment-and-operations.md` §5 |
| 未決議題與風險？ | `08-open-questions-and-decisions.md` | `QUALITY_REVIEW_REPORT.md` |

---

## 6. 文件一致性規則

1. **單一可信來源**：同一項規格（欄位長度、API 端點、結果代碼）以 `04-interface-specification.md` 為主要定義位置，其他文件以連結引用，不得複製。
2. **名稱統一**：程式碼中的類別名、函式名、欄位名必須與文件中完全一致。例如 `EsunHttpClientConfig`、`build_twd_payment_request`、`TwdMediaRecord`。
3. **證據標記**：每個技術陳述必須標記證據等級（✔ 已確認 / △ 合理推論 / ● 設計建議 / ⚠ 待確認 / 🚫 已知限制）。
4. **版本同步**：文件版本號必須與程式碼變更同步。當 `Cargo.toml` 版本變更時，相關文件需審查。

---

## 7. 文件變更程序

| 步驟 | 動作 | 負責角色 |
|------|------|---------|
| 1 | 識別需要變更的文件與章節 | 發現變更需求的人員 |
| 2 | 讀取 `DOCUMENTATION_MAP.md` 確認相關文件 | 變更提出者 |
| 3 | 修改文件內容，保持證據標記 | 相關負責角色 |
| 4 | 交叉檢查其他文件是否有矛盾 | 文件架構師 |
| 5 | 執行 `cargo clippy --workspace --all-targets -- -D warnings` | 開發者 |
| 6 | 執行 `cargo test --workspace --no-fail-fast` | 開發者 |
| 7 | 更新 `CHANGELOG.md` | 變更提出者 |
| 8 | 更新 `QUALITY_REVIEW_REPORT.md`（若發現缺陷） | 獨立技術審查者 |

---

*本文件地圖是 Wellfit BankPay 文件體系的導航核心。任何新增、移除或重大修改文件的動作，必須先更新本檔。*
