#requires -Version 5.1
<#
.SYNOPSIS
    Import E.Sun bank result file and settle matched payable vouchers.
    Part of the Wellfit BankPay wave (Fuyi ERP <-> E.Sun enterprise net-banking).

.DESCRIPTION
    Invokes the wellfit native binary `bankpay import` sub-command to read the
    bank result file, cross-reference bank references against ledger transaction
    keys, record findings for traceability, and quarantine unparseable rows.

.EXAMPLE
    .\Import-BankResults.ps1 -Input "C:\bank\results\20261103_result.csv" -Batch "BATCH-BANKPAY-20261102"
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$WellfitBin = "$env:LOCALAPPDATA\Temp\wellfit-target\release\wellfit.exe",

    [Parameter(Mandatory = $false)]
    [string]$Db = "data\wellfit_ledger.sqlite",

    [Parameter(Mandatory = $true)]
    [string]$Input,

    [Parameter(Mandatory = $false)]
    [string]$Batch = "BATCH-BANKPAY-RES-$(Get-Date -Format 'yyyyMMdd')",

    [Parameter(Mandatory = $false)]
    [ValidateSet("utf8", "cp950", "big5")]
    [string]$Encoding = "utf8",

    [switch]$Strict
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $WellfitBin)) {
    throw "wellfit binary not found: $WellfitBin"
}

$args = @(
    "bankpay", "import",
    "--input", $Input,
    "--batch", $Batch,
    "--encoding", $Encoding,
    "--db", $Db
)

Write-Host "Importing bank result file..."
Write-Host "  binary : $WellfitBin"
Write-Host "  input  : $Input"
Write-Host "  batch  : $Batch"

& $WellfitBin @args
$exit = $LASTEXITCODE

if ($exit -eq 0) {
    Write-Host "Bankpay import completed successfully."
} elseif ($exit -eq 2) {
    Write-Warning "Bankpay import completed with isolated/unmatched records (exit 2). Review quarantine log."
    if ($Strict) { throw "Strict mode: isolated records detected" }
} else {
    throw "Bankpay import failed with exit code $exit"
}

exit $exit
