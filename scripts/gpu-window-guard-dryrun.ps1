# Pure-CPU dry-run for the GPU window guard template (no CUDA context, read-only nvidia-smi).
#
# A. `ConvertFrom-GpuCsvLine` accepts unit-suffixed / bare numeric forms; empty / invalid -> GUARD_ABORT.
# B. `Start-GuardSampler`/`Stop-GuardSampler` persist every 0.25 s sample (immediate flush), Stop joins
#    the child with timeout; stats match an independent CSV recompute; no residual process.
# C. Injected sequences (no live load) prove the guard rules: util>=80% x3 without own step, free floor,
#    VRAM increment (with injected `-TotalMib` so the half-total limit is reachable), normal/own-step pass.
#
# Output goes to `-OutDir` (default: target/gpu-window-guard-sampler-dryrun/<timestamp>); it never writes
# into committed evidence by default — copy is an explicit evidence step.
param(
    [string]$OutDir = '',
    [double]$Seconds = 3.2
)
$ErrorActionPreference = 'Stop'
$script:failures = @()
$script:total = 0
$script:pass = 0
function Check([string]$Name, [bool]$Ok, [string]$Detail = '') {
    $script:total++
    if ($Ok) { $script:pass++; Write-Output "PASS $Name $Detail" } else { Write-Output "FAIL $Name $Detail"; $script:failures += $Name }
}
if (-not $OutDir) { $OutDir = Join-Path (Split-Path -Parent $PSScriptRoot) ('target/gpu-window-guard-sampler-dryrun/' + (Get-Date -Format 'yyyyMMdd-HHmmss')) }
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Write-Output "dryrun out_dir=$OutDir"
. (Join-Path $PSScriptRoot 'gpu-window-guard.ps1')

# ---- A. unit-suffixed CSV parsing -------------------------------------------------------------
$parserLog = Join-Path $OutDir 'parser-dryrun.log'
$lines = @()
$units = ConvertFrom-GpuCsvLine -Line '4165 MiB, 11875 MiB, 38 %'
$bare = ConvertFrom-GpuCsvLine -Line '4165, 11875, 38'
$lines += "units: used=$($units.used) free=$($units.free) util=$($units.util)"
$lines += "bare:  used=$($bare.used) free=$($bare.free) util=$($bare.util)"
Check 'parser_units' ($units.used -eq 4165 -and $units.free -eq 11875 -and $units.util -eq 38)
Check 'parser_bare' ($bare.used -eq 4165 -and $bare.free -eq 11875 -and $bare.util -eq 38)
$badAbort = $false
try { $null = ConvertFrom-GpuCsvLine -Line '4165 MiB, N/A MiB, 38 %' } catch { $badAbort = $_.Exception.Message -match 'GUARD_ABORT' }
Check 'parser_invalid_is_guard_abort' $badAbort
$emptyAbort = $false
try { $null = ConvertFrom-GpuCsvLine -Line '' } catch { $emptyAbort = $_.Exception.Message -match 'GUARD_ABORT' }
Check 'parser_empty_is_guard_abort' $emptyAbort
$lines | Out-File -LiteralPath $parserLog -Encoding utf8
Write-Output ($lines -join '; ')

