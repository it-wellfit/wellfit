# Wellfit 完整產品生命週期計畫 — 從 0 到 Production

> 計畫編號：KNB-WELL-PROD-001｜版本：v1.0｜日期：2026-10-25
> 狀態：歷史交付計畫；部分狀態與驗證資料已過時，不能作為目前生產就緒證明。
> 範圍：Wellfit Rust 工作區（目前 7 crates）+ ISMS 合規套件 + MTRIX 介接 + Sync CLI（資料傳輸未實作）

---

## 1. 專案現況盤點

### 1.1 已完成資產

| 類別 | 項目 | 狀態 | 驗證結果 |
|---|---|---|---|
| Rust Crate | 7 (wellfit_core, wellfit_reports, wellfit_shopee, wellfit_mtrix, wellfit_cli, wellfit_sync, wellfit_bankpay) | ✔ 建置／測試 | 最近 workspace run：237 passed、3 ignored；不代表容量或生產 SLA 驗收 |
| 驗證規則 | VR-01~VR-15 (15 條) | ✔ 完成 | 47 個單元測試通過 |
| 修復規則 | ANOM-01~ANOM-12 (12 條) | ✔ 完成 | 12 個單元測試通過 |
| ISMS 合規 | 9 工具 + 23 範本 + 15 表單 + 10 主文 + 93 項控制 | ✔ 完成 | PASS=631/WARN=0/FAIL=0 |
| UI/UX | ANSI 顏色、Spinner、ProgressBar、Table、Panel | ✔ 完成 | clippy 0 警告 |
| Sync | wellfit_sync CLI 與設備探索骨架 | 部分實作；無資料傳輸 | 測試僅驗證現有 CLI／設定／探索行為，不代表同步能力 |
| PowerShell 工具 | 9 個腳本 | ✔ 完成 | 全部驗證通過 |
| 文件 | 20+ 份 | ✔ 完成 | 人工審閱 |

### 1.2 待完成項目

| # | 項目 | 優先級 | 說明 |
|---|---|---|---|
| G-01 | 端對端測試覆蓋率 < 90% | 高 | 缺少跨模組 E2E 測試 |
| G-02 | UI/UX 不一致 | 高 | 各 CLI 子命令輸出格式不一 |
| G-03 | 缺少效能基準 | 中 | 無 cargo bench 結果 |
| G-04 | 無自動化 CI/CD | 中 | 僅有本地 build.ps1 |
| G-05 | 錯誤處理不一致 | 中 | 各子命令錯誤格式不一 |
| G-06 | 文件不完整 | 中 | 各 crate 缺少 API 參考文件 |
| G-07 | 遺失測試 | 高 | 部分 crate 單元測試不足 |

---

## 2. 領域知識與業務內容

### 2.1 業務領域

| 領域 | 流程數 | 關鍵問題 | 解決方案 |
|---|---|---|---|
| 財務 (P-FIN) | 6 | 對帳帳號可改收款資訊 | VR-03 日期驗證、VR-05 重複交易 |
| 作業 (P-OPS) | 7 | 匯入批次可被重送 | VR-05 重複交易、ANOM-10 聚合 |
| 零售 (P-RET) | 3 | POS 共用帳號 | A.5.16 唯一識別 |
| 電商 (P-ECS) | 3 | 個資外流 | A.5.17 MFA、A.8.18 特權存取 |
| 人力 (P-HR) | 2 | 離職員帳號未撤銷 | A.5.11 離職撤銷 |
| 資訊 (P-IT) | 2 | 版本發布未授權 | A.8.25 變更管理 |
| 物流 (P-LOG) | 4 | 承運商存取未核准 | A.5.19 供應商安全 |
| 供應商 (P-SUP) | 2 | 服務結束未撤銷 | A.8.31 退出管理 |
| 客服 (P-SVC) | 1 | 無授權個資查詢 | A.5.37 個資查詢 |
| 行銷 (P-MKT) | 1 | 名單來源不可追溯 | A.5.22 測試資料 |
| 實體 (P-PHY) | 1 | 未授權進入 | A.7.1 門禁控制 |
| 治理 (P-GOV) | 3 | 範圍未核准即宣稱 | A.5.26 範圍治理 |

### 2.2 真實世界問題

| 問題 | 影響 | 解決方案 |
|---|---|---|
| 對端檔名含下載殘留 | VR-01 警告 | normalize-filename.ps1 |
| 客戶代碼不在主檔 | VR-08 錯誤 | 異常通知 + 主檔同步 |
| 批次代碼不在主檔 | VR-09 錯誤 | 異常通知 + 主檔同步 |
| 欄位移位 | VR-15 警告 | 對端格式契約 |
| 強制性未定義 | VR-04/VR-10 警告 | 業務規則確認文件 |

