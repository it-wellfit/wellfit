[CmdletBinding()]
param(
    [ValidateRange(1, 1000000)]
    [int]$RecordsPerPipeline = 200000,
    [string]$WellfitExe = (Join-Path $env:LOCALAPPDATA 'Temp\wellfit-target\debug\wellfit.exe'),
    [string]$OutputDirectory = (Join-Path $env:TEMP ("wellfit-synthetic-benchmark-" + [guid]::NewGuid().ToString('N')))
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$resolvedOutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)

if (-not (Test-Path -LiteralPath $WellfitExe -PathType Leaf)) {
    throw "Wellfit executable not found: $WellfitExe"
}
if ($resolvedOutputDirectory.StartsWith(
        $repositoryRoot,
        [StringComparison]::OrdinalIgnoreCase
    )) {
    throw 'Synthetic benchmark inputs and outputs must remain outside the repository.'
}
if (Test-Path -LiteralPath $resolvedOutputDirectory) {
    if ((Get-ChildItem -LiteralPath $resolvedOutputDirectory -Force | Measure-Object).Count -gt 0) {
        throw "Output directory must be empty: $resolvedOutputDirectory"
    }
}
else {
    New-Item -ItemType Directory -Path $resolvedOutputDirectory | Out-Null
}

$utf8 = [Text.UTF8Encoding]::new($false)

function Write-SyntheticCsv {
    param(
        [string]$Path,
        [string]$Header,
        [scriptblock]$RowFactory
    )

    $writer = [IO.StreamWriter]::new($Path, $false, $utf8, 1MB)
    try {
        $writer.NewLine = "`n"
        $writer.WriteLine($Header)
        for ($index = 1; $index -le $RecordsPerPipeline; $index++) {
            $writer.WriteLine((& $RowFactory $index))
        }
    }
    finally {
        $writer.Dispose()
    }
}

function Quote-ProcessArgument {
    param([string]$Value)
    '"' + $Value.Replace('"', '\"') + '"'
}

function Get-PhysicalLineCount {
    param([string]$Path)

    $reader = [IO.StreamReader]::new($Path)
    $count = 0
    try {
        while ($null -ne $reader.ReadLine()) {
            $count++
        }
    }
    finally {
        $reader.Dispose()
    }
    $count
}

function Invoke-MeasuredPipeline {
    param(
        [string]$Name,
        [string[]]$Arguments,
        [string]$InputPath,
        [string]$DatabasePath,
        [string]$OutputPath
    )

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $WellfitExe
    $startInfo.Arguments = (($Arguments | ForEach-Object { Quote-ProcessArgument $_ }) -join ' ')
    $startInfo.WorkingDirectory = $repositoryRoot
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $stopwatch = [Diagnostics.Stopwatch]::StartNew()
    if (-not $process.Start()) {
        throw "$Name process did not start."
    }
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $peakWorkingSet = [long]0
    $workingSetSamples = 0
    if (-not $process.HasExited) {
        $process.Refresh()
        $peakWorkingSet = $process.WorkingSet64
        $workingSetSamples++
    }
    while (-not $process.WaitForExit(100)) {
        $process.Refresh()
        if ($process.WorkingSet64 -gt $peakWorkingSet) {
            $peakWorkingSet = $process.WorkingSet64
        }
        $workingSetSamples++
    }
    $stopwatch.Stop()
    $stdout = $stdoutTask.GetAwaiter().GetResult()
    $stderr = $stderrTask.GetAwaiter().GetResult()
    $cpuTime = $process.TotalProcessorTime
    $exitCode = $process.ExitCode
    $process.Dispose()

    if ($exitCode -ne 0) {
        throw "$Name failed with exit code $exitCode.`n$stdout`n$stderr"
    }

    $acceptedPattern = if ($Name -eq 'Shopee') {
        'orders accepted\s*:?\s*(\d+)'
    }
    else {
        'records accepted\s*:?\s*(\d+)'
    }
    $acceptedMatch = [regex]::Match($stdout, $acceptedPattern)
    if (-not $acceptedMatch.Success) {
        throw "$Name completed without a parseable accepted-record count.`n$stdout"
    }
    $accepted = [int64]$acceptedMatch.Groups[1].Value
    if ($accepted -ne $RecordsPerPipeline) {
        throw "$Name accepted $accepted records; expected $RecordsPerPipeline.`n$stdout"
    }

    $inputItem = Get-Item -LiteralPath $InputPath
    $databaseItem = Get-Item -LiteralPath $DatabasePath
    $inputLineCount = Get-PhysicalLineCount -Path $InputPath
    if ($inputLineCount -ne ($RecordsPerPipeline + 1)) {
        throw "$Name input has $($inputLineCount - 1) data rows; expected $RecordsPerPipeline."
    }
    $outputLineCount = $null
    $outputHash = $null
    if ($OutputPath -and (Test-Path -LiteralPath $OutputPath)) {
        $outputLineCount = Get-PhysicalLineCount -Path $OutputPath
        if ($outputLineCount -ne ($accepted + 1)) {
            throw "$Name output has $($outputLineCount - 1) rows; expected $accepted."
        }
        $outputHash = (Get-FileHash -LiteralPath $OutputPath -Algorithm SHA256).Hash
    }
    $result = [ordered]@{
        Pipeline = $Name
        Synthetic = $true
        InputRecords = $RecordsPerPipeline
        AcceptedRecords = $accepted
        IsolatedRecords = 0
        InputPhysicalLines = $inputLineCount
        InputBytes = $inputItem.Length
        InputSha256 = (Get-FileHash -LiteralPath $InputPath -Algorithm SHA256).Hash
        WallSeconds = [Math]::Round($stopwatch.Elapsed.TotalSeconds, 3)
        RecordsPerSecond = [Math]::Round($accepted / [Math]::Max($stopwatch.Elapsed.TotalSeconds, 0.001), 2)
        CpuSeconds = [Math]::Round($cpuTime.TotalSeconds, 3)
        PeakWorkingSetBytes = if ($workingSetSamples -gt 0) { $peakWorkingSet } else { $null }
        WorkingSetSampleCount = $workingSetSamples
        LedgerBytes = $databaseItem.Length
        LedgerSha256 = (Get-FileHash -LiteralPath $DatabasePath -Algorithm SHA256).Hash
        OutputPhysicalLines = $outputLineCount
        OutputSha256 = $outputHash
        OutputBytes = if ($OutputPath -and (Test-Path -LiteralPath $OutputPath)) {
            (Get-Item -LiteralPath $OutputPath).Length
        }
        else {
            $null
        }
        Stdout = $stdout.Trim()
        Stderr = $stderr.Trim()
    }
    [pscustomobject]$result
}

