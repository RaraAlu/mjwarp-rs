# Pure-CPU dry-run for the GPU window guard template (no CUDA context, read-only nvidia-smi).
#
# Verifies:
#  A. `ConvertFrom-GpuCsvLine` accepts unit-suffixed (`4165 MiB, 11875 MiB, 38 %`) and bare numeric
#     (`4165, 11875, 38`) forms; invalid input raises `GUARD_ABORT` (not an [int] exception).
#  B. `Start-GuardSampler`/`Stop-GuardSampler` persist every 0.25 s sample to `samples.csv` with
#     immediate flush; Stop joins the child with timeout; stats match an independent CSV recompute;
#     no residual process.
#  C. Guard rules really consume samples: injected sequences (no live load) prove
#     util>=80% for 3 consecutive samples without own step aborts, own-step active does not abort,
#     normal sequence does not abort, and the memory rules (free<10240 / increment>=3072) still fire.
#
# Usage: pwsh -NoProfile -File scripts/gpu-window-guard-dryrun.ps1 [-EvidenceDir <dir>] [-Seconds 3.2]
param(
    [string]$EvidenceDir = '',
    [double]$Seconds = 3.2
)
$ErrorActionPreference = 'Stop'
$script:failures = @()
function Check([string]$Name, [bool]$Ok, [string]$Detail = '') {
    if ($Ok) { Write-Output "PASS $Name $Detail" } else { Write-Output "FAIL $Name $Detail"; $script:failures += $Name }
}
if (-not $EvidenceDir) { $EvidenceDir = Join-Path (Split-Path -Parent $PSScriptRoot) 'docs/development-evidence/gpu-window-guard-sampler/logs' }
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null
. (Join-Path $PSScriptRoot 'gpu-window-guard.ps1')

# ---- A. unit-suffixed CSV parsing -------------------------------------------------------------
$parserLog = Join-Path $EvidenceDir 'parser-dryrun.log'
$lines = @()
$units = ConvertFrom-GpuCsvLine -Line '4165 MiB, 11875 MiB, 38 %'
$bare = ConvertFrom-GpuCsvLine -Line '4165, 11875, 38'
$lines += "units: used=$($units.used) free=$($units.free) util=$($units.util)"
$lines += "bare:  used=$($bare.used) free=$($bare.free) util=$($bare.util)"
Check 'parser_units' ($units.used -eq 4165 -and $units.free -eq 11875 -and $units.util -eq 38)
Check 'parser_bare' ($bare.used -eq 4165 -and $bare.free -eq 11875 -and $bare.util -eq 38)
$aborted = $false
try { $null = ConvertFrom-GpuCsvLine -Line '4165 MiB, N/A MiB, 38 %' } catch { $aborted = $_.Exception.Message -match 'GUARD_ABORT' }
Check 'parser_invalid_is_guard_abort' $aborted
$lines | Out-File -LiteralPath $parserLog -Encoding utf8
Write-Output ($lines -join '; ')

# ---- B. real 0.25 s sampler dry-run (>=3 s) ----------------------------------------------------
$sampleCsv = Join-Path $EvidenceDir 'samples.csv'
$samplerLog = Join-Path $EvidenceDir 'sampler-dryrun.log'
$sampler = Start-GuardSampler -CsvPath $sampleCsv -IntervalMs 250
Start-Sleep -Seconds $Seconds
$stats = Stop-GuardSampler -Sampler $sampler -TimeoutSeconds 10
$rows = @(Import-Csv -LiteralPath $sampleCsv)
$seqOk = $true
$monotonic = $true
$parsable = $true
for ($i = 0; $i -lt $rows.Count; $i++) {
    if ([int]$rows[$i].seq -ne ($i + 1)) { $seqOk = $false }
    if ($rows[$i].timestamp -notmatch '^\d{4}-\d{2}-\d{2}T') { $parsable = $false }
    if ($i -gt 0 -and [datetime]$rows[$i].timestamp -lt [datetime]$rows[$i - 1].timestamp) { $monotonic = $false }
    if ($rows[$i].used_mib -notmatch '^\d+$' -or $rows[$i].free_mib -notmatch '^\d+$' -or $rows[$i].util_percent -notmatch '^\d+$') { $parsable = $false }
}
$indepPeak = if ($rows.Count) { ($rows | ForEach-Object { [int]$_.used_mib } | Measure-Object -Maximum).Maximum } else { $null }
$indepFree = if ($rows.Count) { ($rows | ForEach-Object { [int]$_.free_mib } | Measure-Object -Minimum).Minimum } else { $null }
$indepUtil = if ($rows.Count) { ($rows | ForEach-Object { [int]$_.util_percent } | Measure-Object -Maximum).Maximum } else { $null }
Check 'sampler_rows_equal_stats' ($stats.samples -eq $rows.Count) "stats=$($stats.samples) csv=$($rows.Count)"
Check 'sampler_at_least_10' ($stats.samples -ge 10) "samples=$($stats.samples)"
Check 'sampler_seq_1_n' $seqOk
Check 'sampler_timestamp_monotonic' $monotonic "first=$($stats.first_ts) last=$($stats.last_ts)"
Check 'sampler_fields_parsable' $parsable
Check 'sampler_peak_matches_csv' ($stats.used_peak_mib -eq $indepPeak) "stats=$($stats.used_peak_mib) csv=$indepPeak"
Check 'sampler_free_min_matches_csv' ($stats.free_min_mib -eq $indepFree) "stats=$($stats.free_min_mib) csv=$indepFree"
Check 'sampler_util_max_matches_csv' ($stats.util_max_percent -eq $indepUtil) "stats=$($stats.util_max_percent) csv=$indepUtil"
Check 'sampler_process_exited' ([bool]$stats.process_exited)
Check 'sampler_no_residual_process' ($stats.residual_processes -eq 0) "residual=$($stats.residual_processes)"
"sampler stats: samples=$($stats.samples) used_peak=$($stats.used_peak_mib) free_min=$($stats.free_min_mib) util_max=$($stats.util_max_percent) process_exited=$($stats.process_exited) residual=$($stats.residual_processes)" |
    Out-File -LiteralPath $samplerLog -Encoding utf8

