# R05-S3b hfield validation-window runner (ASCII-only on purpose: PS 5.1 reads .ps1 as ANSI).
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/run-window.ps1 -Attempt 1
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/run-window.ps1 -GuardsOnly
# Discipline:
# - single hard gate: guard precheck (73 frozen sha256 entries) -> any mismatch = invalid round, exit 9
# - 0.25 s GPU sampler runs for the whole entity body
# - entities run the exact entity-table command: cargo test --locked --features cuda-probe --test <name> -- --ignored --test-threads=1 --nocapture
# - zero Rust compile inside the window: deps snapshot before/after must be identical, else exit 7
# - no authoring exception here: thresholds are the original ones (free >= 10240, util < 40)
param(
    [int]$Attempt = 1,
    [string]$RepoRoot = 'D:\Rust\mjwarp-rs',
    [string]$SuiteDir = 'scripts\validate\render-bvh',
    [string]$OutDir = 'target\render-bvh\r05-s3b-window\attempt1',
    [string]$CudaBin = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin',
    [int]$OpenFreeMiB = 10240,
    [int]$OpenUtilLimit = 40,
    [int]$AbortUtilLimit = 80,
    [int]$AbortUtilStreak = 3,
    [int]$BudgetMiB = 3072,
    [switch]$GuardsOnly
)
$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)
$out = Join-Path $RepoRoot $OutDir
$logs = Join-Path $out 'logs'
New-Item -ItemType Directory -Force $logs | Out-Null
$factsPath = Join-Path $out 'facts.txt'
$shaPath = Join-Path $out 'sha-guards.txt'
$samplesPath = Join-Path $out 'samples.csv'
$entityPath = Join-Path $out 'entity-results.csv'
$guardLog = Join-Path $out 'guard.log'
$abortFile = Join-Path $out '.abort'
$stopFile = Join-Path $out '.stop-sampler'
foreach ($p in @($factsPath, $shaPath, $samplesPath, $entityPath, $guardLog, $abortFile, $stopFile)) {
    Remove-Item -Force $p -ErrorAction SilentlyContinue
}
function Add-Txt([string]$path, [string]$text) { [IO.File]::AppendAllText($path, $text + "`n", $utf8) }
function Fact([string]$text) { Add-Txt $factsPath $text }
function Guard([string]$text) { Add-Txt $guardLog $text; Write-Output $text }
function Iso { (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffffffK') }
function GpuSample {
    $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader)
    if (-not $line) { throw 'nvidia-smi returned nothing' }
    $parts = $line -split ',\s*'
    [pscustomobject]@{
        used = [int]($parts[0] -replace ' MiB', '')
        free = [int]($parts[1] -replace ' MiB', '')
        util = [int]($parts[2] -replace ' %', '')
        temp = [int]$parts[3]
    }
}
function ComputeApps {
    $raw = @(& nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader)
    $pattern = 'python|g10_p5_|product_collision|stage2_equivalence|cargo|rustc|nvcc|cl\.exe|render_'
    $filtered = @($raw | Where-Object { $_ -and ($_ -match $pattern) })
    [pscustomobject]@{ raw = $raw; filtered = $filtered }
}
function DepsSnapshot {
    $deps = Join-Path $RepoRoot 'target\debug\deps'
    $exes = @(Get-ChildItem $deps -Filter '*.exe' -ErrorAction SilentlyContinue)
    $max = if ($exes.Count -gt 0) { ($exes | Sort-Object LastWriteTime | Select-Object -Last 1).LastWriteTime } else { $null }
    [pscustomobject]@{ count = $exes.Count; max = $max }
}
function Start-Sampler {
    Start-Job -Name r05s3bsampler -ArgumentList $samplesPath, $stopFile, $abortFile, $AbortUtilLimit, $AbortUtilStreak -ScriptBlock {
        param($csv, $stop, $abort, $limit, $streakLimit)
        $u = New-Object System.Text.UTF8Encoding($false)
        [IO.File]::AppendAllText($csv, "seq,timestamp,used_mib,free_mib,util_percent,temp_c`n", $u)
        [IO.File]::WriteAllText($abort, '', $u)
        $seq = 0
        $streak = 0
        while (-not (Test-Path $stop)) {
            $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader) 2>$null
            if ($line) {
                $parts = $line -split ',\s*'
                $seq++
                $ts = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffffffK')
                $used = [int]($parts[0] -replace ' MiB', '')
                $free = [int]($parts[1] -replace ' MiB', '')
                $util = [int]($parts[2] -replace ' %', '')
                [IO.File]::AppendAllText($csv, "$seq,$ts,$used,$free,$util,$($parts[3])`n", $u)
                if ($util -ge $limit) { $streak++ } else { $streak = 0 }
                if ($streak -ge $streakLimit) {
                    [IO.File]::WriteAllText($abort, "util_streak:$seq", $u)
                    break
                }
            }
            Start-Sleep -Milliseconds 250
        }
    }
}
function Stop-Sampler {
    [IO.File]::WriteAllText($stopFile, 'stop', $utf8)
    if (Get-Job -Name r05s3bsampler -ErrorAction SilentlyContinue) {
        Wait-Job -Name r05s3bsampler -Timeout 30 | Out-Null
        Receive-Job -Name r05s3bsampler | Out-Null
        Remove-Job -Name r05s3bsampler -Force
    }
}
if (-not $GuardsOnly) {
    $violations = @()
    $build = @(Get-Process cargo, rustc, cl, nvcc, link -ErrorAction SilentlyContinue)
    if ($build.Count -gt 0) { $violations += 'rust_build_processes=' + (($build | ForEach-Object ProcessName) -join ',') }
    $ca = ComputeApps
    if ($ca.filtered.Count -gt 0) { $violations += 'filtered_compute_apps=' + ($ca.filtered -join ';') }
    $openSamples = @()
    for ($i = 0; $i -lt 3; $i++) {
        $openSamples += GpuSample
        if ($i -lt 2) { Start-Sleep -Milliseconds 250 }
    }
    foreach ($s in $openSamples) {
        if ($s.free -lt $OpenFreeMiB) { $violations += "open_free=$($s.free)<$OpenFreeMiB" }
        if ($s.util -ge $OpenUtilLimit) { $violations += "open_util=$($s.util)>=${OpenUtilLimit}" }
    }
    if ($violations.Count -gt 0) {
        Guard ('PRECHECK_ABORT ' + ($violations -join ' | '))
        Fact ('attempt=' + $Attempt)
        Fact 'device_validation_window=true'
        Fact 'authoring_exception_used=false'
        Fact ('precheck_abort ' + ($violations -join ' | '))
        Guard 'INVALID_ROUND exit9'
        exit 9
    }
    Guard ("PRECHECK_OK open=" + (($openSamples | ForEach-Object { "$($_.used)/$($_.free)/$($_.util)/$($_.temp)" }) -join ';'))
}
$baseline = @()
for ($i = 0; $i -lt 3; $i++) {
    $baseline += GpuSample
    if ($i -lt 2) { Start-Sleep -Milliseconds 250 }
}
$baselineFree = $baseline[1].free
$floor = [Math]::Min($OpenFreeMiB, $baselineFree - $BudgetMiB)
[IO.File]::WriteAllText((Join-Path $out 'baseline.txt'), ((($baseline | ForEach-Object { "used_mib=$($_.used) free_mib=$($_.free) util_percent=$($_.util) temp_c=$($_.temp)" }) -join "`n") + "`n"), $utf8)
$ca = ComputeApps
$cpuLoad = (Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
$machineLines = @()
$machineLines += "cpu_load_percent=$cpuLoad"
$machineLines += "build_processes=" + ((@(Get-Process cargo, rustc, cl, nvcc, link -ErrorAction SilentlyContinue) | ForEach-Object ProcessName) -join ',')
$machineLines += "compute_apps_raw=$($ca.raw.Count)"
$machineLines += "compute_apps_filtered=$($ca.filtered.Count)"
$machineLines += 'compute_apps_lines:'
$machineLines += @($ca.raw)
[IO.File]::WriteAllText((Join-Path $out 'machine-load.txt'), (($machineLines -join "`n") + "`n"), $utf8)
Fact ('attempt=' + $Attempt)
Fact 'device_validation_window=true'
Fact 'authoring_exception_used=false'
Fact ('window_start=' + (Iso))
Fact ('baseline_used,free,util,temp=' + (($baseline | ForEach-Object { "$($_.used),$($_.free),$($_.util),$($_.temp)" }) -join ' / '))
Fact "baseline_free_mib=$baselineFree"
Fact "floor_mib=$floor"
Fact "budget_increment_limit_mib=$BudgetMiB"
Fact 'feature_profile=cuda-probe'
Fact ('env_cuda_bin=' + $CudaBin)
$guardPath = Join-Path $RepoRoot (Join-Path $SuiteDir 'guards.json')
$guard = ([IO.File]::ReadAllText($guardPath, [Text.Encoding]::UTF8)) | ConvertFrom-Json
Fact ('guards_suite=' + $guard.suite)
Fact ('guards_aggregate_sha256=' + $guard.aggregate_sha256)
$guardOk = 0
$guardBad = 0
foreach ($exe in $guard.frozen_exes) {
    $path = Join-Path $RepoRoot $exe.path
    $hash = if (Test-Path $path) { (Get-FileHash $path -Algorithm SHA256).Hash.ToLower() } else { 'MISSING' }
    $bytes = if (Test-Path $path) { (Get-Item $path).Length } else { -1 }
    $ok = ($hash -eq $exe.sha256) -and ($bytes -eq $exe.bytes)
    if ($ok) { $guardOk++ } else { $guardBad++; Guard "SHA_MISMATCH $($exe.path) expect=$($exe.sha256)/$($exe.bytes) actual=$hash/$bytes" }
    Add-Txt $shaPath ("SHA_OK {0} {1} {2}" -f $hash, $bytes, $exe.path)
}
foreach ($input in $guard.guard_inputs) {
    $path = Join-Path $RepoRoot $input.path
    $hash = if (Test-Path $path) { (Get-FileHash $path -Algorithm SHA256).Hash.ToLower() } else { 'MISSING' }
    $bytes = if (Test-Path $path) { (Get-Item $path).Length } else { -1 }
    $ok = ($hash -eq $input.sha256) -and ($bytes -eq $input.bytes)
    if ($ok) { $guardOk++ } else { $guardBad++; Guard "SHA_MISMATCH $($input.path) expect=$($input.sha256)/$($input.bytes) actual=$hash/$bytes" }
    Add-Txt $shaPath ("SHA_OK {0} {1} {2}" -f $hash, $bytes, $input.path)
}
$guardTotal = $guard.frozen_exes.Count + $guard.guard_inputs.Count
Fact "sha_guards=$guardOk/$guardTotal ($($guard.frozen_exes.Count) exes + $($guard.guard_inputs.Count) read-only inputs), mismatches=$guardBad"
Guard "GUARDS_OK=$guardOk/$guardTotal"
if ($guardBad -gt 0) {
    Fact 'guard_aborted=true'
    Fact ('window_end=' + (Iso))
    Guard 'INVALID_ROUND exit9'
    exit 9
}
if ($GuardsOnly) {
    Fact 'guards_only=true'
    Fact "sha_guards=$guardOk/$guardTotal"
    Guard 'GUARDS_ONLY_DONE'
    exit 0
}
$depsBefore = DepsSnapshot
Fact "deps_before_count=$($depsBefore.count)"
Fact "deps_before_max_write=$($depsBefore.max)"
$sampler = Start-Sampler
$entityStart = Iso
$bodyWatch = [Diagnostics.Stopwatch]::StartNew()
$oldPath = $env:PATH
$env:PATH = "$CudaBin;" + $env:PATH
$failures = @()
Add-Txt $entityPath 'entity,kind,exit,wall_ms,log'
$exeList = @($guard.frozen_exes | Sort-Object { if ($_.source -eq 'new') { 0 } else { 1 } }, entity)
Fact "entity_count=$($exeList.Count)"
$newCount = @($exeList | Where-Object { $_.source -eq 'new' }).Count
$regressionCount = @($exeList | Where-Object { $_.source -eq 'regression' }).Count
Fact "entity_new=$newCount entity_regression=$regressionCount"
foreach ($exe in $exeList) {
    if ((Test-Path $abortFile) -and ([bool](Get-Content $abortFile -Raw))) { $failures += 'sampler_abort'; break }
    $name = $exe.entity
    $logPath = Join-Path $logs "$name.log"
    $watch = [Diagnostics.Stopwatch]::StartNew()
    cmd /c "cargo test --locked --features cuda-probe --test $name -- --ignored --test-threads=1 --nocapture > `"$logPath`" 2>&1"
    $code = $LASTEXITCODE
    $watch.Stop()
    Fact "entity $name exit=$code wall_ms=$($watch.ElapsedMilliseconds)"
    Add-Txt $entityPath ("{0},{1},{2},{3},{4}" -f $name, $exe.source, $code, $watch.ElapsedMilliseconds, $logPath)
    if ($code -ne 0) { $failures += "entity_failed:$name" }
    if ([bool](Get-Content $abortFile -Raw)) { $failures += "sampler_abort_after:$name"; break }
}
$bodyWatch.Stop()
$entityEnd = Iso
$env:PATH = $oldPath
Stop-Sampler
$post = @()
for ($i = 0; $i -lt 3; $i++) {
    $post += GpuSample
    if ($i -lt 2) { Start-Sleep -Milliseconds 250 }
}
Fact ('post_baseline=' + (($post | ForEach-Object { "$($_.used),$($_.free),$($_.util),$($_.temp)" }) -join ' / '))
$depsAfter = DepsSnapshot
Fact "deps_after_count=$($depsAfter.count)"
Fact "deps_after_max_write=$($depsAfter.max)"
$zeroCompile = ($depsBefore.count -eq $depsAfter.count) -and ($depsBefore.max -eq $depsAfter.max)
Fact "zero_rust_compile=$zeroCompile"
$postCa = ComputeApps
Fact "compute_apps_post_raw=$($postCa.raw.Count) filtered=$($postCa.filtered.Count)"
$exeOk = 0
$exeBad = 0
foreach ($exe in $guard.frozen_exes) {
    $hash = (Get-FileHash (Join-Path $RepoRoot $exe.path) -Algorithm SHA256).Hash.ToLower()
    if ($hash -eq $exe.sha256) { $exeOk++ } else { $exeBad++ }
}
Fact "post_window_exe_recheck=$exeOk/$($guard.frozen_exes.Count) mismatches=$exeBad"
$residual = @(Get-Process | Where-Object { $_.ProcessName -match '^render_' })
Fact "residual_processes=$($residual.Count)"
Fact "attempt_valid_rounds=$Attempt"
Fact "body_wall_s=$([Math]::Round($bodyWatch.Elapsed.TotalSeconds, 1))"
Fact "entity_start=$entityStart entity_end=$entityEnd"
Fact ('window_end=' + (Iso))
$rows = @(Import-Csv $samplesPath)
$used = @($rows | ForEach-Object { [int]$_.used_mib })
$free = @($rows | ForEach-Object { [int]$_.free_mib })
$util = @($rows | ForEach-Object { [int]$_.util_percent })
$samples = $rows.Count
$peak = 0
$peakSeq = 0
$min = [int]::MaxValue
$freeMin = [int]::MaxValue
$utilMax = 0
for ($i = 0; $i -lt $samples; $i++) {
    if ($used[$i] -gt $peak) { $peak = $used[$i]; $peakSeq = $i + 1 }
    if ($used[$i] -lt $min) { $min = $used[$i] }
    if ($free[$i] -lt $freeMin) { $freeMin = $free[$i] }
    if ($util[$i] -gt $utilMax) { $utilMax = $util[$i] }
}
$streak = 0
$streakMax = 0
foreach ($v in $util) {
    if ($v -ge $AbortUtilLimit) { $streak++ } else { $streak = 0 }
    if ($streak -gt $streakMax) { $streakMax = $streak }
}
$deltaBaseline = $peak - $baseline[0].used
$deltaFirst = if ($samples -gt 0) { $peak - $used[0] } else { 0 }
$deltaMin = if ($samples -gt 0) { $peak - $min } else { 0 }
$sustained = if ($samples -gt 0) {
    $sortedUsed = @($used | Sort-Object)
    $second = if ($sortedUsed.Count -ge 2) { $sortedUsed[$sortedUsed.Count - 2] } else { $peak }
    $second - $baseline[0].used
} else { 0 }
Fact "samples=$samples"
Fact "used_peak_mib=$peak peak_seq=$peakSeq"
Fact "used_min_mib=$min free_min_mib=$freeMin util_max_percent=$utilMax"
Fact "util_streak_ge_${AbortUtilLimit}=$streakMax"
Fact "delta_vs_guard_baseline_mib=$deltaBaseline delta_vs_first_sample_mib=$deltaFirst delta_vs_sample_min_mib=$deltaMin"
Fact "sustained_delta_mib=$sustained"
Fact "increment_limit_mib=$BudgetMiB increment_exceeded=$([bool]($deltaBaseline -gt $BudgetMiB))"
Fact "floor_breached=$([bool]($freeMin -lt $floor))"
$abortMarker = Get-Content $abortFile -Raw
$guardAborted = [bool]$abortMarker
Fact "guard_aborted=$guardAborted abort_marker=$abortMarker"
$stamps = @($rows | ForEach-Object { [datetimeoffset]::Parse($_.timestamp) })
$diffs = @()
for ($i = 1; $i -lt $stamps.Count; $i++) { $diffs += ($stamps[$i] - $stamps[$i - 1]).TotalSeconds }
$diffs = @($diffs | Sort-Object)
$median = if ($diffs.Count -gt 0) { $diffs[[int][Math]::Floor($diffs.Count / 2)] } else { 0 }
$intMin = if ($diffs.Count -gt 0) { $diffs[0] } else { 0 }
$intMax = if ($diffs.Count -gt 0) { $diffs[$diffs.Count - 1] } else { 0 }
$entityRows = @(Import-Csv $entityPath)
$entityExits = @{}
foreach ($row in $entityRows) { $entityExits[$row.entity] = [int]$row.exit }
$agg = [pscustomobject]@{
    suite = 'r05-s3b-hfield-validate'
    attempt = $Attempt
    device_validation_window = $true
    authoring_exception_used = $false
    samples = $samples
    used_peak_mib = $peak
    peak_seq = $peakSeq
    used_min_mib = $min
    free_min_mib = $freeMin
    util_max_percent = $utilMax
    util_streak_ge_abort_limit = $streakMax
    sample_interval_s = [pscustomobject]@{
        median = [Math]::Round($median, 4)
        min_s = [Math]::Round($intMin, 4)
        max_s = [Math]::Round($intMax, 4)
    }
    floor_mib = $floor
    baseline_free_mib = $baselineFree
    delta_vs_guard_baseline_mib = $deltaBaseline
    delta_vs_first_sample_mib = $deltaFirst
    delta_vs_sample_min_mib = $deltaMin
    sustained_delta_mib = $sustained
    budget_mib = [pscustomobject]@{ increment_limit = $BudgetMiB }
    guards_aggregate_sha256 = $guard.aggregate_sha256
    sha_guards = "$guardOk/$guardTotal"
    frozen_exes = $guard.frozen_exes.Count
    guard_inputs = $guard.guard_inputs.Count
    post_window_exe_recheck = "$exeOk/$($guard.frozen_exes.Count)"
    zero_rust_compile = $zeroCompile
    residual_processes = $residual.Count
    guard_aborted = $guardAborted
    entity_exits = $entityExits
}
[IO.File]::WriteAllText((Join-Path $out 'guard_aggregate.json'), ($agg | ConvertTo-Json -Depth 6), $utf8)
if ($failures.Count -gt 0) {
    Fact ('failures=' + ($failures -join ';'))
    Guard 'INVALID_ROUND exit10'
    exit 10
}
if (-not $zeroCompile) {
    Guard 'COMPILE_DETECTED exit7'
    exit 7
}
Guard 'WINDOW_RUN_DONE'
exit 0
