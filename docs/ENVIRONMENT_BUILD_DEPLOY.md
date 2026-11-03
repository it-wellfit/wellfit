# Wellfit BankPay 環境、建置與部署手冊

**文件編號**：WFLT-DOC-007  
**版本**：1.0  
**狀態**：Draft  
**文件語言**：繁體中文（技術名詞保留標準英文）  
**證據來源**：`Wellfit/AGENTS.md`、`Wellfit/Cargo.toml`、`Wellfit/crates/wellfit_bankpay/` 原始碼、`docs/bankpay/07-deployment-and-operations.md`

---

## 1. 使用說明

本文件說明 Wellfit BankPay 模組的開發環境需求、建置指令、部署前置作業、執行方式、監控與除錯程序。所有指令皆以實際可執行的命令為基礎，未實際驗證的指令皆標記為⚠ 待確認。

---

## 2. 開發環境需求

### 2.1 必要工具與版本

| 工具 | 版本要求 | 用途 | 驗證指令 |
|------|---------|------|---------|
| Rust 工具鏈 | stable-x86_64-pc-windows-msvc | 編譯 Rust 程式碼 | `rustc --version` |
| Cargo | 隨 Rust 安裝 | 專案建置與依賴管理 | `cargo --version` |
| Visual Studio 建置工具 | 2019 或更新 | 提供 `link.exe` (MSVC linker) | `link.exe` 可用 |
| PowerShell | 5.1+ | 執行建置腳本 `build.ps1` | `$PSVersionTable.PSVersion` |

### 2.2 應用程式控制原則限制（⚠ 已知環境限制）

本工作站的應用程式控制原則會**阻擋在桌面路徑下建置的 `.exe` 執行檔**。直接執行 `cargo build` 產出的執行檔可能無法運行。

**解決方式**：必須透過知識庫的 `build.ps1` 腳本，將 target 目錄導向 `%LOCALAPPDATA%\Temp\wellfit-target`（桌面路徑之外）。

```powershell
cd "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit"

powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "<cargo 子命令>" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

`build.ps1` 執行三件事（✔ 已確認，見 `Wellfit/AGENTS.md` §3）：

1. 設定 MSVC 工具鏈 (`stable-x86_64-pc-windows-msvc`)。
2. 載入 Visual Studio 建置環境 (`vcvars64.bat`)，使 `link.exe` 可用。
3. 將 target 目錄導向儲存庫之外（預設 `%LOCALAPPDATA%\Temp\wellfit-target`）。

**協作注意**：`build.ps1` 會把 cargo 指令寫到暫存 `.bat`，檔名帶有行程 ID，多個代理人同時呼叫不會互相覆寫。

---

## 3. 建置指令

從 `Wellfit/` 目錄執行所有指令。

### 3.1 型別檢查（推薦先執行）

```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "check --workspace --all-targets" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

### 3.2 Clippy 零警告檢查（強制）

```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "clippy --workspace --all-targets -- -D warnings" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

> **不變量 #2**：Clippy 必須零警告。任何警告都視為缺陷，必須修正後才能繼續。

### 3.3 建置 CLI (`wellfit.exe`)

```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "build -p wellfit_cli" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

建置後的執行檔位於：`$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe`

### 3.4 建置全部 crate

```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "build --workspace" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

### 3.5 執行測試

```powershell
# 僅 wellfit_bankpay 單元測試
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test -p wellfit_bankpay" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 全部工作區測試（不因單一失敗中止）
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test --workspace --no-fail-fast" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 僅編譯測試二進位檔（不執行，用於 AppLocker 受限環境）
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test -p wellfit_bankpay --no-run" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

---

## 4. 執行 CLI

### 4.1 設定執行檔路徑變數

```powershell
$wf = "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe"
```

### 4.2 BankPay 子命令使用範例

#### 匯出應付憑單（Export）

```powershell
& $wf bankpay export `
  --input data/vouchers.csv `
  --batch BATCH-TEST-001 `
  --batch-date 2026-11-02 `
  --encoding utf8 `
  --esun-url https://127.0.0.1:8443 `
  --source-system S001 `
  --customer-id 12345678 `
  --payer-account ACCT-123 `
  --payer-id 1234567890 `
  --format-type 2 `
  --client-cert-path certs/erp-client.pem `
  --client-key-path certs/erp-key.pem `
  --ca-cert-path certs/esun-ca.pem `
  --db data/wellfit_ledger.sqlite
