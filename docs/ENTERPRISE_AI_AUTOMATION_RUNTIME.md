# 企業級 AI 自動化執行平台 — 架構與工程設計

> 文件編號：KNB-AUTO-RUNTIME-001｜版本：v1.0｜日期：2026-10-27
> 狀態：設計文件（非實作）
> 範圍：通用自動化執行平台，整合 RPA、AI Agent、流程編排、流程驗證與執行底層
> 語言策略：Rust + TLA+ + Zig + Austral（正確性優先）
> 驗證標記：△ 依專業知識與最佳實踐推導

---

## 1. 設計原則與約束

### 1.1 核心原則

1. **職責分離**：RPA 執行、AI 推理、流程編排、流程驗證、執行底層各自獨立
2. **統一治理**：所有操作經過統一的執行與治理架構
3. **多策略降級**：策略降級必須有條件，不可無條件重試
4. **不可逆操作保護**：不可逆操作前確認前次動作是否已生效
5. **證據管理**：所有執行皆產生可追溯的證據記錄
6. **故障恢復**：持久化執行狀態，支援中斷後恢復
7. **資源隔離**：各 Worker 類型獨立資源池
8. **AI 受控**：AI 不持有無限制存取權，所有操作經過授權與預算控制

### 1.2 約束條件

| 約束 | 說明 |
|---|---|
| 非 AGPL | 核心依賴不得使用 AGPL 授權 |
| Rust 為主 | 核心執行底層以 Rust 為主 |
| TLA+ 驗證 | 核心狀態機必須 TLA+ 驗證 |
| Zig 可選 | 僅在基準測試證明瓶頸後引入 |
| Austral 可選 | 僅在需要線性型別保證時引入 |
| Windows 優先 | 先在 Windows + 瀏覽器上穩定，再扩展 OS |

---

## 2. 自動化執行模式（六種）

### 2.1 模式分類

| 模式 | 說明 | 適用場景 | 持久性 |
|---|---|---|---|
| **M1: API/系統整合** | REST、GraphQL、gRPC、資料庫、訊息佇列、原生 SDK | 結構化資料交換、系統間通訊 | 無狀態 |
| **M2: 瀏覽器 RPA** | Web 表單、入口網站、登入、表格、下載、上傳，使用 DOM 與無障礙語義 | Web 應用自動化 | 有狀態（session） |
| **M3: 桌面 RPA** | Windows Win32/WinForms/WPF/Electron，原生 UI Automation、鍵盤滑鼠、視窗管理 | 桌面應用自動化 | 有狀態（session） |
| **M4: 跨系統工作流** | 等待、審批、逾時、重試、補償、恢復，持久化於 agent 記憶體之外 | 長期跨系統任務 | 持久化 |
| **M5: 流程驗證/流程挖掘** | 規範流程 vs 實際事件日誌比對 | 流程合規性分析 | 無狀態 |
| **M6: AI 動態決策** | AI 驅動的動態決策與工具編排，不確定或高風險時停止請求證據或人工審批 | 智慧化自動化 | 有狀態 |

### 2.2 模式選擇矩陣

| 場景 | 推薦模式 | 理由 |
|---|---|---|
| 資料查詢/更新 | M1 (API) | 結構化、可靠、可追溯 |
| Web 表單填寫 | M2 (Browser RPA) | DOM 語義定位，穩定 |
| 桌面應用操作 | M3 (Desktop RPA) | 原生 UI Automation |
| 跨系統長任務 | M4 (Durable Workflow) | 持久化、可恢復 |
| 流程合規檢查 | M5 (Process Mining) | 事件日誌分析 |
| 智慧化決策 | M6 (AI Agent) | 動態規劃、受控執行 |

---

## 3. 多策略自動化執行引擎

### 3.1 策略優先順序

| 優先順序 | 策略 | 說明 | 適用條件 |
|---|---|---|---|
| 1 | 原生 API/協定 | REST, GraphQL, gRPC, DB, MQ, SDK | 目標系統提供 API |
| 2 | DOM/語義瀏覽器控制 | Playwright + 無障礙樹 | Web 應用、DOM 可存取 |
| 3 | 原生桌面 UI Automation | Windows UIA / AT-SPI / macOS Accessibility | 桌面應用、UI Automation 可用 |
| 4 | OCR/影像/視覺識別 | Tesseract + 視覺 Worker | 無可靠語義介面 |
| 5 | 座標/鍵盤/滑鼠模擬 | 最後手段 | 無其他策略可用 |

