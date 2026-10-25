[CmdletBinding()]
param(
    [string]$Executable = (Join-Path $env:LOCALAPPDATA 'Temp\wellfit-target\release\wellfit.exe'),
    [ValidateRange(1, 1000000)]
    [int]$ShopeeRows = 333333,
    [ValidateRange(1, 1000000)]
    [int]$ReportRows = 333333,
    [ValidateRange(4, 1000000)]
    [int]$MtrixRows = 333334,
    [string]$OutputDirectory = (Join-Path $env:TEMP ("wellfit-perf-" + (Get-Date -Format 'yyyyMMdd-HHmmss') + "-" + $PID))
)

$ErrorActionPreference = 'Stop'
$MaximumInputBytes = 64MB

if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) {
    throw "Wellfit executable not found: $Executable"
}
if (Test-Path -LiteralPath $OutputDirectory) {
    throw "Output directory already exists; choose a new path: $OutputDirectory"
}
if (($ShopeeRows + $ReportRows + $MtrixRows) -gt 1000000) {
    throw 'The total benchmark data rows cannot exceed 1,000,000.'
}

$root = [System.IO.Path]::GetFullPath($OutputDirectory)
$inputs = Join-Path $root 'inputs'
$outputs = Join-Path $root 'outputs'
$logs = Join-Path $root 'logs'
New-Item -ItemType Directory -Path $inputs, $outputs, $logs | Out-Null
$utf8 = [System.Text.UTF8Encoding]::new($false)
$culture = [System.Globalization.CultureInfo]::InvariantCulture

function New-Utf8Writer {
    param([string]$Path)
    return [System.IO.StreamWriter]::new($Path, $false, $utf8)
}

function Get-FileEvidence {
    param([string]$Path)
    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256
    return [pscustomobject]@{
        path = $item.FullName
        bytes = $item.Length
        sha256 = $hash.Hash
    }
}

function ConvertTo-ProcessArgument {
    param([string]$Value)
    if ($Value.Contains('"')) {
        throw 'Benchmark arguments must not contain quotation marks.'
    }
    return '"' + $Value + '"'
}

function Invoke-MeasuredCli {
    param(
        [string]$Name,
        [string[]]$Arguments
    )
    $stdout = Join-Path $logs "$Name.stdout.txt"
    $stderr = Join-Path $logs "$Name.stderr.txt"
    $argumentLine = ($Arguments | ForEach-Object { ConvertTo-ProcessArgument $_ }) -join ' '
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Executable
    $startInfo.Arguments = $argumentLine
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    if (-not $process.Start()) {
        throw "Could not start benchmark stage: $Name"
    }
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $peakWorkingSet = $process.WorkingSet64
    while (-not $process.HasExited) {
        $process.Refresh()
        if (-not $process.HasExited -and $process.WorkingSet64 -gt $peakWorkingSet) {
            $peakWorkingSet = $process.WorkingSet64
        }
        Start-Sleep -Milliseconds 50
    }
    $process.WaitForExit()
    $process.Refresh()
    $watch.Stop()
    if ($process.WorkingSet64 -gt $peakWorkingSet) {
        $peakWorkingSet = $process.WorkingSet64
    }
    $stdoutText = $stdoutTask.GetAwaiter().GetResult()
    $stderrText = $stderrTask.GetAwaiter().GetResult()
    [System.IO.File]::WriteAllText($stdout, $stdoutText, $utf8)
    [System.IO.File]::WriteAllText($stderr, $stderrText, $utf8)
    if ($process.ExitCode -ne 0) {
        $exitCode = $process.ExitCode
        $process.Dispose()
        throw "$Name exited with code $exitCode.`n$stdoutText`n$stderrText"
    }
    $process.Dispose()
    return [pscustomobject]@{
        name = $Name
        elapsed_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 3)
        peak_sampled_working_set_bytes = $peakWorkingSet
        stdout = $stdoutText
        stderr = $stderrText
    }
}

