# Write-lock drill: double launch / renew / non-owner release / stale / force clear.
# Touches only .normify/locks/repo-write.lock; never the git index or other paths.
[CmdletBinding()]
param(
    [string]$Repo = ''
)

$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($Repo)) {
    $Repo = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
}
$lockScript = Join-Path $Repo 'scripts/validate/repo-write-lock.ps1'
$lockPath = Join-Path $Repo '.normify/locks/repo-write.lock'
$underTest = 'powershell'  # Windows PowerShell 5.1

$failures = New-Object System.Collections.Generic.List[string]

function Invoke-Lock {
    param([string]$Action, [string]$Owner, [string]$Reason = 'drill', [int]$Ttl = 90)
    $argv = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $lockScript,
        '-Action', $Action, '-Owner', $Owner, '-Repo', $Repo, '-TtlMinutes', "$Ttl")
    if ($Reason -ne '') { $argv += @('-Reason', $Reason) }
    $raw = & $underTest @argv 2>&1
    $code = $LASTEXITCODE
    return [pscustomobject]@{ Exit = $code; Out = (($raw | ForEach-Object { [string]$_ }) -join "`n") }
}

function Expect {
    param([string]$Label, $Result, [int]$Exit, [string]$Contains)
    $ok = ($Result.Exit -eq $Exit) -and ($Result.Out.Contains($Contains))
    if (-not $ok) { $failures.Add($Label) }
    $verdict = 'FAIL'
    if ($ok) { $verdict = 'PASS' }
    Write-Output ('[{0}] {1}: exit={2} expect_exit={3} expect="{4}"' -f $verdict, $Label, $Result.Exit, $Exit, $Contains)
    Write-Output ('        out: {0}' -f $Result.Out.Replace("`n", ' | '))
}

Write-Output '# write-lock drill'
Write-Output ('repo=' + $Repo)
Write-Output ('under_test=' + $underTest)
Write-Output ('time=' + [DateTime]::Now.ToString('yyyy-MM-ddTHH:mm:sszzz', [Globalization.CultureInfo]::InvariantCulture))

# 0) Windows PowerShell 5.1 parse check (0 errors). This drill itself runs on 5.1, so parsing in place
#    proves compatibility of the lock script with the target shell.
$parseTokens = $null
$parseErrors = $null
[void][System.Management.Automation.Language.Parser]::ParseFile($lockScript, [ref]$parseTokens, [ref]$parseErrors)
$parseCount = 0
if ($null -ne $parseErrors) { $parseCount = $parseErrors.Count }
$parseOk = ($parseCount -eq 0)
if (-not $parseOk) { $failures.Add('ps51_parse') }
$parseVerdict = 'FAIL'
if ($parseOk) { $parseVerdict = 'PASS' }
Write-Output ('[{0}] ps51_parse: ps_version={1} parse_errors={2}' -f $parseVerdict, $PSVersionTable.PSVersion.ToString(), $parseCount)
if ($parseCount -gt 0) {
    $parseErrors | Select-Object -First 3 | ForEach-Object { Write-Output ('        ' + $_.Message) }
}

# 1) clean start
if (Test-Path -LiteralPath $lockPath) { Remove-Item -LiteralPath $lockPath -Force }
$r = Invoke-Lock -Action 'status' -Owner 'arch-chat'
Expect 'status_free' $r 0 'REPO_WRITE_LOCK_FREE'

# 2) session A acquires
$r = Invoke-Lock -Action 'acquire' -Owner 'arch-chat' -Reason 'drill-A'
Expect 'acquire_A' $r 0 'REPO_WRITE_LOCK_ACQUIRED'

# 3) session B is refused (exit 17) and the lock file is unchanged
$before = Get-Content -LiteralPath $lockPath -Raw
$r = Invoke-Lock -Action 'acquire' -Owner 'arch-group' -Reason 'drill-B'
Expect 'acquire_B_held' $r 17 'REPO_WRITE_LOCK_HELD'
$after = Get-Content -LiteralPath $lockPath -Raw
if ($before -ne $after) { $failures.Add('lock_unchanged_on_refusal'); Write-Output '[FAIL] lock_unchanged_on_refusal' }
else { Write-Output '[PASS] lock_unchanged_on_refusal' }

# 4) session A renews
$r = Invoke-Lock -Action 'acquire' -Owner 'arch-chat' -Reason 'drill-A-renew'
Expect 'renew_A' $r 0 'REPO_WRITE_LOCK_RENEWED'

# 5) session B cannot release (exit 18) and the lock survives
$r = Invoke-Lock -Action 'release' -Owner 'arch-group'
Expect 'release_not_owner' $r 18 'REPO_WRITE_LOCK_NOT_OWNER'
if (-not (Test-Path -LiteralPath $lockPath)) { $failures.Add('lock_survives_non_owner_release'); Write-Output '[FAIL] lock_survives_non_owner_release' }
else { Write-Output '[PASS] lock_survives_non_owner_release' }

# 6) session A releases
$r = Invoke-Lock -Action 'release' -Owner 'arch-chat'
Expect 'release_A' $r 0 'REPO_WRITE_LOCK_RELEASED'

# 7) status is FREE again
$r = Invoke-Lock -Action 'status' -Owner 'arch-chat'
Expect 'status_free_again' $r 0 'REPO_WRITE_LOCK_FREE'

# 8) expired lock is cleared automatically: A acquires with TTL 0, B then sees a stale lock and acquires
$r = Invoke-Lock -Action 'acquire' -Owner 'arch-chat' -Reason 'drill-A-expired' -Ttl 0
Expect 'acquire_A_ttl0' $r 0 'REPO_WRITE_LOCK_ACQUIRED'
$r = Invoke-Lock -Action 'acquire' -Owner 'arch-group' -Reason 'drill-B-after-expiry'
Expect 'stale_clear_and_acquire_B' $r 0 'REPO_WRITE_LOCK_STALE_CLEARED'

# 9) break needs a reason; break with a reason clears
$r = Invoke-Lock -Action 'break' -Owner 'arch-chat' -Reason ''
Expect 'break_needs_reason' $r 19 'REPO_WRITE_LOCK_BREAK_NEEDS_REASON'
$r = Invoke-Lock -Action 'break' -Owner 'arch-chat' -Reason 'drill force clear'
Expect 'break_with_reason' $r 0 'REPO_WRITE_LOCK_BROKEN'

# 10) final state is FREE with no leftovers
$r = Invoke-Lock -Action 'status' -Owner 'arch-chat'
Expect 'final_status_free' $r 0 'REPO_WRITE_LOCK_FREE'

Write-Output ''
if ($failures.Count -eq 0) {
    Write-Output 'DRILL_OK all checks passed'
    exit 0
}
Write-Output ('DRILL_FAILED ' + ($failures -join ','))
exit 1
