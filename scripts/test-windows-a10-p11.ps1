[CmdletBinding()]
param(
    [string]$RecordRoot
)
# A10@P11 Windows 单平台：CUDA 事件计时与设备显存峰值的 debug/release 可复现记录。
# 固定负载与温机/重复次数在探针内固定（tests/a10_p11_gpu_perf_probe.rs），
# 本脚本只负责构建、运行与落盘，不改产品源码/测试/容差/断言。
# 产物：<RecordRoot>/debug.json、release.json、probe-<profile>.log、unit-tests.log。
$ErrorActionPreference = 'Stop'
if (-not ($IsWindows -or $env:OS -eq 'Windows_NT')) { throw '此脚本只验证 Windows' }
$root = Split-Path -Parent $PSScriptRoot
if (-not $RecordRoot) { $RecordRoot = Join-Path $root 'docs\development-evidence\a10-p11-windows' }
New-Item -ItemType Directory -Force -Path $RecordRoot | Out-Null
$logRoot = Join-Path $root 'target\a10-p11-logs'
New-Item -ItemType Directory -Force -Path $logRoot | Out-Null
$toolchain = '+1.99.0-x86_64-pc-windows-msvc'

function Invoke-Logged {
    param([string[]]$Arguments, [string]$Log)
    # 原生程序把进度写到 stderr；Stop 会把它当终止错误，显式用 Continue 并校验退出码。
    $previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & cargo @Arguments 2>&1 | Tee-Object -FilePath $Log | Write-Host
        return $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previous
    }
}

Push-Location $root
try {
    $gpu = (& nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader) -join '; '
    if ($LASTEXITCODE -ne 0) { throw 'GPU信息读取失败' }
    $head = (& git rev-parse HEAD).Trim()
    $dirty = @(& git status --porcelain)
    Write-Host "GPU=$gpu"
    Write-Host "HEAD=$head dirty=$($dirty.Count)"

    foreach ($profile in @('debug', 'release')) {
        $record = Join-Path $RecordRoot "$profile.json"
        $log = Join-Path $RecordRoot "probe-$profile.log"
        Remove-Item -LiteralPath $record -ErrorAction SilentlyContinue
        $arguments = @($toolchain, 'test', '--locked', '--features', 'cuda-probe', '--test', 'a10_p11_gpu_perf_probe')
        if ($profile -eq 'release') { $arguments += '--release' }
        $arguments += @('--', '--ignored', '--test-threads=1', '--nocapture')
        $env:MJWARP_A10P11_RECORD = $record
        $watch = [Diagnostics.Stopwatch]::StartNew()
        try {
            $code = Invoke-Logged -Arguments $arguments -Log $log
        }
        finally {
            Remove-Item Env:MJWARP_A10P11_RECORD -ErrorAction SilentlyContinue
        }
        $seconds = [math]::Round($watch.Elapsed.TotalSeconds, 3)
        if ($code -ne 0) { throw "$profile 探针失败（exit=$code），日志：$log" }
        if (-not (Test-Path -LiteralPath $record)) { throw "$profile 未写出记录：$record" }
        $parsed = Get-Content -LiteralPath $record -Raw -Encoding utf8 | ConvertFrom-Json
        if ($parsed.task -ne '2026-10-11-roadmap-a10-p11') { throw "$profile 记录任务不符" }
        if ($parsed.environment.profile -ne $profile) { throw "$profile 记录档位不符" }
        if (@($parsed.records).Count -lt 2) { throw "$profile 记录阶段不足" }
        foreach ($phase in $parsed.records) {
            if ($phase.timing.count -ne $parsed.plan.repetitions) { throw "$profile 阶段样本数与固定计划不符" }
            if ($phase.timing.min_ns -le 0) { throw "$profile 阶段事件计时必须为正" }
        }
        Write-Host ("{0}: wall={1}s workload={2} plan={3}+{4}" -f $profile, $seconds,
            $parsed.workload, $parsed.plan.warmup, $parsed.plan.repetitions)
        foreach ($phase in $parsed.records) {
            Write-Host ("  {0} median={1}ns min={2}ns max={3}ns peak_device={4}MiB" -f $phase.phase,
                $phase.timing.median_ns, $phase.timing.min_ns, $phase.timing.max_ns,
                [math]::Round($phase.memory.peak_used_bytes / 1MB, 1))
        }
    }

    # 宿主侧单元测试（默认特性，不需要 GPU），作为记录工具本身的自检证据。
    $unitLog = Join-Path $RecordRoot 'unit-tests.log'
    $unitCode = Invoke-Logged -Arguments @($toolchain, 'test', '--locked', '--lib', 'runtime::tooling') -Log $unitLog
    if ($unitCode -ne 0) { throw "宿主侧单元测试失败（exit=$unitCode），日志：$unitLog" }
    Write-Host 'a10-p11 记录完成'
}
finally {
    Pop-Location
}
