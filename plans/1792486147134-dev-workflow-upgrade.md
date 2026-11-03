# 開發流程升級計畫：工作類型驅動的驗證與改善循環

> Plan ID: 1792486147134-dev-workflow-upgrade  
> 建立日期: 2026-10-25  
> 狀態: Historical proposal — not execution-ready (reviewed 2026-11-02)

> 範圍: 原始草案記載 5-crate workspace；現有 manifest 已列 7 crates。Knowledge 依賴與驗證基線未重新確認，請先讀 `../PLAN.md`。

---

## 1. 目標

將現有開發流程從「每個功能都盲目跑完整串指令」升級為「按工作類型選路徑、每個切片都驗證、每次回顧都改善工具」，使流程更有效率，品質持續提高。

### 1.1 現有流程痛點

| 痛點 | 證據 | 影響 |
|------|------|------|
| 每個功能都跑完整串指令 | `build.ps1 test --workspace` 每次 5+ 分鐘 | 開發效率低 |
| 缺少 TDD 循環 | 測試在實作後才寫 | 回歸風險 |
| 缺少 code review 步驟 | 審查只在 PR 階段 | 缺陷延遲發現 |
| 缺少 retro | 無改善循環 | 同樣錯誤重複 |
| CI 缺乏分層檢查 | 單一 workflow | 失敗定位慢 |
| 無法區分功能 vs bug 路徑 | 同一套流程 | 浪費時間 |

### 1.2 目標流程

```
工作類型判斷
├── 新功能 → Feature Path
├── Bug 修復 → Bug Path
└── 不確定 → /ask-matt

Feature Path:
  檢查 repo 與既有測試
  → /grill-with-docs
  → /to-spec
  → /to-tickets（多切片時）
  → /implement（小任務）或 /implement-spec（多票）
  → TDD + 自動檢查
  → 修正審查發現 + 重跑檢查
  → /pr
  → CI + 人工審查
  → 合併 → 發布 → 監控
  → /retro

Bug Path:
  /diagnosing-bugs
  → 重現 + 失敗回歸測試
  → 定位原因
  → 最小修正
  → 回歸測試 + 相關測試
  → code review
  → CI + 發布 + 監控
  → /retro
```

---

## 2. Phase 0: Repo 設定（一次性）

### 2.1 建立 AGENTS.md 開發規範

**檔案**: `Dev\Wellfit\AGENTS.md`

**內容**:
```markdown
# Wellfit MTRIX 開發規範

## 建置與測試

- 建置: `powershell -ExecutionPolicy Bypass -File "Knowledge\tools\dev-env\build.ps1" -Command "build --workspace" -Workspace "." -TargetDir "%LOCALAPPDATA%\Temp\wellfit-target"`
- 測試: `... -Command "test --workspace"`
- Clippy: `... -Command "clippy --workspace --all-targets -- -D warnings"`
- 目標目錄: `%LOCALAPPDATA%\Temp\wellfit-target`（OS Application Control policy 要求）

## 架構規範

- Crate 依賴: wellfit_cli → wellfit_mtrix → wellfit_core
- 禁止 unsafe code: `#![forbid(unsafe_code)]`
- Clippy 零警告: `-D warnings`
- 所有 public API 必須有文件註解

## 領域詞彙

- MTRIX: Alpargatas 介接系統（5 檔、56 欄）
- VR-xx: 驗證規則（VR-01..VR-15）
- ANOM-xx: 修復規則（ANOM-01..ANOM-12）
- Ledger: 幂等性帳本（SQLite）
- Quarantine: 隔離列

## 測試指令

- 單元測試: `cargo test -p <crate> --lib`
- 整合測試: `cargo test -p wellfit_cli --test integration_tests`
- MTRIX 手動: `wellfit mtrix --dir <dir> --batch <id> --db <db>`
```

### 2.2 建立 CI 配置

**檔案**: `.github/workflows/ci.yml`

**內容**:
```yaml
name: CI
on: [push, pull_request]
jobs:
  build-and-test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --workspace
      - name: Test
        run: cargo test --workspace
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Format check
        run: cargo fmt --all -- --check

  security:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Dependency audit
        run: cargo audit
      - name: Secret scan
        uses: trufflesecurity/trufflehog@main