### 3.2 Target Capability Profile

每個目標系統必須記錄：

| 欄位 | 說明 | 範例 |
|---|---|---|
| `target_id` | 唯一識別碼 | `sap-erp-prod` |
| `available_interfaces` | 可用介面清單 | `[REST, UIA, DOM]` |
| `locator_stability` | 定位器穩定度 (0-1) | `0.95` |
| `preconditions` | 前置條件 | `["user_logged_in", "page_loaded"]` |
| `postconditions` | 後置條件 | `["data_saved", "confirmation_shown"]` |
| `known_limits` | 已知限制 | `["no_batch_api", "session_timeout_30min"]` |
| `risk_level` | 風險等級 | `medium` |
| `verified_strategies` | 已驗證策略歷史 | `[{strategy: "api", success_rate: 0.99, last_verified: "2026-01-15"}]` |
| `irreversible_actions` | 不可逆操作清單 | `["delete_record", "submit_payment"]` |

### 3.3 策略降級規則

```
降級決策樹：
  1. 嘗試策略 N
  2. 觀察結果
  3. 若成功 → commit 證據 → 繼續
  4. 若失敗 →
     a. 判斷是否不可逆操作
        - 是 → 確認前次動作是否已生效
          - 已生效 → 記錄為成功，不重試
          - 未生效 → 嘗試策略 N+1
        - 否 → 嘗試策略 N+1
  5. 若所有策略失敗 → 補償/升級/人工接管
```

---

## 4. 開源 RPA 技術棧

### 4.1 核心技術選型

| 層級 | 技術 | 用途 | 授權 | 備註 |
|---|---|---|---|---|
| 瀏覽器 | **Playwright** | DOM/語義瀏覽器控制 | Apache-2.0 | 首選，語義定位 |
| 桌面 (Windows) | **FlaUI** (via isolated Windows Worker) | 原生 UI Automation | MIT | Rust FFI 成本高時使用 |
| OCR/視覺 | **Tesseract** + 可插拔視覺 Worker | 無語義介面的畫面 | Apache-2.0 | 可插拔，支援多引擎 |
| 相容性 | **Robot Framework** (可選 Worker) | 相容性測試 | Apache-2.0 | 僅相容性，非核心 |
| Linux 桌面 | **AT-SPI** | Linux UI Automation | LGPL-2.1 | 僅 Linux |
| macOS 桌面 | **Accessibility APIs** | macOS UI Automation | Proprietary | 僅 macOS |

### 4.2 作業系統支援策略

| OS | UI Automation | 狀態 | 說明 |
|---|---|---|---|
| Windows | UI Automation (UIA) | **首選** | 完整支援 Win32/WinForms/WPF/Electron |
| Linux | AT-SPI | 次要 | 需要 AT-SPI2 支援 |
| macOS | Accessibility APIs | 次要 | 需要輔助功能權限 |
| 虛擬/遠端桌面 | RDP/VDI 會話管理 | 次要 | 需要會話重連處理 |

**策略**：先在 Windows + 瀏覽器上穩定，再承諾其他作業系統。

### 4.3 Rust vs Zig 分工

| 組件 | 語言 | 理由 |
|---|---|---|
| 控制平面 | Rust | 記憶體安全、生態系成熟 |
| 執行引擎 | Rust | 零 unsafe、所有權保證 |
| 高效能模組 | Zig | 基準測試證明瓶頸後才引入 |
| 介面 | C ABI / IPC | 穩定介面，明確所有權 |

**Zig 引入條件**：僅在基準測試證明 Rust 是瓶頸，且 Zig 能提供可測量的效能提升時才引入。引入方式：穩定 C ABI 或 IPC，明確所有權，fuzzing，邊界測試，長時間測試。

---

## 5. 流程編排（與 RPA 分離）

### 5.1 三種表示方式

