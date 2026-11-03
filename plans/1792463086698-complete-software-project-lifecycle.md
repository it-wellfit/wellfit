# Wellfit 企業級軟體專案生命週期計畫

> 計畫編號：KNB-WELL-LIFE-001｜版本：v1.0｜日期：2026-10-25
> 狀態：Historical proposal — not execution-ready (reviewed 2026-11-02)
> 適用範圍：原始草案記載 6 crates；現有 manifest 已列 7 crates。其程式碼／測試基線與相鄰 lifecycle plans 有重疊，請先讀 `../PLAN.md`。

---

## 1. 專案現況盤點

### 1.1 已建立資產

| 資產類型 | 數量 | 狀態 | 驗證方式 |
|---|---|---|---|
| Rust Crate | 6 (wellfit_core, wellfit_reports, wellfit_shopee, wellfit_mtrix, wellfit_cli, wellfit_sync) | ✔ 已完成 | `cargo check --workspace --all-targets` |
| 驗證規則 (VR-01~VR-15) | 15 | ✔ 已實作 | `cargo test --workspace` |
| 修復規則 (ANOM-01~ANOM-12) | 12 | ✔ 已實作 | `cargo test --workspace` |
| CLI 子命令 | 6 (status/shopee/reports/mtrix/audit/spec) | ✔ 已實作 | `cargo test --test integration_tests` |
| 合規工具 | 9 | ✔ 已實作 | `verify-compliance.ps1` PASS=631/WARN=0/FAIL=0 |
| 分析文件 | 9+ | ✔ 已建立 | 人工審閱 |
| 管理文件 | 6+ | ✔ 已建立 | 人工審閱 |
| 架構文件 | 4 | ✔ 已建立 | 人工審閱 |

### 1.2 待完成項目（優先級）

| # | 項目 | 優先級 | 說明 |
|---|---|---|---|
| G-01 | 端對端測試覆蓋率 < 90% | 高 | 僅 131 單元測試 + 12 整合測試，缺少跨模組 E2E |
| G-02 | UI/UX 不一致 | 高 | 各 CLI 子命令輸出格式不一致 |
| G-03 | 缺少效能基準 | 中 | 無 `cargo bench` 結果 |
| G-04 | 無自動化 CI/CD | 中 | 僅有本地 build.ps1 |
| G-05 | 錯誤處理不一致 | 中 | 各子命令錯誤格式不一 |
| G-06 | 文件不完整 | 中 | 各 crate 缺少 API 參考文件 |
| G-07 | 遺失測試 | 高 | wellfit_reports 與 wellfit_sync 單元測試為 0 |

---

## 2. 產品規格（可測量）

### 2.1 功能需求（已實作，維持現狀）

| ID | 需求 | 優先級 | 驗收標準 |
|---|---|---|---|
| FR-01 | MTRIX 五檔解析 | P0 | 5/5 檔解析，25/25 列接受 |
| FR-02 | VR-01~VR-15 驗證規則 | P0 | 每條規則至少 1 個單元測試 |
| FR-03 | ANOM-01~ANOM-12 修復規則 | P0 | 每條規則至少 1 個單元測試 |
| R-07 | 零 unsafe / 零 clippy 警告 | P0 | `cargo clippy --workspace --all-targets -- -D warnings` |
| R-08 | 零浮點金額 | P0 | 所有金額使用 ScaledDecimal |
| R-09 | 欄位名稱解析，禁止序位退讓 | P0 | 所有欄位透過名稱解析 |
| R-10 | 合規驗證 0 FAIL | P0 | `verify-compliance.ps1` PASS=631/WARN=0/FAIL=0 |

### 2.2 非功能需求（可測量）

