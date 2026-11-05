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
        Write-Host "【乾跑模式】將執行：Start-Process "$uri"" -ForegroundColor Yellow
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
