<#
.SYNOPSIS
    Cross-session repo write lock: only one session may write the repository at a time.

.DESCRIPTION
    Lock file: <repo>/.normify/locks/repo-write.lock (key=value lines: pid/owner/host/started/expires/reason).

    Rules (same spirit as the window <out-dir>/.window-lock, but liveness is per session, not per process):
      - acquire: when another owner holds a live (not expired) lock, print REPO_WRITE_LOCK_HELD and exit 17
        without touching the lock file; an expired lock is stale - it is cleared and reported as
        REPO_WRITE_LOCK_STALE_CLEARED; re-acquiring with the same owner renews the lock
        (REPO_WRITE_LOCK_RENEWED).
      - release: only the owner may release, otherwise REPO_WRITE_LOCK_NOT_OWNER and exit 18.
      - status: read only; prints the current lock or REPO_WRITE_LOCK_FREE.
      - break: force clear (use only after confirming the holder stopped writing); prints the cleared
        owner/age, the caller records the reason.

    Why TTL instead of pid: every command runs in a short-lived process, so the acquiring pid is gone
    as soon as acquire returns; using pid liveness would report a writing session as free. Liveness is
    therefore the expires timestamp (default 90 minutes, -TtlMinutes). The pid is still recorded for
    troubleshooting, and every write command re-acquiring the lock renews it.

.PARAMETER Action
    acquire / release / status / break.

.PARAMETER Owner
    Session identity, default "<user>@<host>". Cross-session use should pass an explicit session name
    (for example arch-chat / arch-group).

.PARAMETER Reason
    Why the lock is taken or cleared; written into the lock file (required for break).

.PARAMETER Repo
    Repository root, default two levels above this script (scripts/validate -> repo).

.PARAMETER TtlMinutes
    Lock lifetime in minutes, default 90.

.EXAMPLE
    powershell -NoProfile -File scripts/validate/repo-write-lock.ps1 -Action acquire -Owner arch-chat -Reason "night CPU slice"

.EXAMPLE
    powershell -NoProfile -File scripts/validate/repo-write-lock.ps1 -Action release -Owner arch-chat
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('acquire', 'release', 'status', 'break')]
    [string]$Action,
    [string]$Owner = '',
    [string]$Reason = '',
    [string]$Repo = '',
    [int]$TtlMinutes = 90
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($Repo)) {
    $Repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}
if ([string]::IsNullOrWhiteSpace($Owner)) {
    $Owner = '{0}@{1}' -f $env:USERNAME, $env:COMPUTERNAME
}

$lockDir = Join-Path $Repo '.normify\locks'
$lockPath = Join-Path $lockDir 'repo-write.lock'

function Get-WriteLock {
    if (-not (Test-Path -LiteralPath $lockPath)) {
        return $null
    }
    $map = @{}
    foreach ($line in (Get-Content -LiteralPath $lockPath -Encoding UTF8)) {
        $index = $line.IndexOf('=')
        if ($index -le 0) { continue }
        $key = $line.Substring(0, $index).Trim()
        $value = $line.Substring($index + 1).Trim()
        if ($key -ne '') { $map[$key] = $value }
    }
    return $map
}

function Get-LockField($lock, [string]$name) {
    if ($null -eq $lock) { return '' }
    if (-not $lock.ContainsKey($name)) { return '' }
    return [string]$lock[$name]
}

function Get-LockAgeMinutes($lock) {
    $started = Get-LockField $lock 'started'
    if ($started -eq '') { return -1.0 }
    try {
        $when = [DateTime]::Parse($started, [Globalization.CultureInfo]::InvariantCulture)
        return [Math]::Round(([DateTime]::Now - $when).TotalMinutes, 1)
    } catch {
        return -1.0
    }
}

function Test-LockExpired($lock) {
    $expires = Get-LockField $lock 'expires'
    if ($expires -eq '') { return $true }
    try {
        $when = [DateTime]::Parse($expires, [Globalization.CultureInfo]::InvariantCulture)
        return ($when -le [DateTime]::Now)
    } catch {
        return $true
    }
}

function Remove-WriteLock {
    if (Test-Path -LiteralPath $lockPath) {
        Remove-Item -LiteralPath $lockPath -Force
    }
}

