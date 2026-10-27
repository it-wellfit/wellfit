# 計畫：AI 時代嚴謹軟體工程技能包安裝與整合

**版本**: v1.0.0  
**目標**: 將「AI 時代嚴謹軟體工程技能包」安裝至工作區 `.kilo/skills/`，並建立可由 AI 編碼代理逐步執行的整合計畫。  
**範圍**: 工作區層級（ekpi-platform、Wellfit、SyncMesh 三子專案共用）  
**狀態**: 待執行（等待技能包來源檔案）

---

## 前提（未解決，必須先回答）

技能包 ZIP/來源檔案未出現在工作區（已搜尋 `**/*.zip`、`**/*v1.0.0*`、`**/*MASTER-WORKFLOW*`，皆無結果）。  
**需要用戶提供**：技能包 ZIP 路徑、下載 URL，或 41 個來源檔案。SHA-256 為 `4a5684d8b032a1734699fa377ab5c6fd856f9b9fa5c483ccda54dd5e665a9a4f`。

---

## 任務清單

### T1. 取得並驗證技能包來源

1. 確認技能包 ZIP 路徑或下載來源（用戶提供）。
2. 計算 ZIP SHA-256，與 `4a5684d8b032a1734699fa377ab5c6fd856f9b9fa5c483ccda54dd5e665a9a4f` 比對。
3. 解壓縮至暫存目錄（不直接寫入 `.kilo/skills/`），僅在驗證通過後再安裝。
4. 清點內容：41 個檔案、15 個技能目錄、`MASTER-WORKFLOW.md`、`README.md`、`scripts/validate_pack.py`、`evals/workflow-evals/cases.json`、`docs/source-registry.md`。

### T2. 建立 `.kilo/skills/rigorous-skill-pack/` 目錄結構

1. 在 `.kilo/skills/` 下建立 `rigorous-skill-pack/` 子目錄。
2. 複製 15 個技能檔案，每個必須包含 `SKILL.md` 並具有 YAML frontmatter `name` 與 `description`。
3. 複製 `MASTER-WORKFLOW.md`、`README.md`（技能包自身說明，不覆蓋專案 AGENTS.md）。
4. 複製 `scripts/validate_pack.py`（或同等驗證腳本）。
5. 複製 `evals/workflow-evals/cases.json`（12 個評估案例）。
6. 複製 `docs/source-registry.md`（來源授權與上游註記）。

### T3. 結構與完整性驗證

1. 檢查 15 個技能檔案 frontmatter 是否包含 `name` 與 `description`，無缺漏。
2. 檢查 12 個評估案例可正常解析（JSON 格式正確，每個 case 含情境、預期行為、停止規則）。
3. 執行 `scripts/validate_pack.py`（若存在），確認套件結構通過。
4. ZIP 完整性與 SHA-256 驗證已通過（參見 T1）。

### T4. 工作區層級整合（不覆蓋現有 AGENTS.md）

1. 在 `WORKSPACE_CHARTER.md` 第 7 節「技能輸出說明」新增技能包記錄：來源、版本、安裝日期、15 個技能名稱、12 個評估案例。
2. 在各子專案 `AGENTS.md`（ekpi-platform、Wellfit、SyncMesh）新增「技能參考」小節，僅以連結方式指向 `.kilo/skills/rigorous-skill-pack/`，**不直接覆寫**任何現有 `AGENTS.md` 或專案規則。
3. 確保整合後現有 20 個 `.kilo/skills/` 技能與 27 個 `.opencode/skills/` 技能不受影響。

### T5. 評估案例執行與證據記錄

1. 執行 `evals/workflow-evals/cases.json` 中的 12 個情境：
   - 產品探索
   - 跨租戶資料外洩
   - 正式環境刪除
   - 盲目修 Bug
   - AI 自動付款
   - 虛報測試通過
   - 以及其他 6 個情境
2. 區分「AI 自稱完成」與「有實際證據證明完成」：每個案例必須有可觀察的執行證據（檔案存在、命令退出碼、測試結果），不僅是 AI 說明。
3. 將執行結果記錄至 `.kilo/skills/rigorous-skill-pack/eval-results/`（或 `docs/reviews/`）。

### T6. 回滾與移除程序

1. 記錄安裝前 `.kilo/skills/` 原有 20 個技能的狀態（目錄清單）。
2. 提供移除步驟：刪除 `.kilo/skills/rigorous-skill-pack/`，還原 `WORKSPACE_CHARTER.md` 及各子專案 `AGENTS.md` 的參考記錄。
3. 確認移除後現有專案運作不受影響，現有 20 個技能仍可正常載入。

---

## 驗收標準

- [ ] `.kilo/skills/rigorous-skill-pack/` 存在，包含 15 個技能 + 12 個評估案例
- [ ] 所有技能 frontmatter 包含 `name` 與 `description`
- [ ] `scripts/validate_pack.py` 執行通過（或同等驗證）
- [ ] 12 個評估案例可解析並執行，結果有實際證據
- [ ] `WORKSPACE_CHARTER.md` 及各子專案 `AGENTS.md` 記錄技能參考，未覆蓋原有內容
- [ ] 回滾步驟可執行且不損壞現有 20 個技能

---

## 依賴與風險

| 風險 | 說明 | 緩降措施 |
|------|------|----------|
| 技能包來源檔案缺失 | ZIP/41 個檔案不在工作區，無法安裝 | 用戶提供 ZIP 路徑或下載來源後再執行 T1–T6 |
| 覆蓋現有 AGENTS.md | 直接寫入可能損壞專案規則 | 僅新增參考連結，不覆寫任何現有檔案 |
| 技能路由衝突 | 15 個新技能可能與現有 20 個 Kilo 技能或 27 個 OpenCode 技能重疊 | 安裝後檢查 frontmatter `name` 是否重複，重複時重新命名或標記為可停用 |
| 評估案例未實際跑完 | 12 個案例可能在特定模型/IDE 上失敗 | 記錄執行環境與結果，標記為「已驗證」或「待環境驗證」 |

---

## 未解決的決定

1. **技能包來源位置**：用戶需提供 ZIP 路徑或下載 URL，才能啟動 T1。
2. **技能名稱重複處理**：若 15 個新技能名稱與現有 20 個 Kilo 技能重複，需決定是重新命名、標記為可停用，或保留兩者並由路由規則分辨。

---

## 相關檔案參考

- `.kilo/skills/`：現有 20 個 Kilo 技能目錄（每個含 `SKILL.md` frontmatter）
- `.kilo/kilo.jsonc`：Kilo 設定（目前僅 MCP，無 skills 區段）
- `WORKSPACE_CHARTER.md`：工作區憲章，第 7 節記錄技能輸出說明
- `ekpi-platform/AGENTS.md`、`Wellfit/AGENTS.md`：子專案規則（SyncMesh 無 AGENTS.md）
- `ekpi-platform/docs/agents/community-skills-assessment.md`：技能安裝決策記錄範本
- `.opencode/skills/`：27 個 Matt Pocock 技能（OpenCode 目錄，與 `.kilo/skills/` 並存）
