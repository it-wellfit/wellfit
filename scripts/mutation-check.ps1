# scripts/mutation-check.ps1 — verify no tooling mutates the version-controlled checkout.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\mutation-check.ps1
#
# Snapshots SHA-256 of every tracked file before and after running scripts\check.ps1;
# any difference is a failure (tooling must never write to the live repo).

param(
    [string]$TargetDir = "$env:LOCALAPPDATA\Temp\wellfit-target"
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location -LiteralPath $root

function Snapshot-Tracked {
    param([string]$Label)
    # -z gives NUL-terminated paths so filenames with spaces/quotes are safe.
    $raw = git ls-files -z
    $files = $raw -split "`0" | Where-Object { $_ -ne "" }
    $lines = foreach ($f in $files) {
        $full = Join-Path $root $f
        if (Test-Path -LiteralPath $full) {
            $h = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash
            "$h  $f"
        }
    }
    $out = Join-Path $env:TEMP "wellfit-snapshot-$Label.txt"
    $lines | Sort-Object | Set-Content -LiteralPath $out -Encoding utf8
    return $out
}

Write-Host "=== Snapshotting tracked files (before) ===" -ForegroundColor Cyan
$before = Snapshot-Tracked "before"

Write-Host "=== Running scripts\check.ps1 ===" -ForegroundColor Cyan
& (Join-Path $root "scripts\check.ps1")
$checkExit = $LASTEXITCODE

Write-Host "=== Snapshotting tracked files (after) ===" -ForegroundColor Cyan
$after = Snapshot-Tracked "after"

$diff = Compare-Object (Get-Content -LiteralPath $before) (Get-Content -LiteralPath $after)
if ($diff) {
    Write-Host "MUTATION DETECTED: tracked files changed during check:" -ForegroundColor Red
    $diff | ForEach-Object { Write-Host $_.InputObject -ForegroundColor Red }
    exit 1
} else {
    Write-Host "No mutation: tracked files byte-identical before/after." -ForegroundColor Green
}

if ($checkExit -ne 0) {
    Write-Host "check.ps1 failed (exit $checkExit)" -ForegroundColor Red
    exit $checkExit
}
Write-Host "mutation-check PASSED" -ForegroundColor Green
exit 0
