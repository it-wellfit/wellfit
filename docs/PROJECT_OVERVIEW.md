# Wellfit BankPay 專案概覽與快速入門

**文件編號**：WFLT-DOC-003  
**版本**：1.0  
**狀態**：Draft  
**適用產品**：Wellfit / Fuyi ERP (FlapsErp) 應付憑單匯款波 — E.Sun Bank 銀企直連整合  
**文件語言**：繁體中文（技術名詞保留標準英文）

---

## 1. 專案是什麼？

Wellfit BankPay 是 Dachen Industrial（大前實業）輔翼 ERP (FlapsErp) 應付憑單匯款波中的 **E.Sun Bank (玉山銀行) 企業銀企直連整合模組**。它實作為 Rust crate `wellfit_bankpay`，內建於 `wellfit.exe` 單一原生二進位發行檔中，透過 CLI 的 `bankpay` 子命令對外提供匯出、匯入與追溯查詢三項功能。

### 1.1 核心目的

本模組解決 Fuyi ERP 系統在應付憑單 (AP Voucher) 結算上的三個核心痛點：

| 痛點 | 說明 | 影響 |
|------|------|------|
| P1 | 人工在銀行入口網站重複鍵入已在 ERP 中的付款資料 | 重鍵錯誤、重複付款風險 |
| P2 | 無即時付款狀態，需人工輪詢入口網站 | 對帳延遲、AP 總帳過時 |
| P3 | 無自動化稽軌跡連結 ERP 憑單 ↔ 銀行交易 | 稽核缺口、可追溯性失敗 |

### 1.2 整合方式（已確認：API-based）

根據 E.Sun 技術文件 V2.8，整合不是傳統的媒體檔案交換（file-based batch upload/download），而是透過本機安裝的 **玉山銀企直連安控元件**（Security Component）提供的 REST API。ERP 透過 HTTPS (TLS 1.2+, 2-Way SSL) 與本機安控元件（預設連接埠 8443）通訊，安控元件再透過 eToken 簽名後轉發至銀行後端。

---

## 2. 系統邊界與不處理的事項

### 2.1 在範圍內（V2.8 核心）

| # | 能力 | API 端點 |
|---|------|---------|
| 1 | TWD 付款釋放（單筆/批次） | `POST /payment/createTwdPaymentInstruction` |
| 2 | TWD 交易狀態查詢 | `POST /payment/retrieveTwdPaymentInstructionResult` |

### 2.2 本階段不在範圍內

- 銀行端憑證簽發 / eToken 設備配發（銀行責任）
- FlapsErp 的自定義 UI（超過現有畫面）
- 行動裝置 / 瀏覽器型付款釋放（僅桌面 eToken）
- 多銀行抽象層（本階段僅 E.Sun）
- FX（外幣）付款與對帳單查詢等其他 6 個 API 端點（僅定義，未實作）

### 2.3 假設條件

| ID | 假設 | 來源 |
|----|------|------|
| A.1 | 安控元件已安裝於釋放管理員工作站與 ERP (API) 伺服器 | `01-problem-and-scope.md` |
| A.2 | E.Sun 以書面提供 SourceSystemId、ClientId 與啟用碼 | `01-problem-and-scope.md` |
| A.3 | E.Sun 提供 2-Way SSL 憑證檔案 | `01-problem-and-scope.md` |
| A.4 | 連接埠 8443（內部）與 9443（外部）已開啟 | `01-problem-and-scope.md` |
| A.5 | ERP 操作員持有有效 eToken 並登錄於銀行網銀流程 | `01-problem-and-scope.md` |

---

## 3. 技術棧（已確認：以原始碼為準）

| 層級 | 技術 | 版本 | 用途 |
|------|------|------|------|
| 語言 | Rust | edition 2021 | 所有程式碼 |
| 工作區 | Cargo workspace | resolver 2 | 7 個 crate |
| HTTP 客戶端 | ureq | 2.10 | REST API 呼叫 |
| TLS | rustls / rustls-pki-types / webpki-roots | 0.23 / 1 / 0.26 | 2-Way SSL |
| 編碼 | encoding_rs | 0.8 | UTF-8 ↔ Big5 / Big5-HKSCS |
| 序列化 | serde / serde_json | 1.0 | JSON 請求/回應 |
| 雜湊 | sha2 | 0.10 | SHA-256 內容雜湊 |
| 日期時間 | chrono | 0.4 | 時間戳記 |
| CLI 解析 | clap | 4.5 | 命令列介面 |
| 資料庫 | rusqlite | 0.32 (bundled) | Idempotency Ledger (SQLite) |
| 定點小數 | wellfit_core::ScaledDecimal | workspace | 金額整數分運算 |

**編譯限制**：`#![forbid(unsafe_code)]` 在所有 crate 中強制執行，不允許 unsafe 程式碼。

---

## 4. 快速入門：從零到可執行

### 4.1 前置條件