| 表示 | 用途 | 格式 | 版本控制 |
|---|---|---|---|
| 視覺流程圖 | 業務人員編輯 | BPMN 2.0 XML | Git |
| 版本化類型化規格 | 程式化定義 | Rust struct + TLA+ | Git |
| 持久化執行記錄 | 執行歷史 | Event Sourcing | DB + Object Storage |

### 5.2 編排引擎選型

| 引擎 | 用途 | 授權 | 說明 |
|---|---|---|---|
| **Temporal (Rust SDK)** | 程式化持久化執行 | MIT | 優先，程式化工作流 |
| **Flowable (BPMN/DMN)** | 業務標準化流程 | Apache-2.0 | 業務人員可編輯 |
| **Windmill** | 輕量流程編排 | AGPL-3.0 | 輕量部署，非核心 |

**規則**：不讓兩個引擎同時修改同一流程狀態，除非職責明確分離。

**偏好**：Rust 核心 + 程式化持久化工作流， acknowledging 編輯器建置成本。

---

## 6. 模組化控制平台架構

### 6.1 五層架構

```
Layer 1: 使用者與設計層
  - 視覺編輯器 (BPMN/DMN)
  - 程式碼編輯器 (Rust/Python/Go)
  - 版本控制 (Git)
  - 審批流程
  - 重播/回放

Layer 2: Rust 控制平面
  - API Gateway (REST/gRPC/WebSocket)
  - Policy Engine (授權/預算/風控)
  - Task Scheduler (排程/優先級)
  - Tool Registry (工具註冊/發現)
  - Evidence Service (證據管理)
  - Audit Service (稽核日誌)

Layer 3: 編排與執行層
  - AI Planner & Agent Runtime (受控)
  - Durable Workflow Engine (Temporal)
  - API Worker (REST/GraphQL/gRPC/DB/MQ)
  - Browser Worker (Playwright)
  - Desktop Worker (UIA/AT-SPI/Accessibility)
  - Vision Worker (Tesseract/OCR)

Layer 4: 持久化與事件層
  - PostgreSQL (狀態/歷史)
  - Object Storage (證據/檔案)
  - Workflow History (Event Sourcing)
  - Optional: NATS JetStream (訊息)

Layer 5: 全域工程治理
  - TLA+ (核心狀態機驗證)
  - 整合測試 + 故障注入
  - OpenTelemetry (可觀測性)
  - SBOM (軟體物料清單)
  - License 檢查
  - 基準測試
  - 備份與恢復演練
```

### 6.2 部署策略

**初始**：單一 Rust 控制平面 + PostgreSQL + 一個工作流引擎

**擴展**：僅在瓶頸或安全邊界需要時，才增加 Worker 和 NATS

---

## 7. 類型化 Action Contract

### 7.1 ActionRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action_id: Uuid,
    pub task_id: Uuid,
    pub action_type: ActionType,
    pub target: TargetRef,
    pub parameters: serde_json::Value,
    pub preconditions: Vec<Condition>,
    pub postconditions: Vec<Condition>,
    pub authorization: AuthorizationContext,
    pub budget: ExecutionBudget,
    pub deadline: Option<DateTime<Utc>>,
    pub retry_policy: RetryPolicy,
    pub compensation: Option<CompensationSpec>,
    pub evidence_requirements: EvidenceRequirements,
    pub idempotency_key: Option<String>,
}
```

### 7.2 ActionOutcome

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionOutcome {
    Success {
        result: serde_json::Value,
        evidence: Vec<EvidenceRef>,
        duration_ms: u64,
    },
    Failure {
        error: ActionError,
        evidence: Vec<EvidenceRef>,
        retryable: bool,
    },
    Uncertain {
        reason: String,
        partial_evidence: Vec<EvidenceRef>,
        requires_verification: bool,
        suggested_action: String,
    },
    Timeout {
        elapsed_ms: u64,
        partial_evidence: Vec<EvidenceRef>,
        may_have_succeeded: bool,
    },
    Compensated {
        compensation_result: serde_json::Value,
        original_error: ActionError,
    },
    Escalated {
        escalation_reason: String,
        human_review_required: bool,
    },
}
```

### 7.3 Uncertain 結果處理

