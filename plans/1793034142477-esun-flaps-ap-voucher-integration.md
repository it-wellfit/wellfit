# 玉山銀行網銀 ↔ 輔翼ERP(FlapsErp) 應付憑單雙向串接與可追溯查詢 — 實作計畫

**版本**: v1.0  
**狀態**: Superseded historical proposal — do not execute this file-based design

**適用**: E.Sun Bank 企業網銀(玉山收付通) 與 FlapsErp (C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp) 應付憑單  
**關鍵約定**: 雙向、快速、安全、易追查、多格式、符合法規、持續演進；不使用 Docker、不使用商業 RPA 授權、Windows 原生二進位部署

> **Disposition (2026-11-02):** This plan predates review of the local [`E.Sun V2.8 technical specification`](../docs/bankpay/references/【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf) and contains superseded file-drop assumptions in its decisions, flows, tasks, risks, and acceptance criteria. The supported design is API-first through the bank security component; JSON (`formatType=2`) is the default, with the documented one-debit/one-credit file payload (`formatType=1`) only where applicable. Use [`docs/bankpay/`](../docs/bankpay/) for maintained requirements and decisions. Do not execute this historical plan. Sandbox/account access and end-to-end validation remain pending. The 6-crate, 148-test, and Clippy claims below are also unverified historical assertions.

---

## 1. 背景與現有資產

### 1.1 FlapsErp (輔翼ERP 桌面版)
- 路徑: `C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp`，Electron 殼，內建 `winax` (COM) + `ffi-napi` (原生 DLL) + `iconv-lite` + `jspdf`
- 主程序 `main.js` + `mainApi.js`，全面同步 IPC (`sendSync`)，COM 物件在主執行緒操作，無快取/批次
- 支援匯出 CSV/JSON/PDF；可作為應付憑單資料來源與寫回入口
- 逆向分析見 `Dev/flaps_erp_performance_analysis.md`

### 1.2 Wellfit Rust 工作區 (Dev/Wellfit/)
- 6 crates 已建置並通過 (148 測試全 pass、clippy 0 警告、cargo fmt)
- `wellfit_core::ledger::IdempotencyLedger` — SQLite 冪等帳本 (`file_batches`, `transaction_keys`, `quarantine_log`, `validation_findings`, `cumulative_balance`, `access_audit_log`)，支援 `BatchScope::Exclusive/PerRun`
- `wellfit_core::release_gate::ReleaseGate` — Two-Way Door / One-Way Door 狀態機 (`DraftGenerated → VerifiedPass/Quarantined → AuthorizedDispatch → Completed/Failed/DispatchUncertain`)
- `wellfit_core::contract::ColumnContract` — 僅名稱解析，無序位退讓
- `wellfit_core::text::split_lines` — CR/CRLF/LF 皆可
- `wellfit_core::scaled` — 定點十進位，零浮點 (金額用此)
- `wellfit_core::federation::OperationalRealityFeedback` — SHA-256 數位簽章回饋封包
- `wellfit_core::mdm.rs` — MDM 目錄與累計餘額

### 1.3 玉山銀行 (E.Sun) 企業網銀
- 2022-11 與鼎新電腦簽 MOU，ERP–銀行 API 直連；加密模組直連銀行，交易結果即時回饋 ERP
- 「玉山收付通」企業收付產品：支援媒體檔案加密、批次匯款/薪轉/開票
- 有開放 API 專區與 TSP 合作機制

### 1.4 工作區憲章不變量
- Rust，`#![forbid(unsafe_code))`，clippy 零警告，零 JS 展示層
- Windows 原生二進位 (sc.exe/systemd/launchd)，無 Docker
- PowerShell + Windmill 原生二進位作為自動化與排序
- UTF-8 內部，CP950/Big5 與台灣銀行/ERP 交換

---

## 2. 設計決定 (已與使用者確認)

