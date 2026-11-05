# A1 端到端測試基礎設施 — flapserp:// 開頁指令範本

**文件編號：** A1-E2E-2026-11-04
**版本：** v1.0
**日期：** 2026-11-04
**適用專案：** Wellfit 自動化工具鏈
**相關提案：** OQ-F1_F2_輔翼ERP正式需求提案_v2.1.md（CH-1: flapserp:// 自訂協定）

---

## 1. 目的

本文件提供 flapserp:// 自訂協定開頁指令範本，供 A1 端到端測試基礎設施使用。透過 flapserp:// 協定可啟動 FlapsErp 應用程式並傳遞啟動參數，作為 ERP 自動化測試的入口點。

## 2. flapserp:// 協定規格

### 2.1 基本格式

flapserp://<base64(JSON 參數)>

### 2.2 JSON 啟動參數範本

{
  "action": "open",
  "target": "main",
  "params": {
    "module": "dashboard",
    "action": "open",
    "params": {}
  },
  "metadata": {
    "source": "wellfit_e2e_test",
    "timestamp": "2026-11-04T00:00:00Z",
    "correlation_id": "flaps-test-001"
  }
}

### 2.3 範例：開啟進貨訂單模組

{
  "action": "open",
  "target": "main",
  "params": {
    "module": "purchase_order",
    "action": "open",
    "params": {
      "view": "list",
      "filter": "status=pending"
    }
  },
  "metadata": {
    "source": "wellfit_e2e_test",
    "timestamp": "2026-11-04T08:00:00Z",
    "correlation_id": "flaps-test-001"
  }
}

轉換為 base64 後組裝 flapserp:// URI：

flapserp://<base64 編碼後的完整字串>

---

## 3. PowerShell 開頁指令範本

`powershell
# 輔翼 ERP flapserp:// 開頁指令範本
# 用途：A1 端到端測試基礎設施 — 開啟 FlapsErp 並傳遞啟動參數
# 使用方式：.\Invoke-FlapserpOpen.ps1 -Action open -Module "purchase_order" -BatchId "TEST-001"
#         乾跑模式（僅顯示將執行的 URI，不實際啟動）：.\Invoke-FlapserpOpen.ps1 -Action "open" -Module "purchase_order" -BatchId "TEST-001" -DryRun

param(
    [string]$Action = "open",
    [string]$Module = "dashboard",
    [string]$BatchId = "TEST-001",
    [string]$ActionParam = "",
    [switch]$DryRun
)

function Invoke-FlapserpOpen {
    param(
        [string]$Action,
        [string]$Module,
        [string]$BatchId,
        [string]$ActionParam
    )

    # 建立 JSON 啟動參數
    $jsonPayload = @{
        action = $Action
        target = "main"
        params = @{
            module = $Module
            action = $Action
            params = @{}
        }
        metadata = @{
            source = "wellfit_e2e_test"
            timestamp = (Get-Date -Format "o")
            correlation_id = $BatchId
        }
    } | ConvertTo-Json -Depth 3

    # base64 編碼
    $base64Payload = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($jsonPayload))

    # 建立 flapserp:// URI
    $uri = "flapserp://$base64Payload"

    Write-Host "flapserp:// URI: $uri" -ForegroundColor Cyan

    if ($DryRun) {
        Write-Host "【乾跑模式】將執行：Start-Process """ -ForegroundColor Yellow
        return
    }

    # 實際啟動 FlapsErp
    Start-Process $uri
    Write-Host "已啟動 FlapsErp：$uri" -ForegroundColor Green
}

# 執行範例（取消註解以實際執行）：
# Invoke-FlapserpOpen -Action "open" -Module "purchase_order" -BatchId "TEST-001"

# 乾跑模式（僅顯示將執行的 URI，不實際啟動）：
# Invoke-FlapserpOpen -Action "open" -Module "purchase_order" -BatchId "TEST-001" -DryRun
`

---

## 4. 驗證清單

| 項目 | 預期結果 | 驗證方式 |
|---|---|---|
| flapserp:// 協定可解析 | FlapsErp 啟動並顯示指定畫面 | 觀察視窗標題與內容 |
| JSON 參數正確傳遞 | 目標模組正確顯示 | 對照預期畫面 |
| 追蹤碼正確傳遞 | 日誌中可追蹤 correlation_id | 檢查 FlapsErp 日誌 |
| 重複執行不衝突 | 多次開啟不產生錯誤 | 重複執行相同 batch_id |

---

## 5. 後續步驟

1. 將此範本提供給測試團隊
2. 在輔翼 ERP 虛擬機環境中執行實測
3. 根據實測結果調整參數格式
4. 將測試結果整合進端到端測試套件