function Get-StatusEvidence {
    param([string]$Name, [string]$Database)
    $status = Invoke-MeasuredCli -Name "$Name-status" -Arguments @('status', '--db', $Database)
    $isolatedMatch = [regex]::Match($status.stdout, 'isolated records\s*:\s*(\d+)')
    $errorsMatch = [regex]::Match($status.stdout, 'validation errors\s*:\s*(\d+)')
    $warningsMatch = [regex]::Match($status.stdout, 'validation warnings\s*:\s*(\d+)')
    if (-not $isolatedMatch.Success -or -not $errorsMatch.Success -or -not $warningsMatch.Success) {
        throw "Could not parse ledger status for ${Name}:`n$($status.stdout)"
    }
    return [pscustomobject]@{
        isolated_records = [int]$isolatedMatch.Groups[1].Value
        validation_errors = [int]$errorsMatch.Groups[1].Value
        validation_warnings = [int]$warningsMatch.Groups[1].Value
        database_bytes = (Get-Item -LiteralPath $Database).Length
    }
}

$shopeeInput = Join-Path $inputs 'shopee.csv'
$reportsInput = Join-Path $inputs 'reports.csv'
$shopeeOutput = Join-Path $outputs 'shopee-erp.csv'
$shopeeDb = Join-Path $root 'shopee.sqlite'
$reportsDb = Join-Path $root 'reports.sqlite'
$mtrixDb = Join-Path $root 'mtrix.sqlite'

$writer = New-Utf8Writer $shopeeInput
try {
    $writer.WriteLine('ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT')
    for ($i = 0; $i -lt $ShopeeRows; $i++) {
        $id = $i.ToString('D10', $culture)
        $writer.WriteLine("SYN$id,CUST-SYN,2026-10-23 08:15:00,100,HAV-TOP-BLK-39,1,100,100")
    }
}
finally {
    $writer.Dispose()
}

$writer = New-Utf8Writer $reportsInput
try {
    $writer.WriteLine('ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV')
    for ($i = 0; $i -lt $ReportRows; $i++) {
        $id = $i.ToString('D10', $culture)
        $writer.WriteLine("SYN$id,SyntheticItem$id,WH-SYN,0,1,0,0,1")
    }
}
finally {
    $writer.Dispose()
}

$mtrixDirectory = Join-Path $inputs 'mtrix'
New-Item -ItemType Directory -Path $mtrixDirectory | Out-Null
$customerFile = Join-Path $mtrixDirectory 'ALTHCI23102026000000000.csv'
$inventoryFile = Join-Path $mtrixDirectory 'ALTHES23102026000000000.csv'
$salesForceFile = Join-Path $mtrixDirectory 'ALTHFV23102026000000000.csv'
$productFile = Join-Path $mtrixDirectory 'ALTHPD23102026000000000.csv'
$salesFile = Join-Path $mtrixDirectory 'ALTHVD23102026000000000.csv'

$writer = New-Utf8Writer $customerFile
try {
    $writer.WriteLine('VENDOR_ID;DIST_ID;CUSTOMER_ID;CUSTOMER_NAME;ADRESS;DISTRICT;ZIP_CODE;CITY;STATE;SALES_CONTACT;PHONE_NUMBERS;CUSTOMER_ID;ROUTE;STORE_TYPE;PROPORTION;COUNTRY_ISO')
    $customerColumns = @(
        '6063343900011222', '029864857023000', '995A', 'SyntheticCustomer',
        '', '', '', '', '', '', '', '995A', 'SYN', 'Online', '0', 'USA'
    )
    $writer.WriteLine([string]::Join(';', $customerColumns))
}
finally {
    $writer.Dispose()
}

$writer = New-Utf8Writer $inventoryFile
try {
    $writer.WriteLine('INVENTORY_DATE;VENDOR_ID;DIST_ID;BATCH_CODE;QUANTITY_STOCK;COUNTRY_ISO')
    $writer.WriteLine('20261023;6063343900011222;029864857023000;7898656600087;1;USA')
}
finally {
    $writer.Dispose()
}

$writer = New-Utf8Writer $salesForceFile
try {
    $writer.WriteLine('VENDOR_ID;DIST_ID;CUSTOMER_ID;MANAGER_CODE;MANAGER_NAME;SUPERVISOR_ID;SUPERVISOR_NAME;SALESPERSON_CODE;SALESPERSON_NAME;COUNTRY')
    $writer.WriteLine('6063343900011222;029864857023000;995A;SYN-MGR;SyntheticManager;;;SYN-SP;SyntheticSeller;USA')
}
finally {
    $writer.Dispose()
}