| 情境 | 處理方式 |
|---|---|
| 逾時後外部已收到 | 驗證後置條件，確認是否已生效 |
| Worker 崩潰後寫入 | 重新查詢目標系統狀態 |
| 桌面失去焦點 | 重新定位目標，驗證前次動作 |
| 網路中斷後重連 | 檢查冪等性，避免重複執行 |

### 7.4 決定性執行迴圈

```
observe → identify target → validate preconditions → authorize → execute → observe result → validate postconditions → commit evidence → continue/retry/compensate/escalate
```

---

## 8. 事件記錄

### 8.1 事件類型

| 事件 | 說明 | 載荷 |
|---|---|---|
| TaskAccepted | 任務被接受 | task_id, process_version, timestamp |
| AuthorizationGranted | 授權通過 | task_id, actor, permissions, timestamp |
| ActionStarted | 動作開始 | action_id, strategy, target, timestamp |
| ActionSucceeded | 動作成功 | action_id, result, evidence_refs, timestamp |
| ActionFailed | 動作失敗 | action_id, error, retryable, timestamp |
| ActionUncertain | 結果不確定 | action_id, reason, partial_evidence, timestamp |
| ApprovalRequested | 請求審批 | task_id, approval_type, approver, timestamp |
| ApprovalGranted | 審批通過 | task_id, approver, comments, timestamp |
| ApprovalRejected | 審批拒絕 | task_id, approver, reason, timestamp |
| CompensationStarted | 補償開始 | action_id, compensation_action, timestamp |
| CompensationCompleted | 補償完成 | action_id, result, timestamp |
| TaskCompleted | 任務完成 | task_id, result, total_duration, timestamp |
| TaskFailed | 任務失敗 | task_id, error, timestamp |
| TaskEscalated | 任務升級 | task_id, reason, human_review, timestamp |

### 8.2 事件屬性（每個事件必須包含）

| 屬性 | 說明 |
|---|---|
| task_id | 任務唯一識別碼 |
| process_version | 流程版本 |
| action_version | 動作版本 |
| timestamp | ISO 8601 時間戳 |
| actor | 執行者（系統/使用者/Agent） |
| source_system | 來源系統 |
| evidence_refs | 證據參考清單 |
| result | 結果摘要 |

---

## 9. 流程驗證與流程挖掘

### 9.1 流程合規檢查

| 檢查類型 | 說明 | 方法 |
|---|---|---|
| 流程依賴 | 分析流程間的依賴關係 | DAG 分析 |
| 規範要求 | 比較實際執行與規範流程 | Conformance Checking |
| 因果關係 | 區分流程依賴、規範要求與因果關係 | Process Mining (Alpha/Petri Net) |

**重要**：流程圖不應被視為因果模型。流程依賴、規範要求、因果關係必須明確區分。

### 9.2 流程挖掘方法

| 方法 | 用途 | 工具 |
|---|---|---|
| Alpha Algorithm | 從事件日誌發現流程 | PM4Py (Python) |
| Heuristic Miner | 處理雜訊事件日誌 | PM4Py |
| Inductive Miner | 保證 soundness | PM4Py |
| Conformance Checking | 比較規範 vs 實際 | PM4Py |

---

## 10. 資源治理

### 10.1 資源池設計

| 資源池 | Worker 類型 | 限制 |
|---|---|---|
| API Pool | REST/GraphQL/gRPC/DB/MQ | 連線池大小、請求佇列 |
| Browser Pool | Playwright | 瀏覽器實例數、Context 數 |
| Desktop Pool | UIA/AT-SPI/Accessibility | 會話數、視窗數 |
| Vision Pool | Tesseract/OCR | CPU 核心、記憶體 |
| Agent Pool | AI Planner/Agent | Token 預算、併發數 |

### 10.2 資源治理機制

| 機制 | 說明 | 實作 |
|---|---|---|
| 有界管線 | 佇列有上限 | Bounded Channel |
| 任務輸入/輸出上限 | 防止記憶體爆炸 | Max bytes per task |
| Deadline | 任務超時自動取消 | Timeout enforcement |
| 取消 | 使用者/系統取消任務 | Cancellation token |
| 租戶公平配額 | 每租戶資源限制 | Per-tenant quota |
| 背壓/拒絕 | 資源不足時拒絕新任務 | Backpressure signal |
| 健康檢查 | Worker 健康監控 | Health endpoint |
| 重啟 | 不健康 Worker 自動重啟 | Supervisor pattern |
| 回收 | 高 RSS/長時間 Worker 回收 | Memory watchdog |