$shopeeInput = Join-Path $resolvedOutputDirectory 'shopee_synthetic.csv'
$shopeeOutput = Join-Path $resolvedOutputDirectory 'shopee_import_synthetic.csv'
$shopeeDatabase = Join-Path $resolvedOutputDirectory 'shopee_synthetic.sqlite'
$reportsInput = Join-Path $resolvedOutputDirectory 'reports_synthetic.csv'
$reportsDatabase = Join-Path $resolvedOutputDirectory 'reports_synthetic.sqlite'

Write-SyntheticCsv -Path $shopeeInput `
    -Header 'ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT' `
    -RowFactory {
        param($index)
        'SYNTH-{0:D10},SYNTH-CUSTOMER,2026-11-02 08:00:00,100,HAV-TOP-BLK-39,2,50,100' -f $index
    }
Write-SyntheticCsv -Path $reportsInput `
    -Header 'ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV' `
    -RowFactory {
        param($index)
        'SYNTH-ITEM-{0:D10},Synthetic product {0:D10},WH-SYNTH,100,20,10,0,110' -f $index
    }

$maximumInputBytes = 100MB
foreach ($inputPath in @($shopeeInput, $reportsInput)) {
    $inputLength = (Get-Item -LiteralPath $inputPath).Length
    if ($inputLength -gt $maximumInputBytes) {
        throw "Synthetic input exceeds Wellfit's current 100 MiB input cap: $inputPath ($inputLength bytes)"
    }
}

$runId = [guid]::NewGuid().ToString('N')
$results = @(
    Invoke-MeasuredPipeline -Name 'Shopee' `
        -Arguments @(
            '--color', 'never', 'shopee', '--input', $shopeeInput,
            '--output', $shopeeOutput, '--db', $shopeeDatabase,
            '--batch', "SYNTH-SHOPEE-$runId"
        ) `
        -InputPath $shopeeInput -DatabasePath $shopeeDatabase -OutputPath $shopeeOutput
    Invoke-MeasuredPipeline -Name 'Reports' `
        -Arguments @(
            '--color', 'never', 'reports', '--input', $reportsInput,
            '--db', $reportsDatabase, '--batch', "SYNTH-REPORTS-$runId"
        ) `
        -InputPath $reportsInput -DatabasePath $reportsDatabase -OutputPath ''
)

$manifest = [ordered]@{
    ReportType = 'Synthetic local ingestion capacity measurement'
    GeneratedUtc = [DateTime]::UtcNow.ToString('o')
    Machine = [Environment]::MachineName
    ProcessorCount = [Environment]::ProcessorCount
    RecordsPerPipeline = $RecordsPerPipeline
    DataIsSynthetic = $true
    Scope = 'Shopee and Reports only; excludes MTRIX, BankPay, and Sync'
    MeasurementLimitations = @(
        'Working set is sampled on startup and every 100 ms; null means the process exited before sampling, and samples may miss shorter peaks.',
        'Each pipeline is run once; this is not a cold/warm-cache or concurrency test.',
        'Synthetic results are not production SLA evidence.',
        'Sync has no data-transfer implementation; no throughput is inferred for it.'
    )
    Pipelines = $results
}
$manifestPath = Join-Path $resolvedOutputDirectory 'benchmark.json'
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding utf8

$results | Select-Object Pipeline, InputRecords, AcceptedRecords, WallSeconds, RecordsPerSecond,
    CpuSeconds, PeakWorkingSetBytes, InputBytes, LedgerBytes | Format-Table -AutoSize
Write-Output "Synthetic benchmark report: $manifestPath"
