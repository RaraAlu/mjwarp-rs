# CPU-only finalizer for the approved GPU capture window (attempt-3).
# The GPU work already happened once (round1 + round2 raw files saved); a runner post-processing bug
# (PowerShell 5.1 native-stderr ErrorActionPreference) stopped the runner after guard end. This script
# performs the remaining CPU-only steps: sha compare, canonical form, input re-verify, old-CPU-vs-new-GPU
# comparison, final artifact copy (only when status=ok) and capture-meta.json.
#
# capture-meta.json is written via kv_to_json.py because PowerShell 5.1 ConvertTo-Json crashes on the
# nested OrderedDictionary used here ("Capacity was less than the current size").
#
# Usage: pwsh -File finalize_capture.ps1 -Stamp 20261026-012956
param([Parameter(Mandatory = $true)][string]$Stamp)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$env:PYTHONIOENCODING = 'utf-8'
$repo = 'D:\Rust\mjwarp-rs'
$evidence = Join-Path $repo 'docs\development-evidence\g10-p5-g2-gpu-reference-capture'
$logs = Join-Path $evidence 'logs'
$rounds = Join-Path $evidence 'rounds'
$python = Join-Path $repo 'target\mjwarp-parallel\venv\Scripts\python.exe'
$canonicalize = Join-Path $evidence 'canonicalize.py'
$compare = Join-Path $evidence 'compare_references.py'
$verify = Join-Path $evidence 'verify_inputs.py'
$kvToJson = Join-Path $evidence 'kv_to_json.py'
$round1Raw = Join-Path $rounds 'round1-raw.json'
$round2Raw = Join-Path $rounds 'round2-raw.json'
$round1Canon = Join-Path $rounds 'round1-canonical.json'
$round2Canon = Join-Path $rounds 'round2-canonical.json'
$oldRef = Join-Path $repo 'docs\development-evidence\g10-p5-gjk-epa\upstream-mjwarp-f32.json'
$oldScript = Join-Path $repo 'docs\development-evidence\g10-p5-gjk-epa\capture_mjwarp_f32.py'
$finalRef = Join-Path $repo 'docs\development-evidence\g10-p5-gjk-epa\upstream-mjwarp-f32-gpu.json'
$utf8 = New-Object System.Text.UTF8Encoding($false)