| 類別 | 指標 | 當前值 | 目標值 | 驗證方法 |
|---|---|---|---|---|
| 效能 | MTRIX 處理時間 | < 5s/批次 | < 1s/批次 | `cargo bench` |
| 可擴展 | 同時處理檔案數 | 5 檔 | 100+ 檔 | 壓力測試 |
| 可用性 | 系統正常時間 | 未測量 | 99.99% | 故障注入測試 |
| 維護性 | 程式碼複雜度 | 未測量 | < 5 CC | `cargo clippy` + 人工審核 |
| 安全性 | 弱點數量 | 未掃描 | 零高危 | `cargo audit` |
| 可移植性 | 支援平台數 | 3 平台 | 4 平台 | 多平台編譯測試 |
| 測試覆蓋率 | 單元測試覆蓋率 | 估計 >85% | > 90% | `cargo tarpaulin` |

### 2.3 UI/UX 專業化需求

| 維度 | 具體要求 | 當前狀態 | 目標 |
|---|---|---|---|
| 視覺一致性 | 各子命令輸出相同顏色方案、圖示系統 | 各子命令輸出格式不一 | 所有子命令使用相同 UI 模組 |
| 導航 | `--help` 清晰顯示所有子命令 | 已實作，可改善 | 所有子命令具備 `--help`，嵌套清晰 |
| 無障礙 | 尊重 `NO_COLOR`/`CI`，不僅依賴顏色 | 已部分實作 | 所有訊息皆具圖示 + 文字 |
| 響應性 | 終端寬度自適應 | 固定寬度 80 | 自適應終端寬度 |
| 易用性 | 錯誤訊息包含建議動作 | 部分錯誤無建議 | 所有錯誤皆含建議動作 |

---

## 3. 系統架構（多層）

### 3.1 架構圖（Mermaid）

```
Layer 0: 硬體與基礎設施
  x86_64 / ARM64 CPU · 記憶體 4-256GB · 儲存 SSD/NVMe/HDD/RAID
  網路 1GbE-10GbE / WiFi
                              │
                              ▼
Layer 1: 作業系統層
  Windows 10/11 · Linux (Ubuntu/Debian/CentOS) · macOS · NAS OS
                              │
                              ▼
Layer 2: 執行時層
  Rust Runtime (零 GC, 零開銷) · SQLite (內嵌) · PostgreSQL
  TLS 1.3 (rustls, 純 Rust)
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Layer 3: 應用服務層                                                │
│  ┌──────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │ wellfit_core │  │ wellfit_reports│  │ wellfit_shopee│            │
│  │    MDM      │  │   Report     │  │   Shopee    │            │
│  │   帳本      │  │   Automator   │  │   Engine    │            │
│  │   ReleaseGate│  │              │  │             │        │
│  └──────────────┘  └──────────────┘   └─────────────┘                │
│                                                                   │
│  ┌──────────────┐  ┌─────────────┐  ┌──────────────┐            │
│  │ wellfit_mtrix │  │ wellfit_cli │  │ wellfit_sync  │            │
│  │   MTRIX     │  │   CLI       │  │   Sync      │            │
│  │   Engine    │  │   Entrypoint│  │   Daemon    │            │
│  └──────────────┘  └─────────────┘  └──────────────┘            │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Layer 4: 整合層                                                    │
│  Windmill Flow Engine · Prometheus · Grafana · Loki                │
│  Keycloak SSO · HashiCorp Vault · 各種 PowerShell 腳本            │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Layer 5: 安全層                                                  │
│  RBAC · 稽核日誌 · SHA-256 雜湊 · TLS 1.3                          │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 依賴關係圖（DAG）

```
wellfit_core (共享核心)
    │
    ├──► wellfit_reports (報表自動化)
    │       │
    │       └──► ReportAutomator.parse_native_csv_report()
    │
    ├──► wellfit_shopee (蝦皮進貨)
    │       │
    │       └──► ShopeeInboundEngine.process_raw_orders()
    │
    ├──► wellfit_mtrix (MTRIX 介接引擎)
    │       │
    │       ├──► MtrixEngine.process_drop()
    │       │       │
    │       │       ├──► check_filename() (VR-01)
    │       │       ├──► compare_header() (VR-02)
    │       │       ├──► validate_row() (VR-03~VR-07)
    │       │       ├──► apply_cross_file_rules() (VR-08~VR-11, VR-13, VR-14)
    │       │       ├──► format_warnings() (VR-15)
    │       │       ├──► repair_header() (ANOM-01~ANOM-03, ANOM-12)
    │       │       └──► repair_row() (ANOM-01, ANOM-05~ANOM-11)
    │       │
    │       └──► ExceptionNotification (異常通知)
    │
    ├──► wellfit_cli (CLI 進入點)
    │       │
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

