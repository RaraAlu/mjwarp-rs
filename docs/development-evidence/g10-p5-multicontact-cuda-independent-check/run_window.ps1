# G10@P5 multicontact QA window.
# Conditions: HEAD bound to expected prefix; product commit pinned (docs-only diff allowed);
# prebuilt QA dumper only (no compilation in window); baseline recheck; PATH prepend;
# 0.25 s sampling; hard guard 3072 MiB / 50% (abort); soft threshold +1536 MiB record-only;
# per-case GPU kernel run + QA's own python comparator (early stop on failure).
# NOTE: keep this file ASCII-only (PowerShell 5.1 reads .ps1 as ANSI).

param(
    [string]$Repo = 'D:\Rust\mjwarp-rs',
    [int]$ExpectedBaseline = 3641,
    [string]$ExpectedHead = '4c7be3c6',
    [string]$ProductHead = '9a282074'
)
$ErrorActionPreference = 'Stop'
$evidence = Join-Path $Repo 'docs\development-evidence\g10-p5-multicontact-cuda-independent-check'
$log = Join-Path $evidence 'window-log.txt'
$samplesPath = Join-Path $evidence 'window-memory-samples.txt'
$outputPath = Join-Path $evidence 'window-output.txt'
$stopFlag = Join-Path $env:TEMP 'qa-multicontact-window-stop.flag'
$samplesTmp = Join-Path $env:TEMP 'qa-multicontact-window-samples.txt'
$dumper = Join-Path $Repo 'target\qa-multicontact\dumper-target\debug\qa-multicontact-dumper.exe'
$casesDir = Join-Path $Repo 'docs\development-evidence\g10-p5-gpu-kernels-independent-check\qa-gpu\cases'
$prodOut = Join-Path $Repo 'target\qa-multicontact\prod-out'
$python = Join-Path $Repo 'target\mjwarp-parallel\venv\Scripts\python.exe'
$compare = Join-Path $evidence 'qa_multicontact_check.py'
$compareLauncher = Join-Path $evidence 'qa_launcher_regression_check.py'
$cudaBin = Join-Path $Repo 'target\toolchains\cuda-12.8.1\bin'
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
    $raw = ((& nvidia-smi --query-gpu=memory.used,memory.total,memory.free,utilization.gpu,temperature.gpu,driver_version --format=csv,noheader) -join '')
    $parts = $raw -split ',\s*'
    return @{
        Used = [int]($parts[0] -replace '[^0-9]', '')
        Total = [int]($parts[1] -replace '[^0-9]', '')
        Free = [int]($parts[2] -replace '[^0-9]', '')
        Raw = $raw
    }
}

function ComputeApps {
    return @(& nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>$null)
}

$env:PATH = $cudaBin + ';' + $env:PATH
$multicontactCases = @('cylinder-box', 'box-box-margin', 'mesh-mesh')
$launcherCases = @('hfield-sphere', 'sdf-box', 'ellipsoid-box')
$caseList = @($multicontactCases + $launcherCases)
Note "window=start task=multicontact-independent-check cases=$($caseList -join ',')"
$head = (& git -C $Repo rev-parse HEAD).Trim()
Note "head=$head expect_prefix=$ExpectedHead"
if (-not $head.StartsWith($ExpectedHead)) {
    Note 'ABORT: HEAD does not match approved condition'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 3
}
Note "product_head=$ProductHead (HEAD may only differ from it by docs/evidence/.normify changes)"
$codeDiff = @(& git -C $Repo diff "$ProductHead..$head" --stat)
foreach ($line in $codeDiff) { Note "diff_stat=$line" }
$productDiff = @(& git -C $Repo diff "$ProductHead..$head" --stat -- src tests Cargo.toml Cargo.lock fixtures)
if ($productDiff.Count -gt 0) {
    foreach ($line in $productDiff) { Note "ABORT_DIFF=$line" }
    Note 'ABORT: HEAD differs from product HEAD in src/tests/Cargo/fixtures'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 8
}
Note 'product_diff_empty=true (src/tests/Cargo.toml/Cargo.lock/fixtures)'
$worktree = @(& git -C $Repo status --porcelain -- src tests Cargo.toml Cargo.lock fixtures)
Note "worktree_product_changes_count=$($worktree.Count)"
if ($worktree.Count -gt 0) {
    foreach ($line in $worktree) { Note "ABORT_WORKTREE=$line" }
    Note 'ABORT: product worktree is not clean'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 9
}
Note "path_prepend=$cudaBin"
$nvcc = (& nvcc --version) | Select-Object -Last 1
Note "nvcc=$nvcc"
foreach ($dll in 'nvrtc64_120_0.dll', 'nvrtc64_120_0.alt.dll', 'nvrtc-builtins64_128.dll') {
    $p = Join-Path $cudaBin $dll
    Note "nvrtc_dll_sha256=$dll=$((Get-FileHash $p -Algorithm SHA256).Hash.ToLower())"
}
if (-not (Test-Path $dumper)) {
    Note "ABORT: prebuilt dumper missing: $dumper"
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 7
}
$dumperItem = Get-Item $dumper
Note "dumper_exe=$dumper"
Note "dumper_exe_sha256=$((Get-FileHash $dumper -Algorithm SHA256).Hash.ToLower())"
Note "dumper_built_at=$($dumperItem.LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss')) (prebuilt; no compilation in window)"
Note "cases_dir=$casesDir"

