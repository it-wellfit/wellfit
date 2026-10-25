# daily-backup.ps1
# Wellfit SQLite 資料庫每日備份腳本
# 依據 ISO 27001:2022 A.8.13 資訊備份控制項

param(
    [string]$DbPath = "data\wellfit_ledger.sqlite",
    [string]$BackupDir = "$env:LOCALAPPDATA\Temp\wellfit-backup"
)

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$backupFile = Join-Path $BackupDir "wellfit_ledger_$timestamp.sqlite"

# 確保備份目錄存在
if (-not (Test-Path -LiteralPath $BackupDir)) {
    New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
}

# SQLite 線上備份（使用 .backup 命令）
$sqlite3 = Get-Command sqlite3 -ErrorAction SilentlyContinue
if ($sqlite3) {
    & sqlite3 $DbPath ".backup '$backupFile'"
} else {
    # 備用：直接複製（需確保無寫入鎖）
    Copy-Item -LiteralPath $DbPath -Destination $backupFile
}

# 驗證備份完整性
if (Test-Path -LiteralPath $backupFile) {
    $size = (Get-Item -LiteralPath $backupFile).Length
    Write-Host "Backup created: $backupFile ($size bytes)"
    # 驗證 SQLite 完整性
    if ($sqlite3) {
        $integrity = & sqlite3 $backupFile "PRAGMA integrity_check;"
        if ($integrity -eq "ok") {
            Write-Host "Integrity check: OK"
        } else {
            Write-Error "Integrity check failed: $integrity"
            exit 1
        }
    }
} else {
    Write-Error "Backup failed!"
    exit 1
}

# 清理 7 天前的備份
Get-ChildItem -LiteralPath $BackupDir -Filter "wellfit_ledger_*.sqlite" |
    Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } |
    Remove-Item -Force

Write-Host "Backup complete. Retention: 7 days."