---

## 4. 敏捷迭代實作計畫

### Sprint 1: 基礎架構（第 1 週）

| 項目 | 動作 | 驗收標準 |
|---|---|---|
| 1.1 | 建立 `.gitignore`，確保 `.env`、金鑰、客戶原始資料不入倉庫 | 所有敏感檔案被排除 |
| 1.2 | 建立 `.env.example` 模板 | 所有環境變數有範本 |
| 1.3 | 建立 `tests/` 目錄結構，整合所有測試 | 測試目錄結構清晰 |
| 1.4 | 建立 `.cargo/config.toml`，設定 target 目錄為 `%LOCALAPPDATA%\Temp\wellfit-target` | 避免應用程式控制問題 |
| 1.5 | 建立 `build.ps1`，統一建置與測試入口 | 可重複執行，無錯誤 |

### Sprint 2: UI/UX 專業化（第 2 週）

| 項目 | 動作 | 驗收標準 |
|---|---|---|
| 2.1 | 統一所有 CLI 輸出格式，使用 `ui.rs` 模組 | 所有子命令輸出格式一致 |
| 2.2 | 為所有錯誤訊息添加錯誤代碼與建議動作 | 所有錯誤包含錯誤代碼與建議 |
| 2.3 | 為所有子命令添加 `--help` 說明 | 每個子命令都有完整說明 |
| 2.4 | 實作 `wellfit --help` 主幫助畫面 | 包含所有子命令說明 |
| 2.5 | 實作錯誤代碼系統（ERR-001~ERR-99） | 每個錯誤有唯一識別碼 |

### Sprint 3: 測試覆蓋率提升（第 3-4 週）

| 項目 | 動作 | 驗收標準 |
|---|---|---|
| 3.1 | 為 wellfit_reports 編寫單元測試 | 測試覆蓋率 > 80% |
| 3.2 | 為 wellfit_sync 編寫單元測試 | 測試覆蓋率 > 80% |
| 3.3 | 為 wellfit_cli 編寫整合測試 | 覆蓋所有子命令 |
| 3.4 | 為 wellfit_mtrix 編寫邊界測試案例 | 涵蓋所有邊界情況 |
| 3.5 | 建立 `cargo bench` 基準測試 | 處理時間可量化 |

### Sprint 5: 安全性與證照管理（第 5 週）

| 項目 | 動作 | 驗收標準 |
|---|---|---|
| 5.1 | 建立 `.env.example` 模板 | 所有環境變數有範本 |
| 5.2 | 建立 `config.example.toml` | 所有配置有範本 |
| 5.3 | 執行 `cargo audit`，修復所有高危漏洞 | 零高危漏洞 |
| 5.4 | 實作密鑰輪替機制 | 密鑰可輪替 |
| 5.5 | 實作存取記錄（Audit Log） | 所有敏感操作有日誌 |

### Sprint 6: 部署與 CI/CD（第 6 週）

| 項目 | 動作 | 驗收標準 |
|---|---|---|
| 6.1 | 建立 GitHub Actions 工作流 | 自動執行 build、test、clippy |
| 6.2 | 建立多平台構建腳本 | Windows/Linux/macOS 皆可建置 |
| 6.3 | 建立發布腳本 | 自動產生版本標記 |
| 6.4 | 建立安裝腳本 | 一鍵安裝 |

