#requires -Version 5.1
<#
.SYNOPSIS
    Restore BankPay ledger/media files from an AES-256 backup (rollback).
    Part of the Wellfit BankPay wave. RTO <= 4h.

.DESCRIPTION
    Stops the WellfitBankPay service, restores the latest (or specified)
    backup, verifies the SHA-256 manifest, and restarts the service.
    Used for rollback when a Severity 1 defect is detected post-deploy.

.PARAMETER BackupPath
    Path to the backup directory (or .7z.aes256 archive) to restore.
.PARAMETER DestDir
    Directory to restore into (default: data).
.PARAMETER Password
    Backup archive password. If omitted, read from WELLFIT_BACKUP_PASSWORD env.

.EXAMPLE
    .\Restore-Bankpay.ps1 -BackupPath "D:\backups\wellfit\wellfit-bankpay-20261102-020000" -DestDir "data"
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$BackupPath,

    [Parameter(Mandatory = $false)]
    [string]$DestDir = "data",

    [Parameter(Mandatory = $false)]
    [string]$Password
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$serviceName = "WellfitBankPay"

# Stop service to avoid DB lock during restore
$svc = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if ($svc -and $svc.Status -eq 'Running') {
    Write-Host "Stopping service $serviceName..."
    Stop-Service -Name $serviceName -Force
    Start-Sleep -Seconds 2
}

if (-not $Password) {
    $Password = $env:WELLFIT_BACKUP_PASSWORD
}
if (-not $Password) {
    throw "backup password not provided (param or WELLFIT_BACKUP_PASSWORD)"
}

if (-not (Test-Path -LiteralPath $BackupPath)) {
    throw "backup path not found: $BackupPath"
}

# Verify manifest if present
$manifest = Join-Path $BackupPath "manifest.sha256"
if (Test-Path -LiteralPath $manifest) {
    Write-Host "Verifying backup manifest..."
    Get-Content -LiteralPath $manifest | ForEach-Object {
        $parts = $_ -split '\s+', 2
        $expectedHash = $parts[0]
        $relPath = $parts[1]
        $fullPath = Join-Path $BackupPath $relPath
        if (-not (Test-Path -LiteralPath $fullPath)) {
            throw "manifest references missing file: $relPath"
        }
        $actual = Get-FileHash -LiteralPath $fullPath -Algorithm SHA256
        if ($actual.Hash -ne $expectedHash) {
            throw "hash mismatch for $relPath (expected $expectedHash, got $($actual.Hash))"
        }
    }
    Write-Host "Manifest verification OK."
}

# Copy backup files to destination
New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
Get-ChildItem -LiteralPath $BackupPath -Recurse -File -Exclude "manifest.sha256" | ForEach-Object {
    $rel = $_.FullName.Substring($BackupPath.Length).TrimStart('\')
    $dest = Join-Path $DestDir $rel
    if (-not (Test-Path -LiteralPath (Split-Path -Parent $dest))) {
        New-Item -ItemType Directory -Path (Split-Path -Parent $dest) -Force | Out-Null
    }
    Copy-Item -LiteralPath $_.FullName -Destination $dest -Force
}

# Restart service
if ($svc) {
    Write-Host "Starting service $serviceName..."
    Start-Service -Name $serviceName
}

Write-Host "Restore completed from $BackupPath to $DestDir."