$pre = GpuMemory
Note "baseline_memory=$($pre.Raw)"
Note "baseline_used_mib=$($pre.Used)"
$drift = [math]::Abs($pre.Used - $ExpectedBaseline)
Note "baseline_drift_mib=$drift expected=$ExpectedBaseline limit=200"
if ($drift -gt 200) {
    Note 'ABORT: baseline drift > 200 MiB'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 4
}
$appsBefore = ComputeApps
$taskBefore = @($appsBefore | Where-Object { $_ -match 'qa-multicontact|dumper|probe|python|cargo|rustc|nvcc' })
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
Note "guard_policy=hard=$hardMiB MiB or 50pct abort; soft=$softMiB MiB record-only; sample=0.25 s"
Note "stop_policy_running=free<10240 MiB or task-related compute process -> abort; hard guard increment>=3072 MiB"

$sampler = Start-Job -ScriptBlock {
    param($stopFlag, $outFile)
    while (-not (Test-Path $stopFlag)) {
        $raw = & nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader
        $raw | Out-File -FilePath $outFile -Append -Encoding ASCII
        Start-Sleep -Milliseconds 250
    }
} -ArgumentList $stopFlag, $samplesTmp

$startTime = Get-Date
$peak = $pre.Used
$softExceeded = $false
$failedCase = $null
$caseResults = New-Object System.Collections.Generic.List[string]
$output = New-Object System.Collections.Generic.List[string]

foreach ($case in $caseList) {
    $isMulticontact = $multicontactCases -contains $case
    $isLauncher = $launcherCases -contains $case
    if (-not ($isMulticontact -or $isLauncher)) {
        $failedCase = $case
        Note "ABORT: unknown case $case"
        break
    }
    $phase = if ($isMulticontact) { 'multicontact' } else { 'launcher-regression' }
    Note "step=run-dumper case=$case phase=$phase"
    $caseStart = Get-Date
    if ($isMulticontact) {
        $kernel = if ($case -eq 'cylinder-box') { 'contact_multi' } elseif ($case -eq 'box-box-margin') { 'boxbox' } else { 'contact' }
        Note "kernel_override=$kernel"
        $runOut = & $dumper $casesDir $prodOut $case $kernel 2>&1
    } else {
        $runOut = & $dumper $casesDir $prodOut $case 2>&1
    }
    $code = $LASTEXITCODE
    $output.Add("=== case=$case dumper_exit=$code ===")
    foreach ($line in $runOut) { $output.Add([string]$line); Note ('dumper: ' + [string]$line) }
    if ($code -ne 0) { $failedCase = $case; Note "ABORT: dumper exit code $code"; break }
    $now = GpuMemory
    if ($now.Used -gt $peak) { $peak = $now.Used }
    $delta = $peak - $pre.Used
    Note "guard phase=after-$case used_mib=$($now.Used) free_mib=$($now.Free) peak_delta_mib=$delta"
    if ($now.Free -lt 10240) {
        $failedCase = $case
        Note "ABORT: free < 10240 MiB (free=$($now.Free))"
        break
    }
    $appsMid = ComputeApps
    $taskMid = @($appsMid | Where-Object { $_ -match 'qa-multicontact|dumper' })
    if ($taskMid.Count -gt 0) {
        $failedCase = $case
        Note "ABORT: task-related compute process present mid-window: $([string]::Join('|', $taskMid))"
        break
    }
    Note "free_check phase=after-$case free_mib=$($now.Free) threshold_mib=10240 ok=true"
    if ($delta -gt $softMiB) {
        $softExceeded = $true
        Note "soft_threshold_exceeded delta=$delta MiB > $softMiB MiB (record only, continue)"
    }
    if ($delta -ge $hardMiB -or $peak -gt [int]($pre.Total / 2)) {
        $failedCase = $case
        Note "ABORT: hard guard triggered (delta=$delta MiB / peak=$peak MiB)"
        break
    }
    Note "step=compare case=$case phase=$phase"
    if ($isMulticontact) {
        $cmpOut = & $python $compare --numeric-only --case $case --dumps $prodOut 2>&1
    } else {
        $cmpOut = & $python $compareLauncher --numeric-only --case $case --dumps $prodOut 2>&1
    }
    $cmpCode = $LASTEXITCODE
    $output.Add("=== case=$case compare_exit=$cmpCode ===")
    foreach ($line in $cmpOut) { $output.Add([string]$line) }
    foreach ($line in $cmpOut) {
        if ([string]$line -match 'bit_equal|failures|FAIL') { Note ('compare: ' + [string]$line) }
    }
    if ($cmpCode -ne 0) {
        $failedCase = $case
        Note "STOP: case=$case not byte-identical to reference (compare exit=$cmpCode); remaining cases not run"
        break
    }
    if ($isMulticontact) {
        $caseResults.Add("${case}: compared vs qa-upstream-gpu.json (compare exit 0)")
    } else {
        $caseResults.Add("${case}: bitwise identical to G5 probe out1 (launcher regression, compare exit 0)")
    }
    Note "case_result=pass case=$case phase=$phase wall_ms=$([int]((Get-Date) - $caseStart).TotalMilliseconds)"
}

