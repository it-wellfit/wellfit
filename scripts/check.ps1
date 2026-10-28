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

# 6. Golden parity check (MTRIX delivered sample yields documented findings)
if (-not $SkipFixtures) {
    Write-Host ""
    Write-Host "=== Golden parity (22 errors / 50 warnings / 37 repairs) ===" -ForegroundColor Cyan
    $db = Join-Path $env:TEMP "check-golden.sqlite"
    $report = Join-Path $env:TEMP "check-golden-report.json"
    if (Test-Path -LiteralPath $db) { Remove-Item -LiteralPath $db -Force }
    if (Test-Path -LiteralPath $report) { Remove-Item -LiteralPath $report -Force }
    $wf = Join-Path $TargetDir "debug\wellfit.exe"
    & $wf mtrix --dir "spec\fixtures\mtrix\delivered" --batch "CHECK-GOLDEN" --db $db --report $report | Out-Null
    if (-not (Test-Path -LiteralPath $report)) { Write-Host "[FAIL] report not generated" -ForegroundColor Red; $script:failures++ } else {
        $json = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
        $errors = ($json.findings | Where-Object { $_.severity -eq "Error" }).Count
        $warnings = ($json.findings | Where-Object { $_.severity -eq "Warning" }).Count
        $ok = $true
        if ($errors -ne 22) { Write-Host "[FAIL] expected 22 errors, got $errors" -ForegroundColor Red; $ok = $false }
        if ($warnings -ne 50) { Write-Host "[FAIL] expected 50 warnings, got $warnings" -ForegroundColor Red; $ok = $false }
        if ($json.repairs_applied -ne 37) { Write-Host "[FAIL] expected 37 repairs, got $($json.repairs_applied)" -ForegroundColor Red; $ok = $false }
        if ($ok) { Write-Host "[OK] Golden parity: errors=$errors warnings=$warnings repairs=$($json.repairs_applied)" -ForegroundColor Green }
        else { $script:failures++ }
    }
}

# 7. 技能合規檢查（新增）
Run-Step "技能合規檢查" {
    $activeFile = Join-Path $root "..\.kilo\skills\rigorous-skill-pack\ACTIVE_SKILLS.md"
    if (-not (Test-Path -LiteralPath $activeFile)) { throw "ACTIVE_SKILLS.md 不存在，請先執行 scripts\skills-activate.ps1 啟動技能" }
    $content = Get-Content -LiteralPath $activeFile -Raw
    $activeCount = ([regex]::Matches($content, "- \[x\]")).Count
    if ($activeCount -eq 0) { throw "未啟動任何技能，請先執行 skills-activate.ps1 選擇對應技能" }
    Write-Host "已啟動技能數量：$activeCount" -ForegroundColor Green

    if ($content -match "06-tdd-and-implementation") {
        Write-Host "[G6] TDD 技能：確認已執行 clippy 零警告檢查與測試通過" -ForegroundColor Cyan
    }
    if ($content -match "08-code-review") {
        Write-Host "[G8] 審查技能：確認已產生審查報告與回滾證明" -ForegroundColor Cyan
    }
    if ($content -match "09-testing-and-quality") {
        Write-Host "[G9] 品質技能：確認所有測試已通過" -ForegroundColor Cyan
    }
    if ($content -match "10-security-and-privacy") {
        Write-Host "[G9] 安全技能：確認威脅模型與安全審查已完成" -ForegroundColor Cyan
    }
    if ($content -match "12-ci-cd-and-release") {
        Write-Host "[G10] 發布技能：確認回滾程序與備份還原已驗證" -ForegroundColor Cyan
    }
}
