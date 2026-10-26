# GPU window runner for the G2/G6 GPU reference capture (window count = 1).
#
# Default (no -Execute): prints the plan + pre-window GPU snapshot only (nvidia-smi reads; no GPU context).
# -Execute: runs the approved window: guard v2, 0.25 s sampling, one preflight + two independent capture
# rounds (separate processes, separate kernel caches), then CPU-only post-processing (sha compare,
# canonical row-order normalization, input contract check, old-CPU-vs-new-GPU comparison table).
#
# Usage:
#   pwsh -File run_capture_gpu.ps1                 # plan only
#   pwsh -File run_capture_gpu.ps1 -Execute        # inside the approved window
#
param([switch]$Execute, [string]$CapturedAt = '')
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$env:PYTHONIOENCODING = 'utf-8'
$repo = 'D:\Rust\mjwarp-rs'
$evidence = Join-Path $repo 'docs\development-evidence\g10-p5-g2-gpu-reference-capture'
$logs = Join-Path $evidence 'logs'
$rounds = Join-Path $evidence 'rounds'
$python = Join-Path $repo 'target\mjwarp-parallel\venv\Scripts\python.exe'
$capture = Join-Path $evidence 'capture_mjwarp_f32_gpu.py'
$canonicalize = Join-Path $evidence 'canonicalize.py'
$compare = Join-Path $evidence 'compare_references.py'
$verify = Join-Path $evidence 'verify_inputs.py'
$preflightPy = Join-Path $evidence 'preflight_gpu.py'
$cacheRoot = Join-Path $repo 'target\g10-p5-g2\warp-cache-gpu'
$cache1 = Join-Path $cacheRoot 'round1'
$cache2 = Join-Path $cacheRoot 'round2'
$round1Raw = Join-Path $rounds 'round1-raw.json'
$round2Raw = Join-Path $rounds 'round2-raw.json'
$round1Canon = Join-Path $rounds 'round1-canonical.json'
$round2Canon = Join-Path $rounds 'round2-canonical.json'
$oldRef = Join-Path $repo 'docs\development-evidence\g10-p5-gjk-epa\upstream-mjwarp-f32.json'
$oldScript = Join-Path $repo 'docs\development-evidence\g10-p5-gjk-epa\capture_mjwarp_f32.py'
$finalRef = Join-Path $repo 'docs\development-evidence\g10-p5-gjk-epa\upstream-mjwarp-f32-gpu.json'
$guardScript = Join-Path $repo 'scripts\gpu-window-guard.ps1'
$stamp = (Get-Date).ToUniversalTime().ToString('yyyyMMdd-HHmmss')
$utf8 = New-Object System.Text.UTF8Encoding($false)

function Write-Text([string]$path, [string]$text) {
    [System.IO.File]::WriteAllText($path, $text, $utf8)
}
function Append-Text([string]$path, [string]$text) {
    [System.IO.File]::AppendAllText($path, $text, $utf8)
}
function Sha256Of([string]$path) {
    if (-not (Test-Path $path)) { return 'missing' }
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}
function Gpu-Raw([string]$path) {
    $sb = New-Object System.Text.StringBuilder
    [void]$sb.AppendLine('=== nvidia-smi --query-gpu=name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu,driver_version,compute_cap ===')
    [void]$sb.AppendLine((& nvidia-smi --query-gpu=name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu,driver_version,compute_cap --format=csv,noheader | Out-String).Trim())
    [void]$sb.AppendLine('=== nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv (raw) ===')
    [void]$sb.AppendLine((& nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv | Out-String).Trim())
    Write-Text $path $sb.ToString()
}
function Run-Python([string]$label, [string[]]$arguments, [string]$logPath, [string]$workdir) {
    $sb = New-Object System.Text.StringBuilder
    [void]$sb.AppendLine("CMD: $python $($arguments -join ' ')")
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    # Native stderr must not terminate under the script-wide ErrorActionPreference=Stop;
    # route both streams through files so the full traceback is preserved in the log.
    $outFile = "$logPath.stdout.tmp"
    $errFile = "$logPath.stderr.tmp"
    $process = Start-Process -FilePath $python -ArgumentList $arguments -WorkingDirectory $workdir -NoNewWindow -Wait -PassThru -RedirectStandardOutput $outFile -RedirectStandardError $errFile
    $code = $process.ExitCode
    $watch.Stop()
    $stdout = if (Test-Path $outFile) { Get-Content -LiteralPath $outFile -Raw -ErrorAction SilentlyContinue } else { '' }
    $stderr = if (Test-Path $errFile) { Get-Content -LiteralPath $errFile -Raw -ErrorAction SilentlyContinue } else { '' }
    if ($null -eq $stdout) { $stdout = '' }
    if ($null -eq $stderr) { $stderr = '' }
    [void]$sb.AppendLine($stdout.TrimEnd())
    if ($stderr.Trim()) { [void]$sb.AppendLine('STDERR:'); [void]$sb.AppendLine($stderr.TrimEnd()) }
    [void]$sb.AppendLine("EXIT=$code WALL_MS=$($watch.ElapsedMilliseconds)")
    Write-Text $logPath $sb.ToString()
    Remove-Item -LiteralPath $outFile, $errFile -Force -ErrorAction SilentlyContinue
    Write-Host ("{0}: exit={1} wall={2}ms" -f $label, $code, $watch.ElapsedMilliseconds)
    return @{ exit = $code; wall_ms = $watch.ElapsedMilliseconds; log = $logPath }
}

