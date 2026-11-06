#Requires -Version 5.1
<#
.SYNOPSIS
  FlapsErp 效能基線量測工具（C-03 / T-101）
.DESCRIPTION
  三個唯讀探測器：
    baseline - FlapsErp 行程採樣（RAM/CPU/執行緒/分頁），10 次取樣
    com      - COM 每呼叫延遲基準（1000 迴路，P50/P95，使用 Scripting.FileSystemObject 唯讀操作）
    cdp      - Chromium DevTools Protocol 埠探測（9222，唯讀 /json/list）
  輸出：JSON + Markdown 至 artifacts/perf/<timestamp>/
  紅線：唯讀觀測；不寫入 C:\SYSTEM_PROGRAM_DIRECTORY。
  回滾：刪除 artifacts/perf/<timestamp> 目錄即可。
.NOTES
  無 BOM 安全：本檔僅含 ASCII 字面量；中文輸出由 UTF-8 資料檔處理。
#>
param(
  [ValidateSet('baseline','com','cdp','all')]
  [string]$Probe = 'all',
  [string]$OutRoot = (Join-Path $PSScriptRoot '..\..\artifacts\perf')
)
$ErrorActionPreference = 'Continue'
$ts = Get-Date -Format 'yyyyMMdd-HHmmss'
$outDir = Join-Path $OutRoot $ts
New-Item -ItemType Directory -Path $outDir -Force | Out-Null
$result = [ordered]@{ timestamp = $ts; host = $env:COMPUTERNAME; user = $env:USERNAME; probes = [ordered]@{} }

# ---------- baseline ----------
function Invoke-BaselineProbe {
  $samples = 10
  $found = Get-Process -Name 'FlapsErp' -ErrorAction SilentlyContinue
  if (-not $found) {
    return @{ status = 'APP-NOT-RUNNING'; note = 'Launch FlapsErp then re-run: & "C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\FlapsErp.exe"; or schedule via Task Scheduler 09:00/14:00 daily.' }
  }
  $data = @()
  for ($i = 0; $i -lt $samples; $i++) {
    $p = Get-Process -Name 'FlapsErp' -ErrorAction SilentlyContinue
    foreach ($proc in $p) {
      $data += [pscustomobject]@{
        t = (Get-Date).ToString('o'); pid = $proc.Id
        wsMB = [math]::Round($proc.WorkingSet64/1MB,1)
        pmMB = [math]::Round($proc.PagedMemorySize64/1MB,1)
        cpuSec = [math]::Round($proc.TotalProcessorTime.TotalSeconds,1)
        threads = $proc.Threads.Count; handles = $proc.HandleCount
      }
    }
    Start-Sleep -Milliseconds 1000
  }
  $ws = $data | ForEach-Object { $_.wsMB } | Sort-Object
  return @{
    status = 'OK'; samples = $samples; instances = ($data | Select-Object -Unique pid).Count
    wsMB_min = $ws[0]; wsMB_max = $ws[-1]; wsMB_median = $ws[[int][math]::Floor($ws.Count/2)]
    rows = $data
  }
}

