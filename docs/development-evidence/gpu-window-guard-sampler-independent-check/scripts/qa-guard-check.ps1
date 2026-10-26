# QA independent check for GPU window guard sampler fix (CPU only; read-only nvidia-smi)
param([string]$OutDir = 'D:\Rust\mjwarp-rs\target\qa-guard\out', [string]$Root = 'D:\Rust\mjwarp-rs')
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$script:failures = @()
function Check([string]$Name, [bool]$Ok, [string]$Detail = '') {
    if ($Ok) { Write-Output "PASS $Name $Detail" } else { Write-Output "FAIL $Name $Detail"; $script:failures += $Name }
}
function Invoke-Inject([scriptblock]$Action) {
    $caught = $false
    $msg = ''
    try { & $Action } catch { $caught = $true; $msg = $_.Exception.Message }
    return [pscustomobject]@{ caught = $caught; msg = $msg }
}
$tpl = Join-Path $Root 'scripts\gpu-window-guard.ps1'
. $tpl

# ---------- A. parser ----------
$u = ConvertFrom-GpuCsvLine -Line '4165 MiB, 11875 MiB, 38 %'
$b = ConvertFrom-GpuCsvLine -Line '4165,11875,38'
$e = ConvertFrom-GpuCsvLine -Line '4165 MiB, 11875 MiB, 38 %, 33'
Check 'parser_units' ($u.used -eq 4165 -and $u.free -eq 11875 -and $u.util -eq 38) "used=$($u.used) free=$($u.free) util=$($u.util)"
Check 'parser_bare' ($b.used -eq 4165 -and $b.free -eq 11875 -and $b.util -eq 38) "used=$($b.used) free=$($b.free) util=$($b.util)"
Check 'parser_extra_col_temperature_ok' ($e.used -eq 4165 -and $e.free -eq 11875 -and $e.util -eq 38) "used=$($e.used)"
$invalid = @(
    @('two_columns', '4165 MiB, 11875 MiB'),
    @('na_column', '4165 MiB, N/A MiB, 38 %'),
    @('non_numeric', 'abc, def, ghi'),
    @('decimal', '4165.5 MiB, 11875 MiB, 38 %'),
    @('negative', '-1 MiB, 11875 MiB, 38 %')
)
foreach ($case in $invalid) {
    $isAbort = $false
    $isInt = $false
    $detail = ''
    try { $null = ConvertFrom-GpuCsvLine -Line $case[1] } catch {
        $detail = $_.Exception.Message
        $isAbort = $detail -match 'GUARD_ABORT'
        $isInt = $detail -match 'Int32|System\.Int32'
    }
    Check ("parser_invalid_" + $case[0]) ($isAbort -and -not $isInt) ("msg=" + ($detail -replace "`r|`n", ' ').Substring(0, [math]::Min(70, $detail.Length)))
}
# empty string: not in the acceptance list; record actual behaviour as observation data
$emptyMsg = ''
try { $null = ConvertFrom-GpuCsvLine -Line '' } catch { $emptyMsg = $_.Exception.Message }
Write-Output ("INFO parser_empty_line_error=" + ($emptyMsg -replace "`r|`n", ' '))
# live nvidia-smi line through the parser (read-only query)
$live = & nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu --format=csv,noheader 2>&1 | Select-Object -First 1
$liveParsed = ConvertFrom-GpuCsvLine -Line ([string]$live)
Check 'parser_live_units_line' ($liveParsed.used -ge 0 -and $liveParsed.free -gt 0 -and $liveParsed.util -ge 0 -and $liveParsed.util -le 100) "line='$live' -> used=$($liveParsed.used) free=$($liveParsed.free) util=$($liveParsed.util)"
$snap = Get-GpuSnapshot
Check 'live_snapshot_via_parser' ([int]$snap.used -ge 0 -and [int]$snap.free -gt 0) "used=$($snap.used) free=$($snap.free) util=$($snap.util)"
$liveApps = @(Get-GpuContextApps)
Write-Output "INFO live_contexts=$($liveApps.Count)"

