# Wellfit 安全工作區審查（Phase 1–3 交付）

**文件編號：** WFLT-SEC-001  
**版本：** 1.0  
**日期：** 2026-11-04  
**範圍：** Wellfit 9-crate 工作區（聚焦 Phase 1–3 新增模組）  
**狀態：** 本機工程審查；不構成 ISO 27001 驗證、滲透測試或生產安全保證  
**驗證標記：** ✔ 已實測｜△ 依現況推導｜⚠ 未驗證

---

## 1. 審查範圍

| 類別 | 對象 |
|---|---|
| 新增 crate | `wellfit_assist`、`wellfit_integration` |
| 新增模組 | `wellfit_core::{automation, audit_integrity, customer_automation, inventory_automation}`、`wellfit_shopee::procurement_automation`、`wellfit_mtrix::outbound_automation`、`wellfit_bankpay::payable_automation` |
| 既有機制 | IdempotencyLedger、ReleaseGate、MDM、access_control、jit_permission、rustls HTTP client |

## 2. 依賴安全

### 2.1 漏洞掃描（✔ 已實測）

| 工具 | 命令 | 結果 |
|---|---|---|
| cargo audit | `cargo audit --file Wellfit/Cargo.lock` | **170 crate 依賴，0 漏洞，0 警告** |
| cargo deny（advisories） | `cargo deny check` | `advisories ok` |

Dev 根工作區（FlapsErp root，非本審查範圍）另有 1 筆已設定允許的 unmaintained 警告（`basic-toml` via askama，RUSTSEC-2026-0374），與 Wellfit 無關。

### 2.2 授權合規（✔ 已實測）

`cargo deny check` 修正後結果：**`advisories ok, bans ok, licenses ok, sources ok`**

| 發現 | 處置 |
|---|---|
| `CDLA-Permissive-2.0` 未列於 allowlist（來源：`webpki-roots` 0.26.11 / 1.0.11，TLS 信任錨資料） | 已加入 `deny.toml` allowlist，附理由註解；此為 Linux Foundation 寬鬆資料授權，無 copyleft 義務 |
| `r-efi`（LGPL-2.1-or-later OR MIT OR Apache-2.0 三選一） | 通過：SPDX 析取式中 MIT 已允許 |

### 2.3 來源驗證（✔ 已實測）

`sources ok`：所有依賴來自 crates.io 官方索引，無 git 來源、無未知 registry。

## 3. 程式碼安全不變量（✔ 已實測）

| 不變量 | 落地機制 | 驗證 |
|---|---|---|
| 零 unsafe | 每個 `.rs` 首行 `#![forbid(unsafe_code)]` | 全部新增 13 檔案遵守 |
| Clippy 零警告 | `-D warnings` | 全工作區 0 警告 |
| 金額／數量禁用浮點 | `i64` cents、`ScaledDecimal` | 新模組全部整數運算 |
| 欄位以名稱解析 | `ColumnContract`（禁序位退讓） | 三個新自動化模組均先 resolve 後讀取 |
| 錯誤不靜默 | malformed 資料一律隔離或報錯，不歸零 | 各引擎既有測試覆蓋 |
| 密碼學用具 | `sha2`（SHA-256 稽核鏈）、`rustls`（TLS 1.2+） | 無自製密碼學 |

## 4. 密鑰與機密掃描（✔ 已實測）

- 掃描模式：`(password|api[_-]?key|secret|token)\s*[:=]\s*["'][^"']{8,}`
- 掃描範圍：`Wellfit/crates/**`
- 結果：1 筆命中 — `wellfit_sync/src/config_tests.rs:70` 之 `"test-token-123"`，為**測試 fixture 字面值**，非真實憑證；處置：接受（測試資料）。
- 真實憑證一律經環境變數／本機設定注入（`.env` 已列入 `.gitignore`；`.env.example` 僅含欄位範例）。

## 5. 稽核與可追溯（✔ 已實測）

| 機制 | 實作 | 測試證據 |
|---|---|---|
| SHA-256 稽核鏈 | `audit_integrity::{compute_hash, verify_hash_chain, AuditEntry}` | 13 測試（含竄改偵測） |
| 每個自動化管線執行產生 audit_hash | 5 個 orchestrator | 各 orchestrator 測試驗證 64-hex 輸出 |
| 冪等鎖 | `IdempotencyLedger`（SQLite、穩定業務鍵） | ledger 測試 + replay 測試 |
| 發布閘門 | `ReleaseGate`（白名單、狀態機） | release_gate 測試 |

## 6. 網路邊界（△ 依現況推導）

| 項目 | 狀態 |
|---|---|
| TLS | `rustls` 0.23，TLS 1.2+，`ring` backend；webpki-roots 信任錨 |
| 憑證釘選 | `wellfit_bankpay` 支援 pinned hash 設定（測試覆蓋） |
| 外部呼叫 | 僅 E.Sun 銀行 API（sandbox pending，OQ-E3）；無其他外部連線 |
| 資料外傳 | 無：所有模組本機執行；Sync 資料傳輸尚未實作 |

## 7. 存取控制（△ 依現況推導）

- `access_control`：RBAC + ABAC、SoD（職責分離）違規偵測（既有測試）。
- `jit_permission`：時間窗授權、核准流程、到期回收（既有測試）。
- ⚠ 未執行：端對端權限實測（需部署環境與操作者）；列於上線前待辦 P-02。

## 8. 發現與處置彙總

| # | 發現 | 等級 | 處置 | 狀態 |
|---|---|---|---|---|
| S-01 | `CDLA-Permissive-2.0` 授權未允許（webpki-roots） | 低（合規） | 加入 allowlist + 理由 | ✔ 已修正 |
| S-02 | 測試檔含 token 字面值 | 極低 | 確認非真實憑證，接受 | ✔ 已確認 |
| S-03 | 端對端權限／憑證實測未執行 | 中 | 列入上線前待辦（P-02） | ⚠ 待辦 |
| S-04 | BankPay 真實銀行連線未驗證（沙盒憑證 pending） | 中 | 憑證取得後回歸（OQ-E3） | ⚠ 待辦 |
| S-05 | 無 SAST/DAST 深度掃描 | 中 | 建議上線前執行（P-07 CI 擴充） | ⚠ 待辦 |

## 9. 限制

本審查為離線工程審查：不含滲透測試、原廠元件（FlapsErp）黑箱安全評估、生產環境組態檢視或營運監控驗證。所有「✔」項均可由第 2–6 節所列命令重現；「⚠」項在上線前處理。

---

*重現命令（repository 根目錄）：*

```powershell
cargo audit --file Cargo.lock
cargo deny check
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