$writer = New-Utf8Writer $productFile
try {
    $writer.WriteLine('TRANSACTION_DATE;DIST_ID;VENDOR_ID;VENDOR_COMPANY_NAME;PRODUCT_CODE;PACKAGE_TYPE;BARCODE;BARCODE_TYPE;DESCRIPTION;PRODUCT_DIV;PRODUCT_STATUS;COUNTRY_ISO')
    $writer.WriteLine('20261023;029864857023000;6063343900011222;SyntheticVendor;SYN-ITEM;0;7898656600087;1;Synthetic product;SYN;A;USA')
}
finally {
    $writer.Dispose()
}

$salesRows = $MtrixRows - 4
$writer = New-Utf8Writer $salesFile
try {
    $writer.WriteLine('VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO')
    for ($i = 0; $i -lt $salesRows; $i++) {
        $document = $i.ToString('D10', $culture)
        $writer.WriteLine("6063343900011222;029864857023000;995A;20261023;$document;7898656600087;1;100;SYN-SP;N;00000;USA")
    }
}
finally {
    $writer.Dispose()
}

$inputFiles = @(
    (Get-FileEvidence $shopeeInput),
    (Get-FileEvidence $reportsInput),
    (Get-FileEvidence $customerFile),
    (Get-FileEvidence $inventoryFile),
    (Get-FileEvidence $salesForceFile),
    (Get-FileEvidence $productFile),
    (Get-FileEvidence $salesFile)
)
$oversized = @($inputFiles | Where-Object { $_.bytes -gt $MaximumInputBytes })
if ($oversized.Count -gt 0) {
    throw "Input exceeds the 64 MiB cap: $($oversized.path -join ', ')"
}

$shopeeRun = Invoke-MeasuredCli -Name 'shopee' -Arguments @(
    'shopee', '--input', $shopeeInput, '--output', $shopeeOutput,
    '--batch', 'SYNTHETIC-BENCH-SHOPEE', '--db', $shopeeDb
)
$shopeeCounts = [regex]::Match($shopeeRun.stdout, 'wave 1 shopee: (\d+) order\(s\) accepted, (\d+) record\(s\) isolated')
if (-not $shopeeCounts.Success -or [int]$shopeeCounts.Groups[1].Value -ne $ShopeeRows -or [int]$shopeeCounts.Groups[2].Value -ne 0) {
    throw "Shopee count verification failed:`n$($shopeeRun.stdout)"
}
$shopeeStatus = Get-StatusEvidence -Name 'shopee' -Database $shopeeDb
if ($shopeeStatus.isolated_records -ne 0 -or $shopeeStatus.validation_errors -ne 0 -or $shopeeStatus.validation_warnings -ne 0) {
    throw "Shopee ledger recorded an unexpected issue: $($shopeeStatus | ConvertTo-Json -Compress)"
}

$reportsRun = Invoke-MeasuredCli -Name 'reports' -Arguments @(
    'reports', '--input', $reportsInput, '--batch', 'SYNTHETIC-BENCH-REPORTS',
    '--db', $reportsDb
)
$reportsCounts = [regex]::Match($reportsRun.stdout, 'wave 2 reports: (\d+) record\(s\) accepted, (\d+) record\(s\) isolated')
if (-not $reportsCounts.Success -or [int]$reportsCounts.Groups[1].Value -ne $ReportRows -or [int]$reportsCounts.Groups[2].Value -ne 0) {
    throw "Reports count verification failed:`n$($reportsRun.stdout)"
}
$reportsStatus = Get-StatusEvidence -Name 'reports' -Database $reportsDb
if ($reportsStatus.isolated_records -ne 0 -or $reportsStatus.validation_errors -ne 0 -or $reportsStatus.validation_warnings -ne 0) {
    throw "Reports ledger recorded an unexpected issue: $($reportsStatus | ConvertTo-Json -Compress)"
}