function Write-Text([string]$path, [string]$text) { [System.IO.File]::WriteAllText($path, $text, $utf8) }
function Append-Text([string]$path, [string]$text) { [System.IO.File]::AppendAllText($path, $text, $utf8) }
function Sha256Of([string]$path) {
    if (-not (Test-Path $path)) { return 'missing' }
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}
function Run-Py([string]$label, [string[]]$arguments, [string]$logPath) {
    $sb = New-Object System.Text.StringBuilder
    [void]$sb.AppendLine("CMD: $python $($arguments -join ' ')")
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $outFile = "$logPath.stdout.tmp"
    $errFile = "$logPath.stderr.tmp"
    $process = Start-Process -FilePath $python -ArgumentList $arguments -WorkingDirectory $repo -NoNewWindow -Wait -PassThru -RedirectStandardOutput $outFile -RedirectStandardError $errFile
    $watch.Stop()
    $stdout = if (Test-Path $outFile) { Get-Content -LiteralPath $outFile -Raw -ErrorAction SilentlyContinue } else { '' }
    $stderr = if (Test-Path $errFile) { Get-Content -LiteralPath $errFile -Raw -ErrorAction SilentlyContinue } else { '' }
    if ($null -eq $stdout) { $stdout = '' }
    if ($null -eq $stderr) { $stderr = '' }
    [void]$sb.AppendLine($stdout.TrimEnd())
    if ($stderr.Trim()) { [void]$sb.AppendLine('STDERR:'); [void]$sb.AppendLine($stderr.TrimEnd()) }
    [void]$sb.AppendLine("EXIT=$($process.ExitCode) WALL_MS=$($watch.ElapsedMilliseconds)")
    Write-Text $logPath $sb.ToString()
    Remove-Item -LiteralPath $outFile, $errFile -Force -ErrorAction SilentlyContinue
    Write-Host ("{0}: exit={1} wall={2}ms" -f $label, $process.ExitCode, $watch.ElapsedMilliseconds)
    return @{ exit = $process.ExitCode }
}
function Get-Sample-Summary([string]$path) {
    $summary = [ordered]@{ samples = 0; peak_used_mib = 0; min_free_mib = [int]::MaxValue; max_util = 0; max_temp_c = 0 }
    if (-not (Test-Path $path)) { return $summary }
    foreach ($line in Get-Content -LiteralPath $path) {
        $parts = $line.Split(';')
        if ($parts.Count -lt 2) { continue }
        $values = $parts[1].Split(',')
        if ($values.Count -lt 4) { continue }
        $used = [int]$values[0].Trim(); $free = [int]$values[1].Trim(); $util = [int]$values[2].Trim(); $temp = [int]$values[3].Trim()
        $summary.samples++
        if ($used -gt $summary.peak_used_mib) { $summary.peak_used_mib = $used }
        if ($free -lt $summary.min_free_mib) { $summary.min_free_mib = $free }
        if ($util -gt $summary.max_util) { $summary.max_util = $util }
        if ($temp -gt $summary.max_temp_c) { $summary.max_temp_c = $temp }
    }
    if ($summary.samples -eq 0) { $summary.min_free_mib = 0 }
    return $summary
}
function Get-Snapshot([string]$path) {
    $result = [ordered]@{ gpu_line = ''; compute_apps = 0 }
    if (-not (Test-Path $path)) { return $result }
    $lines = Get-Content -LiteralPath $path
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^NVIDIA') { $result.gpu_line = $lines[$i] }
        if ($lines[$i] -match '^pid, process_name') {
            $result.compute_apps = @($lines[($i + 1)..($lines.Count - 1)] | Where-Object { $_ -match '^\d+' }).Count
        }
    }
    return $result
}

$guardLog = Join-Path $logs "window-$Stamp-guard.log"
$samplesFile = Join-Path $logs "window-$Stamp-samples.txt"
$preFile = Join-Path $logs "window-$Stamp-pre.txt"
$postFile = Join-Path $logs "window-$Stamp-post.txt"
$gitFile = Join-Path $logs "window-$Stamp-git.txt"

$guardText = Get-Content -LiteralPath $guardLog -Raw
$peakMatch = [regex]::Match($guardText, 'guard_end[^\n]*peak_used_mib=(\d+)[^\n]*samples=(\d+)[^\n]*residual_context_pids=([^\s]*)[^\n]*aborted=(\w+)')
$guardPeak = if ($peakMatch.Success) { [int]$peakMatch.Groups[1].Value } else { 0 }
$guardSamples = if ($peakMatch.Success) { [int]$peakMatch.Groups[2].Value } else { 0 }
$guardResidual = if ($peakMatch.Success) { $peakMatch.Groups[3].Value } else { 'unknown' }
$baselineMatch = [regex]::Match($guardText, 'guard_baseline used_mib=(\d+) free_mib=(\d+)')
$baselineUsed = if ($baselineMatch.Success) { [int]$baselineMatch.Groups[1].Value } else { 0 }
$baselineFree = if ($baselineMatch.Success) { [int]$baselineMatch.Groups[2].Value } else { 0 }
$samples = Get-Sample-Summary $samplesFile
$pre = Get-Snapshot $preFile
$post = Get-Snapshot $postFile
$gitBeforeMatch = [regex]::Match((Get-Content -LiteralPath $gitFile -Raw), 'OLD_REF_SHA256=([0-9a-f]+)\s+OLD_SCRIPT_SHA256=([0-9a-f]+)')
$oldRefBefore = if ($gitBeforeMatch.Success) { $gitBeforeMatch.Groups[1].Value } else { Sha256Of $oldRef }
$oldScriptBefore = if ($gitBeforeMatch.Success) { $gitBeforeMatch.Groups[2].Value } else { Sha256Of $oldScript }

