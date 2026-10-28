#requires -Version 5.1
<#
.SYNOPSIS
    E.Sun Bank payable voucher export -> bank media file -> dispatch.
    Part of the Wellfit BankPay wave (Fuyi ERP <-> E.Sun enterprise net-banking).

.DESCRIPTION
    Invokes the wellfit native binary `bankpay export` sub-command to read
    FlapsErp payable vouchers, lock idempotency keys, assemble the encrypted
    bank media file, and dispatch via the release gate.

    Intended to be triggered by Windmill off-peak (02:00-06:00) or run
    standalone from PowerShell 5.1.

.NOTES
    Application Control policy: all build artefacts live under
    %LOCALAPPDATA%\Temp\wellfit-target. No Docker, no commercial RPA.
#>

[CmdletBinding()]
param(
    /// Path to the wellfit native binary
    [Parameter(Mandatory = $false)]
    [string]$WellfitBin = "$env:LOCALAPPDATA\Temp\wellfit-target\release\wellfit.exe",

    /// Ledger database path
    [Parameter(Mandatory = $false)]
    [string]$Db = "data\wellfit_ledger.sqlite",

    /// Source voucher export file (CSV) from FlapsErp
    [Parameter(Mandatory = $true)]
    [string]$Input,

    /// Batch identifier
    [Parameter(Mandatory = $false)]
    [string]$Batch = "BATCH-BANKPAY-$(Get-Date -Format 'yyyyMMdd')",

    /// Batch date for idempotency keys (ISO-8601). Default: today.
    [Parameter(Mandatory = $false)]
    [string]$BatchDate = (Get-Date -Format 'yyyy-MM-dd'),

    /// Target encoding: utf8, cp950, big5
    [Parameter(Mandatory = $false)]
    [ValidateSet("utf8", "cp950", "big5")]
    [string]$Encoding = "utf8",

    /// E.Sun security component base URL
    [Parameter(Mandatory = $false)]
    [string]$EsunUrl = "https://127.0.0.1:8443",

    /// SourceSystemId agreed with E.Sun
    [Parameter(Mandatory = $false)]
    [string]$SourceSystem,

    /// Enterprise group tax ID (8-10 digits)
    [Parameter(Mandatory = $false)]
    [string]$CustomerId,

    /// Payer E.Sun account number
    [Parameter(Mandatory = $false)]
    [string]$PayerAccount,

    /// Payer tax ID
    [Parameter(Mandatory = $false)]
    [string]$PayerId,

    /// Format type: 1=file (base64 media), 2=json
    [Parameter(Mandatory = $false)]
    [ValidateSet("1", "2")]
    [string]$FormatType = "2",

    /// If set, fail the script when isolated records > 0
    [switch]$Strict
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $WellfitBin)) {
    throw "wellfit binary not found: $WellfitBin"
}

$args = @(
    "bankpay", "export",
    "--input", $Input,
    "--batch", $Batch,
    "--batch-date", $BatchDate,
    "--encoding", $Encoding,
    "--esun-url", $EsunUrl,
    "--format-type", $FormatType,
    "--db", $Db
)

if ($SourceSystem) { $args += @("--source-system", $SourceSystem) }
if ($CustomerId) { $args += @("--customer-id", $CustomerId) }
if ($PayerAccount) { $args += @("--payer-account", $PayerAccount) }
if ($PayerId) { $args += @("--payer-id", $PayerId) }

Write-Host "Exporting payable vouchers to bank media file..."
Write-Host "  binary : $WellfitBin"
Write-Host "  input  : $Input"
Write-Host "  batch  : $Batch"

& $WellfitBin @args
$exit = $LASTEXITCODE

if ($exit -eq 0) {
    Write-Host "Bankpay export completed successfully."
} elseif ($exit -eq 2) {
    Write-Warning "Bankpay export completed with isolated records (exit 2). Review quarantine log."
    if ($Strict) { throw "Strict mode: isolated records detected" }
} else {
    throw "Bankpay export failed with exit code $exit"
}

exit $exit