| # | 決策 | 內容 |
|---|------|------|
| D1 | 整合模式 | **雙向媒體檔案交換**：ERP→銀行產生加密媒體檔(批次付款)；銀行→ERP 回傳結果檔，更新憑單狀態。後續可升級為 API 直連。 |
| D2 | 實作位置 | 新增 Wellfit crate `wellfit_bankpay` (Dev/Wellfit/crates/wellfit_bankpay/)，依賴 `wellfit_core` |
| D3 | 可追溯查詢 | 新增 CLI 子命令 `wellfit bankpay query --voucher \| --tx \| --batch`，交叉對照 銀行參考號 ↔ ERP 憑單編號 ↔ 帳本批次 ↔ 檔案 SHA-256 |
| D4 | 編碼 | 內部 UTF-8；銀行/ERP 交換 CP950/Big5 via iconv-lite (與現有 codebase 一致) |
| D5 | 冪等鍵 | `voucher_no + amount_cents + batch_date`，寫入 `transaction_keys`，防重複付款 |
| D6 | 發布閘門 | `ReleaseGate` two-way door (驗證 payload) → one-way door (發送)；`DispatchUncertain` 需人工/狀態查詢複原 |
| D7 | 稽核與保留 | `validation_findings` + `quarantine_log` + `access_audit_log`，保留 ≥ 1 年 (法規) |
| D8 | 部署 | `wellfit_bankpay.exe` 原生二進位，Windows sc.exe 服務或排程工作，Windmill 協調，無 Docker |

---

## 3. 資料流

### 3.1 匯出方向 (ERP → 銀行：付款)
```
FlapsErp 匯出應付憑單 (CSV/JSON, via COM 或檔案匯出)
    │
    ▼
wellfit_bankpay 讀取 → ColumnContract 欄位解析 → scaled 金額 → iconv-lite UTF-8↔CP950/Big5
    │
    ▼
產生 E.Sun 加密媒體檔 (批次付款檔)
    │
    ▼
ReleaseGate.verify_payload → VerifiedPass / Quarantined
    │
    ▼
ReleaseGate.dispatch (安全通道送銀行) → Completed / Failed / DispatchUncertain
    │
    ▼
ledger: register_file_batch + check_and_lock_order (冪等鎖) + validation_findings
```

### 3.2 匯入方向 (銀行 → ERP：結果)
```
銀行結果檔 (加密媒體檔)
    │
    ▼
wellfit_bankpay 解密 → 解析 → 比對 bank_ref ↔ ledger.transaction_keys
    │
    ▼
更新憑單狀態 (已付款/失敗/不確定) → 寫回 FlapsErp (COM 或檔案匯入)
    │
    ▼
未比對 → quarantine_log；所有發現 → validation_findings
```

### 3.3 可追溯查詢
```
wellfit bankpay query --voucher <AP-00123>
    → batch_id, bank_ref, file_sha256, status, findings, timestamps

wellfit bankpay query --tx <E.SUN-REF-999>
    → voucher_no, batch_id, status, amount

wellfit bankpay query --batch <BATCH-20261102-001>
    → 批次內所有憑單與對應銀行參考號
```
所有查詢結果寫入 `validation_findings`，供稽核追溯。

---

## 4. 實作任務 (依序執行)