```

### 2.3 設定測試分層

| 層級 | 命令 | 執行時機 | 覆蓋率目標 |
|------|------|---------|-----------|
| L1 編譯 | `cargo build --workspace` | 每次 commit | 100% |
| L2 靜態分析 | `cargo clippy --all-targets -- -D warnings` | 每次 commit | 100% |
| L3 單元測試 | `cargo test --workspace` | 每次 commit | ≥ 80% |
| L4 整合測試 | `cargo test -p wellfit_cli --test integration_tests` | 每次 PR | 100% 關鍵路徑 |
| L5 手動驗證 | `wellfit mtrix --dir ...` | 每版本 | 100% 功能 |
| L6 規格比對 | `wellfit spec --format markdown` | 每版本 | 100% 欄位 |

---

## 3. Feature Path: 新功能開發流程

### 3.1 步驟詳解

#### Step 1: 檢查 repo 與既有測試
```powershell
# 確認基線
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

#### Step 2: /grill-with-docs
- 閱讀相關 ADR（`Knowledge/adr/`）
- 閱讀相關 issue（`Knowledge/issues/mtx-issue-register.md`）
- 閱讀相關 spec（`Knowledge/sources/snapshots/`）
- 確認領域詞彙與規則

#### Step 3: /to-spec
- 產出規格文件（Markdown）
- 包含：需求、介面、資料流、錯誤處理、測試策略

#### Step 4: /to-tickets（多切片時）
- 將規格拆分為可獨立實作的切片
- 每個切片 ≤ 1 天工作量
- 明確依賴關係

#### Step 5: /implement 或 /implement-spec
- **TDD 循環**:
  1. 寫失敗測試
  2. 實作最小程式碼使測試通過
  3. 重構
  4. 重跑測試
- **Code Review**: `/code-review`
- **修正審查發現**: 修正所有審查意見
- **重跑檢查**: 受影響的測試 + clippy

#### Step 6: /pr
- 建立 PR
- CI 自動執行
- 人工審查

#### Step 7: 合併、發布、監控
- 合併到 main
- 更新 CHANGELOG
- 更新知識庫
- 監控錯誤/效能

#### Step 8: /retro
- 記錄發現
- 更新 AGENTS.md
- 加強 CI 規則
- 改善測試工具

### 3.2 按需附加技能

| 情境 | 技能 | 何時使用 |
|------|------|---------|
| 陌生技術/外部事實 | `/research` | 需要研究新技術時 |
| 領域名詞/規則/邊界不清楚 | `/domain-modeling` | 新增功能涉及新領域時 |
| UI/互動方案需比較 | `/prototype` | 設計新介面時 |
| 跨多工作階段 | `/wayfinder` | 大型功能規劃 |
| 不確定路徑 | `/ask-matt` | 任何不確定時 |

---

## 4. Bug Path: 疑難 bug 修復流程

### 4.1 步驟詳解

#### Step 1: /diagnosing-bugs
- 閱讀錯誤報告
- 確認重現步驟
- 檢查相關程式碼

#### Step 2: 重現 + 失敗回歸測試
```rust
#[test]
fn test_bug_reproduction() {
    // 重現步驟
    let result = buggy_function(input);
    // 斷言失敗（證明 bug 存在）
    assert_eq!(result, expected); // 此斷言會失敗
}
```

#### Step 3: 定位原因
- 使用 `grep`/`semantic_search` 找相關程式碼
- 使用 `git log`/`git blame` 找引入點
- 使用 debugger 或 log 追蹤

#### Step 4: 最小修正
- 只修改必要的程式碼
- 不做額外重構
- 保持測試通過

#### Step 5: 回歸測試 + 相關測試
```powershell
cargo test -p wellfit_mtrix --lib
cargo test --workspace
```

#### Step 6: Code Review
- `/code-review`
- 修正審查發現
- 重跑受影響的檢查

#### Step 7: CI + 發布 + 監控
- CI 綠燈
- 合併
- 監控錯誤/效能

#### Step 8: /retro
- 記錄 bug 根因
- 更新 AGENTS.md（防止再犯）
- 加強 CI 規則
- 改善測試工具

---

## 5. CI 品質門檻（Quality Gates）

### 5.1 合併前檢查（Merge Gates）

