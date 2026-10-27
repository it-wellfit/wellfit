#requires -Version 5.1
<#
.SYNOPSIS
    Traceable query: cross-reference voucher / bank reference / batch.
    Part of the Wellfit BankPay wave.

.DESCRIPTION
    Invokes the wellfit native binary `bankpay query` sub-command. The query is
    recorded as an INFO finding in validation_findings so the act of tracing is
    itself auditable.

    Lookup kinds:
      voucher  - cross-reference by ERP voucher number
      tx       - cross-reference by E.Sun bank reference
      batch    - cross-reference by batch id

.EXAMPLE
    .\Invoke-BankpayQuery.ps1 -Kind voucher -Key AP-00123
    .\Invoke-BankpayQuery.ps1 -Kind tx -Key EREF-999
    .\Invoke-BankpayQuery.ps1 -Kind batch -Key BATCH-BANKPAY-20261102
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("voucher", "tx", "batch")]
    [string]$Kind,

    [Parameter(Mandatory = $true)]
    [string]$Key,

    [Parameter(Mandatory = $false)]
    [string]$WellfitBin = "$env:LOCALAPPDATA\Temp\wellfit-target\release\wellfit.exe",

    [Parameter(Mandatory = $false)]
    [string]$Db = "data\wellfit_ledger.sqlite"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $WellfitBin)) {
    throw "wellfit binary not found: $WellfitBin"
}

$args = @(
    "bankpay", "query",
    "--kind", $Kind,
    "--key", $Key,
    "--db", $Db
)

Write-Host "Bankpay traceable query (kind=$Kind, key=$Key)..."
& $WellfitBin @args
$exit = $LASTEXITCODE

if ($exit -ne 0) {
    throw "Bankpay query failed with exit code $exit"
}

exit $exit