```

#### 匯入銀行結果（Import）

```powershell
& $wf bankpay import `
  --input data/bank-results.csv `
  --batch BATCH-TEST-RES-001 `
  --encoding utf8 `
  --db data/wellfit_ledger.sqlite
```

#### 追溯查詢（Query）

```powershell
& $wf bankpay query --kind voucher --key AP-001 --db data/wellfit_ledger.sqlite
& $wf bankpay query --kind tx --key ETRACE-9001 --db data/wellfit_ledger.sqlite
& $wf bankpay query --kind batch --key BATCH-TEST-001 --db data/wellfit_ledger.sqlite
```

### 4.3 ⚠ 重要：CLI Export 參數未使用問題

在 `crates/wellfit_cli/src/main.rs:2010–2014` 的 `run_bankpay` Export 分支中，`source_system`、`customer_id`、`payer_account`、`payer_id`、`format_type` 參數被標記為 `_`（未使用），**未傳遞至 `BankpayEngine::export_vouchers()` 或 `build_twd_payment_request()`**。

這意味著：
- CLI 匯出命令雖接受這些參數，但目前**不實際使用**它們。
- `export_vouchers()` 使用 CSV 檔案作為輸入，不調用 `build_twd_payment_request()`。
- TLS 憑證路徑參數（`client_cert_path`、`client_key_path`、`ca_cert_path`）會被傳遞至 `EsunHttpClientConfig`，但 `export_vouchers()` 的 transport 閉包目前僅回傳 `Ok(true)`，**不實際發送網路請求**。

**修補建議**：若要啟用 API-based 匯出，需修改 `run_bankpay` 以將這些參數傳遞至引擎，並實作真實的 TLS transport 閉包。

---

## 5. 設定檔（wellfit.toml）

CLI 支援 `--config` 參數指定設定檔路徑，預設為 `./wellfit.toml`。

### 5.1 BankPay 區塊設定範例（⚠ 部分為建議，未在程式碼中讀取）

```toml
[bankpay]
# 安全元件基礎 URL（內部 8443，外部 9443）
base_url_internal = "https://localhost:8443"
base_url_external = "https://eosp.esunbank.com.tw:9443"

# 從 E.Sun 取得的識別碼
source_system_id = "<從銀行取得>"
customer_id = "<企業集團統編，8-10 碼>"

# TLS 憑證路徑（PEM 格式，勿提交至版本控制）
client_cert_path = "certs/erp-client.pem"
client_key_path = "certs/erp-key.pem"
ca_cert_path = "certs/esun-ca.pem"

# 超時與重試
timeout_secs = 30
max_retries = 3

# 預設編碼
default_encoding = "utf8"
```

> **⚠ 待確認**：`wellfit.toml` 的 `[bankpay]` 區塊目前**未在程式碼中讀取**。CLI 參數是硬編碼的預設值，設定檔僅作為文件參考。實際部署時需確認設定檔讀取邏輯是否已實作，或以 CLI 參數傳遞所有值。

---

## 6. TLS 憑證設定（2-Way SSL）

### 6.1 憑證需求

根據 `docs/bankpay/05-security-and-audit.md` §1 與 `http.rs`：

| 憑證 | 用途 | 來源 | 格式 |
|------|------|------|------|
| 客戶端憑證 (`client_cert_path`) | ERP 身分識別 | ERP 提供（銀行交換） | PEM |
| 客戶端私鑰 (`client_key_path`) | 簽名請求 | ERP 提供 | PEM |
| CA 憑證 (`ca_cert_path`) | 驗證伺服器憑證 | 玉山銀行提供 | PEM |

### 6.2 憑證管理規則（✔ 已確認，見 `05-security-and-audit.md` §7）

1. 憑證檔案**不得提交至版本控制**（gitignored）。
2. 憑證透過 `certs/` 目錄或作業系統憑證存放區提供。
3. `docs/bankpay/references/` 僅保留銀行技術文件（PDF/PPT），不保留密鑰。
4. 私鑰密碼與啟用碼必須來自秘密存放區，絕不寫入程式碼或設定檔。

### 6.3 TLS 設定流程（✔ 已確認，見 `http.rs:58–109`）

```
1. 檢查是否提供任何憑證路徑 (ca/client_cert/client_key)
   ↓ 若皆未提供 → 回傳 None，使用 ureq 預設 TLS
2. 若有 ca_cert_path：
   - 讀取 PEM 檔案
   - 解析 CertificateDer 陣列
   - 建置 RootCertStore，加入 CA 憑證
   ↓ 若無 ca_cert_path → 使用 webpki_roots::TLS_SERVER_ROOTS