# ---- C. injected-sequence guard verification (no live load) ------------------------------------
$injectLog = Join-Path $EvidenceDir 'injection-dryrun.log'
$inject = @()
function Expect-Abort([string]$Name, [scriptblock]$Action) {
    $caught = $false
    try { & $Action } catch { $caught = $_.Exception.Message -match 'GUARD_ABORT' }
    $script:inject += "case=$Name abort=$caught"
    Check "inject_$Name" $caught
}
function Expect-Pass([string]$Name, [scriptblock]$Action) {
    $threw = $false
    try { & $Action } catch { $threw = $true; $script:inject += "case=$Name abort=true msg=$($_.Exception.Message)" }
    if (-not $threw) { $script:inject += "case=$Name abort=false" }
    Check "inject_$Name" (-not $threw)
}
$appNone = @()
# normal sequence: 3 samples, util 30/40/30 -> no abort, streak resets
$state = New-GuardState -Label 'inject-normal' -LogPath (Join-Path $EvidenceDir 'inject-normal.log')
Expect-Pass 'normal_sequence' {
    $null = Assert-Guard -State $state -Phase 'n1' -Snapshot @{ used = 4165; free = 11875; util = 30 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'n2' -Snapshot @{ used = 4165; free = 11875; util = 40 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'n3' -Snapshot @{ used = 4165; free = 11875; util = 30 } -Apps $appNone
}
# util streak without own step: 85/90/88 -> abort (3 consecutive >= 80)
$state = New-GuardState -Label 'inject-util' -LogPath (Join-Path $EvidenceDir 'inject-util.log')
Expect-Abort 'util_streak_without_own_step' {
    $null = Assert-Guard -State $state -Phase 'u1' -Snapshot @{ used = 4165; free = 11875; util = 85 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'u2' -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'u3' -Snapshot @{ used = 4165; free = 11875; util = 88 } -Apps $appNone
}
# own GPU step active: same util no abort
$state = New-GuardState -Label 'inject-ownstep' -LogPath (Join-Path $EvidenceDir 'inject-ownstep.log')
Expect-Pass 'util_with_own_step_active' {
    $null = Assert-Guard -State $state -Phase 'o1' -OwnGpuStepActive -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'o2' -OwnGpuStepActive -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'o3' -OwnGpuStepActive -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
}
# memory rules unchanged: free<10240 aborts, used increment>=3072 aborts
$state = New-GuardState -Label 'inject-free' -LogPath (Join-Path $EvidenceDir 'inject-free.log')
Expect-Abort 'free_below_floor' {
    $null = Assert-Guard -State $state -Phase 'f1' -Snapshot @{ used = 4165; free = 10000; util = 10 } -Apps $appNone
}
$state = New-GuardState -Label 'inject-delta' -LogPath (Join-Path $EvidenceDir 'inject-delta.log')
Expect-Abort 'increment_over_limit' {
    $null = Assert-Guard -State $state -Phase 'd1' -Snapshot @{ used = 4165; free = 11875; util = 10 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'd2' -Snapshot @{ used = 7400; free = 8640; util = 10 } -Apps $appNone
}
$inject | Out-File -LiteralPath $injectLog -Encoding utf8
$inject | ForEach-Object { Write-Output $_ }

if ($script:failures.Count -gt 0) {
    Write-Output "DRYRUN RESULT: FAIL ($($script:failures -join ','))"
    exit 1
}
Write-Output "DRYRUN RESULT: PASS (parser+sampler+guard-injection)"