---

## 3. 系統分析與設計

### 3.1 需求追蹤矩陣 (RTM)

| 需求 ID | 需求描述 | 優先級 | 實作狀態 | 驗證方法 |
|---|---|---|---|---|
| R-01 | MTRIX 五檔解析 | P0 | ✔ 已實作 | integration_test |
| R-02 | 15 驗證規則 | P0 | ✔ 已實作 | validators.rs 單元測試 |
| R-03 | 12 修復規則 | P0 | ✔ 已實作 | repairs.rs 單元測試 |
| R-04 | CR/CRLF/LF 統一處理 | P0 | ✔ 已實作 | text.rs 單元測試 |
| R-05 | JSON 報告輸出 | P0 | ✔ 已實作 | integration_test |
| R-06 | 例外通知機制 | P1 | ✔ 已實作 | integration_test |
| R-07 | 零 unsafe / 零 clippy 警告 | P0 | ✔ 已實作 | cargo clippy |
| R-08 | 零浮點金額 | P0 | ✔ 已實作 | scaled.rs 單元測試 |
| R-09 | 欄位名稱解析 | P0 | ✔ 已實作 | contract.rs 單元測試 |
| R-10 | 合規驗證 0 FAIL | P0 | ✔ 已實作 | verify-compliance.ps1 |
| R-11 | 多平台同步守護程式 | P1 | 部分實作：設定、健康檢查及 peer 探索；沒有資料傳輸 | cargo test 不驗證資料同步 |
| R-12 | Windmill 自動化部署 | P2 | ✔ 已設計 | 設計文件 |
| R-13 | ISMS 93 項控制 | P0 | ✔ 已實作 | build-control-trace |
| R-14 | 四維控制狀態 | P0 | ✔ 已實作 | V-05 |
| R-15 | 深度級別 D1/D2/D3 | P1 | ✔ 已實作 | V-13 |

### 3.2 系統架構圖 (Mermaid)

```mermaid
graph TD
    subgraph CORE ["Wellfit Rust 工作區"]
        C1[wellfit_core] --> C2[wellfit_reports]
        C1 --> C3[wellfit_shopee]
        C1 --> C4[wellfit_mtrix]
        C1 --> C5[wellfit_cli]
        C1 --> C6[wellfit_sync]
        C4 --> C5
    end

    subgraph ISMS ["ISMS 合規套件"]
        I1[10 主文] --> I7[93 項控制]
        I2[23 範本] --> I5[11 值域]
        I4[9 工具] --> I2
    end

    subgraph MTRIX ["MTRIX 介接"]
        M1[ALTHCI 16欄] --> M6[VR-01~VR-15]
        M2[ALTHES 13欄] --> M6
        M3[ALTHFV 15欄] --> M6
        M4[ALTHPD 16欄] --> M6
        M5[ALTHVD 17欄] --> M6
        M6 --> M7[ANOM-01~ANOM-12]
        M6 --> M8[例外通知]
    end

    subgraph AUTO ["自動化部署"]
        A1[Windmill] --> C4
        A2[PowerShell] --> C4
        A2 --> I4
    end

    C4 --> M1
    C4 --> M2
    C4 --> M3
    C4 --> M4
    C4 --> M5
    I4 --> I1
    I4 --> I2

    style C1 fill:#e6f3ff,stroke:#0066cc,stroke-width:2px
    style I7 fill:#fff4e6,stroke:#ff9900
    style M6 fill:#ffe6e6,stroke:#cc0000
    style A1 fill:#e6ffe6,stroke:#009900
```

---

## 4. 敏捷迭代實作計畫

### Sprint 1: 基礎架構（已完成）
- [x] 建立 wellfit_core crate (MDM, Ledger, ReleaseGate, ScaledDecimal, text, contract)
- [x] 建立 wellfit_reports crate (ReportAutomator)
- [x] 建立 wellfit_shopee crate (ShopeeInboundEngine)
- [x] 建立 wellfit_mtrix crate (MtrixEngine, spec, validators, repairs)
- [x] 建立 wellfit_cli crate (6 子命令 CLI)
- [x] AGENTS.md 不變量定義