3. 若有 client_cert_path + client_key_path：
   - 讀取客戶端憑證與私鑰 PEM
   - 解析 CertificateDer 與 PrivateKeyDer
   - builder.with_client_auth_cert(certs, key)
   ↓ 若僅提供一個 → 使用 with_no_client_auth()
4. 回傳 Arc<rustls::ClientConfig>
```

### 6.4 ⚠ 已知缺口：憑證釘固未實作

`05-security-and-audit.md` §2 列為 gap：目前 `EsunHttpClient` 使用 rustls 預設憑證驗證，**未實作憑證釘固 (certificate pinning)**。實際部署前應：
- 擷取銀行安全元件伺服器憑證的 SHA-256 指紋
- 在 `build_tls_config()` 中加入憑證釘固驗證

---

## 7. 部署前置作業清單（✔ 已確認，見 `07-deployment-and-operations.md` §1）

| # | 項目 | 說明 | 狀態 |
|---|------|------|------|
| 1 | 出站 IP 列入銀行防火牆白名單 | 企業對外連線 IP 需經銀行核准 | ⚠ 待確認 |
| 2 | 企業身分與帳戶約定 | 申請為企業網銀身分，約定轉帳限額與指定帳戶 | ⚠ 待確認 |
| 3 | eToken (FXML) 發行與測試 | 硬體權杖用於釋放簽名 | ⚠ 待確認 |
| 4 | 主機名稱與 MAC 地址提交 | 釋放管理員工作站與 ERP/API 伺服器 | ⚠ 待確認 |
| 5 | 連接埠開啟 | 8443（內部）、9443（外部） | ⚠ 待確認 |
| 6 | 安控元件安裝 | 以系統管理員身分安裝於兩台主機 | ⚠ 待確認 |
| 7 | 綁定資訊 | 統編 + ClientId + 啟用碼 | ⚠ 待確認 |
| 8 | 憑證交換 | 銀行憑證 + ERP 憑證相互安裝 | ⚠ 待確認 |
| 9 | SourceSystemId 取得 | 從銀行電子郵件取得 | ⚠ 待確認 |

**全部 9 項目前皆為⚠ 待確認**，等待 E.Sun 客戶經理提供（見 `08-open-questions-and-decisions.md` OQ-E3）。

---

## 8. 網路拓撲（✔ 已確認，見 `07-deployment-and-operations.md` §4）

```
ERP (API) Server ──8443──► Release-Manager Workstation ──9443──► Bank (eosp*.esunbank.com.tw)
        │                          │
        └─────── (兩台皆需連到銀行 9443) ──────────────────────┘
```

| 路徑 | 連接埠 | 說明 |
|------|--------|------|
| ERP → 釋放管理員工作站 | 8443 | 內部防火牆，安全元件通訊 |
| ERP → 銀行 | 9443 | 外部防火牆，直接連銀行後端 |
| 釋放管理員工作站 → 銀行 | 9443 | 外部防火牆，eToken 簽名轉發 |

**環境網域**（見 `05-security-and-audit.md` §1.1）：
- DEV/測試：`http://IP:8080`
- UAT：`eospu.testesunbank.com.tw` (9443)
- 正式：`eosp.esunbank.com.tw` (9443)

---

## 9. 營運手冊（參考 `07-deployment-and-operations.md`）

### 9.1 健康檢查

- 啟動時驗證安全元件 `/health` 回報版本 V2.8，不符則快速失敗。
- 每 60 秒定期健康檢查，失敗則發出警報。

### 9.2 憑證到期監控

- 監控憑證到期日，提前 30 天發出警報。
- 續約流程：聯繫銀行、交換新憑證檔案、重新啟動元件。

### 9.3 重試與斷路器（⚠ 部分為建議，未在程式碼中實作）

| 錯誤類型 | 建議處理 | 程式碼實作狀態 |
|---------|---------|-------------|
| 傳輸錯誤 | 指數退避 (1s, 2s, 4s)，最多 3 次重試 | ⚠ 未實作 (建議) |
| `910068` (銀行寫入失敗) | 帶退避重試 | ⚠ 未實作 |
| 連續 5 次失敗 | 開啟斷路器，60 秒後半開 | ⚠ 未實作 |

### 9.4 DispatchUncertain 對帳

- 逾時後狀態變為 `DispatchUncertain`。
- 透過 `/payment/retrieveTwdPaymentInstructionResult` 重新查詢銀行狀態。
- 若銀行確認 → `Completed`；否則 → `Failed`，通知財務。

---

