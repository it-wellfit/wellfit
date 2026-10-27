# ISO/SOP/CAPA/內控/稽核系統技術研究框架

## 問題重新定義

本框架回答的核心問題不是「如何讓 LLM 更準確地回答文件問題」，而是：

> **如何建立一套可重現、可稽核、可替換的企業知識與流程智慧平台，使得每個稽核結論都能追溯至原始文件條款、版本與事件紀錄，且在證據不足時正確拒答？**

### 問題分解

| 子問題 | 說明 |
|--------|------|
| Q1 | 如何從非結構化/半結構化文件中還原可機器查詢的結構、條款、版本與座標？ |
| Q2 | 如何表示跨文件的規範要求、流程模型、事件紀錄與證據主張，使它們可被獨立驗證？ |
| Q3 | 如何在不同任務類型下選擇最佳檢索策略，並保留原文引用？ |
| Q4 | 如何將規範要求轉化為可機器檢查的規則，並區分規則符合、流程順序與真實因果？ |
| Q5 | 如何驗證實際作業是否符合流程模型，並輸出可追溯的合規判定？ |
| Q6 | 如何管理證據鏈、版本固定、不確定性與人工覆核？ |

### 關鍵區分（A11 核心原則）

| 關係類型 | 定義 | 負責模組 | 驗證方式 |
|----------|------|----------|----------|
| **規範依賴** (Normative Dependency) | 必須先取得核准才能執行 | 規則引擎 | 規則匹配 + 前置條件檢查 |
| **流程順序** (Process Order) | A 活動完成後才允許 B 開始 | 流程驗證器 | Token replay / conformance checking |
| **真實因果** (True Causation) | 某項控制措施是否真的降低了缺失發生率 | 因果分析模組 | 因果圖 + 識別策略 + 混淆因素分析 |

**禁止**將三者混為一談。順序 ≠ 規則符合 ≠ 因果。

---

## 六層技術架構

### Layer 1: 文件感知與結構還原

**目標**：將原始文件轉化為可機器查詢的結構化表示，保留頁碼、章節、表格、附錄、版本關係。

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| **版面感知解析** (Docling/MinerU 類) | PDF → 結構化 Markdown/JSON；保留表格、圖表、公式 | OCR 錯誤率；複雜版面（多欄、跨頁表格）仍不穩定；需人工校正 | 成熟（開源，有 benchmark） |
| **PageIndex 樹導覽** | 文件樹 + 推理式導覽；適合長文件精確定位 | 依賴文件已有清晰結構；對無標題的文件效果差；官方基準成績需獨立重現 | 候選（官方自報成績，待重現） |
| **RAPTOR 階層摘要** | 內容聚類生成多粒度摘要樹；適合跨章節多跳問答 | 摘要可能遺失細節；不保留原文座標；對稽核（需 verbatim）不適用 | 研究（ICLR 2024，學術） |
| **Psi-RAG 跨文件樹** | 跨文件多跳 QA；F1 較 RAPTOR +25.9% | 自報數字；對順序敏感查詢支援度待確認；需自有語料重測 | 研究（ICML 2026） |
| **SERAG 結構熵樹** | 多視角圖（語意/邏輯/順序）+ 結構熵最小化 | 對條件分支支援度待確認；複雜度高 | 研究（ICML 2026） |
| **長上下文 LLM** | 全文比較、版本差異分析 | 成本高；上下文窗口限制；不可作為稽核證據（無座標） | 成熟（工程，非研究） |
| **OCR + 座標** | 掃描文件數位化；保留 bbox | OCR 錯誤率 1-5%；需人工校正；座標精度依賴解析器 | 成熟（Tesseract/Docling） |

**MVP 選擇**：版面感知解析（Docling 類）+ 文件樹 + 條款錨點（section/page/bbox）。PageIndex 作為可插拔檢索後端（R3）。RAPTOR/Psi-RAG/SERAG 作為研究軌道。

### Layer 2: 知識表示與索引