# ---------- B. thresholds / defaults ----------
$defaults = New-GuardState -Label 'qa-defaults' -LogPath (Join-Path $OutDir 'defaults.log')
Check 'threshold_free_floor_10240' ($defaults.free_floor_mib -eq 10240) "value=$($defaults.free_floor_mib)"
Check 'threshold_increment_3072' ($defaults.increment_limit_mib -eq 3072) "value=$($defaults.increment_limit_mib)"
Check 'threshold_util_80' ($defaults.util_abort_percent -eq 80) "value=$($defaults.util_abort_percent)"
Check 'threshold_util_consecutive_3' ($defaults.util_abort_consecutive -eq 3) "value=$($defaults.util_abort_consecutive)"

$emptyApps = @()
$oneForeign = @()
$oneForeign += [ordered]@{ pid = 999999; name = 'foreign.exe'; path = 'D:\Tools\foreign.exe' }
$oneDesktop = @()
$oneDesktop += [ordered]@{ pid = 999998; name = 'chrome.exe'; path = 'C:\Program Files\Google\Chrome\Application\chrome.exe' }

# ---------- C. guard rules through injected snapshots/apps ----------
$st = New-GuardState -Label 'qa-normal' -LogPath (Join-Path $OutDir 'normal.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase n1 -Snapshot @{ used = 4000; free = 12000; util = 30 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase n2 -Snapshot @{ used = 4000; free = 12000; util = 40 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase n3 -Snapshot @{ used = 4000; free = 12000; util = 30 } -Apps $emptyApps
}
Check 'inject_normal_no_abort' (-not $r.caught) $r.msg
$normalLog = [IO.File]::ReadAllText((Join-Path $OutDir 'normal.log'))
Check 'log_sample_format_unchanged' ($normalLog -match 'guard_sample phase=n1 n=1 used_mib=4000 free_mib=12000 util=30 delta_mib=0 contexts=\d+ own_step=False') 'guard_sample field order'
Write-Output ("INFO injected_empty_apps_log=" + (($normalLog -split "`n" | Where-Object { $_ -match 'guard_baseline' }) -join ''))

$st = New-GuardState -Label 'qa-free-ok' -LogPath (Join-Path $OutDir 'free-ok.log')
$r = Invoke-Inject { $null = Assert-Guard -State $st -Phase f0 -Snapshot @{ used = 4000; free = 10240; util = 10 } -Apps $emptyApps }
Check 'inject_free_10240_not_abort' (-not $r.caught) $r.msg
$st = New-GuardState -Label 'qa-free-bad' -LogPath (Join-Path $OutDir 'free-bad.log')
$r = Invoke-Inject { $null = Assert-Guard -State $st -Phase f0 -Snapshot @{ used = 4000; free = 10239; util = 10 } -Apps $emptyApps }
Check 'inject_free_10239_abort' ($r.caught -and $r.msg -match 'GUARD_ABORT') $r.msg
Check 'log_free_reason' (([IO.File]::ReadAllText((Join-Path $OutDir 'free-bad.log'))) -match 'guard_stop reason=free_below_floor') 'log reason'

$st = New-GuardState -Label 'qa-delta-ok' -LogPath (Join-Path $OutDir 'delta-ok.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase d1 -Snapshot @{ used = 4000; free = 12000; util = 10 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase d2 -Snapshot @{ used = 7071; free = 12000; util = 10 } -Apps $emptyApps
}
Check 'inject_delta_3071_not_abort' (-not $r.caught) $r.msg
$st = New-GuardState -Label 'qa-delta-bad' -LogPath (Join-Path $OutDir 'delta-bad.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase d1 -Snapshot @{ used = 4000; free = 12000; util = 10 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase d2 -Snapshot @{ used = 7072; free = 12000; util = 10 } -Apps $emptyApps
}
Check 'inject_delta_3072_abort' ($r.caught -and $r.msg -match 'GUARD_ABORT') $r.msg
Check 'log_delta_reason' (([IO.File]::ReadAllText((Join-Path $OutDir 'delta-bad.log'))) -match 'guard_stop reason=increment_over_limit') 'log reason'
$stSmall = New-GuardState -Label 'qa-half' -TotalMib 4000 -LogPath (Join-Path $OutDir 'half.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $stSmall -Phase h1 -Snapshot @{ used = 500; free = 12000; util = 10 } -Apps $emptyApps
    $null = Assert-Guard -State $stSmall -Phase h2 -Snapshot @{ used = 2500; free = 12000; util = 10 } -Apps $emptyApps
}
Check 'inject_half_total_limit_2000' ($r.caught -and $r.msg -match '>= 2000') $r.msg