## 10. 監控與警報（參考 `07-deployment-and-operations.md` §7）

| 指標 | 警報閾值 | 來源 |
|------|---------|------|
| API 成功率 | < 95% (15 分鐘) | 需實作監控 |
| P95 狀態查詢延遲 | > 5 秒 | 需實作監控 |
| 連續傳輸失敗 | ≥ 5 次 | 需實作監控 |
| 憑證到期 | < 30 天 | 需實作監控 |
| 隔離列比例 | > 2% (批次) | 可從 Ledger findings 查詢 |

> **⚠ 待確認**：上述監控指標目前為文件建議，程式碼中**未實作自動監控與警報**。部署時需額外實作監控層，或整合現有監控系統。

---

## 11. 除錯指南

### 11.1 常見問題

| 問題 | 可能原因 | 診斷方式 | 解決方式 |
|------|---------|---------|---------|
| `wellfit.exe` 無法執行 | 桌面路徑下 `.exe` 被應用程式控制阻擋 | 檢查執行檔路徑是否在 `%LOCALAPPDATA%\Temp\wellfit-target` | 透過 `build.ps1` 重新建置 |
| `link.exe` 找不到 | Visual Studio 建置環境未載入 | `build.ps1` 應自動載入 `vcvars64.bat` | 確認 Visual Studio 已安裝 |
| `cargo test` 整合測試失敗 | Windows AppLocker 阻擋 temp 目錄二進位檔 | 檢查測試二進位檔是否可執行 | 使用 `--no-run` 僅編譯，或在 CI 執行 |
| 憑證檔案載入失敗 | 路徑錯誤或檔案不存在 | 檢查 `--client-cert-path` 等參數指向 | 確認憑證檔案存在且可讀 |
| 匯出不派送 | Transport 閉包僅回傳 `Ok(true)`，不發送網路請求 | 檢查 `run_bankpay` Export 分支 (main.rs:2046–2050) | 實作真實 TLS transport 閉包 |
| CLI 參數未生效 | `source_system` 等參數被 `_` 忽略 | 檢查 `main.rs:2010–2014` | 修補程式碼將參數傳遞至引擎 |

### 11.2 查看日誌與發現

BankPay 所有驗證事件皆記錄在 Idempotency Ledger 的 `validation_findings` 表。可透過 `wellfit status` 命令查看：

```powershell
& $wf status --db data/wellfit_ledger.sqlite
```

這會顯示：
- 導出匯入記錄的批次數量
- 隔離記錄數量
- 驗證錯誤與警告數量
- 最近隔離記錄的詳細資訊

---

## 12. 版本升級與回滾

### 12.1 版本升級

1. 更新 `Cargo.toml` 中的 `wellfit_bankpay` 版本（若為外部 crate）或工作區版本。
2. 更新相關文件版本號。
3. 執行 `cargo check --workspace --all-targets` 確認無編譯錯誤。
4. 執行 `cargo clippy --workspace --all-targets -- -D warnings` 確認零警告。
5. 執行 `cargo test --workspace --no-fail-fast` 確認所有測試通過。
6. 更新 `CHANGELOG.md`。

### 12.2 回滾

1. 使用 Git 將 `wellfit_bankpay` crate 回退至前一個提交。
2. 重新建置與測試。
3. 若資料庫結構有變更，需評估是否需要遷移腳本（目前 Ledger 為 SQLite，結構由程式碼動態管理）。
4. 通知財務團隊，恢復至手動銀行入口網站操作（見 `07-deployment-and-operations.md` §8）。

---

## 13. 已知環境限制

| 限制 | 說明 | 影響 | 緩降措施 |
|------|------|------|---------|
| Windows AppLocker | 桌面路徑下 `.exe` 被阻擋 | 無法直接執行 `cargo build` 產出的執行檔 | 透過 `build.ps1` 導向 temp 目錄 |
| 沙箱憑證不可用 | E.Sun 沙箱 (OQ-E3) 憑證尚未取得 | 無法對真實安全元件進行端對端測試 | 建置模擬安全元件進行契約測試 |
| 憑證釘固未實作 | `http.rs` 未實作憑證釘固 | 中間人攻擊風險較高 | 部署前補實作憑證釘固 |
| 監控未實作 | 無自動警報與指標收集 | 問題發現依賴人工檢查 | 整合現有監控系統或實作簡易監控 |

---

*本文件以 `Wellfit/AGENTS.md`、`Wellfit/Cargo.toml` 與實際原始碼為證據基礎。未實際驗證的指令與設定皆以 ⚠ 待確認 標記。*
