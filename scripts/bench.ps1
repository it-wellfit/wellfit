# scripts/bench.ps1 — measure MTRIX processing time on delivered samples.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\bench.ps1
# Target: < 5s per file.

param(
    [string]$TargetDir = "$env:LOCALAPPDATA\Temp\wellfit-target"
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location -LiteralPath $root

$wf = Join-Path $TargetDir "debug\wellfit.exe"
if (-not (Test-Path -LiteralPath $wf)) {
    Write-Host "Building wellfit_cli..." -ForegroundColor Cyan
    & (Join-Path $root "..\..\Knowledge\tools\dev-env\build.ps1") -Command "build -p wellfit_cli" -Workspace $root -TargetDir $TargetDir
}

$db = Join-Path $env:TEMP "bench.sqlite"
$report = Join-Path $env:TEMP "bench-report.json"
$outdir = Join-Path $env:TEMP "bench-repaired"
if (Test-Path -LiteralPath $db) { Remove-Item -LiteralPath $db -Force }
if (Test-Path -LiteralPath $outdir) { Remove-Item -LiteralPath $outdir -Recurse -Force }

$sw = [System.Diagnostics.Stopwatch]::StartNew()
& $wf mtrix --dir "spec\fixtures\mtrix\delivered" --batch "BENCH" --db $db --report $report --output-dir $outdir | Out-Null
$sw.Stop()

$elapsed = $sw.Elapsed.TotalSeconds
$files = 5
$perFile = $elapsed / $files
Write-Host "Total: $([math]::Round($elapsed,3))s  Per file: $([math]::Round($perFile,3))s  (target <5s/file)" -ForegroundColor $(if ($perFile -lt 5) { "Green" } else { "Red" })

if ($perFile -ge 5) { exit 1 }
exit 0