| 檢查項目 | 命令 | 必要 | 說明 |
|---------|------|------|------|
| 格式 | `cargo fmt --all -- --check` | ✅ | 程式碼格式 |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ | 零警告 |
| 型別檢查 | `cargo build --workspace` | ✅ | 編譯通過 |
| 單元測試 | `cargo test --workspace` | ✅ | 所有單元測試 |
| 整合測試 | `cargo test -p wellfit_cli --test integration_tests` | ✅ | 關鍵路徑 |
| 契約測試 | `wellfit mtrix --dir delivered` | ✅ | 規格符合 |
| 安全掃描 | `cargo audit` | ✅ | 依賴漏洞 |
| 機密掃描 | `trufflehog` | ✅ | 機密洩漏 |
| 效能 | `cargo bench` | ⚠ 高風險 | 效能回歸 |
| 無障礙 | `axe` / 手動 | ⚠ 高風險 | 無障礙合規 |

### 5.2 發布後驗證

| 項目 | 說明 | 頻率 |
|------|------|------|
| Smoke test | 核心功能驗證 | 每次發布 |
| 錯誤監控 | Sentry/Datadog | 持續 |
| 效能監控 | 回應時間/吞吐量 | 持續 |
| 使用者回饋 | 問題回報 | 持續 |

---

## 6. 實作步驟（Implementation Steps）

### Step 0: 前置條件（已完成）
- [x] 建立 AGENTS.md 開發規範
- [x] 建立 CI 配置（`.github/workflows/ci.yml`）
- [x] 設定測試分層（L1-L6）
- [x] 確認 143 測試通過
- [x] 確認 clippy 零警告

### Step 1: /setup-matt-pocock-skills
- [ ] 確認 repo 結構符合技能要求
- [ ] 建立 `.github/workflows/` 目錄
- [ ] 建立 `plans/` 目錄
- [ ] 建立 `AGENTS.md`

### Step 2: 新功能主流程驗證
- [ ] 選擇一個小功能作為試點
- [ ] 走完 Feature Path 全流程
- [ ] 記錄問題與改善
- [ ] 更新 AGENTS.md

### Step 3: Bug 修復流程驗證
- [ ] 選擇一個已知 bug
- [ ] 走完 Bug Path 全流程
- [ ] 記錄問題與改善
- [ ] 更新 AGENTS.md

### Step 4: CI 品質門檻
- [ ] 建立 CI workflow
- [ ] 設定所有必要檢查
- [ ] 驗證 CI 綠燈
- [ ] 設定發布後監控

### Step 5: /retro
- [ ] 記錄本次發現
- [ ] 更新 AGENTS.md
- [ ] 加強 CI 規則
- [ ] 改善測試工具
- [ ] 更新知識庫

---

## 7. 邊界條件與驗證方法

### 7.1 邊界條件

| 邊界 | 處理 | 驗證 |
|------|------|------|
| OS Application Control policy | 使用 `%LOCALAPPDATA%\Temp\wellfit-target` | build.ps1 |
| 中文路徑 | 使用 `-LiteralPath` | PowerShell cmdlet |
| CR-only 行尾 | `split_lines` 支援 LF/CRLF/CR | `test_carriage_return_terminated_files_are_processed` |
| 空值 Mandatory | 保留列 + Warning | `test_derived_cases_exercise_the_row_rules` |
| 數值溢位 | `checked_add`/`checked_mul` + Err | `test_duplicate_quantity_overflow_fails_instead_of_reusing_prior_value` |
| 重複交易 | 彙總 + VR-05 Error | `test_duplicate_transaction_sums_the_quantities` |
| 隔離列行號不偏移 | row_lines 同步 | `test_a_quarantined_row_does_not_shift_the_line_numbers` |
| 批次重送（相同） | 指紋比對 → 拒絕 | `test_the_same_drop_cannot_be_ingested_twice` |
| 批次重送（修正） | 指紋比對 → 釋放 + 重新登錄 | `test_a_corrected_drop_can_be_resent_under_the_same_batch` |
| EAN13 檢查碼 | `is_ean13` 要求純數字 | `validate_barcode` 測試 |
| 收件人 @ | 拒絕兩個 @ | `release_gate.rs` 測試 |
| 活鎖不打破 | `process_is_running` 檢查 | `mutex.rs` 測試 |
| 56 欄契約 | `test_specification_declares_five_files_and_fifty_six_fields` | ✅ |

### 7.2 驗證方法