function New-WriteLock([string]$lockOwner, [string]$lockReason) {
    if (-not (Test-Path -LiteralPath $lockDir)) {
        New-Item -ItemType Directory -Force -Path $lockDir | Out-Null
    }
    $now = [DateTime]::Now
    $stamp = 'yyyy-MM-ddTHH:mm:sszzz'
    $lines = @(
        ('pid={0}' -f $PID),
        ('owner={0}' -f $lockOwner),
        ('host={0}' -f $env:COMPUTERNAME),
        ('started={0}' -f $now.ToString($stamp, [Globalization.CultureInfo]::InvariantCulture)),
        ('expires={0}' -f $now.AddMinutes($TtlMinutes).ToString($stamp, [Globalization.CultureInfo]::InvariantCulture)),
        ('reason={0}' -f $lockReason)
    )
    Set-Content -LiteralPath $lockPath -Value $lines -Encoding UTF8
}

$lock = Get-WriteLock

switch ($Action) {
    'status' {
        if ($null -eq $lock) {
            Write-Output 'REPO_WRITE_LOCK_FREE'
            exit 0
        }
        $state = 'held'
        if (Test-LockExpired $lock) { $state = 'expired' }
        Write-Output ('REPO_WRITE_LOCK_{0} pid={1} owner={2} host={3} started={4} expires={5} age_min={6} reason={7}' -f `
            $state.ToUpperInvariant(), (Get-LockField $lock 'pid'), (Get-LockField $lock 'owner'), (Get-LockField $lock 'host'), `
            (Get-LockField $lock 'started'), (Get-LockField $lock 'expires'), (Get-LockAgeMinutes $lock), (Get-LockField $lock 'reason'))
        exit 0
    }
    'acquire' {
        if ($null -ne $lock) {
            $heldOwner = Get-LockField $lock 'owner'
            $heldPid = Get-LockField $lock 'pid'
            if (Test-LockExpired $lock) {
                Write-Output ('REPO_WRITE_LOCK_STALE_CLEARED previous_owner={0} previous_pid={1} age_min={2}' -f `
                    $heldOwner, $heldPid, (Get-LockAgeMinutes $lock))
                Remove-WriteLock
            } elseif ($heldOwner -eq $Owner) {
                New-WriteLock $Owner $Reason
                $renewed = Get-WriteLock
                Write-Output ('REPO_WRITE_LOCK_RENEWED pid={0} owner={1} expires={2}' -f `
                    $PID, $Owner, (Get-LockField $renewed 'expires'))
                exit 0
            } else {
                Write-Output ('REPO_WRITE_LOCK_HELD pid={0} owner={1} started={2} expires={3} age_min={4} reason={5}' -f `
                    $heldPid, $heldOwner, (Get-LockField $lock 'started'), (Get-LockField $lock 'expires'), `
                    (Get-LockAgeMinutes $lock), (Get-LockField $lock 'reason'))
                exit 17
            }
        }
        New-WriteLock $Owner $Reason
        Write-Output ('REPO_WRITE_LOCK_ACQUIRED pid={0} owner={1} ttl_min={2}' -f $PID, $Owner, $TtlMinutes)
        exit 0
    }
    'release' {
        if ($null -eq $lock) {
            Write-Output 'REPO_WRITE_LOCK_FREE'
            exit 0
        }
        $heldOwner = Get-LockField $lock 'owner'
        if ($heldOwner -ne $Owner) {
            Write-Output ('REPO_WRITE_LOCK_NOT_OWNER requested_owner={0} held_owner={1} pid={2}' -f `
                $Owner, $heldOwner, (Get-LockField $lock 'pid'))
            exit 18
        }
        Remove-WriteLock
        Write-Output ('REPO_WRITE_LOCK_RELEASED owner={0}' -f $Owner)
        exit 0
    }
    'break' {
        if ([string]::IsNullOrWhiteSpace($Reason)) {
            Write-Output 'REPO_WRITE_LOCK_BREAK_NEEDS_REASON'
            exit 19
        }
        if ($null -eq $lock) {
            Write-Output 'REPO_WRITE_LOCK_FREE'
            exit 0
        }
        Write-Output ('REPO_WRITE_LOCK_BROKEN previous_owner={0} previous_pid={1} age_min={2} reason={3}' -f `
            (Get-LockField $lock 'owner'), (Get-LockField $lock 'pid'), (Get-LockAgeMinutes $lock), $Reason)
        Remove-WriteLock
        exit 0
    }
}