### 10.3 瀏覽器資源管理

- 瀏覽器常駐，但限制 Context 數量
- 每個 Context 使用後清理
- 高 RSS 時回收瀏覽器實例
- 長時間運行時自動重啟

---

## 11. 正確性閉環

### 11.1 TLA+ 驗證

| 驗證目標 | 說明 | 範圍 |
|---|---|---|
| 核心狀態機 | 任務狀態轉移 | Task, Action, Approval |
| 重試邏輯 | 重試計數、退避、斷路器 | Retry, Backoff, Circuit Breaker |
| Lease 管理 | 資源租約、超時、釋放 | Lease, Timeout, Release |
| 取消機制 | 任務/動作取消傳播 | Cancellation, Propagation |
| 併發控制 | 多 Worker 併發安全 | Concurrency, Lock, Deadlock |

### 11.2 Rust 測試

| 測試類型 | 說明 | 工具 |
|---|---|---|
| 單元測試 | 模組級測試 | cargo test |
| 整合測試 | 跨模組測試 | cargo test --test |
| 屬性測試 | Property-based testing | proptest |
| Fuzz 測試 | 邊界條件測試 | cargo fuzz |

### 11.3 故障注入

| 故障類型 | 說明 | 驗證目標 |
|---|---|---|
| Worker 崩潰 | 模擬 Worker 異常終止 | 恢復機制 |
| 網路中斷 | 模擬網路斷線 | 重試與補償 |
| 資料庫故障 | 模擬 DB 不可用 | 降級與恢復 |
| 記憶體壓力 | 模擬 OOM | 資源回收 |
| 時鐘偏移 | 模擬時鐘不同步 | 時間相關邏輯 |

### 11.4 稽核事件

所有重要操作產生稽核事件，包含：
- 事件類型、時間戳、執行者
- 來源系統、目標系統
- 輸入/輸出摘要
- 證據參考
- 結果狀態

---

## 12. 防護機制

### 12.1 AI 權限控制

| 控制措施 | 說明 |
|---|---|
| 無 unrestricted access | AI 不持有無限制存取權 |
| 工具註冊 | 所有工具必須註冊並授權 |
| 授權檢查 | 每個操作經過授權檢查 |
| 資源預算 | 每個 Agent 有資源預算限制 |
| 執行狀態管理 | 所有執行狀態由系統管理 |
| 獨立策略檢查 | 策略檢查獨立於 AI 推理 |
| 人類確認 | 高風險操作需人類確認 |

### 12.2 安全邊界

| 邊界 | 說明 |
|---|---|
| 模型不能修改策略 | Policy 獨立於模型 |
| 模型不能跳過審批 | Approval 強制執行 |
| 不可信內容不能作為指令 | Web content treated as data |
| 獨立策略檢查 | Policy engine independent |
| 人類確認高風險 | Human-in-the-loop for high-risk |

---

## 13. 驗收標準

### 13.1 正確性

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 動作契約完整性 | ActionRequest/ActionOutcome 完整 | 單元測試 |
| 狀態機正確性 | TLA+ 驗證通過 | TLA+ model checker |
| 補償邏輯正確性 | 補償動作正確執行 | 整合測試 |
| 冪等性保證 | 重複執行不產生副作用 | 屬性測試 |

### 13.2 RPA 穩定性

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 定位器穩定度 | locator_stability > 0.9 | 實測 |
| 策略成功率 | 每個策略成功率 > 0.95 | 實測 |
| 降級正確性 | 降級不產生副作用 | 整合測試 |
| 會話恢復 | 會話中斷後可恢復 | 故障注入 |

### 13.3 併發性

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 無 data race | Rust 所有權保證 | cargo test |
| 無死鎖 | 併發測試 | stress test |
| 資源公平 | 租戶配額公平 | 負載測試 |

### 13.4 記憶體

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 無記憶體洩漏 | RSS 穩定 | 長時間測試 |
| 記憶體上限 | Worker RSS < 512MB | 監控 |
| 回收機制 | 高 RSS 自動回收 | 故障注入 |

