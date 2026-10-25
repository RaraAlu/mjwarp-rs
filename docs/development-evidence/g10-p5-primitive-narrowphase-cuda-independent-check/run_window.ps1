# G10@P5 primitive narrowphase QA window.
# Conditions: HEAD 9d72159c; prebuilt QA dumper only (no compilation in window);
# baseline recheck (drift <= 200 MiB, no task-related compute process); PATH prepend;
# 0.25 s sampling; hard guard 3072 MiB / 50% (abort); soft threshold +1536 MiB record-only;
# per-case GPU kernel run + QA's own python comparator (early stop on failure).
# NOTE: keep this file ASCII-only (PowerShell 5.1 reads .ps1 as ANSI).

param(
    [string]$Repo = 'D:\Rust\mjwarp-rs',
    [string]$Cases = 'sphere-box-shallow,sphere-box-deep,capsule-box,hfield-sphere,sdf-box,ellipsoid-box',
    [int]$ExpectedBaseline = 3693,
    [string]$ExpectedHead = '6e2bd529',
    [string]$ProductHead = '9d72159c'
)
$ErrorActionPreference = 'Stop'
$evidence = Join-Path $Repo 'docs\development-evidence\g10-p5-primitive-narrowphase-cuda-independent-check'
$log = Join-Path $evidence 'window-log.txt'
$samplesPath = Join-Path $evidence 'window-memory-samples.txt'
$outputPath = Join-Path $evidence 'window-output.txt'
$stopFlag = Join-Path $env:TEMP 'qa-primitive-window-stop.flag'
$samplesTmp = Join-Path $env:TEMP 'qa-primitive-window-samples.txt'
$dumper = Join-Path $Repo 'target\qa-primitive\dumper-target\debug\qa-primitive-dumper.exe'
$casesDir = Join-Path $Repo 'docs\development-evidence\g10-p5-gpu-kernels-independent-check\qa-gpu\cases'
$prodOut = Join-Path $Repo 'target\qa-primitive\prod-out'
$python = Join-Path $Repo 'target\mjwarp-parallel\venv\Scripts\python.exe'
$compare = Join-Path $evidence 'qa_primitive_check.py'
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
$caseList = @($Cases -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' })
Note "window=start task=primitive-narrowphase-independent-check cases=$($caseList -join ',')"
$head = (& git -C $Repo rev-parse HEAD).Trim()
Note "head=$head expect_prefix=$ExpectedHead"
if (-not $head.StartsWith($ExpectedHead)) {
    Note 'ABORT: HEAD does not match approved condition'
    $lines | Set-Content -Path $log -Encoding UTF8
    exit 3
}
Note "path_prepend=$cudaBin"
Note "product_head=$ProductHead (HEAD may only differ from it by docs/evidence text changes)"
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
foreach ($case in $caseList) {
    $metaPath = Join-Path (Join-Path $casesDir $case) 'meta.txt'
    Note "case_meta=$case kind=$(((Select-String -Path $metaPath -Pattern '^kind ').Line) -replace '^kind ', '')"
}

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
$taskBefore = @($appsBefore | Where-Object { $_ -match 'qa-primitive|dumper|probe|python|cargo|rustc|nvcc' })
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
$output = New-Object System.Collections.Generic.List[string]

$primitiveCases = @('sphere-box-shallow', 'sphere-box-deep', 'capsule-box')
$launcherCases = @('hfield-sphere', 'sdf-box', 'ellipsoid-box')
$compareLauncher = Join-Path $evidence 'qa_launcher_regression_check.py'

foreach ($case in $caseList) {
    $isPrimitive = $primitiveCases -contains $case
    $isLauncher = $launcherCases -contains $case
    if (-not ($isPrimitive -or $isLauncher)) {
        $failedCase = $case
        Note "ABORT: unknown case $case (only primitive 3 or launcher 3 allowed)"
        break
    }
    $phase = if ($isPrimitive) { 'primitive' } else { 'launcher-regression' }
    Note "step=run-dumper case=$case phase=$phase"
    $caseStart = Get-Date
    if ($isPrimitive) {
        $runOut = & $dumper $casesDir $prodOut $case primitive 2>&1
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
    Note "step=compare case=$case phase=$phase"
    if ($isPrimitive) {
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
        Note "STOP: case=$case not byte-identical to upstream reference (compare exit=$cmpCode); remaining cases not run"
        break
    }
    if ($isPrimitive) {
        $caseResults.Add("${case}: bitwise identical to qa-upstream-gpu.json (compare exit 0)")
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
foreach ($row in $sampleRows) {
    $first = ($row -split ',')[0]
    $v = [int]($first -replace '[^0-9]', '')
    if ($v -gt $samplePeak) { $samplePeak = $v }
}
$post = GpuMemory
$appsAfter = ComputeApps
$taskAfter = @($appsAfter | Where-Object { $_ -match 'qa-primitive|dumper' })
$residual = @(Get-Process -Name 'qa-primitive-dumper' -ErrorAction SilentlyContinue)
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
    "reference_primitive=docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json (QA committed upstream f32 GPU capture)",
    "reference_launcher=docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/<case>/out1 (G5 probe output)",
    'command=qa-primitive-dumper <cases-dir> <out-dir> <case> [primitive] (per-case process, early stop)'
)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($outputPath, ($header + $output), $utf8NoBom)
$lines | Set-Content -Path $log -Encoding UTF8
if ($failedCase) { exit 6 }
exit 0
