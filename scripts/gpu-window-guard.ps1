# GPU window guard template (project-wide, refined 2026-10-25).
#
# Why: the guard's object is the GPU resource (exclusive CUDA compute + VRAM), not the mere existence of
# a process. A CPU-only test binary (no CUDA context) must be recorded but must NOT abort the window.
#
# Usage (dot-source from a window script):
#   . scripts/gpu-window-guard.ps1
#   $state = New-GuardState -Label 'my-window' -TotalMib 16376
#   Assert-Guard -State $state -Phase 'preconditions'      # before starting
#   ... run one measured step ...
#   Assert-Guard -State $state -Phase 'after-step-1'       # after each step
#   Complete-Guard -State $state                           # end baseline / residual check
#
# Rules (ABORT when any of these holds):
#   1. A NEW pid appears in `nvidia-smi --query-compute-apps` whose process name does not match the
#      allowed own-exe patterns (default: this task's test exes; baseline pids are always allowed).
#   2. Free VRAM drops below 10240 MiB.
#   3. VRAM increment vs baseline is >= 3072 MiB (or > half of total; the stricter one is used).
# CPU-only processes (cargo/rustc/test exe without CUDA context) are recorded as observations only,
# because they never appear in --query-compute-apps and do not occupy the GPU.
#
# All facts are appended to the guard log; nothing is written to the repo.

function New-GuardState {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [int]$TotalMib = 16376,
        [string[]]$AllowedExePatterns = @('g10_p5_*', 'render_*', 'mjwarp_rs-*'),
        [int]$FreeFloorMib = 10240,
        [int]$IncrementLimitMib = 3072,
        [string]$LogPath = ''
    )
    if (-not $LogPath) { $LogPath = Join-Path $PWD ("window-guard-$Label.log") }
    $state = [ordered]@{
        label = $Label
        total_mib = $TotalMib
        allowed_patterns = $AllowedExePatterns
        baseline_used_mib = 0
        baseline_free_mib = 0
        baseline_cuda_pids = @()
        peak_used_mib = 0
        aborted = $false
        samples = 0
        observations = @()
        log_path = $LogPath
        free_floor_mib = $FreeFloorMib
        increment_limit_mib = $IncrementLimitMib
    }
    "guard_init label=$Label total_mib=$TotalMib increment_limit_mib=$IncrementLimitMib free_floor_mib=$FreeFloorMib" |
        Out-File -LiteralPath $LogPath -Encoding utf8
    return $state
}

function Write-GuardLine {
    param($State, [string]$Message)
    $Message | Out-File -LiteralPath $State.log_path -Append -Encoding utf8
}

function Get-CudaUsage {
    $line = (& nvidia-smi --query-gpu=memory.used,memory.free --format=csv,noheader 2>&1 | Select-Object -First 1)
    $parts = @(($line -split ',') | ForEach-Object { $_.Trim() } | Where-Object { $_ -match '^\d+' })
    if ($parts.Count -lt 2) { throw "GUARD_ABORT: cannot parse nvidia-smi memory line: $line" }
    return @{ used = [int]$parts[0]; free = [int]$parts[1] }
}

function Get-CudaComputeApps {
    $apps = @()
    $rows = & nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1
    foreach ($row in $rows) {
        if ($row -match '^\s*(\d+)\s*,\s*(.+)$') {
            $apps += [ordered]@{ pid = [int]$Matches[1]; name = $Matches[2].Trim() }
        }
    }
    return $apps
}

function Test-OwnExe {
    param($State, [string]$Name)
    foreach ($pattern in $State.allowed_patterns) {
        if ($Name -like $pattern) { return $true }
    }
    return $false
}

function Assert-Guard {
    param(
        [Parameter(Mandatory = $true)]$State,
        [string]$Phase = 'sample'
    )
    $usage = Get-CudaUsage
    $apps = Get-CudaComputeApps
    $pids = @($apps | ForEach-Object { $_.pid })
    if (-not $State.baseline_used_mib) {
        $State.baseline_used_mib = $usage.used
        $State.baseline_free_mib = $usage.free
        $State.baseline_cuda_pids = $pids
        Write-GuardLine -State $State -Message "guard_baseline used_mib=$($usage.used) free_mib=$($usage.free) cuda_pids=$($pids -join ',')"
    }
    $State.samples++
    if ($usage.used -gt $State.peak_used_mib) { $State.peak_used_mib = $usage.used }
    $delta = $usage.used - $State.baseline_used_mib
    Write-GuardLine -State $State -Message "guard_sample phase=$Phase n=$($State.samples) used_mib=$($usage.used) free_mib=$($usage.free) delta_mib=$delta cuda_pids=$($pids -join ',')"

    $foreign = @()
    foreach ($app in $apps) {
        if ($State.baseline_cuda_pids -contains $app.pid) { continue }
        if (Test-OwnExe -State $State -Name $app.name) { continue }
        $foreign += "$($app.pid):$($app.name)"
    }
    if ($foreign.Count -gt 0) {
        $State.aborted = $true
        Write-GuardLine -State $State -Message "guard_stop reason=new_cuda_compute_process apps=$($foreign -join '|')"
        throw "GUARD_ABORT($($State.label)): new CUDA compute process $($foreign -join ',')"
    }
    if ($usage.free -lt $State.free_floor_mib) {
        $State.aborted = $true
        Write-GuardLine -State $State -Message "guard_stop reason=free_below_floor free_mib=$($usage.free) floor=$($State.free_floor_mib)"
        throw "GUARD_ABORT($($State.label)): free VRAM $($usage.free) MiB < $($State.free_floor_mib) MiB"
    }
    $half = [int]($State.total_mib / 2)
    $limit = if ($State.increment_limit_mib -lt $half) { $State.increment_limit_mib } else { $half }
    if ($delta -ge $limit) {
        $State.aborted = $true
        Write-GuardLine -State $State -Message "guard_stop reason=increment_over_limit delta_mib=$delta limit_mib=$limit"
        throw "GUARD_ABORT($($State.label)): VRAM increment $delta MiB >= $limit MiB"
    }
    return @{ used = $usage.used; free = $usage.free; delta = $delta }
}

function Complete-Guard {
    param([Parameter(Mandatory = $true)]$State)
    $usage = Get-CudaUsage
    $apps = Get-CudaComputeApps
    $pids = @($apps | ForEach-Object { $_.pid })
    $residual = @($pids | Where-Object { $State.baseline_cuda_pids -notcontains $_ })
    Write-GuardLine -State $State -Message "guard_end used_mib=$($usage.used) free_mib=$($usage.free) peak_used_mib=$($State.peak_used_mib) samples=$($State.samples) residual_cuda_pids=$($residual -join ',') aborted=$($State.aborted)"
    foreach ($observation in $State.observations) {
        Write-GuardLine -State $State -Message "guard_end_observation $observation"
    }
    if ($residual.Count -gt 0) { throw "GUARD_RESIDUAL($($State.label)): $($residual -join ',')" }
    return @{
        peak_used_mib = $State.peak_used_mib
        samples = $State.samples
        residual = $residual
        log_path = $State.log_path
    }
}