$stopTime = Get-Date
New-Item -ItemType File -Path $stopFlag -Force | Out-Null
Wait-Job $sampler -Timeout 10 | Out-Null
Remove-Job $sampler -Force -ErrorAction SilentlyContinue
Remove-Item $stopFlag -ErrorAction SilentlyContinue
if (Test-Path $samplesTmp) { Move-Item $samplesTmp $samplesPath -Force } else { New-Item -ItemType File -Path $samplesPath -Force | Out-Null }
$sampleRows = @(Get-Content $samplesPath | Where-Object { $_.Trim() -ne '' })
$samplePeak = 0
$minFree = [int]::MaxValue
foreach ($row in $sampleRows) {
    $parts = $row -split ',\s*'
    $used = [int]($parts[0] -replace '[^0-9]', '')
    $free = [int]($parts[1] -replace '[^0-9]', '')
    if ($used -gt $samplePeak) { $samplePeak = $used }
    if ($free -lt $minFree) { $minFree = $free }
}
$post = GpuMemory
$appsAfter = ComputeApps
$taskAfter = @($appsAfter | Where-Object { $_ -match 'qa-multicontact|dumper' })
$residual = @(Get-Process -Name 'qa-multicontact-dumper' -ErrorAction SilentlyContinue)
Note "after_memory=$($post.Raw)"
Note "after_compute_apps_count=$($appsAfter.Count)"
Note "after_compute_apps_task_related=$([string]::Join('|', $taskAfter))"
Note "residual_qa_dumper_processes=$($residual.Count)"
Note "after_used_mib=$($post.Used)"
Note "peak_used_mib_post_query=$peak"
Note "sample_peak_mib=$samplePeak"
Note "sample_peak_delta_mib=$($samplePeak - $pre.Used)"
Note "sample_min_free_mib=$minFree"
Note "sample_count=$($sampleRows.Count)"
Note "soft_threshold_exceeded=$softExceeded"
Note "elapsed_seconds=$([math]::Round(($stopTime - $startTime).TotalSeconds, 1))"
foreach ($r in $caseResults) { Note "case_result=$r" }
if ($failedCase) { Note "window=failed case=$failedCase" } else { Note 'window=done' }
$dumperSha = (Get-FileHash $dumper -Algorithm SHA256).Hash.ToLower()
$header = @(
    "window_head=$head",
    "product_head=$ProductHead",
    "path_prepend=$cudaBin",
    "nvcc=$nvcc",
    "dumper_exe_sha256=$dumperSha",
    "reference_multicontact=docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json (QA committed upstream f32 GPU capture)",
    "reference_launcher=docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/<case>/out1 (G5 probe output)",
    'command=qa-multicontact-dumper <cases-dir> <out-dir> <case> [kernel] (per-case process, early stop)'
)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($outputPath, ($header + $output), $utf8NoBom)
$lines | Set-Content -Path $log -Encoding UTF8
if ($failedCase) { exit 6 }
exit 0