### Sprint 2: MTRIX 介接引擎（已完成）
- [x] spec.rs: 56 欄位字典、15 驗證規則、12 修復規則
- [x] validators.rs: VR-01~VR-15 實作
- [x] repairs.rs: ANOM-01~ANOM-12 實作
- [x] mtrix_engine.rs: 三階段管線 (Accept → Repair → Validate)
- [x] CR/CRLF/LF 統一處理 (text.rs)
- [x] 例外通知機制 (ExceptionNotification)

### Sprint 3: ISMS 合規套件（已完成）
- [x] 10 主文文件 (KNB-CMP-001~010)
- [x] 23 範本 CSV + 26 Schema
- [x] 15 表單
- [x] 9 工具腳本
- [x] 11 值域檔
- [x] 3 ADR
- [x] 93 項 Annex A 控制 + 5 項組織附加控制

### Sprint 4: UI/UX 專業化（已完成）
- [x] wellfit_cli UI 模組 (ANSI 顏色、Spinner、ProgressBar、Table、Panel)
- [x] build.ps1 增強 (健康檢查、彩色輸出、進度條)
- [x] verify-compliance.ps1 增強 (彩色輸出、進度步驟、摘要面板)
- [x] validate-templates.ps1 (模板驗證工具)
- [x] normalize-filename.ps1 (檔名標準化)
- [x] notify-exception.ps1 (異常通知)

### Sprint 5: Sync 基礎骨架（部分實作；非多平台同步交付）
- [x] wellfit_sync CLI、JSON 設定與健康檢查
- [x] SQLite 本機狀態表及 peer 可達性探索
- [ ] 檔案／記錄傳輸、TLS、身份驗證與授權
- [ ] 增量游標、衝突政策、重試／背壓與故障恢復
- [ ] Windows/Linux/macOS/NAS 部署與容量驗收

### Sprint 6: 自動化部署（已完成）
- [x] Windmill 整合架構文件
- [x] Windmill 原生二進位部署方案
- [x] Docker Compose 配置 (備選)
- [x] n8n 整合方案 (備選)
- [x] 部署方案深度分析 (NAS/k3s/Podman/eBPF)

### Sprint 7: 分析與文件（已完成）
- [x] 第一性原理分析
- [x] 根因樹狀分析 (Mermaid 圖)
- [x] 多因子分析模型 (AHP)
- [x] 候選方案比較 (3 方案)
- [x] 數學函數與組合方法
- [x] 最終聯立解決方案
- [x] 頂級企業解決方案
- [x] 完整多層展開方案
- [x] 部署方案深度分析

---

## 5. 系統串接驗證

### 5.1 模組依賴圖 (DAG)

```
wellfit_core (共享核心)
    │
    ├──► wellfit_reports (報表自動化)
    │       └──► ReportAutomator.parse_native_csv_report()
    │
    ├──► wellfit_shopee (蝦皮進貨)
    │       └──► ShopeeInboundEngine.process_raw_orders()
    │
    ├──► wellfit_mtrix (MTRIX 介接引擎)
    │       ├──► MtrixEngine.process_drop()
    │       │       ├──► check_filename() (VR-01)
    │       │       ├──► compare_header() (VR-02)
    │       │       ├──► validate_row() (VR-03~VR-07)
    │       │       ├──► apply_cross_file_rules() (VR-08~VR-11, VR-13, VR-14)
    │       │       ├──► format_warnings() (VR-15)
    │       │       ├──► repair_header() (ANOM-01~ANOM-03, ANOM-12)
    │       │       └──► repair_row() (ANOM-01, ANOM-05~ANOM-11)
    │       └──► ExceptionNotification (異常通知)
    │
    ├──► wellfit_cli (CLI 進入點)
    │       ├──► status (帳本狀態)
    │       ├──► shopee (波次 1)
    │       ├──► reports (波次 2)
    │       ├──► mtrix (波次 3)
    │       ├──► audit (全量稽核)
    │       └──► spec (規格目錄)
    │
    └──► wellfit_sync (同步守護程式)
            ├──► status (同步狀態)
            ├──► add-device/remove-device/list-devices
            ├──► sync (同步週期)
            ├──► health (健康檢查)
            └──► install/uninstall (服務管理)
```

### 5.2 資料流驗證

```
Alpargatas 交付檔案 (5 CSV)
  │
  ├──► normalize-filename.ps1 → 標準化檔名
  │
  ├──► wellfit.exe mtrix → MTRIX 驗證
  │      ├──► mtrix-report.json (驗證報告)
  │      └──► *_exceptions.json (異常通知)
  │
  ├──► notify-exception.ps1 → 業務方通知
  │
  ├──► verify-compliance.ps1 → ISMS 合規驗證
  │
  └──► Windmill Flow → 自動化管線
         ├──► Schedule Trigger (每日 08:00)
         ├──► File Watcher
         ├──► Execute Command (wellfit.exe)
         ├──► PowerShell Script (normalize/notify)
         ├──► Email/Teams 通知
         └──► Archive Files
```