$st = New-GuardState -Label 'qa-util-ok' -LogPath (Join-Path $OutDir 'util-ok.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase u1 -Snapshot @{ used = 4000; free = 12000; util = 79 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase u2 -Snapshot @{ used = 4000; free = 12000; util = 79 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase u3 -Snapshot @{ used = 4000; free = 12000; util = 79 } -Apps $emptyApps
}
Check 'inject_util_79_no_abort' (-not $r.caught) $r.msg
$st = New-GuardState -Label 'qa-util-reset' -LogPath (Join-Path $OutDir 'util-reset.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase r1 -Snapshot @{ used = 4000; free = 12000; util = 85 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase r2 -Snapshot @{ used = 4000; free = 12000; util = 85 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase r3 -Snapshot @{ used = 4000; free = 12000; util = 30 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase r4 -Snapshot @{ used = 4000; free = 12000; util = 85 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase r5 -Snapshot @{ used = 4000; free = 12000; util = 85 } -Apps $emptyApps
}
Check 'inject_util_streak_resets' (-not $r.caught) $r.msg
$st = New-GuardState -Label 'qa-util-bad' -LogPath (Join-Path $OutDir 'util-bad.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase u1 -Snapshot @{ used = 4000; free = 12000; util = 80 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase u2 -Snapshot @{ used = 4000; free = 12000; util = 90 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase u3 -Snapshot @{ used = 4000; free = 12000; util = 88 } -Apps $emptyApps
}
Check 'inject_util_3x80_abort' ($r.caught -and $r.msg -match 'GUARD_ABORT') $r.msg
Check 'log_util_reason' (([IO.File]::ReadAllText((Join-Path $OutDir 'util-bad.log'))) -match 'guard_stop reason=sustained_util_without_own_step') 'log reason'
$st = New-GuardState -Label 'qa-util-own' -LogPath (Join-Path $OutDir 'util-own.log')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase o1 -OwnGpuStepActive -Snapshot @{ used = 4000; free = 12000; util = 95 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase o2 -OwnGpuStepActive -Snapshot @{ used = 4000; free = 12000; util = 95 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase o3 -OwnGpuStepActive -Snapshot @{ used = 4000; free = 12000; util = 95 } -Apps $emptyApps
}
Check 'inject_util_with_own_step_no_abort' (-not $r.caught) $r.msg

$st = New-GuardState -Label 'qa-apps' -LogPath (Join-Path $OutDir 'apps.log') -OwnExeNames @('own.exe')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase a1 -Snapshot @{ used = 4000; free = 12000; util = 10 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase a2 -Snapshot @{ used = 4000; free = 12000; util = 10 } -Apps $oneDesktop
}
Check 'inject_new_desktop_context_no_abort' (-not $r.caught) $r.msg
$st = New-GuardState -Label 'qa-apps-bad' -LogPath (Join-Path $OutDir 'apps-bad.log') -OwnExeNames @('own.exe')
$r = Invoke-Inject {
    $null = Assert-Guard -State $st -Phase a1 -Snapshot @{ used = 4000; free = 12000; util = 10 } -Apps $emptyApps
    $null = Assert-Guard -State $st -Phase a2 -Snapshot @{ used = 4000; free = 12000; util = 10 } -Apps $oneForeign
}
Check 'inject_new_foreign_context_abort' ($r.caught -and $r.msg -match 'foreign GPU context') $r.msg

# ---------- D. sampler persistence (real child process, >=3 s) ----------
$csv = Join-Path $OutDir 'samples.csv'
$sampler = Start-GuardSampler -CsvPath $csv -IntervalMs 250
Start-Sleep -Milliseconds 400
$child = Get-Process -Id $sampler.process.Id -ErrorAction SilentlyContinue
$childPath = if ($child) { $child.Path } else { 'exited-early' }
Write-Output "INFO child_pid=$($sampler.process.Id) child_path=$childPath"
Start-Sleep -Seconds 1
$midRows = @(Import-Csv -LiteralPath $csv -ErrorAction SilentlyContinue)
$midCount = $midRows.Count
Start-Sleep -Milliseconds 2200
$stats = Stop-GuardSampler -Sampler $sampler -TimeoutSeconds 10
$rows = @(Import-Csv -LiteralPath $csv)
$header = ([IO.File]::ReadAllLines($csv))[0]
Check 'sampler_csv_header' ($header -eq 'seq,timestamp,used_mib,free_mib,util_percent,temperature_c') "header=$header"
Check 'sampler_rows_equal_stats' ($stats.samples -eq $rows.Count) "stats=$($stats.samples) csv=$($rows.Count)"
Check 'sampler_at_least_10' ($stats.samples -ge 10) "samples=$($stats.samples)"
Check 'sampler_flushed_before_stop' ($midCount -gt 0) "rows_after_1.4s=$midCount"
$seqOk = $true; $mono = $true; $parse = $true
for ($k = 0; $k -lt $rows.Count; $k++) {
    if ([int]$rows[$k].seq -ne ($k + 1)) { $seqOk = $false }
    if ($rows[$k].timestamp -notmatch '^\d{4}-\d{2}-\d{2}T') { $parse = $false }
    if ($rows[$k].used_mib -notmatch '^\d+$' -or $rows[$k].free_mib -notmatch '^\d+$' -or $rows[$k].util_percent -notmatch '^\d+$' -or $rows[$k].temperature_c -notmatch '^\d+$') { $parse = $false }
    if ($k -gt 0 -and ([datetime]$rows[$k].timestamp -lt [datetime]$rows[$k - 1].timestamp)) { $mono = $false }
}
Check 'sampler_seq_1_n' $seqOk
Check 'sampler_timestamp_monotonic' $mono "first=$($stats.first_ts) last=$($stats.last_ts)"
Check 'sampler_fields_parsable' $parse
$peak = (($rows | ForEach-Object { [int]$_.used_mib }) | Measure-Object -Maximum).Maximum
$freeMin = (($rows | ForEach-Object { [int]$_.free_mib }) | Measure-Object -Minimum).Minimum
$utilMax = (($rows | ForEach-Object { [int]$_.util_percent }) | Measure-Object -Maximum).Maximum
Check 'sampler_peak_matches_csv' ($stats.used_peak_mib -eq $peak) "stats=$($stats.used_peak_mib) csv=$peak"
Check 'sampler_free_min_matches_csv' ($stats.free_min_mib -eq $freeMin) "stats=$($stats.free_min_mib) csv=$freeMin"
Check 'sampler_util_max_matches_csv' ($stats.util_max_percent -eq $utilMax) "stats=$($stats.util_max_percent) csv=$utilMax"
Check 'sampler_process_exited' ([bool]$stats.process_exited) "exited=$($stats.process_exited)"
Check 'sampler_stop_file_removed' (-not (Test-Path -LiteralPath $sampler.stop_file)) $sampler.stop_file
$lockOk = $false
try {
    $handle = [IO.File]::Open($csv, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    $handle.Close()
    $lockOk = $true
} catch { $lockOk = $false }
Check 'sampler_csv_no_open_handle' $lockOk "exclusive open after stop"
Start-Sleep -Milliseconds 500
$loopProcs = @()
if (Get-Command Get-CimInstance -ErrorAction SilentlyContinue) {
    $loopProcs = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -and $_.CommandLine -match 'gpu-guard-sampler-loop\.ps1' })
}
Check 'sampler_no_residual_loop_process' ($loopProcs.Count -eq 0) "residual=$($loopProcs.Count)"
Check 'sampler_residual_counter_zero' ($stats.residual_processes -eq 0) "stats_residual=$($stats.residual_processes)"

# ---------- E. pwsh resolver behaviour (record; fallback claim checked separately) ----------
$pwshCmd = Get-Command pwsh -ErrorAction SilentlyContinue
$toolchains = @(Get-ChildItem -Path (Join-Path $Root 'target\toolchains') -Filter pwsh.exe -Recurse -ErrorAction SilentlyContinue | Select-Object -ExpandProperty FullName)
$resolved = Resolve-GuardPwsh
Write-Output "INFO resolver shell=$($PSVersionTable.PSVersion) pwsh_on_path=$([bool]$pwshCmd) toolchain=$($toolchains -join ';') resolved=$resolved"
Check 'resolve_guard_pwsh_usable' ([bool]$resolved) "resolved=$resolved"

# ---------- summary ----------
Write-Output ("SUMMARY failures=" + $script:failures.Count + " " + ($script:failures -join ','))
if ($script:failures.Count -gt 0) { exit 1 }
Write-Output 'QA GUARD CHECK: PASS'
exit 0
