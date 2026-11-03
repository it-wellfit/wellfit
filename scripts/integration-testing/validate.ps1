# E.Sun Bank Integration Environment Validation Script
# Run with: powershell -ExecutionPolicy Bypass -File scripts\validate-integration-environment.ps1

$ErrorActionPreference = "Continue"
$allPassed = $true

function Test-Check {
    param($Name, $Condition, $Message)
    if ($Condition) {
        Write-Host "[PASS] $Name" -ForegroundColor Green
    } else {
        Write-Host "[FAIL] $Name - $Message" -ForegroundColor Red
        $script:allPassed = $false
    }
}

Write-Host "=== E.Sun Bank Integration Environment Validation ===" -ForegroundColor Cyan

# Check Rust toolchain
Write-Host "`nChecking Rust toolchain..." -ForegroundColor Yellow
$rustVersion = rustc --version 2>&1
Test-Check "Rust compiler" ($rustVersion -match "rustc") "rustc not found"
$cargoVersion = cargo --version 2>&1
Test-Check "Cargo" ($cargoVersion -match "cargo") "cargo not found"

# Check PowerShell version
Write-Host "`nChecking PowerShell..." -ForegroundColor Yellow
$psVersion = $PSVersionTable.PSVersion
Test-Check "PowerShell 5.1+" ($psVersion.Major -ge 5) "PowerShell version too old"

# Check certificate files
Write-Host "`nChecking certificates..." -ForegroundColor Yellow
Test-Check "CA certificate" (Test-Path "C:\certs\ca.crt") "Missing C:\certs\ca.crt"
Test-Check "Client certificate" (Test-Path "C:\certs\client.crt") "Missing C:\certs\client.crt"
Test-Check "Client key" (Test-Path "C:\certs\client.key") "Missing C:\certs\client.key"

# Check mock files
Write-Host "`nChecking mock data..." -ForegroundColor Yellow
Test-Check "Mock payment response" (Test-Path "C:\mocks\payment_response.json") "Missing C:\mocks\payment_response.json"
Test-Check "Mock status response" (Test-Path "C:\mocks\status_response.json") "Missing C:\mocks\status_response.json"

# Check test data
Write-Host "`nChecking test data..." -ForegroundColor Yellow
Test-Check "Test voucher file" (Test-Path "C:\test_data\voucher_sample.csv") "Missing C:\test_data\voucher_sample.csv"
Test-Check "Test result file" (Test-Path "C:\test_data\result_sample.csv") "Missing C:\test_data\result_sample.csv"

# Check .env file
Write-Host "`nChecking environment configuration..." -ForegroundColor Yellow
Test-Check ".env file" (Test-Path "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit\.env") "Missing .env file"

# Check WDAC policy
Write-Host "`nChecking WDAC..." -ForegroundColor Yellow
Test-Check "WDAC policy XML" (Test-Path "config\wdac-policy.xml") "Missing config\wdac-policy.xml"

# Check build tools
Write-Host "`nChecking build environment..." -ForegroundColor Yellow
$buildScript = "..\..\Knowledge\tools\dev-env\build.ps1"
Test-Check "Build script" (Test-Path $buildScript) "Missing build.ps1"

# Check Wellfit project structure
Write-Host "`nChecking Wellfit project..." -ForegroundColor Yellow
Test-Check "Cargo.toml" (Test-Path "Cargo.toml") "Missing Cargo.toml"
Test-Check "wellfit_bankpay crate" (Test-Path "crates\wellfit_bankpay") "Missing wellfit_bankpay crate"
Test-Check "API spec doc" (Test-Path "docs\esun-bank-flaps-erp-integration\api-spec-v28.md") "Missing API spec doc"

# Summary
Write-Host "`n=== Validation Summary ===" -ForegroundColor Cyan
if ($allPassed) {
    Write-Host "All checks passed!" -ForegroundColor Green
} else {
    Write-Host "Some checks failed. Please review the errors above." -ForegroundColor Red
    exit 1
}
