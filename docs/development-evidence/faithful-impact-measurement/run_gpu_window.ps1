# Faithful-FMA impact measurement: GPU window runner (runs INSIDE the approved window).
# Pre/post nvidia-smi snapshots + query-compute-apps; 0.25 s memory/util/temp sampling
# while the tests run; one mode at a time; no other GPU work is started here.
# Usage: -Modes both|gnu-default|gnu-fma   (default both)
param(
    [ValidateSet('both', 'gnu-default', 'gnu-fma')]
    [string]$Modes = 'both'
)
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$tree = Join-Path $repo 'target\worktrees\faithful-5e000cb9'
$out = Join-Path $repo 'target\faithful'
$evidence = Join-Path $repo 'docs\development-evidence\faithful-impact-measurement'
$logs = Join-Path $evidence 'logs'
New-Item -ItemType Directory -Force -Path $logs | Out-Null
$stamp = (Get-Date).ToUniversalTime().ToString('yyyyMMdd-HHmmss')
$cudaPath = Join-Path $repo 'target\toolchains\cuda-12.8.1'

$tests = @(
    @{ target = 'g10_p5_nvrtc_launcher'; filter = 'product_nvrtc_launcher_matches_g5_probe_out1'; item = 'launcher regression (hfield-sphere 314 / sdf-box 3122 / ellipsoid-box 80)' },
    @{ target = 'g10_p5_primitive_cuda'; filter = 'primitive_launcher_matches_qa_upstream_gpu'; item = 'primitive 3 cases (80/80 x3)' },
    @{ target = 'g10_p5_multicontact_cuda'; filter = 'multicontact_launcher_matches_qa_upstream_gpu'; item = 'multicontact 3 cases (314/314, 391/392, 80/80)' },
    @{ target = 'render_kernel_depth_seg'; filter = 'depth_and_seg_match_frozen_reference_for_both_cameras'; item = 'S7b1 depth/seg (120,000/120,000)' },
    @{ target = 'render_kernel_rgb'; filter = 'rgb_matches_reference_and_depth_seg_do_not_regress'; item = 'S7b2 RGB (<=1 LSB; depth/seg no regression)' },
    @{ target = 'render_kernel_flags'; filter = 'flag_variants_match_own_references'; item = 'S7b3 flag variants' },
    @{ target = 'render_kernel_flags'; filter = 'aa2_matches_same_recipe_reference'; item = 'S7b3 AA spp=2' },
    @{ target = 'render_kernel_flags'; filter = 'baseline_no_regression_after_flags_and_aa'; item = 'S7b3 baseline no-regression' },
    @{ target = 'render_e2e'; filter = 'end_to_end_two_cameras_match_same_recipe_reference'; item = 'S8 e2e same-recipe' },
    @{ target = 'render_e2e'; filter = 'product_camlight_poses_match_reference_bits'; item = 'S8 camlight pose bits' }
)

function Gpu-Snapshot([string]$path) {
    & nvidia-smi --query-gpu=name,memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader *> $path
    "=== compute apps ===" | Out-File -Encoding utf8 -Append $path
    & nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader *>> $path
}

Gpu-Snapshot (Join-Path $logs ("gpu-window-{0}-pre.txt" -f $stamp))

$flagFile = Join-Path $out 'sampler.on'
Set-Content -Path $flagFile -Value 'on'
$stopFile = Join-Path $out 'sampler.stop'
Remove-Item $stopFile -Force -ErrorAction SilentlyContinue
$samplesFile = Join-Path $logs ("gpu-window-{0}-samples.txt" -f $stamp)
$cpuObsFile = Join-Path $logs ("gpu-window-{0}-cpu-process-observations.txt" -f $stamp)
$freeFloorMiB = 10240
$incrementLimitMiB = 3072
$startUsed = [int]((& nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits))
$baselinePids = @(& nvidia-smi --query-compute-apps=pid --format=csv,noheader | ForEach-Object { $_.Trim() } | Where-Object { $_ -match '^\d+$' })
$sampler = Start-Job -ArgumentList $flagFile, $stopFile, $samplesFile, $cpuObsFile, $freeFloorMiB, $incrementLimitMiB, $startUsed, $baselinePids -ScriptBlock {
    param($flagFile, $stopFile, $samplesFile, $cpuObsFile, $freeFloorMiB, $incrementLimitMiB, $startUsed, $baselinePids)
    while (Test-Path $flagFile) {
        $now = (Get-Date).ToUniversalTime().ToString('HH:mm:ss.fff')
        $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader,nounits) -join ';'
        Add-Content -Path $samplesFile -Value "$now,$line" -Encoding utf8
        $parts = ($line -split ',')
        if ($parts.Count -ge 4) {
            $used = [int]$parts[0]
            $free = [int]$parts[1]
            if ($free -lt $freeFloorMiB) { Set-Content -Path $stopFile -Value "free_mib_lt_$freeFloorMiB"; break }
            if (($used - $startUsed) -ge $incrementLimitMiB) { Set-Content -Path $stopFile -Value "used_increment_ge_$incrementLimitMiB"; break }
        }
        # refined guard: only a NEW CUDA compute process that is not this task's test exe aborts.
        $pids = @(& nvidia-smi --query-compute-apps=pid --format=csv,noheader | ForEach-Object { $_.Trim() } | Where-Object { $_ -match '^\d+$' })
        foreach ($appPid in $pids) {
            if ($baselinePids -contains $appPid) { continue }
            $proc = Get-Process -Id ([int]$appPid) -ErrorAction SilentlyContinue
            $name = if ($proc) { $proc.ProcessName } else { 'unknown' }
            if ($name -match '^(g10_p5_|render_)') { continue }
            Add-Content -Path $cpuObsFile -Value "$now new_cuda_compute pid=$appPid name=$name -> ABORT" -Encoding utf8
            Set-Content -Path $stopFile -Value "new_cuda_compute_pid_${appPid}_$name"
            break
        }
        if (Test-Path $stopFile) { break }
        # CPU-only processes are recorded but never abort the window.
        $cpuProcs = @(Get-Process | Where-Object { $_.ProcessName -match 'mjwarp|rustc|cargo|g10_p5|render_' -and $_.ProcessName -notmatch '^(g10_p5_|render_)' })
        foreach ($proc in $cpuProcs) {
            Add-Content -Path $cpuObsFile -Value "$now cpu_only pid=$($proc.Id) name=$($proc.ProcessName) (recorded, not aborting)" -Encoding utf8
        }
        Start-Sleep -Milliseconds 250
    }
}