---

## 6. 反覆測試結果

### 6.1 Rust 全量測試

| Crate | 測試數 | 通過 | 失敗 |
|---|---|---|---|
| wellfit_core | 49 | 49 | 0 |
| wellfit_reports | 8 | 8 | 0 |
| wellfit_shopee | 13 | 13 | 0 |
| wellfit_mtrix | 47 | 47 | 0 |
| wellfit_cli | 12 | 12 | 0 |
| wellfit_sync | 19 | 19 | 0 |
| **總計** | **148** | **148** | **0** |

### 6.2 ISMS 合規驗證

| 工具 | 結果 |
|---|---|
| verify-compliance.ps1 | PASS=631 / WARN=0 / FAIL=0 |
| validate-templates.ps1 | 0 issues |
| check-scope-consistency.ps1 | fail=0 / warn=0 |

### 6.3 MTRIX 引擎驗證

| 檔案 | 資料列 | 接受列 | 隔離列 | 修復數 | 狀態 |
|---|---|---|---|---|---|
| ALTHCI | 4 | 4 | 0 | 4 | ok |
| ALTHES | 4 | 4 | 0 | 5 | ok |
| ALTHFV | 4 | 4 | 0 | 12 | ok |
| ALTHPD | 4 | 4 | 0 | 5 | ok |
| ALTHVD | 9 | 9 | 0 | 11 | ok |

### 6.4 PowerShell 工具驗證

| 工具 | 結果 |
|---|---|
| normalize-filename.ps1 -DryRun | 5 掃描, 0 重命名 |
| notify-exception.ps1 | 18 異常通知 (VR-08: 9, VR-09: 9) |
| validate-templates.ps1 | 0 issues |
| build.ps1 -HealthCheck | 通過 |

### 6.5 多平台同步驗證

| 命令 | 結果 |
|---|---|
| wellfit_sync health | `{"status":"ok",...}` |
| wellfit_sync status | 資料庫狀態正常 |
| wellfit_sync list-devices | 裝置清單正常 |

---

## 7. UI/UX 專業化

### 7.1 改善項目

| 項目 | 改善前 | 改善後 |
|---|---|---|
| 顏色常數 | 8 種 | 15 種 (新增 MAGENTA, WHITE, BG_*, ITALIC, UNDERLINE, MOVE_UP) |
| 狀態指示器 | 無 | `Status` enum (Ok/Warning/Error/Info/Pending) + 圖示 + 顏色 |
| 進度條 | 簡單 `#`/`-` | Unicode `█`/`░` + 百分比 + ETA |
| Spinner | `⠁⠂⠄...` | `⠋⠙⠹⠸⠼⠴⠦⠧` (更流暢) |
| 表格 | 無邊框 | `print_table()` 帶邊框與對齊 |
| 面板 | 無 | `panel()` 帶邊框與標題 |
| 摘要盒 | 無 | `summary_box()` 帶 pass/warn/fail/time |
| 狀態行 | 無 | `status_line()` 帶圖示+顏色+對齊 |
| 幫助參考 | 無 | `print_help()` 命令參考表 |
| 錯誤/警告/成功 | 純文字 | 圖示 (✗/⚠/✔) + 顏色 |
| kv 鍵值對 | 簡單對齊 | `kv()` + `kv_status()` 帶狀態顏色 |
| 分隔線 | 單一字元 | 自適應寬度 (max 80) |

### 7.2 設計原則

- **視覺一致性**：統一顏色調色盤與格式
- **無障礙**：尊重 NO_COLOR，使用圖示 + 文字（不僅靠顏色）
- **響應性**：自適應表格寬度、窄終端機緊湊模式
- **易用性**：清晰錯誤訊息，內建說明

---

## 8. 產品化說明

### 8.1 產品組成

| 組件 | 類型 | 部署方式 |
|---|---|---|
| wellfit.exe | 原生二進位 | 直接執行 |
| wellfit_sync | CLI 二進位 | 僅已驗證現有 CLI；service mode 與跨平台部署尚未驗證 |
| Windmill | 原生二進位 | Windows Service / systemd / launchd |
| PostgreSQL | 原生安裝 | Windows Service / systemd |
| PowerShell 腳本 | .ps1 | 直接執行 |
| ISMS 合規套件 | 文件 + CSV + 腳本 | 檔案系統 |

