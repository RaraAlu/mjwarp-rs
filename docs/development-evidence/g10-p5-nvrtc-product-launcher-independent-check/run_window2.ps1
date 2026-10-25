# G10@P5 NVRTC launcher QA window - run2 (remaining cases; approved decision A).
# Conditions: HEAD dfcc1bdc; prebuilt dumper only; baseline recheck; PATH prepend;
# 0.25 s sampling; hard guard 3072 MiB / 50% (abort); soft threshold +1536 MiB record-only;
# per-case byte compare with early stop.
# NOTE: keep this file ASCII-only (PowerShell 5.1 reads .ps1 as ANSI).

param(
    [string]$Repo = 'D:\Rust\mjwarp-rs'
)
$ErrorActionPreference = 'Stop'
$evidence = Join-Path $Repo 'docs\development-evidence\g10-p5-nvrtc-product-launcher-independent-check'
$log = Join-Path $evidence 'window-log2.txt'
$samplesPath = Join-Path $evidence 'window-memory-samples2.txt'
$outputPath = Join-Path $evidence 'window-output2.txt'
$stopFlag = Join-Path $env:TEMP 'qa-nvrtc-window2-stop.flag'
$samplesTmp = Join-Path $env:TEMP 'qa-nvrtc-window2-samples.txt'
$dumper = Join-Path $Repo 'target\qa-nvrtc\dumper-target\debug\qa-nvrtc-dumper.exe'
$casesDir = Join-Path $Repo 'docs\development-evidence\g10-p5-gpu-kernels\gpu-window\cases'
$prodOut = Join-Path $Repo 'target\qa-nvrtc\prod-out'
$python = Join-Path $Repo 'target\mjwarp-parallel\venv\Scripts\python.exe'
$cudaBin = Join-Path $Repo 'target\toolchains\cuda-12.8.1\bin'
$expectedHead = 'dfcc1bdc'
$expectedBaseline = 3669
$softMiB = 1536
$hardMiB = 3072

Remove-Item $log, $samplesPath, $outputPath, $samplesTmp, $stopFlag -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $prodOut | Out-Null
$lines = New-Object System.Collections.Generic.List[string]
function Note([string]$text) {
    $stamp = (Get-Date).ToString('HH:mm:ss.fff')
    $line = "[$stamp] $text"
    $lines.Add($line)
    Write-Output $line
}

function GpuMemory {
    $raw = ((& nvidia-smi --query-gpu=memory.used,memory.total,utilization.gpu,temperature.gpu,driver_version --format=csv,noheader) -join '')
    $parts = $raw -split ',\s*'
    return @{
        Used = [int]($parts[0] -replace '[^0-9]', '')
        Total = [int]($parts[1] -replace '[^0-9]', '')
        Raw = $raw
    }
}

function ComputeApps {
    return @(& nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>$null)
}

$env:PATH = $cudaBin + ';' + $env:PATH
Note 'window=start run=2 cases=sdf-box,ellipsoid-box'
$head = (& git -C $Repo rev-parse HEAD).Trim()
Note "head=$head expect_prefix=$expectedHead"
if (-not $head.StartsWith($expectedHead)) {
    Note 'ABORT: HEAD does not match approved condition'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 3
}
Note "path_prepend=$cudaBin"
$nvcc = (& nvcc --version) | Select-Object -Last 1
Note "nvcc=$nvcc"
foreach ($dll in 'nvrtc64_120_0.dll', 'nvrtc64_120_0.alt.dll', 'nvrtc-builtins64_128.dll') {
    $p = Join-Path $cudaBin $dll
    Note "nvrtc_dll_sha256=$dll=$((Get-FileHash $p -Algorithm SHA256).Hash.ToLower())"
}

$pre = GpuMemory
Note "baseline_memory=$($pre.Raw)"
Note "baseline_used_mib=$($pre.Used)"
$drift = [math]::Abs($pre.Used - $expectedBaseline)
Note "baseline_drift_mib=$drift expected=$expectedBaseline limit=200"
if ($drift -gt 200) {
    Note 'ABORT: baseline drift > 200 MiB'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 4
}
$appsBefore = ComputeApps
$taskBefore = @($appsBefore | Where-Object { $_ -match 'qa-nvrtc|dumper|probe|python|cargo|rustc' })
Note "baseline_compute_apps_count=$($appsBefore.Count)"
Note "baseline_compute_apps_task_related=$([string]::Join('|', $taskBefore))"
if ($taskBefore.Count -gt 0) {
    Note 'ABORT: task-related compute process present'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 5
}
$desktop = Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First 5 |
    ForEach-Object { '{0}:{1}MiB' -f $_.ProcessName, [int]($_.WorkingSet64 / 1MB) }
Note "desktop_process_top5=$([string]::Join(', ', $desktop))"
Note "dumper_exe_sha256=$((Get-FileHash $dumper -Algorithm SHA256).Hash.ToLower())"
Note "guard_policy=hard=$hardMiB MiB or 50pct abort; soft=$softMiB MiB record-only; sample=0.25 s"