$modesToRun = @()
if ($Modes -eq 'both') { $modesToRun = @('gnu-default', 'gnu-fma') } else { $modesToRun = @($Modes) }
$windowStart = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
$results = @()
$guardTripped = ''
foreach ($mode in $modesToRun) {
    if (Test-Path $stopFile) { $guardTripped = (Get-Content $stopFile -Raw).Trim(); break }
    $env:CARGO_TARGET_DIR = Join-Path $out ("{0}-cuda" -f $mode)
    if ($mode -eq 'gnu-fma') { $env:RUSTFLAGS = '-C target-feature=+fma' } else { Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue }
    $env:CUDA_PATH = $cudaPath
    $env:PATH = (Join-Path $cudaPath 'bin') + ';' + $env:PATH
    foreach ($test in $tests) {
        if (Test-Path $stopFile) { $guardTripped = (Get-Content $stopFile -Raw).Trim(); break }
        $log = Join-Path $logs ("gpu-{0}-{1}.log" -f $mode, $test.target)
        Push-Location $tree
        $argv = @('test', '--locked', '--features', 'cuda-probe', '--test', $test.target, '--', '--ignored', '--test-threads=1', '--nocapture', $test.filter)
        $sw = [Diagnostics.Stopwatch]::StartNew()
        & cargo @argv *> $log
        $exit = $LASTEXITCODE
        $sw.Stop()
        Pop-Location
        $keyLines = @(Select-String -Path $log -Pattern 'bit_equal|bit-equal|120,000|120000|s7b|s8\[|LSB|max_|mismatch|case=|frame|FAILED|panicked|test result' -ErrorAction SilentlyContinue | ForEach-Object { $_.Line.Trim() })
        $results += [ordered]@{
            mode = $mode
            item = $test.item
            target = $test.target
            filter = $test.filter
            exit = $exit
            wall_s = [math]::Round($sw.Elapsed.TotalSeconds, 1)
            log = ("docs/development-evidence/faithful-impact-measurement/logs/gpu-{0}-{1}.log" -f $mode, $test.target)
            key_lines = $keyLines
        }
        Write-Host ("{0} / {1}: exit={2} wall={3}s" -f $mode, $test.target, $exit, [math]::Round($sw.Elapsed.TotalSeconds, 1))
    }
    # env is cleared only after ALL tests of a mode (keeps CARGO_TARGET_DIR/RUSTFLAGS stable per mode)
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
}
$windowEnd = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
Remove-Item $flagFile -Force -ErrorAction SilentlyContinue
Wait-Job $sampler -Timeout 30 | Out-Null
Remove-Job $sampler -Force -ErrorAction SilentlyContinue
Gpu-Snapshot (Join-Path $logs ("gpu-window-{0}-post.txt" -f $stamp))

$samples = @()
if (Test-Path $samplesFile) {
    $samples = @(Get-Content $samplesFile | Where-Object { $_ -match '^\d{2}:\d{2}:\d{2}' })
}
$used = @(); $free = @(); $util = @(); $temp = @()
foreach ($s in $samples) {
    $parts = ($s -split ',')
    if ($parts.Count -ge 5) {
        $used += [int]$parts[1]; $free += [int]$parts[2]; $util += [int]$parts[3]; $temp += [int]$parts[4]
    }
}
$summary = [ordered]@{
    window_start_utc = $windowStart
    window_end_utc = $windowEnd
    modes = $modesToRun
    tests_per_mode = $tests.Count
    guard_tripped = $guardTripped
    guard_rules = "abort: new CUDA compute pid (not this task's g10_p5_*/render_* test exe) OR free<10240 MiB OR used increment>=3072 MiB; CPU-only processes recorded only"
    cpu_process_observations = ("docs/development-evidence/faithful-impact-measurement/logs/gpu-window-{0}-cpu-process-observations.txt" -f $stamp)
    baseline_compute_app_pids = $baselinePids.Count
    start_used_mib = $startUsed
    samples = [ordered]@{
        count = $samples.Count
        interval_ms = 250
        used_mib_max = if ($used.Count) { ($used | Measure-Object -Maximum).Maximum } else { $null }
        used_mib_min = if ($used.Count) { ($used | Measure-Object -Minimum).Minimum } else { $null }
        free_mib_min = if ($free.Count) { ($free | Measure-Object -Minimum).Minimum } else { $null }
        util_max = if ($util.Count) { ($util | Measure-Object -Maximum).Maximum } else { $null }
        temp_max = if ($temp.Count) { ($temp | Measure-Object -Maximum).Maximum } else { $null }
        file = ("docs/development-evidence/faithful-impact-measurement/logs/gpu-window-{0}-samples.txt" -f $stamp)
    }
    results = $results
}
$summary | ConvertTo-Json -Depth 6 | Out-File -Encoding utf8 (Join-Path $logs ("gpu-window-{0}-summary.json" -f $stamp))
Write-Host 'GPU_WINDOW_DONE'