Write-Host '=== G2/G6 GPU reference capture window plan ==='
Write-Host "HEAD: $((& git -C $repo rev-parse --short HEAD))"
Write-Host 'Steps: preconditions snapshot -> guard baseline -> preflight (device/versions)'
Write-Host '       -> round1 (cache round1, independent process) -> round2 (cache round2, independent process)'
Write-Host '       -> post snapshot/guard end -> CPU post-processing (sha/canonical/input/compare)'
Write-Host 'Window estimate: preflight ~5 s, round1 ~20-40 s (warp CUDA kernel JIT on CPU), round2 ~10-30 s.'
Write-Host 'GPU memory estimate: CUDA context + small arrays, delta < 800 MiB; guard floor free>=10240 MiB / delta<3072 MiB.'
Write-Host ''

New-Item -ItemType Directory -Force -Path $logs, $rounds | Out-Null
Gpu-Raw (Join-Path $logs "window-$stamp-pre.txt")
Write-Host "pre snapshot: $(Join-Path $logs "window-$stamp-pre.txt")"

if (-not $Execute) {
    Write-Host ''
    Write-Host 'PLAN ONLY: no GPU context created by this run. Re-run with -Execute inside the approved window.'
    exit 0
}

if ($CapturedAt -eq '' -and $env:MJWARP_CAPTURE_AT) { $capturedAt = $env:MJWARP_CAPTURE_AT }
elseif ($CapturedAt -eq '') { $capturedAt = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ') }
else { $capturedAt = $CapturedAt }
$env:MJWARP_CAPTURE_AT = $capturedAt
Write-Host "captured_at (both rounds): $capturedAt"

$oldRefShaBefore = Sha256Of $oldRef
$oldScriptShaBefore = Sha256Of $oldScript
$gitBefore = (& git -C $repo status --porcelain) -join "`n"
Write-Text (Join-Path $logs "window-$stamp-git.txt") "GIT_BEFORE`n$gitBefore`n`nOLD_REF_SHA256=$oldRefShaBefore`nOLD_SCRIPT_SHA256=$oldScriptShaBefore"

. $guardScript
# Local override (documented in README): the shared guard template's Get-GpuSnapshot casts
# nvidia-smi's unit-suffixed CSV ("4165 MiB") to int and crashes; v2 logic is unchanged, we only
# supply a unit-less parse. The shared template file itself is NOT modified by this task.
function Get-GpuSnapshot {
    $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu --format=csv,noheader,nounits 2>&1 | Select-Object -First 1)
    $parts = @(($line -split ',') | ForEach-Object { $_.Trim() })
    if ($parts.Count -lt 3 -or $parts[0] -notmatch '^\d+$' -or $parts[1] -notmatch '^\d+$' -or $parts[2] -notmatch '^\d+$') {
        throw "GUARD_ABORT: cannot parse nvidia-smi memory/util line: $line"
    }
    return @{ used = [int]$parts[0]; free = [int]$parts[1]; util = [int]$parts[2] }
}
$guardLog = Join-Path $logs "window-$stamp-guard.log"
$state = New-GuardState -Label 'g2-gpu-ref-capture' -TotalMib 16376 -OwnExeNames @('python.exe') -LogPath $guardLog
$samplesFile = Join-Path $logs "window-$stamp-samples.txt"
$flagFile = Join-Path $repo 'target\g2-capture-sampler.on'
$stopFile = Join-Path $repo 'target\g2-capture-sampler.stop'
$wallWatch = [System.Diagnostics.Stopwatch]::StartNew()
$aborted = $false
$abortMessage = ''
$sampler = $null
function Assert-WallLimit {
    if ($wallWatch.Elapsed.TotalSeconds -gt 300) {
        throw "GUARD_ABORT: window wall exceeded 300 s (approved budget 120 s, abort limit 5 min)"
    }
}