$sampler = Start-Job -ScriptBlock {
    param($stopFlag, $outFile)
    while (-not (Test-Path $stopFlag)) {
        $raw = & nvidia-smi --query-gpu=memory.used,utilization.gpu,temperature.gpu --format=csv,noheader
        $raw | Out-File -FilePath $outFile -Append -Encoding ASCII
        Start-Sleep -Milliseconds 250
    }
} -ArgumentList $stopFlag, $samplesTmp

$startTime = Get-Date
$peak = $pre.Used
$softExceeded = $false
$failedCase = $null
$caseResults = New-Object System.Collections.Generic.List[string]
$cases = 'sdf-box', 'ellipsoid-box'
$output = New-Object System.Collections.Generic.List[string]

foreach ($case in $cases) {
    Note "step=run-dumper case=$case"
    $caseStart = Get-Date
    $runOut = & $dumper $casesDir $prodOut $case 2>&1
    $code = $LASTEXITCODE
    $output.Add("=== case=$case dumper_exit=$code ===")
    foreach ($line in $runOut) { $output.Add([string]$line); Note ('dumper: ' + [string]$line) }
    if ($code -ne 0) { $failedCase = $case; Note "ABORT: dumper exit code $code"; break }
    $now = GpuMemory
    if ($now.Used -gt $peak) { $peak = $now.Used }
    $delta = $peak - $pre.Used
    Note "guard phase=after-$case used_mib=$($now.Used) peak_delta_mib=$delta"
    if ($delta -gt $softMiB) {
        $softExceeded = $true
        Note "soft_threshold_exceeded delta=$delta MiB > $softMiB MiB (record only, continue)"
    }
    if ($delta -ge $hardMiB -or $peak -gt [int]($pre.Total / 2)) {
        $failedCase = $case
        Note "ABORT: hard guard triggered (delta=$delta MiB / peak=$peak MiB)"
        break
    }
    Note "step=compare case=$case"
    $cmpOut = & $python (Join-Path $evidence 'qa_nvrtc_check.py') --numeric-only --case $case 2>&1
    $cmpCode = $LASTEXITCODE
    $output.Add("=== case=$case compare_exit=$cmpCode ===")
    foreach ($line in $cmpOut) { $output.Add([string]$line) }
    foreach ($line in $cmpOut) {
        if ([string]$line -match 'bit_equal|bit_total|status|structure_bit_total_recomputed') {
            Note ('compare: ' + [string]$line)
        }
    }
    if ($cmpCode -ne 0) {
        $failedCase = $case
        Note "STOP: case=$case not byte-identical to out1 (compare exit=$cmpCode); remaining cases not run"
        break
    }
    $caseResults.Add("${case}: bitwise identical to out1 (compare exit 0)")
    Note "case_result=pass case=$case wall_ms=$([int]((Get-Date) - $caseStart).TotalMilliseconds)"
}

$stopTime = Get-Date
New-Item -ItemType File -Path $stopFlag -Force | Out-Null
Wait-Job $sampler -Timeout 10 | Out-Null
Remove-Job $sampler -Force -ErrorAction SilentlyContinue
Remove-Item $stopFlag -ErrorAction SilentlyContinue
if (Test-Path $samplesTmp) { Move-Item $samplesTmp $samplesPath -Force } else { New-Item -ItemType File -Path $samplesPath -Force | Out-Null }
$sampleRows = @(Get-Content $samplesPath | Where-Object { $_.Trim() -ne '' })
$samplePeak = 0
foreach ($row in $sampleRows) {
    $first = ($row -split ',')[0]
    $v = [int]($first -replace '[^0-9]', '')
    if ($v -gt $samplePeak) { $samplePeak = $v }
}
$post = GpuMemory
$appsAfter = ComputeApps
$taskAfter = @($appsAfter | Where-Object { $_ -match 'qa-nvrtc|dumper' })
$residual = @(Get-Process -Name 'qa-nvrtc-dumper' -ErrorAction SilentlyContinue)
Note "after_memory=$($post.Raw)"
Note "after_compute_apps_count=$($appsAfter.Count)"
Note "after_compute_apps_task_related=$([string]::Join('|', $taskAfter))"
Note "residual_qa_dumper_processes=$($residual.Count)"
Note "after_used_mib=$($post.Used)"
Note "peak_used_mib_post_query=$peak"
Note "sample_peak_mib=$samplePeak"
Note "sample_peak_delta_mib=$($samplePeak - $pre.Used)"
Note "sample_count=$($sampleRows.Count)"
Note "soft_threshold_exceeded=$softExceeded"
Note "elapsed_seconds=$([math]::Round(($stopTime - $startTime).TotalSeconds, 1))"
foreach ($r in $caseResults) { Note "case_result=$r" }
if ($failedCase) { Note "window=failed case=$failedCase" } else { Note 'window=done' }
$dumperSha = (Get-FileHash $dumper -Algorithm SHA256).Hash.ToLower()
$header = @(
    "window_head=$head",
    "path_prepend=$cudaBin",
    "nvcc=$nvcc",
    "dumper_exe_sha256=$dumperSha",
    'command=qa-nvrtc-dumper <cases-dir> <out-dir> <case> (per-case, early stop)'
)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($outputPath, ($header + $output), $utf8NoBom)
$lines | Set-Content -Path $log -Encoding UTF8
if ($failedCase) { exit 6 }
exit 0