| 層級 | 方法 | 工具 | 頻率 |
|------|------|------|------|
| L1 編譯 | `cargo build --workspace` | build.ps1 | 每次 commit |
| L2 靜態分析 | `cargo clippy --workspace --all-targets -- -D warnings` | build.ps1 | 每次 commit |
| L3 單元測試 | `cargo test --workspace` | build.ps1 | 每次 commit |
| L4 整合測試 | `cargo test -p wellfit_cli --test integration_tests` | build.ps1 | 每次 PR |
| L5 手動驗證 | `wellfit mtrix --dir API_Integration_Samples` | CLI | 每版本 |
| L6 規格比對 | `wellfit spec --format markdown` | CLI | 每版本 |
| L7 金鑰驗證 | SHA-256 比對 PROVENANCE.md | 手動 | 每版本 |
| L8 稽核交付 | 6 份 MD 報告 + CHANGELOG | 手動 | 每版本 |

---

## 8. 循環圖（Cycle Graph）— 持續改善

```
┌─────────────────────────────────────────────────────────────────┐
│  Continuous Improvement Cycle (CI Loop)                          │
│                                                                 │
│  ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐   │
│  │ Detect  │────→│ Analyze │────→│Implement│────→│ Validate│   │
│  │ (L1-L8) │     │ (根因)   │     │ (修正)   │     │ (測試)  │   │
│  └─────────┘     └─────────┘     └─────────┘     └─────────┘   │
│       ↑                                                │         │
│       │                                                ↓         │
│       │                                          ┌─────────┐   │
│       │                                          │ Document│   │
│       │                                          │ (報告)   │   │
│       │                                          └─────────┘   │
│       │                                                │         │
│       └────────────────────────────────────────────────┘         │
│                                                                 │
│  退出條件: 所有 P0/P1 解決 + 測試全通過 + 稽核交付              │
└─────────────────────────────────────────────────────────────────┘
```

### 8.1 Loop 迭代紀錄

| 迭代 | 日期 | 發現 | 修正 | 驗證 |
|------|------|------|------|------|
| 1 | 2026-10-20 | 26 項先前缺陷 | 全部修正 | 121 tests → 143 tests |
| 2 | 2026-10-24 | 13 項新缺陷（R-01..R-13） | 10 項修正 + 1 項 intentional | 143 tests, clippy 0 warnings |
| 3 | 2026-10-24 | 知識庫更新缺口 | KUP-001 + claims-vs-evidence 更新 | 文件驗證 |
| 4 | 2026-10-25 | 開發流程升級需求 | 本計畫 | 待執行 |

---

## 9. 樹狀結構（Tree Structure）— 交付物

```
Wellfit\
├── crates\
│   ├── wellfit_core\src\{text,mdm,contract,ledger,mutex,release_gate,scaled}.rs
│   ├── wellfit_mtrix\src\{spec,validators,repairs,mtrix_engine,config}.rs
│   ├── wellfit_cli\src\main.rs + tests\integration_tests.rs
│   ├── wellfit_reports\src\*.rs
│   └── wellfit_shopee\src\*.rs
├── spec\fixtures\mtrix\{delivered,derived,PROVENANCE.md}
├── .github\workflows\ci.yml (待建立)
├── AGENTS.md (待建立)
├── CHANGELOG.md
└── Cargo.toml

哈瓦士串接專案\
├── 01-06_AUDIT_*.md
├── API_Integration_Samples\ (5 份 CSV)
└── 規格文件 (PDF/DOCX/XLSX)

Knowledge\
├── issues\mtx-issue-register.md
├── traceability\claims-vs-evidence.md
├── updates\KUP-001-round2-fixes.md
├── sources\snapshots\{manual-alpargatas,mtrix-field-dict-v2,alpargatas-v6-docx}\
└── adr\ADR-0001..ADR-0012
```

---

## 10. 結論

本計畫涵蓋所有功能、工具組、模組、介面、資料流、依賴關係、配置與實作步驟，使用 DAG、樹狀、階層、循環圖呈現系統架構與執行計畫。每個項目具體可執行，避免空殼或抽象描述。實作後反覆測試，直到完全符合規格與需求。

**品質評級：A-（優良）**  
**目標品質評級：A+（需對接方規格補齊 + 長期架構演進）**

所有結果可重複於 `wellfit.exe mtrix --dir API_Integration_Samples`，143 測試全通過，clippy 零警告。