### 13.5 恢復性

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 恢復時間 | RTO < 5min | 故障注入 |
| 資料損失 | RPO < 1min | 備份還原測試 |
| 任務恢復 | 中斷後可恢復 | 故障注入 |

### 13.6 安全性

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 授權檢查 | 每個操作經過授權 | 安全測試 |
| 資料隔離 | 租戶資料隔離 | 安全測試 |
| 稽核日誌 | 所有操作可追溯 | 稽核檢查 |

### 13.7 可維護性

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 程式碼複雜度 | CC < 10 | clippy |
| 測試覆蓋率 | > 80% | cargo tarpaulin |
| 文件完整性 | 所有 public API 有文件 | cargo doc |

### 13.8 可審計性

| 標準 | 說明 | 驗證方法 |
|---|---|---|
| 事件完整性 | 所有操作產生事件 | 事件日誌檢查 |
| 證據可追溯 | 證據可追溯到操作 | 稽核檢查 |
| 版本可追溯 | 版本可追溯到程式碼 | Git tag |

---

## 14. 六階段路線圖

### Phase 1: 統一執行協定與控制平面 (Week 1-4)

| 任務 | 內容 | 交付物 | 驗證指標 |
|---|---|---|---|
| T-1.1 | Action Contract 定義 | Rust struct | 編譯通過 |
| T-1.2 | Event 系統 | Event enum + store | 單元測試 |
| T-1.3 | Policy Engine | 授權檢查 | 單元測試 |
| T-1.4 | Task Scheduler | 排程器 | 單元測試 |
| T-1.5 | API Gateway | REST/gRPC 端點 | 整合測試 |

### Phase 2: 真實瀏覽器與桌面 RPA (Week 5-12)

| 任務 | 內容 | 交付物 | 驗證指標 |
|---|---|---|---|
| T-2.1 | Playwright 整合 | Browser Worker | E2E 測試 |
| T-2.2 | Windows UIA 整合 | Desktop Worker | E2E 測試 |
| T-2.3 | Tesseract 整合 | Vision Worker | E2E 測試 |
| T-2.4 | Target Capability Profile | Profile 系統 | 單元測試 |
| T-2.5 | 多策略降級 | 降級邏輯 | 整合測試 |

### Phase 3: 持久化工作流與審批 (Week 13-20)

| 任務 | 內容 | 交付物 | 驗證指標 |
|---|---|---|---|
| T-3.1 | Temporal 整合 | Durable Workflow | E2E 測試 |
| T-3.2 | 審批流程 | Approval 系統 | 整合測試 |
| T-3.3 | 補償機制 | Compensation 系統 | 整合測試 |
| T-3.4 | 恢復機制 | Recovery 系統 | 故障注入 |

### Phase 4: 視覺流程與企業整合 (Week 21-28)

| 任務 | 內容 | 交付物 | 驗證指標 |
|---|---|---|---|
| T-4.1 | BPMN 編輯器 | 視覺編輯器 | 使用者測試 |
| T-4.2 | Flowable 整合 | BPMN 引擎 | 整合測試 |
| T-4.3 | 企業系統連接器 | REST/GraphQL/gRPC/DB/MQ | 整合測試 |
| T-4.4 | 身分認證整合 | OIDC/SAML | 安全測試 |

### Phase 5: 流程驗證與知識整合 (Week 29-36)

| 任務 | 內容 | 交付物 | 驗證指標 |
|---|---|---|---|
| T-5.1 | 流程挖掘 | PM4Py 整合 | 流程分析報告 |
| T-5.2 | 知識圖譜 | 知識系統 | 整合測試 |
| T-5.3 | RAG 整合 | 檢索系統 | E2E 測試 |
| T-5.4 | AI Agent 整合 | 受控 Agent | 安全測試 |

### Phase 6: 效能與規模 (Week 37-44)

| 任務 | 內容 | 交付物 | 驗證指標 |
|---|---|---|---|
| T-6.1 | 效能基準測試 | Benchmark 報告 | p50/p95/p99 |
| T-6.2 | Zig 模組 (如需要) | 高效能模組 | 基準測試 |
| T-6.3 | 水平擴展 | 多節點部署 | 負載測試 |
| T-6.4 | NATS 整合 (如需要) | 訊息系統 | 吞吐量測試 |