$round1 = Get-Content -LiteralPath $round1Raw -Raw | ConvertFrom-Json
$capturedAt = $round1.provenance.captured_at
$twoCaptures = @($round1.cases.PSObject.Properties | ForEach-Object { [bool]$_.Value.two_captures_bitwise_identical })
$counts = @($round1.cases.PSObject.Properties | ForEach-Object { [int]$_.Value.output.count })

$raw1 = Sha256Of $round1Raw
$raw2 = Sha256Of $round2Raw
$null = Run-Py 'canonicalize-round1' @('-B', $canonicalize, $round1Raw, $round1Canon) (Join-Path $logs "window-$Stamp-canonicalize1.txt")
$null = Run-Py 'canonicalize-round2' @('-B', $canonicalize, $round2Raw, $round2Canon) (Join-Path $logs "window-$Stamp-canonicalize2.txt")
$canonSha1 = Sha256Of $round1Canon
$canonSha2 = Sha256Of $round2Canon
$verifyRun = Run-Py 'verify-inputs' @('-B', $verify) (Join-Path $logs "window-$Stamp-verify-inputs.txt")
$compareRun = Run-Py 'compare-cpu-vs-gpu' @('-B', $compare, $oldRef, $round1Raw, $round2Raw) (Join-Path $logs "window-$Stamp-compare.txt")

if ($raw1 -eq $raw2) { $twoRoundMode = 'raw_identical' }
elseif ($canonSha1 -eq $canonSha2) { $twoRoundMode = 'canonical_identical_raw_order_differs' }
else { $twoRoundMode = 'differ' }
$twoCapturesAll = -not ($twoCaptures -contains $false)
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
    Copy-Item -LiteralPath $round1Raw -Destination $finalRef -Force
    $finalSha = Sha256Of $finalRef
    $finalSource = 'round1-raw'
}

$gitAfter = (& git -C $repo status --porcelain) -join "`n"
Append-Text $gitFile "`nGIT_AFTER`n$gitAfter`n`nFINAL_SHA256=$finalSha"

