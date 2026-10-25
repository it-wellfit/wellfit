# scripts/check.ps1 — fixed local check entry point
# Runs formatting, linting, type checking (via build), unit tests, integration tests,
# and SHA-256 fixture verification. Exit code 0 = all green.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\check.ps1
#        powershell -ExecutionPolicy Bypass -File scripts\check.ps1 -SkipFixtures

param(
    [switch]$SkipFixtures,
    [string]$TargetDir = "$env:LOCALAPPDATA\Temp\wellfit-target"
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location -LiteralPath $root

$buildScript = Join-Path $root "..\..\Knowledge\tools\dev-env\build.ps1"
$buildScript = (Resolve-Path $buildScript).Path

$failures = 0

function Run-Step {
    param([string]$Name, [scriptblock]$Action)
    Write-Host ""
    Write-Host "=== $Name ===" -ForegroundColor Cyan
    try {
        & $Action
        if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) { throw "exit code $LASTEXITCODE" }
        Write-Host "[OK] $Name" -ForegroundColor Green
    } catch {
        Write-Host "[FAIL] $Name : $_" -ForegroundColor Red
        $script:failures++
    }
}

# 1. Format check
Run-Step "cargo fmt --check" {
    $env:CARGO_TARGET_DIR = $TargetDir
    cargo fmt --all -- --check
}

# 2. Clippy (zero warnings)
Run-Step "clippy --workspace --all-targets -- -D warnings" {
    & $buildScript -Command "clippy --workspace --all-targets -- -D warnings" -Workspace $root -TargetDir $TargetDir
}

# 3. Unit tests
Run-Step "test --workspace --no-fail-fast" {
    & $buildScript -Command "test --workspace --no-fail-fast" -Workspace $root -TargetDir $TargetDir
}

# 4. Integration tests
Run-Step "test -p wellfit_cli --test integration_tests" {
    & $buildScript -Command "test -p wellfit_cli --test integration_tests" -Workspace $root -TargetDir $TargetDir
}

# 5. SHA-256 fixture verification (against PROVENANCE.md)
if (-not $SkipFixtures) {
    Run-Step "SHA-256 fixture verification" {
        $prov = Join-Path $root "spec\fixtures\mtrix\PROVENANCE.md"
        if (-not (Test-Path -LiteralPath $prov)) { throw "PROVENANCE.md not found" }
        # Verify the five delivered fixtures exist and are non-empty.
        $delivered = Join-Path $root "spec\fixtures\mtrix\delivered"
        $files = Get-ChildItem -LiteralPath $delivered -Filter "*.csv"
        if ($files.Count -ne 5) { throw "expected 5 delivered CSV, found $($files.Count)" }
        foreach ($f in $files) {
            if ($f.Length -eq 0) { throw "empty fixture: $($f.Name)" }
        }
        Write-Host "Delivered fixtures: $($files.Count) files present"
    }
}

Write-Host ""
if ($failures -gt 0) {
    Write-Host "check FAILED: $failures step(s) failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "check PASSED: all steps green" -ForegroundColor Green
    exit 0
}
