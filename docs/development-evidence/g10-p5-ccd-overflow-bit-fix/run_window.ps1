# G10@P5 修正片 · GPU 窗口执行脚本（authoring-only，窗口内一次性运行；不写 G5 冻结证据目录）。
#
# 用法（仓库根目录，获批窗口后）：
#   powershell -NoProfile -ExecutionPolicy Bypass -File `
#     docs/development-evidence/g10-p5-ccd-overflow-bit-fix/run_window.ps1
#
# 步骤：基线/守卫 → 构建 4 个探针 → 复制回归 case 输入到 target → 修正前设备溢出位 →
#       修正后设备溢出位（两次）→ 上游强制 horizon 捕获（两次）→ G5 探针回归（7 case × 2）
#       → G5 witness 探针回归（3 case × 2）→ 结束基线与峰值。
# 输出（全部落本片证据目录或 target/）：
#   window-log.txt / window-memory-samples.txt / build-report.txt
#   device/<case>/run{1,2}/*.bin          （修正后设备，max_horizon=2）
#   device-before/<case>/run{1,2}/*.bin   （修正前设备，HEAD 源）
#   upstream-forced-horizon-run{1,2}.json （上游 CUDA 强制 horizon 捕获）
#   device-regression/<case>/out{1,2}|witness{1,2}/*.bin（G5 回归重跑输出）
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Set-Location $root
# 全部用绝对路径：`Start-Job` 子进程的 cwd 不可继承，G5 窗口曾因相对路径丢失逐秒采样。
$evidence = Join-Path $root 'docs/development-evidence/g10-p5-ccd-overflow-bit-fix'
$g5Cases = Join-Path $root 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases'
$log = Join-Path $evidence 'window-log.txt'
$samples = Join-Path $evidence 'window-memory-samples.txt'
$work = Join-Path $root 'target/g10-p5-p58'
$python = Join-Path $root 'target/mjwarp-parallel/venv/Scripts/python.exe'
Remove-Item $log, $samples -ErrorAction SilentlyContinue
$started = Get-Date

function Log([string]$text) {
    ('[' + (Get-Date -Format 'HH:mm:ss.fff') + '] ' + $text) | Tee-Object -FilePath $log -Append
}
function Smi([string]$query) {
    ((nvidia-smi --query-gpu=$query --format=csv,noheader 2>&1) -join ' | ')
}
function ComputeApps {
    @((nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1) |
        Where-Object { $_ -and $_ -notmatch 'No running|not supported' })
}
function UsedMiB {
    $value = (nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits 2>$null)
    if (-not $value) { return 0 }
    return [int](($value -join '').Trim())
}
function Assert-Guard([string]$phase, [int]$baseline, [int]$total, [int]$increment) {
    $used = UsedMiB
    $delta = $used - $baseline
    Log ("guard phase={0} used_mib={1} delta_mib={2} increment_mib={3}" -f $phase, $used, $delta, $increment)
    $limit = [Math]::Max($increment, [int]($total / 2))
    if ($delta -ge $limit) {
        Log ("guard_stop phase={0} delta_mib={1} limit_mib={2}" -f $phase, $delta, $limit)
        throw "峰值守卫触发：$phase delta=${delta}MiB ≥ ${limit}MiB"
    }
}
function Copy-Dir([string]$source, [string]$destination) {
    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    & robocopy $source $destination /E /NFL /NDL /NJH /NJS /NC /NS /R:1 /W:1 | Out-Null
}
function Run-Step([string]$name, [scriptblock]$body) {
    Log "step=$name"
    $stepStarted = Get-Date
    & $body
    Log ("step={0} done wall_seconds={1}" -f $name, [math]::Round(((Get-Date) - $stepStarted).TotalSeconds, 1))
}