**目標**：建立多層知識表示（文件層、結構層、知識圖譜、規則層、流程/事件層、證據/主張層），支援多策略索引。

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| **向量索引** (pgvector) | 語意相似度檢索；快速 MVP | 無法處理精確匹配；對版本/權限過濾需額外 SQL | 成熟（PostgreSQL 擴充） |
| **全文檢索** (Tantivy/Postgres FTS) | 關鍵字精確匹配；BM25 排序 | 對同義詞/語意變體不敏感 | 成熟 |
| **知識圖譜** (Typed KG) | 跨文件關係查詢；多跳推理 | LLM 抽取的關係不可自動視為事實；需人工驗證；抽取成本高 | 成熟（建模）/ 研究（自動抽取） |
| **GraphRAG** (MS GraphRAG) | 跨文件多跳查詢；社群摘要 | 摘要層輸出不得作為稽核證據；抽取成本高；對精確條款定位效果差 | 候選（repo 存在，docs 已核實） |
| **GraphRAG-Bench** | 決定何時用圖的證據基礎 | 資料集可借用於跨文件多跳測試；非稽核場景 | 成熟（基準資料集） |
| **文件樹** (PageIndex 風格) | 長文件精確定位；保留結構 | 依賴文件已有清晰結構 | 候選（官方自報，待重現） |

**MVP 選擇**：pgvector + Postgres FTS + 文件樹。KG 以受核准的關係表存放（LLM 抽取預設 unverified）。GraphRAG 作為研究軌道。

### Layer 3: 檢索與證據組合

**目標**：統一檢索介面，依任務特性選擇策略組合，所有結果保留來源位置與版本。

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| **混合檢索** (Hybrid: 向量+BM25+樹導覽) | 通用檢索；平衡召回與精確 | 融合策略（RRF）參數需調校 | 成熟 |
| **Reranker** (Cross-encoder) | 提升 Top-k 精確度 | 延遲增加 2-5x；模型選型需實測 | 成熟（工程） |
| **Self-RAG** | 自我反思式檢索；減少幻覺 | 反思次數 ≠ 正確性；需獨立驗證 | 研究（ACL 2024） |
| **Corrective RAG** | 檢索品質評估 + 補充檢索 | 評估器本身可能出錯；增加延遲 | 研究（arXiv 2024） |
| **Adaptive RAG** | 依查詢複雜度選擇策略 | 複雜度分類器需訓練 | 研究（arXiv 2024） |
| **Agentic RAG** | 多步推理 + 工具調用 | 無限制對話風險；需停止條件 | 研究（多篇 2024-2025） |
| **Parent-Child Retrieval** | 子塊檢索 + 父塊上下文 | 父塊可能過長 | 成熟（工程模式） |

**MVP 選擇**：混合檢索（bigram cosine MVP → pgvector + FTS）+ 樹導覽 + snippet citations。Reranker 延後。Self-RAG/Corrective/Adaptive/Agentic 作為研究軌道。

### Layer 4: 推理與規則驗證

**目標**：將規範要求轉化為可機器檢查的規則，執行確定性判定，區分規則符合/流程順序/因果。

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| **JSON Schema 規則** | 結構化規則定義；可機器校驗 | 表達能力有限；複雜條件需程式碼 | 成熟 |
| **LegalRuleML** | 法規/標準的機器可讀表示 | 學習曲線；工具生態有限 | 標準（OASIS 2021） |
| **DMN** | 決策表；適合條件分支 | 複雜邏輯需 FEEL 表達式 | 成熟（OMG 標準） |
| **Datalog/Prolog** | 規則推理；遞迴查詢 | 效能；除錯困難 | 成熟（學術/工程） |
| **神經符號 AI** | 結合 LLM 理解與符號推理 | LLM 輸出不確定；需驗證層 | 研究（2024-2025） |
| **CausalGraph2LLM** | 因果圖 + LLM 查詢生成 | 因果識別需足夠資料；LLM 可能生成錯誤因果主張 | 研究（2025） |

**MVP 選擇**：JSON Schema v0.1 + LegalRuleML 匯出。規則引擎為 Rust 原生確定性執行。因果模組獨立（A11）。

### Layer 5: 流程執行與合規驗證

