# GPU window guard template (project-wide, refined 2026-10-25 v2).
#
# Why (v2): on Windows WDDM, `nvidia-smi --query-compute-apps` reports **GPU context** processes,
# including desktop apps (explorer/chrome/WeChat/Feishu/NVIDIA Overlay) and entries shown as
# `[Insufficient Permissions]`. Counting every new pid as an intruder would abort windows on ordinary
# desktop churn. So v2 classifies by exe path:
#   - desktop/system paths and unknown-permission entries -> observation only;
#   - own task exes (explicit names, or `<repo>/target/` + allowed wildcard) -> allowed with a note;
#   - anything else -> abort, recording pid:name:path.
# Memory guard and sustained utilisation remain the primary signals.
#
# Usage (dot-source from a window script):
#   . scripts/gpu-window-guard.ps1
#   $state = New-GuardState -Label 'my-window' -TotalMib 16376 -OwnExeNames @('g10_p5_nvrtc_launcher-abc.exe')
#   Assert-Guard -State $state -Phase 'preconditions'
#   ... run one measured step ...
#   Assert-Guard -State $state -Phase 'after-step-1' -OwnGpuStepActive
#   Complete-Guard -State $state
#
# Abort rules
#   1. A NEW GPU-context process whose exe path is neither desktop/system-whitelisted nor under the
#      repository `target\` tree (and not an own exe) appears -> abort.
#   2. Free VRAM < 10240 MiB.
#   3. VRAM increment vs baseline >= 3072 MiB (or > half of total; the stricter one wins).
#   4. GPU utilisation >= 80% for 3 consecutive samples while no own GPU step is active.
# CPU-only processes (cargo/rustc/test exe without GPU context) are never abort signals; they are
# invisible to --query-compute-apps and are only mentioned if the caller logs them separately.

function New-GuardState {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [int]$TotalMib = 16376,
        [string[]]$OwnExeNames = @(),
        [string[]]$OwnExePatterns = @(),
        [string]$RepoRoot = '',
        [int]$FreeFloorMib = 10240,
        [int]$IncrementLimitMib = 3072,
        [int]$UtilAbortPercent = 80,
        [int]$UtilAbortConsecutive = 3,
        [string]$LogPath = ''
    )
    if (-not $RepoRoot) { $RepoRoot = Split-Path -Parent $PSScriptRoot }
    if (-not $LogPath) { $LogPath = Join-Path $PWD ("window-guard-$Label.log") }
    if ($OwnExeNames.Count -eq 0 -and $OwnExePatterns.Count -eq 0) {
        $OwnExePatterns = @('mjwarp_rs-*')
    }
    $state = [ordered]@{
        label = $Label
        total_mib = $TotalMib
        own_exe_names = $OwnExeNames
        own_exe_patterns = $OwnExePatterns
        repo_root = $RepoRoot
        baseline_used_mib = 0
        baseline_free_mib = 0
        baseline_cuda_pids = @()
        peak_used_mib = 0
        aborted = $false
        samples = 0
        high_util_streak = 0
        observations = @()
        log_path = $LogPath
        free_floor_mib = $FreeFloorMib
        increment_limit_mib = $IncrementLimitMib
        util_abort_percent = $UtilAbortPercent
        util_abort_consecutive = $UtilAbortConsecutive
    }
    "guard_init label=$Label total_mib=$TotalMib increment_limit_mib=$IncrementLimitMib free_floor_mib=$FreeFloorMib util_abort=${UtilAbortPercent}pct_x${UtilAbortConsecutive} own_names=$($OwnExeNames -join ',') own_patterns=$($OwnExePatterns -join ',') repo_root=$RepoRoot" |
        Out-File -LiteralPath $LogPath -Encoding utf8
    if ($OwnExeNames.Count -eq 0) {
        "guard_wildcard_tradeoff own_patterns=$($OwnExePatterns -join ',') note=wildcard-allows-other-task-test-exes-to-avoid-false-abort; prefer explicit -OwnExeNames per window" |
            Out-File -LiteralPath $LogPath -Append -Encoding utf8
    }
    return $state
}

function Write-GuardLine {
    param($State, [string]$Message)
    $Message | Out-File -LiteralPath $State.log_path -Append -Encoding utf8
}

function Get-GpuSnapshot {
    $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu --format=csv,noheader 2>&1 | Select-Object -First 1)
    $parts = @(($line -split ',') | ForEach-Object { $_.Trim() } | Where-Object { $_ -match '^\d+' })
    if ($parts.Count -lt 3) { throw "GUARD_ABORT: cannot parse nvidia-smi memory/util line: $line" }
    return @{ used = [int]$parts[0]; free = [int]$parts[1]; util = [int]$parts[2] }
}

function Get-GpuContextApps {
    $apps = @()
    $rows = & nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1
    foreach ($row in $rows) {
        if ($row -match '^\s*(\d+)\s*,\s*(.+)$') {
            $name = $Matches[2].Trim()
            $path = ''
            if ($name -notmatch '^\[Insufficient Permissions\]') {
                try {
                    $process = Get-Process -Id ([int]$Matches[1]) -ErrorAction Stop
                    if ($process.Path) { $path = $process.Path }
                } catch { $path = '' }
            }
            $apps += [ordered]@{ pid = [int]$Matches[1]; name = $name; path = $path }
        }
    }
    return $apps
}

function Test-DesktopOrSystemPath {
    param([string]$Name, [string]$Path)
    if ($Name -match '^\[Insufficient Permissions\]') { return $true }
    if (-not $Path) { return $true }   # unknown path cannot be attributed; observe only
    $patterns = @(
        '^C:\\Windows\\',
        '^C:\\Program Files',
        '^C:\\Program Files \(x86\)',
        '^C:\\Users\\[^\\]+\\AppData\\Local\\(Microsoft|Google|Tencent|Feishu|NVIDIA|Programs)\\.*',
        '^C:\\ProgramData\\'
    )
    foreach ($pattern in $patterns) {
        if ($Path -match $pattern) { return $true }
    }
    return $false
}