$gpu = Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu,driver_version'
$total = [int]((Smi 'memory.total') -replace '[^0-9]', '')
$baseline = UsedMiB
$apps = ComputeApps
# WDDM 下 `--query-compute-apps` 会列出桌面/GUI 上下文（Edge/Chrome/explorer 等），
# 不能据此判断；只拦截本任务相关计算进程（python/probe/nvcc/cargo/warp）。
$computeLike = @($apps | Where-Object { $_ -match '(?i)python|probe|nvcc|cargo|warp' })
Log ('head_revision=' + (git rev-parse HEAD))
Log 'window=start'
Log ("baseline_memory=" + $gpu)
Log ("baseline_used_mib=" + $baseline)
Log ("baseline_compute_apps=" + $(if ($apps.Count) { $apps -join ' | ' } else { '<none>' }))
Log ("guard_policy=increment_limit_mib=3072 total_half_limit_mib=" + [int]($total / 2) + " sample_interval_s=0.5")
if ($computeLike.Count -gt 0) {
    Log ("window=abort reason=compute_like_apps_present " + ($computeLike -join ' | '))
    throw '开窗前存在本任务相关的 GPU 计算进程，放弃窗口'
}

$sampler = Start-Job -ScriptBlock {
    param($path)
    while ($true) {
        $value = (nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits 2>$null)
        if ($value) { $value.Trim() | Out-File -Append -FilePath $path -Encoding ascii }
        Start-Sleep -Milliseconds 500
    }
} -ArgumentList $samples