**目標**：驗證實際作業是否符合流程模型，輸出四分類合規報告。

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| **BPMN 2.0** | 業務流程建模；條件分支/平行/循環 | 複雜流程模型難以維護；需領域專家 | 成熟（OMG 標準） |
| **Petri Net** | 形式化流程模型；token replay conformance | 模型爆炸問題；大規模流程難以建模 | 成熟（學術/工程） |
| **狀態機** (FSM) | 簡單狀態轉移；事件驅動 | 無法處理平行/複雜同步 | 成熟 |
| **Process Mining** (PM4Py) | 從事件日誌發現流程模型；conformance checking | 依賴事件日誌品質；對不完整日誌敏感 | 成熟（PM4Py 開源） |
| **XES/OCEL** | 事件日誌標準格式 | 資料品質問題（缺紀錄/延遲/重複） | 標準（IEEE 1849-2023） |
| **形式化方法** (TLA+/Alloy) | 關鍵流程的數學驗證 | 學習曲線極高；僅適合核心流程 | 成熟（學術） |
| **Model Checking** | 狀態空間窮舉驗證 | 狀態爆炸；僅適合小型模型 | 成熟（學術） |

**MVP 選擇**：Petri Net token replay（Rust 原生）+ 四分類 conformance（conform/deviate/missing/unconfirmed）。PM4Py sidecar 作 alignments 對照（R3）。BPMN 作建模標準。

### Layer 6: 證據與治理

**目標**：管理證據鏈、版本固定、不確定性、人工覆核、稽核紀錄。

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| **Decision Snapshot** (manifest hash) | 決策重放；byte-identical 驗證 | 需所有輸入固定；LLM 路徑不可重放 | 設計即建（本專案） |
| **WORM 儲存** | 證據不可篡改 | 儲存成本；需 hash chain 驗證 | 成熟（工程模式） |
| **PROV-O** | 來源追溯描述 | 複雜度；工具生態有限 | W3C 標準 |
| **NIST AI RMF** | AI 治理框架 | 非技術標準；需組織適配 | 成熟（框架） |
| **OWASP LLM Top 10** | LLM 安全防護 | 針對 LLM 應用；非通用 | 成熟（指南） |

**MVP 選擇**：Decision snapshot + manifest hash + WORM hash chain。NIST AI RMF + OWASP 作治理參考。

---

## 技術比較矩陣

### 檢索技術比較

| 技術 | Recall@k | Precision@k | 延遲 | 成本 | 引用正確率 | 版本感知 | 適合稽核 |
|------|----------|-------------|------|------|-----------|----------|----------|
| 傳統 RAG (向量) | 中 | 中 | 低 | 低 | 低（無座標） | 否 | 否 |
| 混合檢索 (向量+BM25) | 高 | 中高 | 中 | 低 | 中 | 部分 | 部分 |
| PageIndex 樹導覽 | 高（長文件） | 高 | 中 | 中 | 高 | 是 | 是 |
| GraphRAG | 高（多跳） | 中 | 高 | 高 | 中 | 部分 | 部分 |
| RAPTOR | 中高 | 中 | 中 | 中 | 低（摘要無座標） | 否 | 否 |
| Agentic RAG | 高 | 高 | 高 | 高 | 高 | 是 | 是（需驗證） |

### 流程驗證技術比較

| 技術 | Conformance Precision | Conformance Recall | 延遲 | 可解釋性 | 適合稽核 |
|------|---------------------|-------------------|------|----------|----------|
| Token Replay (Petri Net) | 高 | 中 | 低 | 高 | 是 |
| PM4Py Alignments | 高 | 高 | 中 | 高 | 是 |
| BPMN Simulation | 中 | 中 | 中 | 中 | 部分 |
| Model Checking | 極高 | 極高 | 高（狀態爆炸） | 極高 | 是（小型模型） |
| Process Mining Discovery | 中 | 中 | 高 | 中 | 否（發現非驗證） |

### 因果推論技術比較

