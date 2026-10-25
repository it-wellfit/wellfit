# restore.ps1
# Wellfit SQLite 資料庫還原腳本
# 依據 ISO 27001:2022 A.8.13 資訊備份控制項

param(
    [Parameter(Mandatory=$true)]
    [string]$BackupFile,
    [string]$DbPath = "data\wellfit_ledger.sqlite"
)

# 1. 確認備份檔案存在
if (-not (Test-Path -LiteralPath $BackupFile)) {
    Write-Error "Backup file not found: $BackupFile"
    exit 1
}

# 2. 備份當前資料庫（以防還原失敗）
$currentBackup = "$DbPath.pre-restore"
if (Test-Path -LiteralPath $DbPath) {
    Copy-Item -LiteralPath $DbPath -Destination $currentBackup -Force
    Write-Host "Current database backed up to: $currentBackup"
}

# 3. 執行還原
Copy-Item -LiteralPath $BackupFile -Destination $DbPath -Force
Write-Host "Database restored from: $BackupFile"

# 4. 驗證還原後完整性
$sqlite3 = Get-Command sqlite3 -ErrorAction SilentlyContinue
if ($sqlite3) {
    $integrity = & sqlite3 $DbPath "PRAGMA integrity_check;"
    if ($integrity -eq "ok") {
        Write-Host "Integrity check: OK"
    } else {
        Write-Error "Integrity check failed: $integrity"
        Write-Host "Restoring original database..."
        Copy-Item -LiteralPath $currentBackup -Destination $DbPath -Force
        exit 1
    }
    # 檢查資料表是否存在
    $tables = & sqlite3 $DbPath "SELECT name FROM sqlite_master WHERE type='table';"
    Write-Host "Tables found: $($tables -join ', ')"
}

Write-Host "Restore complete. Verify with: wellfit status --db $DbPath"
Write-Host "If verification fails, restore original from: $currentBackup"
