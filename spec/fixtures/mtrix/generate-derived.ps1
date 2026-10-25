<#
.SYNOPSIS
    Build the derived MTRIX test cases from the delivered files.

.DESCRIPTION
    Every derived case is a complete five file set in which exactly one
    delivered file carries one deliberate defect, so that a single validation
    rule can be exercised in isolation.

    Two properties are preserved on purpose:
    - the line terminators of the delivered files (carriage return), because a
      fixture that silently normalises them would hide the very defect the
      engine had to be fixed for;
    - the absence of a byte order mark, because the delivered files have none.

    The script lives outside the folder it rewrites, so that re-running it can
    never delete itself.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File spec\fixtures\mtrix\generate-derived.ps1
#>
[CmdletBinding()]
param(
    [string]$SourceDir = "",
    [string]$TargetDir = ""
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($TargetDir)) {
    $TargetDir = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if ([string]::IsNullOrWhiteSpace($SourceDir)) {
    # The source folder lives outside the repository and its path contains non
    # ASCII characters, which a BOM-less script cannot carry literally. It is
    # therefore discovered by name instead of being written down.
    $wellfit = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))))
    $program = Split-Path -Parent (Split-Path -Parent $wellfit)
    $found = Get-ChildItem -LiteralPath $program -Recurse -Directory -Filter 'API_Integration_Samples' -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($found) { $SourceDir = $found.FullName }
}
if ([string]::IsNullOrWhiteSpace($SourceDir) -or -not (Test-Path -LiteralPath $SourceDir)) {
    throw "Source folder API_Integration_Samples not found; pass -SourceDir explicitly"
}

$delivered = @{}
Get-ChildItem -LiteralPath $SourceDir -Filter '*.csv' | ForEach-Object {
    $delivered[$_.BaseName.Substring(0, 6)] = $_
}
foreach ($prefix in @('ALTHCI', 'ALTHES', 'ALTHFV', 'ALTHPD', 'ALTHVD')) {
    if (-not $delivered.ContainsKey($prefix)) { throw "delivered file for $prefix not found" }
}

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$derivedRoot = Join-Path $TargetDir 'derived'
if (-not (Test-Path -LiteralPath $derivedRoot)) { New-Item -ItemType Directory -Path $derivedRoot -Force | Out-Null }

function Split-Records {
    param([string]$Text)
    return @($Text -split "\r\n|\r|\n")
}

function New-CaseSet {
    param([string]$Name, [hashtable]$Mutations)
    $dir = Join-Path $derivedRoot $Name
    if (Test-Path -LiteralPath $dir) { Remove-Item -LiteralPath $dir -Recurse -Force }
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    foreach ($prefix in @('ALTHCI', 'ALTHES', 'ALTHFV', 'ALTHPD', 'ALTHVD')) {
        $content = [System.IO.File]::ReadAllText($delivered[$prefix].FullName)
        if ($Mutations.ContainsKey($prefix)) {
            $content = $Mutations[$prefix].Invoke($content)
        }
        [System.IO.File]::WriteAllText((Join-Path $dir $delivered[$prefix].Name), $content, $utf8NoBom)
    }
    Write-Output "derived case written: $Name"
}

# Case A - row level value violations.
#   ALTHES: negative stock          -> VR-06, repaired to zero with a warning
#   ALTHPD: package type outside 0/1 -> VR-07, row isolated
#   ALTHVD: impossible calendar date -> VR-03, row isolated
#   ALTHVD: document type outside set -> VR-04, row isolated
New-CaseSet -Name 'case-a-value-violations' -Mutations @{
    'ALTHES' = {
        param($c)
        $lines = Split-Records $c
        for ($i = 1; $i -lt $lines.Count; $i++) {
            if ([string]::IsNullOrWhiteSpace($lines[$i])) { continue }
            $p = $lines[$i] -split ';'
            if ($p.Count -ge 5) { $p[4] = '-12'; $lines[$i] = ($p -join ';') }
            break
        }
        return ($lines -join "`r")
    }
    'ALTHPD' = {
        param($c)
        $lines = Split-Records $c
        for ($i = 1; $i -lt $lines.Count; $i++) {
            if ([string]::IsNullOrWhiteSpace($lines[$i])) { continue }
            $p = $lines[$i] -split ';'
            if ($p.Count -ge 6) { $p[5] = '7'; $lines[$i] = ($p -join ';') }
            break
        }
        return ($lines -join "`r")
    }
    'ALTHVD' = {
        param($c)
        $lines = Split-Records $c
        for ($i = 1; $i -lt $lines.Count; $i++) {
            if ([string]::IsNullOrWhiteSpace($lines[$i])) { continue }
            $p = $lines[$i] -split ';'
            if ($p.Count -ge 10) { $p[3] = '20240230'; $p[9] = 'X'; $lines[$i] = ($p -join ';') }
            break
        }
        return ($lines -join "`r")
    }
}

# Case B - duplicate sales transaction: the same document number and batch code
# twice with the quantity split. The engine must aggregate them (VR-05).
New-CaseSet -Name 'case-b-duplicate-transaction' -Mutations @{
    'ALTHVD' = {
        param($c)
        $lines = [System.Collections.ArrayList]@(Split-Records $c)
        for ($i = 1; $i -lt $lines.Count; $i++) {
            if ([string]::IsNullOrWhiteSpace($lines[$i])) { continue }
            $p = $lines[$i] -split ';'
            if ($p.Count -ge 7) {
                $copy = @($p)
                # A different quantity, so that the aggregation has to add the
                # two rows instead of doubling the first one.
                $copy[6] = '5'
                [void]$lines.Insert($i + 1, ($copy -join ';'))
            }
            break
        }
        return ($lines -join "`r")
    }
}

# Case C - header mismatch: one delivered column renamed. That file is rejected
# on its own while the other four files are still processed (VR-02).
New-CaseSet -Name 'case-c-header-rejected' -Mutations @{
    'ALTHVD' = {
        param($c)
        $lines = Split-Records $c
        $lines[0] = $lines[0] -replace 'BATCH_CODE', 'ITEM_CODE'
        return ($lines -join "`r")
    }
}

# Case D - sales force customer that the customer master does not hold (VR-14).
New-CaseSet -Name 'case-d-salesforce-orphan-customer' -Mutations @{
    'ALTHFV' = {
        param($c)
        $lines = Split-Records $c
        for ($i = 1; $i -lt $lines.Count; $i++) {
            if ([string]::IsNullOrWhiteSpace($lines[$i])) { continue }
            $p = $lines[$i] -split ';'
            if ($p.Count -ge 3) { $p[2] = '999UNKNOWN'; $lines[$i] = ($p -join ';') }
            break
        }
        return ($lines -join "`r")
    }
}

Write-Output "derived fixtures under $derivedRoot"