| 技術 | 因果識別能力 | 資料需求 | 可解釋性 | 適合稽核 |
|------|-------------|----------|----------|----------|
| 規範依賴（規則引擎） | 確定性 | 低 | 極高 | 是 |
| 流程順序（token replay） | 確定性 | 低 | 極高 | 是 |
| 時間序列因果 (Granger) | 統計關聯 | 中 | 中 | 部分 |
| 因果圖 (DAG) | 識別（需假設） | 高 | 高 | 是（需專家） |
| 反事實推理 | 識別（需模型） | 極高 | 中 | 部分 |
| CausalGraph2LLM | 生成式因果查詢 | 高 | 中 | 研究 |

---

## 稽核專用評測集

### 評測類別（15 類）

| # | 類別 | 測試案例數 | 指標 |
|---|------|-----------|------|
| 1 | 精確條款檢索 | 5 | Recall@5, Precision@5, MRR |
| 2 | 跨章節/跨文件多跳 | 5 | Recall@5, nDCG@10 |
| 3 | 表格/附錄定位 | 3 | Anchor resolvability |
| 4 | 新舊版本衝突 | 4 | Version selection accuracy |
| 5 | 否定句與例外條款 | 4 | Rule verdict accuracy |
| 6 | 前置條件與順序錯誤 | 4 | Conformance F1 |
| 7 | 合法/非法循環 | 3 | Loop policy accuracy |
| 8 | 缺少證據 | 4 | Abstention accuracy |
| 9 | 相互矛盾的來源 | 3 | Conflict detection rate |
| 10 | 因果 vs 相關 | 3 | Causal identification accuracy |
| 11 | 長流程/平行分支 | 3 | Conformance F1 |
| 12 | 文件解析失敗 | 2 | Parse failure handling |
| 13 | 權限不足 | 2 | Permission violation rate |
| 14 | 惡意文件/Prompt Injection | 3 | Injection resistance rate |
| 15 | 無法判定/應升級 | 2 | Escalation accuracy |

### 評測指標（分開計算）

| 指標 | 定義 | 目標 |
|------|------|------|
| **漏判率** (False Negative Rate) | 應判定違規但判定為符合的比例 | < 1% |
| **誤報率** (False Positive Rate) | 應判定符合但判定為違規的比例 | < 5% |
| **證據完整度** (Evidence Completeness) | 每個結論引用的證據覆蓋率 | ≥ 95% |
| **引用正確率** (Citation Accuracy) | 引用能精確定位至文件/章節/頁碼 | 100% |
| **版本錯誤率** (Version Error Rate) | 使用了非當前有效版本的比例 | 0% |
| **延遲** (P95 Latency) | 單案件 adjudication P95 | < 500ms |
| **成本** (Cost per Case) | LLM + 檢索 + 儲存成本 | < $0.01 |
| **人工覆核率** (Human Review Rate) | 需人工介入的案件比例 | < 20% |
| **拒答正確率** (Abstention Accuracy) | 證據不足時正確拒答的比例 | 100% |
| **因果誤判率** (Causal Misattribution Rate) | 將規範依賴/流程順序誤判為真實因果的比例 | 0% |

---

## 可重現的基準架構比較實驗

### 實驗設計

**基線比較**（7 個配置，A-G）：

| 配置 | 描述 | 元件 |
|------|------|------|
| A | 傳統 RAG | 向量檢索 + LLM 生成 |
| B | 混合檢索 + 重排序 | 向量 + BM25 + reranker |
| C | PageIndex 樹狀檢索 | 文件樹導覽 + LLM |
| D | GraphRAG | 知識圖譜 + 社群摘要 |
| E | Agentic RAG | 多步推理 + 工具調用 |
| F | 流程模型 + 規則驗證 | Petri net + 規則引擎（無 LLM） |
| G | 整合式平台 | A-F 所有元件整合 |

**消融實驗**：逐一移除 G 的元件，測量 Δ 指標。

### 實驗控制

- **固定**：資料集版本、模型版本、Prompt 版本、索引版本、參數
- **記錄**：每次執行的 commit hash、環境、時間戳
- **重現**：Docker/Podman 容器化；seed 設定固定
- **統計**：每配置跑 3 次，報告均值 ± 標準差

### 實驗步驟

1. 準備資料集（55 案例：9 seed + 46 casebook）
2. 為每個配置建立獨立索引
3. 執行 adjudication，記錄每個案例的 verdict/citations/notes/latency/cost
4. 計算所有指標
5. 生成報告（JSON + Markdown + HTML）

