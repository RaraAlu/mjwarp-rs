# G6-2 性能窗口 · 一次获批窗口内执行（authoring-only；先报 Arch 转 Chief 双签，批了才跑）。
#
# 用法（仓库根目录，获批后）：
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-acceptance-map\perf\run_window.ps1
#
# 前置（窗口外完成）：build_perf_probe.ps1 已构建 probe_perf.exe；cargo test --locked --lib perf_ --no-run 已构建。
# 步骤：基线/守卫 → 校验探针 → 复制 7 个冻结 case 输入 → 设备 kernel-only + 端到端计时（7 case）
#       → host 纯 CPU 单线程吞吐（cargo test --ignored）→ 聚合 → 结束基线与峰值。
# 输出：perf/window-log.txt、perf/window-memory-samples.txt、perf/device/<case>.json、
#       perf/host/<case>.json、perf/device-perf.json、perf/host-perf.json、perf/upstream-cpu.json（窗口外先跑）。
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root
$evidence = Join-Path $root 'docs/development-evidence/g10-p5-acceptance-map/perf'
$g5Cases = Join-Path $root 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases'
$log = Join-Path $evidence 'window-log.txt'
$samples = Join-Path $evidence 'window-memory-samples.txt'
$work = Join-Path $root 'target/g6-perf'
$probe = Join-Path $work 'probe_perf.exe'
$python = Join-Path $root 'target/mjwarp-parallel/venv/Scripts/python.exe'
$cases = @('hfield-sphere', 'hfield-capsule', 'sdf-box', 'ellipsoid-box', 'cylinder-box',
    'box-box-margin', 'mesh-mesh')
Remove-Item $log, $samples -ErrorAction SilentlyContinue
$started = Get-Date
$logEncoding = New-Object System.Text.UTF8Encoding($false)

# 统一 UTF-8 无 BOM 写日志（Windows PowerShell 5.1 的 Tee-Object 会写 UTF-16，工具不可读）。
function Log([string]$text) {
    $line = '[' + (Get-Date -Format 'HH:mm:ss.fff') + '] ' + $text
    Write-Output $line
    [System.IO.File]::AppendAllText($log, $line + "`r`n", $logEncoding)
}
function Log-Raw([string[]]$lines) {
    foreach ($line in $lines) {
        Write-Output $line
        [System.IO.File]::AppendAllText($log, [string]$line + "`r`n", $logEncoding)
    }
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
# WDDM 下 query-compute-apps 也会列出桌面图形上下文；只拦截本任务相关计算进程。
$computeLike = @($apps | Where-Object { $_ -match '(?i)python|probe|nvcc|cargo|warp' })
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1)
$desktopProcesses = @(Get-Process | Sort-Object -Property WorkingSet64 -Descending |
    Select-Object -First 5 | ForEach-Object { "$($_.ProcessName):$([int]($_.WorkingSet64 / 1MB))MiB" })
$headRevision = (git rev-parse HEAD).Trim()
$headShort = (git rev-parse --short HEAD).Trim()
$productDiff = @(git diff 6510317f..HEAD --stat -- src tests Cargo.toml Cargo.lock 2>&1)
$worktreeStatus = @(git status --short 2>&1)
Log ('head_revision=' + (git rev-parse HEAD))
Log ('window_start_head=' + $headShort + ' full=' + $headRevision)
Log ("product_diff_6510317f_to_head=" + $(if ($productDiff.Count -gt 0) { 'NONEMPTY: ' + ($productDiff -join ' ; ') } else { '<empty>' }))
Log ("window_worktree_changes_count=" + $worktreeStatus.Count)
Log ("window_worktree_changes=" + $(if ($worktreeStatus.Count -gt 0) { ($worktreeStatus -join ' | ') } else { '<clean>' }))
Log 'window=start'
Log ("baseline_memory=" + $gpu)
Log ("baseline_used_mib=" + $baseline)
Log ("baseline_compute_apps=" + $(if ($apps.Count) { $apps -join ' | ' } else { '<none>' }))
Log ("baseline_compute_apps_count=" + $apps.Count + " task_related_count=" + $computeLike.Count)
Log ("cpu=" + $cpu.Name.Trim() + " cores=" + $cpu.NumberOfCores + " logical=" + $cpu.NumberOfLogicalProcessors + " max_clock_mhz=" + $cpu.MaxClockSpeed)
Log ("desktop_process_top5=" + ($desktopProcesses -join ', '))
Log ("desktop_process_count=" + @(Get-Process).Count)
Log ("guard_policy=increment_limit_mib=3072 total_half_limit_mib=" + [int]($total / 2) + " sample_interval_s=0.5 sample_columns=memory_used_mib,util_percent,temp_c")
if ($computeLike.Count -gt 0) {
    Log ("window=abort reason=compute_like_apps_present " + ($computeLike -join ' | '))
    throw '开窗前存在本任务相关的 GPU 计算进程，放弃窗口'
}