try {
    $null = Assert-Guard -State $state -Phase 'preconditions'
    Write-Host 'guard: preconditions pass'
    Assert-WallLimit

    Set-Content -LiteralPath $flagFile -Value 'on'
    Remove-Item -LiteralPath $stopFile -Force -ErrorAction SilentlyContinue
    $sampler = Start-Job -ArgumentList $flagFile, $stopFile, $samplesFile -ScriptBlock {
        param($flagFile, $stopFile, $samplesFile)
        $utf8 = New-Object System.Text.UTF8Encoding($false)
        while ((Test-Path $flagFile) -and -not (Test-Path $stopFile)) {
            $now = (Get-Date).ToUniversalTime().ToString('HH:mm:ss.fff')
            $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null) -join ';'
            [System.IO.File]::AppendAllText($samplesFile, "$now;$line`n", $utf8)
            Start-Sleep -Milliseconds 250
        }
    }
    Write-Host "sampler started: $samplesFile"

    $preflight = Run-Python 'preflight' @('-B', $preflightPy) (Join-Path $logs "window-$stamp-preflight.txt") $repo
    if ($preflight.exit -ne 0) { throw "preflight failed (exit $($preflight.exit)), see $($preflight.log)" }
    $null = Assert-Guard -State $state -Phase 'preflight' -OwnGpuStepActive
    Assert-WallLimit

    foreach ($pair in @(@{ name = 'round1'; cache = $cache1; out = $round1Raw }, @{ name = 'round2'; cache = $cache2; out = $round2Raw })) {
        if (Test-Path $pair.cache) { Remove-Item -LiteralPath $pair.cache -Recurse -Force }
        New-Item -ItemType Directory -Force -Path $pair.cache | Out-Null
        $result = Run-Python $pair.name @('-B', $capture, $pair.cache, $pair.out) (Join-Path $logs "window-$stamp-$($pair.name).txt") $repo
        $null = Assert-Guard -State $state -Phase $pair.name -OwnGpuStepActive
        Assert-WallLimit
        if ($result.exit -ne 0) { throw "capture $($pair.name) failed (exit $($result.exit)), see $($result.log)" }
    }
} catch {
    $aborted = $true
    $abortMessage = $_.Exception.Message
    Write-Host "ABORT: $abortMessage"
} finally {
    Remove-Item -LiteralPath $flagFile -Force -ErrorAction SilentlyContinue
    Set-Content -LiteralPath $stopFile -Value 'stop' -ErrorAction SilentlyContinue
    if ($null -ne $sampler) {
        if (Get-Job -Id $sampler.Id -ErrorAction SilentlyContinue) { Wait-Job $sampler -Timeout 10 | Out-Null }
        Remove-Job $sampler -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath $flagFile, $stopFile -Force -ErrorAction SilentlyContinue
}
$sampleCount = (Get-Content -LiteralPath $samplesFile -ErrorAction SilentlyContinue | Measure-Object).Count
Write-Host "sampler stopped: $sampleCount samples"
Gpu-Raw (Join-Path $logs "window-$stamp-post.txt")
$guardEnd = Complete-Guard -State $state
$guardBytes = [System.IO.File]::ReadAllBytes($guardLog)
if ($guardBytes.Length -ge 3 -and $guardBytes[0] -eq 0xEF -and $guardBytes[1] -eq 0xBB -and $guardBytes[2] -eq 0xBF) {
    [System.IO.File]::WriteAllBytes($guardLog, $guardBytes[3..($guardBytes.Length - 1)])
}
Write-Host "guard end: peak=$($guardEnd.peak_used_mib) MiB samples=$($guardEnd.samples) residual=$($guardEnd.residual -join ',')"
if ($aborted) {
    $abortMeta = [ordered]@{
        schema = 1
        task_id = '2026-10-25-g10-p5-g2-gpu-reference-capture'
        kind = 'g2-gpu-reference-capture-window-meta'
        status = 'aborted'
        abort_reason = $abortMessage
        captured_at = $capturedAt
        window_stamp = $stamp
        guard = [ordered]@{ peak_used_mib = $guardEnd.peak_used_mib; samples = $guardEnd.samples; residual_context_pids = $guardEnd.residual; log = "logs/window-$stamp-guard.log" }
        sampler = [ordered]@{ samples = $sampleCount; file = "logs/window-$stamp-samples.txt" }
        note = 'Aborted round is void and not spliced; re-run the whole window after the cause is fixed/approved.'
    }
    Write-Text (Join-Path $logs "window-$stamp-aborted.json") ($abortMeta | ConvertTo-Json -Depth 6)
    Write-Host "STOP: $abortMessage"
    exit 8
}

# ---- CPU-only post-processing (outside the measured GPU section) ----
$raw1 = Sha256Of $round1Raw
$raw2 = Sha256Of $round2Raw
$null = Run-Python 'canonicalize-round1' @('-B', $canonicalize, $round1Raw, $round1Canon) (Join-Path $logs "window-$stamp-canonicalize1.txt") $repo
$null = Run-Python 'canonicalize-round2' @('-B', $canonicalize, $round2Raw, $round2Canon) (Join-Path $logs "window-$stamp-canonicalize2.txt") $repo
$canonSha1 = Sha256Of $round1Canon
$canonSha2 = Sha256Of $round2Canon
$verifyRun = Run-Python 'verify-inputs' @('-B', $verify) (Join-Path $logs "window-$stamp-verify-inputs.txt") $repo
$compareRun = Run-Python 'compare-cpu-vs-gpu' @('-B', $compare, $oldRef, $round1Canon, $round2Canon) (Join-Path $logs "window-$stamp-compare.txt") $repo

if ($raw1 -eq $raw2) { $twoRoundMode = 'raw_identical' }
elseif ($canonSha1 -eq $canonSha2) { $twoRoundMode = 'canonical_identical_raw_order_differs' }
else { $twoRoundMode = 'differ' }

$source = if ($twoRoundMode -eq 'raw_identical') { $round1Raw } else { $round1Canon }
$candidateJson = Get-Content -LiteralPath $source -Raw | ConvertFrom-Json
$twoCaptures = @($candidateJson.cases.PSObject.Properties | ForEach-Object { [bool]$_.Value.two_captures_bitwise_identical })
$twoCapturesAll = -not ($twoCaptures -contains $false)
$counts = @($candidateJson.cases.PSObject.Properties | ForEach-Object { [int]$_.Value.output.count })

$status = 'ok'
$exitCode = 0
if (-not $twoCapturesAll) { $status = 'stop_two_captures_false'; $exitCode = 6 }
elseif ($twoRoundMode -eq 'differ') { $status = 'stop_two_rounds_differ'; $exitCode = 3 }
elseif ($twoRoundMode -ne 'raw_identical') { $status = 'stop_raw_sha_differs_canonical_equal_needs_decision'; $exitCode = 7 }
elseif ($verifyRun.exit -ne 0) { $status = 'stop_input_contract_failed'; $exitCode = 4 }
elseif ($compareRun.exit -ne 0) { $status = 'stop_structural_diff_vs_old_cpu'; $exitCode = 5 }

$finalSha = 'not_written'
$finalSource = 'not_written'
if ($status -eq 'ok') {
    Copy-Item -LiteralPath $source -Destination $finalRef -Force
    $finalSha = Sha256Of $finalRef
    $finalSource = if ($twoRoundMode -eq 'raw_identical') { 'round1-raw' } else { 'round1-canonical' }
}

$meta = [ordered]@{
    schema = 1
    task_id = '2026-10-25-g10-p5-g2-gpu-reference-capture'
    kind = 'g2-gpu-reference-capture-window-meta'
    status = $status
    captured_at = $capturedAt
    head = (& git -C $repo rev-parse HEAD)
    window_stamp = $stamp
    cache_root = 'target/g10-p5-g2/warp-cache-gpu'
    round1 = [ordered]@{ cache = 'target/g10-p5-g2/warp-cache-gpu/round1'; raw_sha256 = $raw1; canonical_sha256 = $canonSha1; log = "logs/window-$stamp-round1.txt" }
    round2 = [ordered]@{ cache = 'target/g10-p5-g2/warp-cache-gpu/round2'; raw_sha256 = $raw2; canonical_sha256 = $canonSha2; log = "logs/window-$stamp-round2.txt" }
    two_round_mode = $twoRoundMode
    two_captures_bitwise_identical = $twoCaptures
    case_counts = $counts
    old_ref_sha256_before = $oldRefShaBefore
    old_ref_sha256_after = (Sha256Of $oldRef)
    old_script_sha256_before = $oldScriptShaBefore
    old_script_sha256_after = (Sha256Of $oldScript)
    final = [ordered]@{
        path = 'docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json'
        source = $finalSource
        sha256 = $finalSha
    }
    guard = [ordered]@{ peak_used_mib = $guardEnd.peak_used_mib; samples = $guardEnd.samples; residual_context_pids = $guardEnd.residual; log = "logs/window-$stamp-guard.log" }
    sampler = [ordered]@{ samples = $sampleCount; file = "logs/window-$stamp-samples.txt" }
    verify_inputs_exit = $verifyRun.exit
    compare_exit = $compareRun.exit
}
$gitAfter = (& git -C $repo status --porcelain) -join "`n"
Append-Text (Join-Path $logs "window-$stamp-git.txt") "`nGIT_AFTER`n$gitAfter"
Write-Text (Join-Path $evidence 'capture-meta.json') ($meta | ConvertTo-Json -Depth 6)
Write-Host "status=$status two_round_mode=$twoRoundMode final_sha=$finalSha"
if ($status -ne 'ok') {
    Write-Host "STOP ($status): facts are recorded in capture-meta.json; do not claim capture; report to Arch."
    exit $exitCode
}
Write-Host 'window complete (facts recorded; no acceptance/claim stated here)'
exit 0