try {
    Run-Step 'build-probes' {
        & (Join-Path $evidence 'build_probes.ps1') 2>&1 | Tee-Object -FilePath $log -Append
        $required = @('overflow_probe_after.exe', 'overflow_probe_before.exe', 'g5_probe.exe',
            'g5_witness_probe.exe')
        foreach ($name in $required) {
            if (-not (Test-Path (Join-Path $work $name))) { throw "构建产物缺失: $name" }
        }
        Log ("probes_built=" + ($required -join ','))
    }

    Run-Step 'prepare-regression-case-inputs' {
        $regressionCases = @('hfield-sphere', 'hfield-capsule', 'sdf-box', 'ellipsoid-box',
            'cylinder-box', 'box-box-margin', 'mesh-mesh')
        foreach ($case in $regressionCases) {
            $destination = Join-Path $work "cases/$case"
            if (Test-Path $destination) { Remove-Item -LiteralPath $destination -Recurse -Force }
            & robocopy (Join-Path $g5Cases $case) $destination /E /XD out1 out2 fmad_false_out1 `
                fmad_false_out2 fmad_true_out1 fmad_true_out2 witness1 witness2 `
                /NFL /NDL /NJH /NJS /NC /NS /R:1 /W:1 | Out-Null
        }
        Log ("regression_cases=" + $regressionCases.Count)
    }
    Assert-Guard 'after-build' $baseline $total 3072

    Run-Step 'overflow-before(HEAD source)' {
        $beforeExe = Join-Path $work 'overflow_probe_before.exe'
        $outRoot = Join-Path $work 'overflow-before'
        foreach ($case in @('cylinder-box', 'box-box-margin', 'mesh-mesh')) {
            $output = & $beforeExe (Join-Path $g5Cases $case) 2 $outRoot 2 2>&1
            $output | Tee-Object -FilePath $log -Append
            if ($LASTEXITCODE -ne 0) { throw "overflow_probe_before $case exit=$LASTEXITCODE" }
        }
        Copy-Dir $outRoot (Join-Path $evidence 'device-before')
    }
    Assert-Guard 'after-overflow-before' $baseline $total 3072

    Run-Step 'overflow-after(worktree source, run1+run2)' {
        $afterExe = Join-Path $work 'overflow_probe_after.exe'
        $outRoot = Join-Path $work 'overflow-after'
        if (Test-Path $outRoot) { Remove-Item -LiteralPath $outRoot -Recurse -Force }
        foreach ($case in @('cylinder-box', 'box-box-margin', 'mesh-mesh')) {
            $output = & $afterExe (Join-Path $g5Cases $case) 2 $outRoot 2 2>&1
            $output | Tee-Object -FilePath $log -Append
            if ($LASTEXITCODE -ne 0) { throw "overflow_probe_after $case exit=$LASTEXITCODE" }
        }
        Copy-Dir $outRoot (Join-Path $evidence 'device')
    }


    foreach ($run in @('1', '2')) {
        Run-Step "upstream-capture-run$run" {
            $out = Join-Path $evidence "upstream-forced-horizon-run$run.json"
            & $python -B (Join-Path $evidence 'capture_forced_horizon.py') --device cuda --out $out 2>&1 |
                Tee-Object -FilePath $log -Append
            if ($LASTEXITCODE -ne 0) { throw "上游捕获 run$run 失败 exit=$LASTEXITCODE" }
        }
        Assert-Guard "after-upstream-run$run" $baseline $total 3072
    }

    Run-Step 'regression-contacts(7 cases x2)' {
        $g5Exe = Join-Path $work 'g5_probe.exe'
        foreach ($case in @('hfield-sphere', 'hfield-capsule', 'sdf-box', 'ellipsoid-box')) {
            $caseDir = Join-Path $work "cases/$case"
            $output = & $g5Exe $caseDir 2 2>&1
            $output | Tee-Object -FilePath $log -Append
            if ($LASTEXITCODE -ne 0) { throw "g5_probe $case exit=$LASTEXITCODE" }
            foreach ($dir in @('out1', 'out2')) {
                Copy-Dir (Join-Path $caseDir $dir) (Join-Path $evidence "device-regression/$case/$dir")
            }
            Copy-Item (Join-Path $caseDir 'probe-report.txt') `
                (Join-Path $evidence "device-regression/$case/probe-report.txt") -Force
        }
    }
    Assert-Guard 'after-regression-contacts' $baseline $total 3072

    Run-Step 'regression-witness(3 cases x2)' {
        $witnessExe = Join-Path $work 'g5_witness_probe.exe'
        foreach ($case in @('cylinder-box', 'box-box-margin', 'mesh-mesh')) {
            $caseDir = Join-Path $work "cases/$case"
            $output = & $witnessExe $caseDir 2 2>&1
            $output | Tee-Object -FilePath $log -Append
            if ($LASTEXITCODE -ne 0) { throw "g5_witness_probe $case exit=$LASTEXITCODE" }
            foreach ($dir in @('witness1', 'witness2')) {
                Copy-Dir (Join-Path $caseDir $dir) (Join-Path $evidence "device-regression/$case/$dir")
            }
            Copy-Item (Join-Path $caseDir 'witness-probe-report.txt') `
                (Join-Path $evidence "device-regression/$case/witness-probe-report.txt") -Force
        }
    }
} finally {
    Stop-Job $sampler -ErrorAction SilentlyContinue
    Remove-Job $sampler -Force -ErrorAction SilentlyContinue
    Log ("after_memory=" + (Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu'))
    $afterApps = ComputeApps
    Log ("after_compute_apps=" + $(if ($afterApps.Count) { $afterApps -join ' | ' } else { '<none>' }))
    Log ("after_used_mib=" + (UsedMiB))
    $values = @(Get-Content $samples -ErrorAction SilentlyContinue |
        Where-Object { $_ -match '^\d+$' } |
        ForEach-Object { [int]($_ -replace '[^0-9]', '') })
    if ($values.Count -gt 0) {
        $peak = ($values | Measure-Object -Maximum).Maximum
        Log ("peak_memory_used_mib=" + $peak)
        Log ("peak_delta_mib=" + ($peak - $baseline))
        Log ("sample_count=" + $values.Count)
        if (($peak - $baseline) -ge [Math]::Max(3072, [int]($total / 2))) {
            Log 'guard_stop_peak=triggered'
        }
    }
    Log ("elapsed_seconds=" + [math]::Round(((Get-Date) - $started).TotalSeconds, 1))
    Log 'window=done'
}
Get-Content $log | Select-Object -Last 25