$mtrixOutput = Join-Path $outputs 'mtrix'
$mtrixReportPath = Join-Path $root 'mtrix-report.json'
$mtrixRun = Invoke-MeasuredCli -Name 'mtrix' -Arguments @(
    'mtrix', '--dir', $mtrixDirectory, '--output-dir', $mtrixOutput,
    '--report', $mtrixReportPath, '--strict', '--batch', 'SYNTHETIC-BENCH-MTRIX',
    '--db', $mtrixDb
)
$mtrixReport = Get-Content -LiteralPath $mtrixReportPath -Raw | ConvertFrom-Json
$mtrixAcceptedRows = ($mtrixReport.files | Measure-Object -Property rows_accepted -Sum).Sum
$mtrixTotalRows = ($mtrixReport.files | Measure-Object -Property rows_total -Sum).Sum
$mtrixErrors = @($mtrixReport.findings | Where-Object { $_.severity -eq 'Error' }).Count
$mtrixWarnings = @($mtrixReport.findings | Where-Object { $_.severity -eq 'Warning' }).Count
$mtrixQuarantined = ($mtrixReport.files | Measure-Object -Property rows_quarantined -Sum).Sum
if ($mtrixTotalRows -ne $MtrixRows -or $mtrixAcceptedRows -ne $MtrixRows -or $mtrixQuarantined -ne 0 -or $mtrixErrors -ne 0 -or $mtrixWarnings -ne 0) {
    throw "MTRIX count or finding verification failed:`n$($mtrixRun.stdout)"
}
$mtrixStatus = Get-StatusEvidence -Name 'mtrix' -Database $mtrixDb
if ($mtrixStatus.isolated_records -ne 0 -or $mtrixStatus.validation_errors -ne 0 -or $mtrixStatus.validation_warnings -ne 0) {
    throw "MTRIX ledger recorded an unexpected issue: $($mtrixStatus | ConvertTo-Json -Compress)"
}

$stageResults = @(
    [pscustomobject]@{
        pipeline = 'Shopee'
        rows = $ShopeeRows
        elapsed_seconds = $shopeeRun.elapsed_seconds
        rows_per_second = [Math]::Round($ShopeeRows / $shopeeRun.elapsed_seconds, 2)
        peak_sampled_working_set_bytes = $shopeeRun.peak_sampled_working_set_bytes
        accepted_rows = [int]$shopeeCounts.Groups[1].Value
        isolated_rows = [int]$shopeeCounts.Groups[2].Value
        ledger = $shopeeStatus
        output_files = @((Get-FileEvidence $shopeeOutput))
        log_files = @($shopeeRun.name + '.stdout.txt', $shopeeRun.name + '.stderr.txt')
    },
    [pscustomobject]@{
        pipeline = 'Reports'
        rows = $ReportRows
        elapsed_seconds = $reportsRun.elapsed_seconds
        rows_per_second = [Math]::Round($ReportRows / $reportsRun.elapsed_seconds, 2)
        peak_sampled_working_set_bytes = $reportsRun.peak_sampled_working_set_bytes
        accepted_rows = [int]$reportsCounts.Groups[1].Value
        isolated_rows = [int]$reportsCounts.Groups[2].Value
        ledger = $reportsStatus
        output_files = @()
        log_files = @($reportsRun.name + '.stdout.txt', $reportsRun.name + '.stderr.txt')
    },
    [pscustomobject]@{
        pipeline = 'MTRIX'
        rows = $MtrixRows
        elapsed_seconds = $mtrixRun.elapsed_seconds
        rows_per_second = [Math]::Round($MtrixRows / $mtrixRun.elapsed_seconds, 2)
        peak_sampled_working_set_bytes = $mtrixRun.peak_sampled_working_set_bytes
        accepted_rows = $mtrixAcceptedRows
        isolated_rows = $mtrixQuarantined
        findings_errors = $mtrixErrors
        findings_warnings = $mtrixWarnings
        ledger = $mtrixStatus
        output_files = @(Get-ChildItem -LiteralPath $mtrixOutput -File | ForEach-Object { Get-FileEvidence $_.FullName })
        log_files = @($mtrixRun.name + '.stdout.txt', $mtrixRun.name + '.stderr.txt')
    }
)

$result = [pscustomobject]@{
    benchmark_type = 'synthetic-performance-only'
    production_acceptance = $false
    authorization = 'One-off local benchmark explicitly authorized by the user; generated data is fictional.'
    generated_at_utc = [DateTime]::UtcNow.ToString('o', $culture)
    output_directory = $root
    total_rows = $ShopeeRows + $ReportRows + $MtrixRows
    max_input_file_bytes = $MaximumInputBytes
    working_set_measurement = 'Maximum sampled process working set, polled every 50 ms; not a hard memory bound.'
    inputs = $inputFiles
    pipelines = $stageResults
    results_json = Join-Path $root 'benchmark-results.json'
}
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $result.results_json -Encoding UTF8
$result