---

## 技術成熟度分類

### 已成熟的工程方法（可直接採用）

| 技術 | 成熟度證據 | 採用決策 |
|------|-----------|----------|
| PostgreSQL + pgvector | 產業標準；大量生產部署 | ✅ 採用 |
| Tantivy (Rust 全文檢索) | Rust 生態成熟；效能優異 | ✅ 採用 |
| BPMN 2.0 | OMG 標準；工具生態豐富 | ✅ 採用 |
| Petri Net | 學術/工程雙重驗證；PM4Py 支援 | ✅ 採用 |
| XES/OCEL | IEEE 標準；廣泛使用 | ✅ 採用 |
| JSON Schema | 廣泛使用；工具成熟 | ✅ 採用 |
| LegalRuleML | OASIS 標準（2021） | ✅ 採用 |
| Rust (unsafe forbidden) | 記憶體安全；確定性執行 | ✅ 採用 |
| axum (Web 框架) | Rust 生態成熟 | ✅ 採用 |
| Maud (模板引擎) | CSS-first；零 JS | ✅ 採用 |

### 仍需實證的架構（需實驗驗證）

| 技術 | 需驗證的假設 | 驗證方法 |
|------|-------------|----------|
| PageIndex 樹導覽 | 在自有文件上是否優於 BM25+向量 | A/B 測試；消融實驗 |
| GraphRAG | 跨文件多跳是否值得抽取成本 | GraphRAG-Bench 資料集 + 自有語料 |
| Reranker | 延遲/成本 vs 精確度提升 | 基準測試 |
| 神經符號 AI | LLM + 規則引擎的整合效果 | 消融實驗 |
| CausalGraph2LLM | 因果查詢生成的準確性 | 人工評審 |

### 研究中的前沿技術（不可直接用於合規路徑）

| 技術 | 狀態 | 限制 | 用途 |
|------|------|------|------|
| Mojo | DNS 未解析；授權未核實 | 不可進入 compliance-critical 路徑 | 未來自建推論核心候選 |
| RAPTOR/Psi-RAG/SERAG | 學術論文；自報成績 | 需獨立重現；對稽核（需 verbatim）不適用 | 研究軌道 |
| Self-RAG/Corrective/Adaptive/Agentic RAG | 研究階段 | 反思次數 ≠ 正確性；需獨立驗證 | 研究軌道 |
| TLA+/Alloy | 形式化驗證 | 學習曲線極高；僅適合核心流程 | 核心流程驗證（選配） |

---

## 避免混淆：順序、規則符合、因果推論

### 三層分離架構

```
┌─────────────────────────────────────────────────┐
│  Layer A: 規範依賴 (Normative Dependency)        │
│  → 規則引擎：前置條件、禁止動作、時間限制          │
│  → 輸出：comply / violate / insufficient / escalate │
├─────────────────────────────────────────────────┤
│  Layer B: 流程順序 (Process Order)               │
│  → Token Replay / Conformance Checking          │
│  → 輸出：conform / deviate / missing / unconfirmed │
├─────────────────────────────────────────────────┤
│  Layer C: 真實因果 (True Causation)              │
│  → 因果分析模組：假設/識別/混淆/反證             │
│  → 輸出：identified / cannot_identify / partially / contradicted │
└─────────────────────────────────────────────────┘
```

### 禁止事項

1. **不得**將流程順序等同於真實因果（A 在 B 之前 ≠ A 導致 B）
2. **不得**將規則符合等同於因果（符合規則 ≠ 控制措施有效）
3. **不得**在資料不足以識別因果時生成確定性因果結論
4. **不得**將 LLM 的反思次數作為正確性證據

### 證據鏈保存

| 元素 | 保存方式 | 驗證方式 |
|------|----------|----------|
| 原始文件 | SHA-256 hash + 物件儲存 | hash 比對 |
| 文件版本 | 版本號 + 生效/失效日期 | 版本解析器 |
| 條款錨點 | section/page/paragraph/bbox | 錨點可解析性檢查 |
| 規則 | JSON Schema + LegalRuleML + 核准紀錄 | Schema 驗證 |
| 事件 | XES/OCEL + 三態（observed/assumed/unconfirmed）+ 時間來源 | 事件完整性檢查 |
| 決策 | Decision snapshot + manifest hash | byte-identical replay |
| 因果假設 | 假設/資料來源/識別策略/混淆/反證 | 人工專家覆核 |

