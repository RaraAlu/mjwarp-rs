# W15/W14 独立复核测试跑批（只编译/跑测试，不启动 GPU 内核）。
# 用法：在仓库内任意位置执行： powershell -NoProfile -File docs/development-evidence/w15-w14-independent-check/run-tests.ps1
$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$cudaBin = Join-Path $root 'target/toolchains/cuda-12.8.1/bin'
$cudaInclude = Join-Path $root 'target/toolchains/cuda-12.8.1/include'
$env:MJWARP_CUDA_INCLUDE = $cudaInclude
$env:PATH = "$cudaBin;$env:PATH"

$cases = @(
    @{ name = 'fmt'; args = @('fmt', '--check') },
    @{ name = 'w15-host'; args = @('test', '--locked', '--features', 'cuda-probe', '--test', 'launcher-warp-headers') },
    @{ name = 'w15-nvrtc'; args = @('test', '--locked', '--features', 'cuda-probe', '--test', 'launcher-warp-headers', '--', '--ignored') },
    @{ name = 'w14-host'; args = @('test', '--locked', '--features', 'cuda-probe', '--test', 'launcher-tile') },
    @{ name = 'w14-nvrtc'; args = @('test', '--locked', '--features', 'cuda-probe', '--test', 'launcher-tile', '--', '--ignored') },
    @{ name = 'artifact-unit'; args = @('test', '--locked', '--features', 'cuda-probe', '--lib', 'runtime::launcher::artifact') },
    @{ name = 'launcher-lib'; args = @('test', '--locked', '--features', 'cuda-probe', '--lib') },
    @{ name = 'clippy-cuda'; args = @('clippy', '--locked', '--all-targets', '--features', 'cuda-probe', '--', '-D', 'warnings') }
)

$results = @()
foreach ($case in $cases) {
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & cargo @($case.args) 2>&1 | Out-String
    $code = $LASTEXITCODE
    $watch.Stop()
    $lines = $output -split "`r?`n" | Where-Object { $_ -match 'test result:|error\[|^error|warning:' } | Select-Object -Last 8
    $results += [pscustomobject]@{
        case     = $case.name
        command  = ('cargo ' + ($case.args -join ' '))
        exit     = $code
        elapsed_ms = $watch.ElapsedMilliseconds
        summary  = ($lines -join "`n")
    }
}

$payload = [pscustomobject]@{
    schema   = 1
    task     = '2026-10-23-w15-w14-independent-check'
    env      = [pscustomobject]@{
        rustc             = (& rustc --version 2>&1 | Out-String).Trim()
        cargo             = (& cargo --version 2>&1 | Out-String).Trim()
        os                = [System.Environment]::OSVersion.VersionString
        mjwarp_cuda_include = $env:MJWARP_CUDA_INCLUDE
        path_prefix       = $cudaBin
        gpu_kernel_launch = $false
    }
    cases    = $results
    all_exit_zero = (@($results | Where-Object { $_.exit -ne 0 }).Count -eq 0)
}

$json = $payload | ConvertTo-Json -Depth 6
$path = Join-Path $PSScriptRoot 'test-runs.json'
[System.IO.File]::WriteAllText($path, $json, (New-Object System.Text.UTF8Encoding($false)))
$results | Format-Table case, exit, elapsed_ms -AutoSize
"ALL_EXIT_ZERO=$($payload.all_exit_zero)"