$twoCapturesJson = '[' + (($twoCaptures | ForEach-Object { if ($_) { 'true' } else { 'false' } }) -join ',') + ']'
$countsJson = '[' + (($counts | ForEach-Object { [string]$_ }) -join ',') + ']'
$kv = New-Object System.Collections.Generic.List[string]
$kv.Add('schema=1')
$kv.Add('task_id=2026-10-25-g10-p5-g2-gpu-reference-capture')
$kv.Add('kind=g2-gpu-reference-capture-window-meta')
$kv.Add("status=$status")
$kv.Add("captured_at=$capturedAt")
$kv.Add('head=' + (& git -C $repo rev-parse HEAD))
$kv.Add("window_stamp=$Stamp")
$kv.Add('window_run=attempt-3 (valid; GPU work once, two independent rounds)')
$kv.Add('attempts.attempt_1=aborted (guard snapshot parser bug; zero GPU work)')
$kv.Add('attempts.attempt_2=aborted (preflight -c quoting bug; zero GPU work)')
$kv.Add('attempts.attempt_3=valid capture rounds; GPU used once')
$kv.Add('cache_root=target/g10-p5-g2/warp-cache-gpu')
$kv.Add('round1.cache=target/g10-p5-g2/warp-cache-gpu/round1')
$kv.Add("round1.raw_sha256=$raw1")
$kv.Add("round1.canonical_sha256=$canonSha1")
$kv.Add("round1.log=logs/window-$Stamp-round1.txt")
$kv.Add('round2.cache=target/g10-p5-g2/warp-cache-gpu/round2')
$kv.Add("round2.raw_sha256=$raw2")
$kv.Add("round2.canonical_sha256=$canonSha2")
$kv.Add("round2.log=logs/window-$Stamp-round2.txt")
$kv.Add("two_round_mode=$twoRoundMode")
$kv.Add("two_captures_bitwise_identical=$twoCapturesJson")
$kv.Add("case_counts=$countsJson")
$kv.Add("old_ref_sha256_before=$oldRefBefore")
$kv.Add('old_ref_sha256_after=' + (Sha256Of $oldRef))
$kv.Add("old_script_sha256_before=$oldScriptBefore")
$kv.Add('old_script_sha256_after=' + (Sha256Of $oldScript))
$kv.Add('final.path=docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json')
$kv.Add("final.source=$finalSource")
$kv.Add("final.sha256=$finalSha")
$kv.Add('guard.template=scripts/gpu-window-guard.ps1 (v2 rules unchanged; runner-local Get-GpuSnapshot override only fixes unit-less parsing)')
$kv.Add("guard.baseline_used_mib=$baselineUsed")
$kv.Add("guard.baseline_free_mib=$baselineFree")
$kv.Add("guard.peak_used_mib=$guardPeak")
$kv.Add("guard.guard_samples=$guardSamples")
$kv.Add("guard.residual_context_pids=$guardResidual")
$kv.Add("guard.log=logs/window-$Stamp-guard.log")
$kv.Add("sampler.samples=$($samples.samples)")
$kv.Add("sampler.peak_used_mib=$($samples.peak_used_mib)")
$kv.Add("sampler.min_free_mib=$($samples.min_free_mib)")
$kv.Add("sampler.max_util_percent=$($samples.max_util)")
$kv.Add("sampler.max_temp_c=$($samples.max_temp_c)")
$kv.Add("sampler.file=logs/window-$Stamp-samples.txt")
$kv.Add("pre_snapshot.file=logs/window-$Stamp-pre.txt")
$kv.Add("pre_snapshot.gpu_line=$($pre.gpu_line)")
$kv.Add("pre_snapshot.compute_contexts=$($pre.compute_apps)")
$kv.Add("post_snapshot.file=logs/window-$Stamp-post.txt")
$kv.Add("post_snapshot.gpu_line=$($post.gpu_line)")
$kv.Add("post_snapshot.compute_contexts=$($post.compute_apps)")
$kv.Add("preflight_log=logs/window-$Stamp-preflight.txt")
$kv.Add("verify_inputs_exit=$($verifyRun.exit)")
$kv.Add("compare_exit=$($compareRun.exit)")
$kvPath = Join-Path $evidence 'capture-meta.kv'
Write-Text $kvPath (($kv -join "`n") + "`n")
$metaRun = Run-Py 'meta-json' @('-B', $kvToJson, $kvPath, (Join-Path $evidence 'capture-meta.json')) (Join-Path $logs "window-$Stamp-meta-json.txt")
Remove-Item -LiteralPath $kvPath -Force -ErrorAction SilentlyContinue
Write-Host "status=$status two_round_mode=$twoRoundMode raw1=$raw1 raw2=$raw2 final_sha=$finalSha"
Write-Host ("window samples: n={0} peak_used={1} MiB min_free={2} MiB max_util={3}% max_temp={4} C" -f $samples.samples, $samples.peak_used_mib, $samples.min_free_mib, $samples.max_util, $samples.max_temp_c)
Write-Host ("pre: {0}" -f $pre.gpu_line)
Write-Host ("post: {0}" -f $post.gpu_line)
Write-Host ("post compute contexts: {0} (pre: {1})" -f $post.compute_apps, $pre.compute_apps)
if ($metaRun.exit -ne 0) { Write-Host 'STOP: capture-meta.json generation failed'; exit 9 }
exit $exitCode