### 版本管理

- **文件版本**：每次匯入建立新版本；舊版本保留；生效/失效日期控制適用範圍
- **規則版本**：rule_version 欄位；draft→active→retired 生命週期；核准紀錄
- **索引版本**：每次重建索引記錄版本；評測時固定索引版本
- **決策版本**：snapshot 記錄所有輸入版本；重放時驗證 byte-identical

### 不確定性管理

| 不確定性類型 | 處理方式 | 輸出 |
|-------------|----------|------|
| 證據不足 | 停止判定；輸出 insufficient_evidence | `insufficient_evidence` + 缺失證據清單 |
| 證據衝突 | 輸出 escalate；不自行擇一 | `escalate` + 衝突證據清單 |
| 無法識別因果 | 輸出 cannot_identify | `cannot_identify` + 資料限制說明 |
| 文件解析失敗 | 標記為 parse_failure；不使用不完整結果 | `parse_failure` + 錯誤詳情 |
| 權限不足 | 拒絕存取；不洩漏資料 | `permission_denied` |

---

## 實施優先順序

| 優先級 | 項目 | 狀態 |
|--------|------|------|
| 1 | Phase 5 Integration（因果模組接入引擎） | ✅ 模組完成，待接入 |
| 2 | Phase 6 Integration（Agent 框架接入引擎） | ✅ 模組完成，待接入 |
| 3 | Phase 3 Completion（向量+Tantivy+reranker+統一檢索） | 待實作 |
| 4 | Phase 4 Completion（BPMN+XES/OCEL+PM4Py） | 待實作 |
| 5 | Phase 2 Completion（知識圖譜+證據/主張層） | 待實作 |
| 6 | Phase 7（API+Web UI+安全） | 待實作 |
| 7 | Phase 8（50+ 案例+CI+消融報告） | 待實作 |
| 8 | Phase 9（WORM+租戶+稽核） | 待實作 |
| 9 | Phase 10（部署文件+本地優先工作流） | 待實作 |
| 10 | Deliveries E-G（報告+文件） | 待實作 |

---

## 開放問題

1. PM4Py sidecar 是否為硬依賴或可選（feature-gated）？
2. 目標部署環境（僅 Windows、Linux、或兩者）？
3. 知識圖譜是否作為獨立 crate 或在 platform 內？
4. 50+ 案例的最低數量（確切數字）？
5. 是否需要支援多租戶（tenant isolation）？
6. 是否需要支援離線 LLM（local model）？

---

## 參考文獻（已核實）

<|constrain|> ID | 標題 | 來源 | 年份 | 驗證狀態 |
|----|------|------|------|----------|
| raptor-2024 | RAPTOR: Recursive Abstractive Processing | ICLR 2024 | 2024 | verified=yes |
| pageindex-2025 | PageIndex: Document Index for Vectorless RAG | GitHub | 2025 | verified=yes |
| graphrag-bench-2026 | When to use Graphs in RAG | ICLR 2026 | 2026 | verified=yes |
| microsoft-graphrag-2024 | Microsoft GraphRAG | official doc | 2024 | verified=yes |
| ares-2024 | ARES: Automated Evaluation Framework | NAACL 2024 | 2024 | verified=yes |
| psi-rag-2026 | Hierarchical Abstract Tree for Cross-Doc RAG | ICML 2026 | 2026 | verified=yes |
| serag-2026 | Token-Free Hierarchical Indexing | ICML 2026 | 2026 | verified=yes |
| stronger-baselines-2025 | Stronger Baselines for RAG with Long-Context LLMs | EMNLP 2025 | 2025 | verified=yes |
| ragas-tool-2024 | RAGAS Evaluation Metrics | official doc | 2024 | verified=yes |

完整文獻清單見 `research/papers.csv`（28 篇）與 `research/standards.csv`（11 條）。