| 項目 | 要求 | 驗證方式 |
|------|------|---------|
| Rust 工具鏈 | stable-x86_64-pc-windows-msvc | `rustc --version` |
| Visual Studio 建置環境 | vcvars64.bat (MSVC linker) | `link.exe` 可用 |
| 應用程式控制原則 | 桌面路徑下 `.exe` 會被阻擋 | 透過 `build.ps1` 導向 temp 目錄 |

> 本工作站的應用程式控制原則會阻擋在桌面路徑下建置的執行檔。所有建置／測試／執行必須透過知識庫的 `build.ps1` 腳本，將 target 目錄導向 `%LOCALAPPDATA%\Temp\wellfit-target`。

### 4.2 建置命令

從 `Wellfit/` 目錄執行：

```powershell
# 型別檢查（零警告）
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "check --workspace --all-targets" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# Clippy 零警告檢查
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "clippy --workspace --all-targets -- -D warnings" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 建置 CLI
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "build -p wellfit_cli" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 執行 BankPay 單元測試
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "test -p wellfit_bankpay" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

### 4.3 快速驗證：執行 BankPay CLI

建置成功後，執行檔位於 `$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe`：

```powershell
$wf = "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"

# 查看 BankPay 子命令說明
& $wf bankpay --help

# 匯出應付憑單（範例，不發送實際網路請求）
& $wf bankpay export `
  --input data/vouchers.csv `
  --batch BATCH-TEST-001 `
  --source-system S001 `
  --customer-id 12345678 `
  --payer-account ACCT-123 `
  --payer-id 1234567890 `
  --format-type 2

# 匯入銀行結果檔
& $wf bankpay import `
  --input data/bank-results.csv `
  --batch BATCH-TEST-RES-001

# 追溯查詢
& $wf bankpay query --kind voucher --key AP-001
```

> **⚠ 重要**：上述 `export` 命令中的輸送（transport）閉包目前僅回傳 `Ok(true)`，不會實際發送網路請求至安控元件。實際部署時需以真實 TLS 傳輸替換此閉包（見 `ENVIRONMENT_BUILD_DEPLOY.md` §5）。

### 4.4 預期首次可驗證成果

執行 `cargo test -p wellfit_bankpay` 後，應看到：

```
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

以及 `cargo clippy --workspace --all-targets -- -D warnings` 零警告。

---

## 5. 專案目前狀態

| 維度 | 狀態 | 說明 |
|------|------|------|
| 程式碼完成度 | ✔ 核心完整 | 匯出、匯入、查詢、媒體檔組裝、TLS 設定皆已實作 |
| 編譯檢查 | ✔ 通過 | `cargo check --workspace --all-targets` 無錯誤 |
| Clippy | ✔ 零警告 | `clippy --workspace --all-targets -- -D warnings` 通過 |
| 單元測試 | ✔ 32/32 通過 | `cargo test -p wellfit_bankpay` 全部通過 |
| 整合測試 | ⚠ 無法本機執行 | Windows AppLocker 阻擋 temp 目錄二進位檔；CI 可執行 |
| 沙箱端對端 | ⚠ 待憑證 | E.Sun 沙箱憑證尚未取得（OQ-E3 未決） |
| 文件完整度 | ● 進行中 | 本文件體系正在建構 |

---

## 6. 常見入門錯誤

| 錯誤 | 原因 | 解決方式 |
|------|------|---------|
| `wellfit.exe` 無法執行 | 桌面路徑下 `.exe` 被應用程式控制原則阻擋 | 透過 `build.ps1` 導向 `%LOCALAPPDATA%\Temp\wellfit-target` |
| `link.exe` 找不到 | Visual Studio 建置環境未載入 | `build.ps1` 會自動載入 `vcvars64.bat` |
| `cargo test` 失敗於 `wellfit_bankpay` integration_tests | 測試二進位檔被 AppLocker 阻擋 | 使用 `cargo test --no-run` 僅編譯，或在 CI 中執行 |
| 憑證檔案載入失敗 | 憑證路徑錯誤或檔案不存在 | 檢查 `--client-cert-path` / `--client-key-path` / `--ca-cert-path` 指向 |
| 匯出不派送 | `source_system` / `customer_id` 為 `Option`，CLI 中未傳遞至 `build_twd_payment_request` | CLI `run_bankpay` 中對應參數目前被標記為 `_`（未使用），需工程修補 |

---

## 7. 後續閱讀路徑

| 下一步 | 文件 | 說明 |
|--------|------|------|
| 理解領域術語 | `GLOSSARY.md` | 統一名稱與定義 |
| 查詢程式碼細節 | `CODE_REFERENCE.md` | 物件、屬性、方法參考 |
| 理解資料流向 | `DATA_MODEL.md` | 資料實體與生命週期 |
| 部署與操作 | `ENVIRONMENT_BUILD_DEPLOY.md` | 建置、部署、監控 |
| 撰寫測試 | `TESTING_QUALITY.md` | 測試策略與品質關卡 |
| 修改與擴充 | `MAINTENANCE_EVOLUTION.md` | 維護與變更指南 |

---

*本文件以 `Wellfit/AGENTS.md` (WFLT-AGNT-001) 與實際原始碼為證據基礎。未經驗證的陳述皆以明確標記識別。*
