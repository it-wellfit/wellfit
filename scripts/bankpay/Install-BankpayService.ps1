#requires -Version 5.1
<#
.SYNOPSIS
    Install the Wellfit BankPay engine as a Windows service via sc.exe.
    Part of the Wellfit BankPay wave (native binary, no Docker).

.DESCRIPTION
    Registers wellfit.exe as a Windows service so the BankPay export/import
    can be triggered by the service or scheduled task. Uses sc.exe (native,
    no Docker) per the FLAPS-RPA-2026 operating constraints.

.PARAMETER Name
    Service name (default: WellfitBankPay).
.PARAMETER BinPath
    Path to wellfit.exe.
.PARAMETER StartType
    Service start type: Auto, Demand, Disabled (default: Demand).

.EXAMPLE
    .\Install-BankpayService.ps1 -Name WellfitBankPay -BinPath "C:\opt\wellfit\wellfit.exe"
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$Name = "WellfitBankPay",

    [Parameter(Mandatory = $true)]
    [string]$BinPath,

    [Parameter(Mandatory = $false)]
    [ValidateSet("Auto", "Demand", "Disabled")]
    [string]$StartType = "Demand"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $BinPath)) {
    throw "wellfit binary not found: $BinPath"
}

# Remove existing service if present (idempotent)
$existing = Get-Service -Name $Name -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Stopping existing service $Name..."
    if ($existing.Status -eq 'Running') { Stop-Service -Name $Name -Force }
    sc.exe delete $Name | Out-Null
    Start-Sleep -Seconds 2
}

# sc.exe create <name> binPath= <path> start= <type>
$startMap = @{ Auto = "auto"; Demand = "demand"; Disabled = "disabled" }
$start = $startMap[$StartType]

Write-Host "Creating service $Name (binPath=$BinPath, start=$start)..."
$create = sc.exe create $Name "binPath= `"$BinPath`"" "start= $start"
if ($LASTEXITCODE -ne 0) {
    throw "sc.exe create failed: $create"
}

Write-Host "Service $Name installed successfully."
Write-Host "Start manually with: sc.exe start $Name"