# ---- B. real 0.25 s sampler dry-run (>=3 s) ----------------------------------------------------
$sampleCsv = Join-Path $OutDir 'samples.csv'
$samplerLog = Join-Path $OutDir 'sampler-dryrun.log'
$sampler = Start-GuardSampler -CsvPath $sampleCsv -IntervalMs 250
Start-Sleep -Seconds $Seconds
$stats = Stop-GuardSampler -Sampler $sampler -TimeoutSeconds 10
$rows = @(Import-Csv -LiteralPath $sampleCsv)
$seqOk = $true; $monotonic = $true; $parsable = $true
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
$injectLog = Join-Path $OutDir 'injection-dryrun.log'
$inject = @()
$appNone = @()
function Run-Expect([string]$Name, [string]$Expect, [string]$LogPath, [scriptblock]$Action) {
    # $Expect = 'none' (no abort) or a concrete guard reason (free_below_floor /
    # increment_over_limit / sustained_util_without_own_step) asserted from the guard log.
    $caught = $false
    $message = ''
    try { & $Action } catch { $caught = $true; $message = $_.Exception.Message }
    if ($Expect -eq 'none') {
        $script:inject += "case=$Name abort=$caught"
        Check "inject_$Name" (-not $caught) ''
        return
    }
    $logText = if (Test-Path -LiteralPath $LogPath) { [IO.File]::ReadAllText($LogPath) } else { '' }
    $reasonOk = ($logText -match ("guard_stop reason=" + [regex]::Escape($Expect)))
    $script:inject += "case=$Name abort=$caught expect_reason=$Expect reason_seen=$reasonOk"
    Check "inject_$Name" ($caught -and $reasonOk) "expect=$Expect seen=$reasonOk"
}
# normal sequence -> no abort
$log = Join-Path $OutDir 'inject-normal.log'
$state = New-GuardState -Label 'inject-normal' -LogPath $log
Run-Expect 'normal_sequence' 'none' $log {
    $null = Assert-Guard -State $state -Phase 'n1' -Snapshot @{ used = 4165; free = 11875; util = 30 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'n2' -Snapshot @{ used = 4165; free = 11875; util = 40 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'n3' -Snapshot @{ used = 4165; free = 11875; util = 30 } -Apps $appNone
}
# util streak without own step -> abort reason=sustained_util_without_own_step
$log = Join-Path $OutDir 'inject-util.log'
$state = New-GuardState -Label 'inject-util' -LogPath $log
Run-Expect 'util_streak_without_own_step' 'sustained_util_without_own_step' $log {
    $null = Assert-Guard -State $state -Phase 'u1' -Snapshot @{ used = 4165; free = 11875; util = 85 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'u2' -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'u3' -Snapshot @{ used = 4165; free = 11875; util = 88 } -Apps $appNone
}
# own GPU step active -> no abort
$log = Join-Path $OutDir 'inject-ownstep.log'
$state = New-GuardState -Label 'inject-ownstep' -LogPath $log
Run-Expect 'util_with_own_step_active' 'none' $log {
    $null = Assert-Guard -State $state -Phase 'o1' -OwnGpuStepActive -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'o2' -OwnGpuStepActive -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'o3' -OwnGpuStepActive -Snapshot @{ used = 4165; free = 11875; util = 90 } -Apps $appNone
}
# free floor -> abort reason=free_below_floor
$log = Join-Path $OutDir 'inject-free.log'
$state = New-GuardState -Label 'inject-free' -LogPath $log
Run-Expect 'free_below_floor' 'free_below_floor' $log {
    $null = Assert-Guard -State $state -Phase 'f1' -Snapshot @{ used = 4165; free = 10000; util = 10 } -Apps $appNone
}
# VRAM increment -> abort reason=increment_over_limit. This card's 16376 MiB makes increment and free
# floor co-fire, so inject -TotalMib 4000 (limit = min(3072, 2000) = 2000) and keep free=11875.
$log = Join-Path $OutDir 'inject-delta.log'
$state = New-GuardState -Label 'inject-delta' -TotalMib 4000 -LogPath $log
Run-Expect 'increment_over_limit' 'increment_over_limit' $log {
    $null = Assert-Guard -State $state -Phase 'd1' -Snapshot @{ used = 4165; free = 11875; util = 10 } -Apps $appNone
    $null = Assert-Guard -State $state -Phase 'd2' -Snapshot @{ used = 7400; free = 11875; util = 10 } -Apps $appNone
}
$inject | Out-File -LiteralPath $injectLog -Encoding utf8
$inject | ForEach-Object { Write-Output $_ }
Write-Output "ASSERTIONS total=$script:total pass=$script:pass fail=$($script:failures.Count)"
if ($script:failures.Count -gt 0) { Write-Output "DRYRUN RESULT: FAIL ($($script:failures -join ','))"; exit 1 }
Write-Output "DRYRUN RESULT: PASS (parser+sampler+guard-injection)"