---

## 6. 風險與緩降

| 風險 | 嚴重度 | 機率 | 緩降措施 |
|---|---|---|---|
| 工具鏈版本不一致 | 中 | 中 | 鎖定 Rust 版本 |
| 依賴套件漏洞 | 高 | 中 | 定期 `cargo audit` |
| 平台相容性問題 | 中 | 高 | 多平台 CI 測試 |
| 效能退化 | 中 | 中 | 基準測試監控 |
| 安全性弱點 | 高 | 低 | 定期安全掃描 |
| 文件過時 | 低 | 高 | 每次變更同步更新文件 |

---

## 7. 最終驗證清單

| 檢查項目 | 預期結果 | 狀態 |
|---|---|---|
| `cargo check --workspace --all-targets` | 0 錯誤 | ✔ 已通過 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 警告 | ✔ 已通過 |
| `cargo test --workspace` | 全部通過 | ✔ 已通過（131+） |
| `verify-compliance.ps1` | PASS>0 / WARN=0 / FAIL=0 | ✔ 已通過 |
| `validate-templates.ps1` | 0 issues | ✔ 已通過 |
| `check-scope-consistency.ps1` | fail=0 / warn=0 | ✔ 已通過 |
| `wellfit.exe mtrix`（5 份實際檔案）| 5/5 解析，25/25 接受 | ✔ 已通過 |
| `wellfit_sync health` | `{"status":"ok",...}` | ✔ 已通過 |

---

## 8. 升級到遠端協作的標準

| 條件 | 閾值 | 說明 |
|---|---|---|
| 開發者數量 | > 3 人 | 需要協作平台 |
| 部署環境數 | > 2 個 | 需要 CI/CD |
| 資料量 | > 1GB/月 | 需要遠端儲存 |
| 安全需求 | 需要 SSO/MFA | 需要 Keycloak |
| 監控需求 | 需要集中監控 | 需要 Prometheus/Grafana |
| 高可用需求 | 需要 99.9%+ | 需要多節點 |

### 升級路徑

```
Phase 1: 本地優先（當前）
  ├── Git 本地倉庫
  ├── SQLite 本地資料庫
  └── 本地驗證腳本

Phase 2: 混合模式（觸發條件達成時）
  ├── Git 遠端倉庫（GitHub/GitLab）
  ├── PostgreSQL 遠端資料庫
  ├── Windmill 工作流引擎
  └── Prometheus/Grafana 監控

Phase 3: 全雲端模式（大規模時）
  ├── Kubernetes 叢集
  ├── 服務網格 (Istio/Linkerd)
  ├── 分散式追蹤 (Jaeger)
  └── 多雲部署（AWS/Azure/GCP）
```

---

## 9. 驗證方法

### 9.1 本地驗證命令

```powershell
# 1. Rust 全量驗證
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 2. ISMS 合規驗證
powershell -ExecutionPolicy Bypass -File Knowledge/tools/compliance/verify-compliance.ps1
powershell -ExecutionPolicy Bypass -File Knowledge/tools/compliance/validate-templates.ps1
powershell -ExecutionPolicy Bypass -File Knowledge/tools/compliance/check-scope-consistency.ps1

# 3. MTRIX 引擎驗證
wellfit.exe mtrix --dir <交付檔案目錄> --report <報告路徑> --batch <批次ID>

# 4. 異常通知
powershell -ExecutionPolicy Bypass -File Knowledge/tools/compliance/notify-exception.ps1 -ReportPath <報告路徑>

# 5. 同步守護程式驗證
wellfit_sync health
wellfit_sync status
```

---

## 10. 結語

本文件為歷史提案，不是目前執行授權。若經核准重新採用，須先依 `../PLAN.md` 對齊範圍與現況，再重驗所有原始驗收條件。