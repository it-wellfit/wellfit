[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet('BankPay', 'MTRIX')]
    [string]$Pipeline,

    [int]$Records,

    [string]$TargetDir = (Join-Path $env:LOCALAPPDATA 'Temp\wellfit-target')
)

$ErrorActionPreference = 'Stop'
if (-not $PSBoundParameters.ContainsKey('Records')) {
    $Records = if ($Pipeline -eq 'MTRIX') { 50000 } else { 250000 }
}
$workspaceRoot = Split-Path -Parent $PSScriptRoot
$buildScript = Join-Path $workspaceRoot '..\..\Knowledge\tools\dev-env\build.ps1'

switch ($Pipeline) {
    'BankPay' {
        if ($Records -lt 1 -or $Records -gt 1000000) {
            throw 'BankPay records must be between 1 and 1,000,000.'
        }
        $env:BANKPAY_BENCH_RECORDS = [string]$Records
        $cargoCommand = 'test -p wellfit_bankpay --test integration_tests --no-run'
        $testName = 'bankpay_synthetic_mock_capacity_benchmark'
        $resultPattern = '^bankpay_synthetic_capacity '
    }
    'MTRIX' {
        if ($Records -lt 1 -or $Records -gt 250000) {
            throw 'MTRIX records per file must be between 1 and 250,000.'
        }
        $env:MTRIX_BENCH_ROWS_PER_FILE = [string]$Records
        $cargoCommand = 'test -p wellfit_mtrix --test synthetic_capacity --no-run'
        $testName = 'mtrix_synthetic_drop_capacity_benchmark'
        $resultPattern = '^mtrix_synthetic_capacity '
    }
}

$token = [guid]::NewGuid().ToString('N')
$buildStdoutPath = Join-Path $env:TEMP "wellfit-benchmark-$token.build.stdout"
$buildStderrPath = Join-Path $env:TEMP "wellfit-benchmark-$token.build.stderr"

try {
    $buildArguments = '-NoProfile -ExecutionPolicy Bypass -File "{0}" -Command "{1}" -TargetDir "{2}"' -f `
        $buildScript, $cargoCommand, $TargetDir
    $buildProcess = Start-Process -FilePath 'powershell.exe' `
        -ArgumentList $buildArguments `
        -RedirectStandardOutput $buildStdoutPath `
        -RedirectStandardError $buildStderrPath `
        -PassThru `
        -Wait
    $buildLog = @(
        (Get-Content -LiteralPath $buildStdoutPath -Raw),
        (Get-Content -LiteralPath $buildStderrPath -Raw)
    ) -join [Environment]::NewLine
    Write-Host $buildLog
    if ($buildProcess.ExitCode -ne 0) {
        throw "Benchmark test build failed with exit code $($buildProcess.ExitCode)."
    }

    $executableMatch = [regex]::Match(
        $buildLog,
        'Executable .+\((?<path>.+\.exe)\)'
    )
    if (-not $executableMatch.Success) {
        throw 'The benchmark test executable path was not reported by Cargo.'
    }
    $testExecutable = $executableMatch.Groups['path'].Value
    if (-not (Test-Path -LiteralPath $testExecutable -PathType Leaf)) {
        throw "Benchmark test executable not found: $testExecutable"
    }

    $startInfo = New-Object System.Diagnostics.ProcessStartInfo
    $startInfo.FileName = $testExecutable
    $startInfo.Arguments = "$testName --ignored --nocapture"
    $startInfo.WorkingDirectory = $workspaceRoot
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $startInfo
    if (-not $process.Start()) {
        throw 'Could not start the benchmark test process.'
    }
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $sampledPeakWorkingSet = 0L
    while (-not $process.HasExited) {
        $process.Refresh()
        if ($process.WorkingSet64 -gt $sampledPeakWorkingSet) {
            $sampledPeakWorkingSet = $process.WorkingSet64
        }
        Start-Sleep -Milliseconds 50
    }
    $process.WaitForExit()
    $process.Refresh()
    $stdout = $stdoutTask.Result
    $stderr = $stderrTask.Result
    if ($process.ExitCode -ne 0) {
        throw "Benchmark process failed with exit code $($process.ExitCode).`n$stdout`n$stderr"
    }

    $benchmarkLine = ($stdout -split '\r?\n') |
        Where-Object { $_ -match $resultPattern } |
        Select-Object -Last 1
    if ([string]::IsNullOrWhiteSpace($benchmarkLine)) {
        throw "Benchmark result line was not emitted.`n$stdout`n$stderr"
    }

    [ordered]@{
        pipeline = $Pipeline
        configured_records = $Records
        process_cpu_seconds = [math]::Round($process.TotalProcessorTime.TotalSeconds, 3)
        process_peak_working_set_bytes = [math]::Max(
            $sampledPeakWorkingSet,
            $process.PeakWorkingSet64
        )
        working_set_sample_interval_ms = 50
        benchmark_result = $benchmarkLine.Trim()
    } | ConvertTo-Json -Compress
}
finally {
    foreach ($path in @($buildStdoutPath, $buildStderrPath)) {
        if (Test-Path -LiteralPath $path -PathType Leaf) {
            Remove-Item -LiteralPath $path -Force
        }
    }
}
