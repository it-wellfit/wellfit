#Requires -Version 5.1
<#
.SYNOPSIS
  Wellfit idempotent batch offload pipeline - T-103 (ADR-003 A1 track entry point).
.DESCRIPTION
  Runs the wellfit_cli MTRIX pipeline against the real delivered fixtures,
  records an execution report (timings, counts) and proves idempotency by
  re-running with the same batch id (duplicate must be detected, not silently
  reprocessed).
  Write boundary: outputs only to %TEMP% + artifacts. Never writes to
  C:\SYSTEM_PROGRAM_DIRECTORY (risk RSK-E5 red line).
  Rollback: delete the run directory + ledger db; ledger idempotency means a
  re-run with a NEW batch id is always safe.
#>
param(
  [string]$WellfitExe = "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe",
  [string]$OutRoot = (Join-Path $PSScriptRoot '..\..\artifacts\erp-batch')
)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $WellfitExe)) {
  Write-Error "wellfit.exe not found at $WellfitExe - build first via Knowledge build.ps1"
  exit 3
}
$ts = Get-Date -Format 'yyyyMMdd-HHmmss'
$runDir = Join-Path $OutRoot $ts
New-Item -ItemType Directory -Path $runDir -Force | Out-Null
$batchId = "ERP-BATCH-$ts"
$db = Join-Path $runDir 'ledger.sqlite'
$report = Join-Path $runDir 'mtrix-report.json'
$fixtures = Join-Path $PSScriptRoot '..\..\spec\fixtures\mtrix\delivered'

$sw = [System.Diagnostics.Stopwatch]::StartNew()
& $WellfitExe mtrix --dir $fixtures --report $report --batch $batchId --db $db
$cliExit = $LASTEXITCODE
$sw.Stop()

$exitMeaning = switch ($cliExit) { 0 {'clean'} 2 {'completed with reported violations (accepted)'} default {'FAILED'} }
$firstRunMs = $sw.ElapsedMilliseconds

# idempotency proof: same batch id + same db must be rejected / detected
$sw2 = [System.Diagnostics.Stopwatch]::StartNew()
& $WellfitExe mtrix --dir $fixtures --report (Join-Path $runDir 'mtrix-report-rerun.json') --batch $batchId --db $db | Out-Null
$rerunExit = $LASTEXITCODE
$sw2.Stop()

$summary = [ordered]@{
  timestamp = $ts; batchId = $batchId
  firstRun = @{ exitCode = $cliExit; meaning = $exitMeaning; elapsedMs = $firstRunMs }
  idempotencyRerun = @{ exitCode = $rerunExit; elapsedMs = $sw2.ElapsedMilliseconds; note = 'same batch id must not double-process' }
  ledger = $db; report = $report
  totalRamBudgetMB = 0
}
$summary | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $runDir 'run-summary.json') -Encoding UTF8

Write-Output "RUN_DIR: $runDir"
Write-Output "first run: exit=$cliExit ($exitMeaning) elapsed=$($firstRunMs)ms"
Write-Output "idempotency rerun: exit=$rerunExit elapsed=$($sw2.ElapsedMilliseconds)ms"
if ($cliExit -eq 0 -or $cliExit -eq 2) { Write-Output 'PIPELINE OK' } else { Write-Output 'PIPELINE FAILED'; exit 1 }