# ---------- com ----------
function Invoke-ComProbe {
  # Per-call COM latency probe using an always-available, read-only COM object.
  # Establishes the per-call overhead floor that the winax sendSync pattern amplifies.
  $iterations = 1000
  $times = New-Object System.Collections.Generic.List[double]
  try {
    $fso = New-Object -ComObject Scripting.FileSystemObject
    $sw = [System.Diagnostics.Stopwatch]::new()
    for ($i = 0; $i -lt $iterations; $i++) {
      $sw.Restart()
      $null = $fso.GetSpecialFolder(2).Path   # read-only: temp folder path
      $sw.Stop()
      $times.Add($sw.Elapsed.TotalMilliseconds)
    }
    [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($fso)
  } catch {
    return @{ status = 'FAIL'; error = $_.Exception.Message }
  }
  $sorted = $times | Sort-Object
  $p50 = $sorted[[int][math]::Floor($iterations*0.50)]
  $p95 = $sorted[[int][math]::Floor($iterations*0.95)]
  $p99 = $sorted[[int][math]::Floor($iterations*0.99)]
  return @{
    status = 'OK'; iterations = $iterations; object = 'Scripting.FileSystemObject.GetSpecialFolder(read-only)'
    ms_p50 = [math]::Round($p50,4); ms_p95 = [math]::Round($p95,4); ms_p99 = [math]::Round($p99,4)
    ms_mean = [math]::Round(($times | Measure-Object -Average).Average,4)
    note = 'Baseline per-call COM overhead floor. Renderer sendSync adds IPC + serialization on top.'
  }
}

# ---------- cdp ----------
function Get-PortOwner {
  param([int]$Port)
  try {
    $conn = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction Stop | Select-Object -First 1
    $proc = Get-Process -Id $conn.OwningProcess -ErrorAction SilentlyContinue
    if ($proc) { return @{ pid = $proc.Id; name = $proc.ProcessName } }
    return @{ pid = $conn.OwningProcess; name = 'unknown' }
  } catch { return $null }
}

function Invoke-CdpProbe {
  # Target verification: /json/list alone proves nothing (any Chromium-based
  # browser listening on 9222 answers, e.g. Edge with --remote-debugging-port).
  # Identity comes from /json/version Browser/User-Agent plus process ownership.
  $ports = @(9222)
  $results = @()
  $erpRunning = [bool](Get-Process -Name 'FlapsErp' -ErrorAction SilentlyContinue)
  foreach ($port in $ports) {
    $owner = Get-PortOwner -Port $port
    $version = $null
    try {
      $resp = Invoke-WebRequest -Uri "http://127.0.0.1:$port/json/version" -TimeoutSec 3 -UseBasicParsing -ErrorAction Stop
      $version = $resp.Content | ConvertFrom-Json
    } catch { }
    if (-not $version) {
      $results += @{ port = $port; status = 'NOT-FOUND'; erpRunning = $erpRunning; ownerPid = $owner.pid; ownerName = $owner.name }
      continue
    }
    $browser = [string]$version.Browser
    $userAgent = [string]$version.'User-Agent'
    $isEdge = ($browser -match 'Edg/') -or ($userAgent -match 'Edg/')
    if (-not $erpRunning) {
      $status = 'FOUND-NOT-FLAPSERP'   # FlapsErp not running: cannot be its CDP
    } elseif ($isEdge) {
      $status = 'FOUND-FOREIGN-BROWSER'
    } elseif ($userAgent -match 'Electron' -or $browser -match 'Chrome/') {
      $status = 'FOUND-CANDIDATE'
    } else {
      $status = 'FOUND-UNVERIFIED'
    }
    $results += @{ port = $port; status = $status; browser = $browser; userAgent = $userAgent; erpRunning = $erpRunning; ownerPid = $owner.pid; ownerName = $owner.name }
  }
  return @{
    status = 'OK'; erpRunning = $erpRunning; probes = $results
    launchHint = 'For A3 track (needs on-site verification + EULA policy): relaunch with --remote-debugging-port=9222 via flapserp:// or shortcut arguments.'
  }
}
switch ($Probe) {
  'baseline' { $result.probes.baseline = Invoke-BaselineProbe }
  'com'      { $result.probes.com      = Invoke-ComProbe }
  'cdp'      { $result.probes.cdp      = Invoke-CdpProbe }
  'all'      {
    $result.probes.baseline = Invoke-BaselineProbe
    $result.probes.com      = Invoke-ComProbe
    $result.probes.cdp      = Invoke-CdpProbe
  }
}

$jsonPath = Join-Path $outDir 'perf-result.json'
$result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $jsonPath -Encoding UTF8

# Markdown summary
$md = New-Object System.Collections.Generic.List[string]
$md.Add("# FlapsErp Perf Baseline - $ts")
$md.Add("")
if ($result.probes.baseline) {
  $b = $result.probes.baseline
  $md.Add("## baseline: $($b.status)")
  if ($b.status -eq 'OK') { $md.Add("- instances=$($b.instances)  wsMB median=$($b.wsMB_median) min=$($b.wsMB_min) max=$($b.wsMB_max)") }
  else { $md.Add("- $($b.note)") }
}
if ($result.probes.com) {
  $c = $result.probes.com
  $md.Add("## com: $($c.status)")
  if ($c.status -eq 'OK') { $md.Add("- $($c.object)  n=$($c.iterations)  P50=$($c.ms_p50)ms P95=$($c.ms_p95)ms P99=$($c.ms_p99)ms mean=$($c.ms_mean)ms") }
  else { $md.Add("- ERROR: $($c.error)") }
}
if ($result.probes.cdp) {
  $d = $result.probes.cdp
  $md.Add("## cdp: erpRunning=$($d.erpRunning)")
  foreach ($p in $d.probes) { $md.Add("- port $($p.port): $($p.status) owner=$($p.ownerName)($($p.ownerPid)) browser=$($p.browser)") }
}
$mdPath = Join-Path $outDir 'perf-summary.md'
$md | Set-Content -LiteralPath $mdPath -Encoding UTF8

Write-Output "OUT_DIR: $outDir"
Write-Output ($md -join "`n")