| ID | 任務 | 內容 | 驗收標準 |
|----|------|------|----------|
| T1 | 建立 `wellfit_bankpay` crate | Cargo.toml (workspace member)、lib.rs 模組宣告 | `cargo check -p wellfit_bankpay` 0 錯誤 |
| T2 | 銀行媒體檔案轉換器 | E.Sun 加密媒體檔產生與解析；CP950/Big5 ↔ UTF-8 轉換 | 可產生/解析樣本檔，編碼無亂碼 |
| T3 | 匯出管線 (ERP→銀行) | 讀取 FlapsErp 憑單 → 驗證 → 鎖冪等鍵 → 產生媒體檔 → ReleaseGate 發送 | 重複憑單被拒，鎖鍵唯一 |
| T4 | 匯入管線 (銀行→ERP) | 解析結果檔 → 比對 bank_ref → 更新憑單狀態 → 寫回 ERP | 未比對進 quarantine_log |
| T5 | 可追溯查詢 CLI | `wellfit bankpay query --voucher / --tx / --batch` | 輸出包含 batch_id、bank_ref、file_sha256、status |
| T6 | 發布閘門與不確定狀態處理 | ReleaseGate two-way/one-way door；DispatchUncertain 複原 | 超時進 DispatchUncertain，狀態查詢後可 Completed/VerifiedPass |
| T7 | 稽核與隔離日誌 | validation_findings、quarantine_log、access_audit_log 寫入 | 每筆發現皆有 rule_code、severity、line、key、reason |
| T8 | 單元測試 | 冪等、編碼、查詢、發布閘門狀態轉換 | `cargo test -p wellfit_bankpay` 全 pass |
| T9 | 程式品質門檻 | `#![forbid(unsafe_code)]`、clippy 0 警告、cargo fmt | `cargo clippy -p wellfit_bankpay -- -D warnings` 0 警告 |
| T10 | PowerShell + Windmill 排程 | 每日離峰觸發、斷路器、重試(5s/15s/30s)、告警(≤5 min) | 排程可觸發，失敗告警送達 |
| T11 | 部署與回滾腳本 | sc.exe 服務安裝、備份(AES-256)、還原演練 | RTO ≤ 4 小時，RPO ≤ 24 小時 |
| T12 | 端對端追溯驗證 | 憑單→媒體檔→銀行參考號→query→稽核鏈端對端 | voucher↔tx↔batch↔hash 互相可查，不中斷 |

---

## 5. 風險與因應

| 風險 | 影響 | 因應措施 |
|------|------|----------|
| E.Sun 媒體檔格式/認證方式未確認 | 高 | OQ-E1~E4 列為阻塞項，開發前取得銀行規格；無規格時以檔案匯出/匯入為先行 fallback |
| FlapsErp COM 物件對應付憑單的 ProgID/方法未知 | 中 | 優先以檔案匯出(CSV/JSON) + 匯入；COM 呼叫作為優化選項 |
| CP950/Big5 編碼亂碼 | 中 | 全端內部 UTF-8，交換邊界 iconv-lite，建立編碼測試用例 |
| DispatchUncertain 狀態堆積 | 中 | 定期狀態查詢複原，人工複核佇列，告警 ≤ 5 分鐘 |
| 重複付款 | 高 | 冪等鍵 `voucher_no+amount_cents+batch_date`，`transaction_keys` 唯一約束 |

---

## 6. 未解決問題 (開發前必須釐清)

| ID | 問題 | 說明 |
|----|------|------|
| OQ-E1 | E.Sun 企業網銀媒體檔規格 | 批次付款檔欄位佈局、加密方式、回傳結果檔格式 |
| OQ-E2 | 銀行認證與安全通道 | 憑證、API Key、IP 白名單、TLS 版本 |
| OQ-E3 | FlapsErp COM 物件明細 | 應付憑單讀/寫的 ProgID/CLSID 與方法清單；或確認檔案匯出/匯入是否足夠 |
| OQ-E4 | 銀行結果送達方式 | 檔案 drop、API callback、或 Email |

---

## 7. 驗收與品質門檻

- `cargo test --workspace` 全部通過 (含新增 crate)
- `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- `cargo fmt -- --check` 通過
- 端對端：建立測試憑單 → 匯出媒體檔 → 產生銀行參考號 → `bankpay query` 三種方式皆可查 → 稽核鏈完整 (voucher↔tx↔batch↔hash)
- 法規：敏感欄位遮蔽 100%，稽核日誌保留 ≥ 1 年，傳輸 TLS 1.2+
- 效能：單筆匯出/匯入 ≤ 5 秒 (檔案模式)，日處理量 ≥ 200 筆

---

## 8. 交付物

1. `Dev/Wellfit/crates/wellfit_bankpay/` 原始碼 (Cargo.toml, lib.rs, bankpay.rs, query.rs, encoding.rs, release.rs)
2. `wellfit bankpay query` CLI 子命令文件
3. PowerShell 排程與告警腳本 (Windmill flow)
4. sc.exe 服務安裝與備份/回滾腳本
5. 測試報告與端對端追溯驗證證據

---

*本計畫僅為設計與任務清單，不包含原始程式碼變更。實作階段請由具備 Rust 與 PowerShell 能力的開發代理人按任務 ID 順序執行。*