function Test-OwnExe {
    param($State, $App)
    foreach ($name in $State.own_exe_names) {
        if ($App.name -eq $name) { return $true }
        if ($App.path -and ([System.IO.Path]::GetFileName($App.path) -eq $name)) { return $true }
    }
    $underTarget = $false
    if ($App.path) {
        $targetRoot = Join-Path $State.repo_root 'target'
        if ($App.path.StartsWith($targetRoot, [System.StringComparison]::OrdinalIgnoreCase)) { $underTarget = $true }
    }
    foreach ($pattern in $State.own_exe_patterns) {
        if ($App.name -like $pattern) {
            return ($underTarget -or $State.own_exe_names.Count -eq 0)
        }
    }
    return $false
}

function Assert-Guard {
    param(
        [Parameter(Mandatory = $true)]$State,
        [string]$Phase = 'sample',
        [switch]$OwnGpuStepActive
    )
    $snapshot = Get-GpuSnapshot
    $apps = Get-GpuContextApps
    $pids = @($apps | ForEach-Object { $_.pid })
    if (-not $State.baseline_used_mib) {
        $State.baseline_used_mib = $snapshot.used
        $State.baseline_free_mib = $snapshot.free
        $State.baseline_cuda_pids = $pids
        Write-GuardLine -State $State -Message "guard_baseline used_mib=$($snapshot.used) free_mib=$($snapshot.free) util=$($snapshot.util) contexts=$($pids.Count)"
    }
    $State.samples++
    if ($snapshot.used -gt $State.peak_used_mib) { $State.peak_used_mib = $snapshot.used }
    $delta = $snapshot.used - $State.baseline_used_mib
    Write-GuardLine -State $State -Message "guard_sample phase=$Phase n=$($State.samples) used_mib=$($snapshot.used) free_mib=$($snapshot.free) util=$($snapshot.util) delta_mib=$delta contexts=$($pids.Count) own_step=$([bool]$OwnGpuStepActive)"

    $foreign = @()
    foreach ($app in $apps) {
        if ($State.baseline_cuda_pids -contains $app.pid) { continue }
        if (Test-OwnExe -State $State -App $app) { continue }
        if (Test-DesktopOrSystemPath -Name $app.name -Path $app.path) {
            $State.observations += "desktop_context pid=$($app.pid) name=$($app.name) path=$($app.path)"
            Write-GuardLine -State $State -Message "guard_observation desktop_context pid=$($app.pid) name=$($app.name) path=$($app.path)"
            continue
        }
        $foreign += "$($app.pid):$($app.name):$($app.path)"
    }
    if ($foreign.Count -gt 0) {
        $State.aborted = $true
        Write-GuardLine -State $State -Message "guard_stop reason=foreign_gpu_context apps=$($foreign -join '|')"
        throw "GUARD_ABORT($($State.label)): foreign GPU context $($foreign -join ',')"
    }
    if ($snapshot.free -lt $State.free_floor_mib) {
        $State.aborted = $true
        Write-GuardLine -State $State -Message "guard_stop reason=free_below_floor free_mib=$($snapshot.free) floor=$($State.free_floor_mib)"
        throw "GUARD_ABORT($($State.label)): free VRAM $($snapshot.free) MiB < $($State.free_floor_mib) MiB"
    }
    $half = [int]($State.total_mib / 2)
    $limit = if ($State.increment_limit_mib -lt $half) { $State.increment_limit_mib } else { $half }
    if ($delta -ge $limit) {
        $State.aborted = $true
        Write-GuardLine -State $State -Message "guard_stop reason=increment_over_limit delta_mib=$delta limit_mib=$limit"
        throw "GUARD_ABORT($($State.label)): VRAM increment $delta MiB >= $limit MiB"
    }
    if (-not $OwnGpuStepActive -and $snapshot.util -ge $State.util_abort_percent) {
        $State.high_util_streak++
        Write-GuardLine -State $State -Message "guard_observation sustained_util streak=$($State.high_util_streak) util=$($snapshot.util)"
        if ($State.high_util_streak -ge $State.util_abort_consecutive) {
            $State.aborted = $true
            Write-GuardLine -State $State -Message "guard_stop reason=sustained_util_without_own_step streak=$($State.high_util_streak) util=$($snapshot.util)"
            throw "GUARD_ABORT($($State.label)): GPU util >= $($State.util_abort_percent)% for $($State.high_util_streak) samples without own step"
        }
    } else {
        $State.high_util_streak = 0
    }
    return @{ used = $snapshot.used; free = $snapshot.free; util = $snapshot.util; delta = $delta }
}

function Complete-Guard {
    param([Parameter(Mandatory = $true)]$State)
    $snapshot = Get-GpuSnapshot
    $apps = Get-GpuContextApps
    $pids = @($apps | ForEach-Object { $_.pid })
    $residual = @($pids | Where-Object { $State.baseline_cuda_pids -notcontains $_ })
    Write-GuardLine -State $State -Message "guard_end used_mib=$($snapshot.used) free_mib=$($snapshot.free) peak_used_mib=$($State.peak_used_mib) samples=$($State.samples) residual_context_pids=$($residual -join ',') aborted=$($State.aborted)"
    foreach ($observation in $State.observations) {
        Write-GuardLine -State $State -Message "guard_end_observation $observation"
    }
    return @{
        peak_used_mib = $State.peak_used_mib
        samples = $State.samples
        residual = $residual
        log_path = $State.log_path
    }
}