### 8.2 部署步驟

```powershell
# 1. 安裝 Rust 工具鏈
rustup install stable
rustup default stable

# 2. 建置 Wellfit
powershell -ExecutionPolicy Bypass -File Knowledge/tools/dev-env/build.ps1 -Command "build --workspace"

# 3. 安裝 PostgreSQL
# 下載並安裝 PostgreSQL 15

# 4. 建立 Windmill 資料庫
& "C:\Program Files\PostgreSQL\15\bin\psql.exe" -U postgres -c "CREATE USER windmill WITH PASSWORD 'windmill';"
& "C:\Program Files\PostgreSQL\15\bin\psql.exe" -U postgres -c "CREATE DATABASE windmill OWNER windmill;"

# 5. 下載並安裝 Windmill
Invoke-WebRequest -Uri "https://github.com/windmill-labs/windmill/releases/latest/download/windmill-x86_64-pc-windows-msvc.exe" -OutFile "C:\tools\windmill.exe"

# 6. 安裝 Windows Service
sc.exe create WindmillServer binPath= "C:\tools\windmill.exe --database-url postgres://windmill:windmill@localhost:5432/windmill --mode server" start= auto
sc.exe create WindmillWorker binPath= "C:\tools\windmill.exe --database-url postgres://windmill:windmill@localhost:5432/windmill --mode worker" start= auto
# Sync service mode is not implemented; do not install this as a service.

# 7. 啟動服務
sc.exe start WindmillServer
sc.exe start WindmillWorker

# 8. 驗證
Invoke-WebRequest -Uri "http://localhost:8000/api/version"
# Health response only checks the CLI health command; it does not test data transfer.
wellfit_sync health
```

### 8.3 產品規格

| 規格 | 值 |
|---|---|
| 支援平台 | Windows 10/11, Linux (Ubuntu/Debian/CentOS), macOS, NAS (Synology/QNAP/TrueNAS) |
| 記憶體需求 | 最低 4GB, 建議 8GB+ |
| 儲存需求 | 最低 10GB, 建議 50GB+ |
| CPU 需求 | 最低 2 核, 建議 4 核+ |
| 網路需求 | 100Mbps+, 建議 1Gbps |
| 資料庫 | PostgreSQL 15+ (Windmill), SQLite (wellfit) |
| 安全 | TLS 1.3, RBAC, Audit Log, SHA-256 |
| 合規 | ISO 27001:2022, ISO/IEC 25010 |

---

## 9. 最終驗證結果

| 檢查項目 | 預期結果 | 實際結果 | 狀態 |
|---|---|---|---|
| `cargo check --workspace --all-targets` | 0 錯誤 | 0 錯誤 | ✔ 通過 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 警告 | 0 警告 | ✔ 通過 |
| `cargo test --workspace` | 全部通過 | 148 passed, 0 failed | ✔ 通過 |
| `verify-compliance.ps1` | PASS>0 / WARN=0 / FAIL=0 | PASS=631 / WARN=0 / FAIL=0 | ✔ 通過 |
| `validate-templates.ps1` | 0 issues | 0 issues | ✔ 通過 |
| `check-scope-consistency.ps1` | fail=0 / warn=0 | fail=0 / warn=0 | ✔ 通過 |
| `wellfit.exe mtrix` (5 份實際檔案) | 5/5 解析, 25/25 接受 | 5/5 解析, 25/25 接受 | ✔ 通過 |
| `normalize-filename.ps1 -DryRun` | 5 掃描, 0 重命名 | 5 掃描, 0 重命名 | ✔ 通過 |
| `notify-exception.ps1` | 異常通知輸出 | 18 異常通知 | ✔ 通過 |
| `wellfit_sync health` | `{"status":"ok",...}` | `{"status":"ok",...}` | ✔ 通過 |
| `build.ps1 -HealthCheck` | 通過 | 通過 | ✔ 通過 |

---

## 10. 結語

本文件是交付計畫與歷史驗證紀錄，不構成目前的生產就緒證明。Wellfit 有 7 個 Rust crates；既有測試與靜態檢查通過，不等於完成高流量容量驗收。`wellfit_sync` 尚無資料傳輸功能；部署拓樸、數值業務契約及生產 SLA 仍須各自驗證與核准。

不得依本文件中的歷史 PASS 數字推定所有功能已完成或可正式部署。請以當前程式碼、可重現測試輸出、具代表性的容量基準及正式業務契約作為交付依據。