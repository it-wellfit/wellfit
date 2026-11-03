# 玉山銀行網銀串接輔翼ERP應付憑單作業優化專案

## 專案概述

**專案名稱**: 玉山銀行網銀串接輔翼ERP應付憑單作業優化專案
**英文名稱**: E.Sun Bank Web Banking Integration with FlapsErp Accounts Payable Voucher Optimization
**版本**: V2.8 (E.Sun Bank API)
**狀態**: Historical secondary summary — implementation completion not verified
**最後更新**: 2026-10-29

> **權威性限制（2026-11-02）：**本資料夾與 `docs/bankpay/` 重複，且本 README 下方仍含媒體檔主流程及已完成宣稱。以本機銀行 V2.8 PDF 為原始規格、以 `docs/bankpay/` 為本專案決策／介面文件；本資料夾保留作歷史摘要，不可作為核准設計或已完成證據。協定採 API-first，JSON 為預設；規格指定的一借一貸可使用檔案載荷。沙箱／正式端對端驗收仍未完成。

---

## 專案目標

本專案實現了玉山銀行企業網銀系統與輔翼ERP系統的雙向整合，主要功能包括：

1. **應付憑單自動化處理** - 從輔翼ERP匯出應付憑單，自動轉換為玉山銀行媒體檔案格式
2. **雙向資料交換** - ERP -> 銀行 (付款檔案) 和 銀行 -> ERP (結果檔案)
3. **可追溯查詢系統** - 完整的稽核鏈追蹤 (憑單 <-> 銀行參考號 <-> 批次 <-> 檔案SHA-256)
4. **冪等性控制** - 防止重複付款
5. **發布閘門** - Two-way door / One-way door 狀態機確保安全性

---

## 技術架構

### 核心技術
- **語言**: Rust 2021 edition
- **安全**: TLS 1.2+, 2-Way SSL, WDAC (Windows Defender Application Control)
- **編碼**: UTF-8 (內部), CP950/Big5 (交換)
- **資料庫**: SQLite (冪等帳本)
- **部署**: Windows 原生二進位 (sc.exe)

### 專案結構
```
Wellfit/
├── crates/wellfit_bankpay/          # Rust 實作核心
├── docs/esun-bank-flaps-erp-integration/  # 專案文件
├── scripts/bankpay/                  # BankPay 整合腳本
├── scripts/integration-testing/       # 整合測試環境設定
└── config/                           # 設定檔 (WDAC 等)
```

---

## 文件索引

### API 規格文件
- **api-spec-v28.md** - 完整的玉山銀行 API V2.8 規格
  - 8 個 API 端點完整說明
  - 請求/回應參數詳細定義
  - 錯誤代碼與狀態碼說明

### 實作指南
- **implementation-guide.md** - 當前實作詳細說明
  - 架構概述與 crate 結構
  - 使用範例與設定說明
  - 安全機制與錯誤處理
  - 已知限制與未來工作

### 測試環境
- **testing-environment.md** - 整合測試環境設定
  - WDAC 設定步驟
  - 證書生成與設定
  - Mock 安全機制
  - 驗證步驟與疑難排解

---

## 主要功能模組

### 1. 應付憑單匯出 (ERP -> 銀行)
- 讀取輔翼 ERP 憑單檔案
- 欄位解析與驗證 (ColumnContract)
- 金額處理 (ScaledDecimal, 零浮點)
- 產生玉山銀行媒體檔案 (422 字元格式)
- 冪等性鎖定 (transaction_keys)
- 發布閘門 (ReleaseGate)

### 2. 銀行結果匯入 (銀行 -> ERP)
- 解析銀行結果檔案
- 匹配憑單與銀行參考號
- 更新憑單狀態
- 隔離未匹配項目 (quarantine_log)
- 稽核日誌 (validation_findings)

### 3. 可追溯查詢
- 憑單查詢 (--voucher)
- 交易查詢 (--tx)
- 批次查詢 (--batch)
- 完整稽核鏈 (voucher <-> tx <-> batch <-> hash)

---

## 環境設定

### 必要目錄
```
C:\certs\          # 測試證書
C:\mocks\          # Mock API 回應
C:\test_data\      # 測試資料
C:\logs\           # 日誌檔案
C:\WDAC\           # WDAC 策略
```

### 設定步驟
1. 執行 `scripts/integration-testing/setup.ps1`
2. 執行 `scripts/integration-testing/validate.ps1`
3. 檢查 `.env` 設定

---

## 快速開始

### 建置專案
```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "build -p wellfit_bankpay" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

### 執行測試
```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "test -p wellfit_bankpay" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

### 程式品質檢查
```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "clippy -p wellfit_bankpay -- -D warnings" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

---
## Project Status

| Item | Status |
|------|--------|
| API V2.8 Implementation | Complete (2/8 — TWD payment + status query; FR-BP-11~16 out of scope) |
| Encoding Conversion | Complete (UTF-8 <-> CP950/Big5) |
| Media File | Complete (422 char format) |
| HTTP Client | Complete (TLS 1.2+, 2-Way SSL) |
| Idempotency Ledger | Complete (SQLite) |
| Release Gate | Complete (Two-way/One-way door) |
| Traceable Query | Complete (voucher/tx/batch) |
| Tests | 46/46 Passed |
| Clippy | 0 Warnings |
| WDAC | Configured |
| Mock Security | Implemented |
| Documentation | Integrated |

---

## Related Documents

- Original Spec: [玉山銀行]企業ERP銀企直連串接_技術文件V2.8_v1.pdf
- Implementation Plan: plans/1793034142477-esun-flaps-ap-voucher-integration.md
- Knowledge Base: ../Knowledge/
- Project Charter: AGENTS.md

---

## Contact Information

- Project Maintenance: Wellfit Knowledge Base Maintainer
- Technical Support: Rust Development Team
- Document Location: docs/esun-bank-flaps-erp-integration/

---

**This document is the overview for the E.Sun Bank Web Banking Integration with FlapsErp Accounts Payable Voucher Optimization Project**
**All related documents and code are located in docs/esun-bank-flaps-erp-integration/ directory**