---

## 15. 最終技術選型

| 類別 | 選擇 | 理由 | 取捨 |
|---|---|---|---|
| 核心語言 | Rust | 記憶體安全、生態系成熟 | 學習曲線高 |
| 正確性驗證 | TLA+ | 狀態機形式驗證 | 僅驗證邏輯，不驗證實作 |
| 高效能模組 | Zig (可選) | 零開銷、C ABI 相容 | 需基準測試證明瓶頸 |
| 線性型別 | Austral (可選) | 線性型別保證 | 生態系不成熟 |
| 瀏覽器 RPA | Playwright | 語義定位、跨瀏覽器 | 需 Node.js runtime |
| 桌面 RPA (Windows) | FlaUI (isolated worker) | 原生 UIA、MIT 授權 | 需 Windows Worker |
| OCR/視覺 | Tesseract + 可插拔 | 開源、可插拔 | 準確率需調校 |
| 流程編排 | Temporal (Rust SDK) | 程式化、持久化、MIT | 編輯器建置成本 |
| 業務流程 | Flowable (BPMN/DMN) | 標準化、Apache-2.0 | Java 依賴 |
| 輕量編排 | Windmill | Rust 後端、AGPL-3.0 | 社群較小 |
| 資料庫 | PostgreSQL | 開源、可靠、功能豐富 | 需維護 |
| 訊息佇列 | NATS JetStream (可選) | 輕量、高效 | 僅在需要時引入 |
| 監控 | Prometheus + Grafana | 開源、業界標準 | 需維護 |
| 日誌 | Loki | 與 Grafana 整合 | 查詢能力有限 |

### Caveat

這是目標架構與候選技術集，需要在實際環境中驗證：
- 各技術的版本相容性
- 各技術的授權合規性
- 各技術的整合成熟度
- 各技術的效能表現
- 各技術的維護成本

所有效能目標（如 50K msg/s）是驗證目標，不是假設能力。需要分別基準測試：
- 訊息處理吞吐量
- 排程延遲
- 瀏覽器操作延遲
- 外部執行延遲
- 端到端完成時間

---

## 16. 驗收標準總表

| 驗收維度 | 標準 | 驗證方法 |
|---|---|---|
| 正確性 | 動作契約完整、狀態機正確、補償邏輯正確、冪等性保證 | TLA+ + 單元測試 + 屬性測試 |
| RPA 穩定性 | 定位器穩定度 > 0.9、策略成功率 > 0.95、降級無副作用 | 實測 + 整合測試 |
| 併發性 | 無 data race、無死鎖、資源公平 | cargo test + stress test |
| 記憶體 | 無記憶體洩漏、Worker RSS < 512MB、自動回收 | 長時間測試 + 監控 |
| 恢復性 | RTO < 5min、RPO < 1min、任務可恢復 | 故障注入 + 備份還原測試 |
| 安全性 | 授權檢查、資料隔離、稽核日誌 | 安全測試 + 稽核檢查 |
| 可維護性 | CC < 10、測試覆蓋率 > 80%、文件完整 | clippy + cargo tarpaulin + cargo doc |
| 可審計性 | 事件完整性、證據可追溯、版本可追溯 | 稽核檢查 |

---

## 17. 結論

本計畫定義了一個企業級 AI 自動化執行平台的完整架構與工程設計。核心策略：

1. **Rust 為主**：記憶體安全、零 unsafe、零 clippy 警告
2. **TLA+ 驗證**：核心狀態機形式驗證
3. **Zig 可選**：僅在基準測試證明瓶頸後引入
4. **多策略 RPA**：API → DOM → UIA → OCR → 座標模擬
5. **持久化編排**：Temporal Rust SDK + Flowable BPMN/DMN
6. **資源治理**：有界管線、配額、背壓、回收
7. **AI 受控**：工具註冊、授權、預算、策略檢查
8. **正確性閉環**：TLA+ + 測試 + 故障注入 + 稽核

所有設計決策均有理由、取捨和整合說明。這是目標架構，需要在實際環境中驗證。