$sampler = Start-Job -ScriptBlock {
    param($path)
    while ($true) {
        $value = (nvidia-smi --query-gpu=memory.used,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null)
        if ($value) { (($value -join '') -replace '\s*,\s*', ',') | Out-File -Append -FilePath $path -Encoding ascii }
        Start-Sleep -Milliseconds 500
    }
} -ArgumentList $samples

try {
    Run-Step 'verify-probe' {
        if (-not (Test-Path $probe)) { throw "探针不存在: $probe（窗口外先跑 build_perf_probe.ps1）" }
        $sha = (Get-FileHash -Algorithm SHA256 $probe).Hash.ToLower()
        Log ("probe_binary_sha256=" + $sha)
        if (-not (Select-String -Path (Join-Path $evidence 'build-report.txt') -Pattern ("probe_binary_sha256=" + $sha) -Quiet)) {
            throw '探针 sha256 与 build-report.txt 不符（重新构建后未更新报告）'
        }
        Log 'probe_sha_matches_build_report=true'
    }

    Run-Step 'prepare-case-inputs' {
        $destinationRoot = Join-Path $work 'cases'
        foreach ($case in $cases) {
            $destination = Join-Path $destinationRoot $case
            if (Test-Path $destination) { Remove-Item -LiteralPath $destination -Recurse -Force }
            New-Item -ItemType Directory -Force -Path $destination | Out-Null
            & robocopy (Join-Path $g5Cases $case) $destination /E /NFL /NDL /NJH /NJS /NC /NS /R:1 /W:1 |
                Out-Null
        }
        Log ("perf_cases=" + ($cases -join ','))
    }
    Assert-Guard 'after-prepare' $baseline $total 3072

    Run-Step 'smoke-device-probe' {
        # 先跑一次 1 case × 0 warmup × 1 rep 的冒烟，确认探针与 case 输入在窗口内可运行；
        # 失败即停，避免窗口被后续 7 case 全量测量浪费。
        $caseDir = Join-Path $work 'cases/hfield-sphere'
        $output = & $probe $caseDir --perf 0 1 2>&1
        Log-Raw $output
        if ($LASTEXITCODE -ne 0) { throw "冒烟失败 exit=$LASTEXITCODE" }
        $report = Join-Path $caseDir 'perf-report.json'
        if (-not (Test-Path $report)) { throw "冒烟未产出 $report" }
        $parsed = Get-Content $report -Raw | ConvertFrom-Json
        Log ("smoke_ok case=hfield-sphere kernel_ns={0} e2e_ns={1} nacon={2}" -f `
                $parsed.kernel_ns[0], $parsed.e2e_ns[0], $parsed.nacon)
        Remove-Item $report -Force -ErrorAction SilentlyContinue
    }
    Assert-Guard 'after-smoke' $baseline $total 3072

    foreach ($case in $cases) {
        Run-Step "device-perf:$case" {
            $caseDir = Join-Path $work "cases/$case"
            $output = & $probe $caseDir --perf 5 50 2>&1
            Log-Raw $output
            if ($LASTEXITCODE -ne 0) { throw "probe_perf $case exit=$LASTEXITCODE" }
            $report = Join-Path $caseDir 'perf-report.json'
            if (-not (Test-Path $report)) { throw "缺少 $report" }
            $targetDir = Join-Path $evidence 'device'
            New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
            Copy-Item $report (Join-Path $targetDir "$case.json") -Force
            # 记录该 case 的 nacon/nccd/overflow（与 G5 已验收数字对照，阈值失败即停）。
            $parsed = Get-Content $report -Raw | ConvertFrom-Json
            Log ("device_case={0} nacon={1} nccd={2} overflow={3} kernel_median_ns={4} e2e_median_ns={5}" -f `
                    $case, $parsed.nacon, $parsed.nccd, ($parsed.overflow -join ','), `
                    $parsed.kernel_median_ns, $parsed.e2e_median_ns)
        }
        Assert-Guard "after-device:$case" $baseline $total 3072
    }

    Run-Step 'host-perf(cpu, single-thread)' {
        $hostOut = Join-Path $evidence 'host'
        $env:G6_PERF_OUT = $hostOut
        New-Item -ItemType Directory -Force -Path $hostOut | Out-Null
        $output = & cargo test --locked --lib perf_ -- --ignored --test-threads=1 2>&1
        Log-Raw $output
        if ($LASTEXITCODE -ne 0) { throw "host perf cargo test exit=$LASTEXITCODE" }
        Remove-Item Env:\G6_PERF_OUT -ErrorAction SilentlyContinue
        $count = @(Get-ChildItem $hostOut -Filter '*.json').Count
        if ($count -lt 9) { throw "host perf JSON 不足 9 个（实际 $count）" }
        Log ("host_perf_json_count=" + $count)
    }
    Assert-Guard 'after-host-perf' $baseline $total 3072

    Run-Step 'artifacts-sha256' {
        $files = @(
            (Join-Path $evidence 'perf_probe.cu'),
            (Join-Path $evidence 'perf_probe_common.cu'),
            (Join-Path $evidence 'perf_probe_timing.cu'),
            (Join-Path $evidence 'split_perf_probe.ps1'),
            (Join-Path $evidence 'build_perf_probe.ps1'),
            (Join-Path $evidence 'build-report.txt'),
            (Join-Path $evidence 'run_window.ps1'),
            (Join-Path $evidence 'aggregate_perf.py'),
            (Join-Path $evidence 'upstream-cpu.json'),
            (Join-Path $evidence 'upstream-cpu.log'),
            (Join-Path $root 'src/physics/collision/perf_report.rs'),
            (Join-Path $root 'src/physics/collision.rs'),
            (Join-Path $root 'src/physics/collision/gjk/case_evidence.rs'),
            (Join-Path $root 'src/physics/collision/sdf/tests.rs'),
            (Join-Path $root 'src/physics/collision/hfield/tests.rs'),
            (Join-Path $evidence 'perf_upstream_cpu.py'),
            (Join-Path $evidence 'README.md'),
            (Join-Path $evidence 'perf-summary.json'),
            $probe
        )
        $entries = foreach ($path in $files) {
            if (Test-Path $path) {
                $hash = (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower()
                "{0}`t{1}" -f ($path.Substring($root.Length + 1).Replace('\', '/')), $hash
            }
        }
        $targetPath = Join-Path $evidence 'artifacts-sha256.txt'
        [System.IO.File]::WriteAllLines($targetPath, $entries, (New-Object System.Text.UTF8Encoding($false)))
        Log ("artifacts_sha256_file=" + $targetPath + " entries=" + @($entries).Count)
    }

    Run-Step 'aggregate' {
        $output = & $python -B (Join-Path $evidence 'aggregate_perf.py') 2>&1
        Log-Raw $output
        if ($LASTEXITCODE -ne 0) { throw "聚合失败 exit=$LASTEXITCODE" }
    }
} finally {
    Stop-Job $sampler -ErrorAction SilentlyContinue
    Remove-Job $sampler -Force -ErrorAction SilentlyContinue
    Log ("after_memory=" + (Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu'))
    $afterApps = ComputeApps
    Log ("after_compute_apps=" + $(if ($afterApps.Count) { $afterApps -join ' | ' } else { '<none>' }))
    Log ("after_compute_apps_count=" + $afterApps.Count)
    Log ("after_used_mib=" + (UsedMiB))
    $rows = @(Get-Content $samples -ErrorAction SilentlyContinue |
        Where-Object { $_ -match '^\d+(,\d+)*$' })
    $values = @($rows | ForEach-Object { [int](($_ -split ',')[0]) })
    if ($values.Count -gt 0) {
        $peak = ($values | Measure-Object -Maximum).Maximum
        Log ("peak_memory_used_mib=" + $peak)
        Log ("peak_delta_mib=" + ($peak - $baseline))
        Log ("sample_count=" + $values.Count)
        $utilValues = @($rows | ForEach-Object { $parts = $_ -split ','; if ($parts.Count -ge 2) { [int]$parts[1] } })
        if ($utilValues.Count -gt 0) {
            Log ("util_max_percent=" + (($utilValues | Measure-Object -Maximum).Maximum) +
                " util_avg_percent=" + [math]::Round(($utilValues | Measure-Object -Average).Average, 2))
        }
        $tempValues = @($rows | ForEach-Object { $parts = $_ -split ','; if ($parts.Count -ge 3) { [int]$parts[2] } })
        if ($tempValues.Count -gt 0) {
            Log ("temp_max_c=" + (($tempValues | Measure-Object -Maximum).Maximum))
        }
        if (($peak - $baseline) -ge [Math]::Max(3072, [int]($total / 2))) {
            Log 'guard_stop_peak=triggered'
        }
    }
    Log ("elapsed_seconds=" + [math]::Round(((Get-Date) - $started).TotalSeconds, 1))
    Log 'window=done'
}
Get-Content $log | Select-Object -Last 20
