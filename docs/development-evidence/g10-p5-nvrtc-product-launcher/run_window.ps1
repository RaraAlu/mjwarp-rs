# G10@P5 NVRTC 产品 launcher · GPU 窗口执行脚本（authoring-only；获批后跑）。
#
# 前置（窗口外完成）：
#   1) `cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher --no-run`
#   2) NVRTC 运行库在 `target/toolchains/cuda-12.8.1/bin`（本脚本把该目录前置到 PATH；
#      没有它 cudarc 找不到 `nvrtc64_*.dll`，NVRTC 编译会失败——窗口与证据必须写清）。
# 步骤：基线/守卫 → 校验 PATH/NVRTC DLL/预构建产物 → 跑 ignored 三 case 对照
#       → 结束基线/峰值/残留 → artifacts sha256。
# 输出：window-log.txt、window-output.txt（测试原始输出）、window-memory-samples.txt、
#       artifacts-sha256.txt。
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Set-Location $root
$evidence = Join-Path $root 'docs/development-evidence/g10-p5-nvrtc-product-launcher'
$cudaBin = Join-Path $root 'target/toolchains/cuda-12.8.1/bin'
$log = Join-Path $evidence 'window-log.txt'
$outputPath = Join-Path $evidence 'window-output.txt'
$samples = Join-Path $evidence 'window-memory-samples.txt'
Remove-Item $log, $samples, $outputPath -ErrorAction SilentlyContinue
$started = Get-Date
$logEncoding = New-Object System.Text.UTF8Encoding($false)

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
function Assert-Guard([string]$phase, [int]$baseline, [int]$total) {
    $used = UsedMiB
    $delta = $used - $baseline
    Log ("guard phase={0} used_mib={1} delta_mib={2}" -f $phase, $used, $delta)
    $limit = [Math]::Max(3072, [int]($total / 2))
    if ($delta -ge $limit) {
        Log ("guard_stop phase={0} delta_mib={1} limit_mib={2}" -f $phase, $delta, $limit)
        throw "峰值守卫触发：$phase delta=${delta}MiB ≥ ${limit}MiB"
    }
}
function Sha256([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }

$env:PATH = $cudaBin + ';' + $env:PATH
$gpu = Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu,driver_version'
$total = [int]((Smi 'memory.total') -replace '[^0-9]', '')
$baseline = UsedMiB
$apps = ComputeApps
$computeLike = @($apps | Where-Object { $_ -match '(?i)python|probe|nvcc|cargo|warp|mjwarp|nvrtc' })
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1)
$desktopProcesses = @(Get-Process | Sort-Object -Property WorkingSet64 -Descending |
    Select-Object -First 5 | ForEach-Object { "$($_.ProcessName):$([int]($_.WorkingSet64 / 1MB))MiB" })
$head = (git rev-parse HEAD).Trim()
$headShort = (git rev-parse --short HEAD).Trim()
$worktree = @(git status --short 2>&1)
$nvccLine = (& (Join-Path $cudaBin 'nvcc.exe') --version 2>&1 | Select-String -Pattern 'release' |
    ForEach-Object { $_.Line.Trim() } | Select-Object -First 1)
$nvrtcDlls = @(Get-ChildItem $cudaBin -Filter 'nvrtc*.dll' | Sort-Object Name |
    ForEach-Object { "$($_.Name)=$(Sha256 $_.FullName)" })
Log 'window=start'
Log 'task=2026-10-25-g10-p5-nvrtc-product-launcher'
Log ("window_start_head=" + $headShort + " full=" + $head)
Log ("path_prepend=" + $cudaBin)
Log ("nvcc=" + $nvccLine)
Log ("nvrtc_dlls=" + ($nvrtcDlls -join ','))
Log ("baseline_memory=" + $gpu)
Log ("baseline_used_mib=" + $baseline)
Log ("baseline_compute_apps_count=" + $apps.Count + " task_related_count=" + $computeLike.Count)
Log ("baseline_compute_apps_task_related=" + $(if ($computeLike.Count) { $computeLike -join ' | ' } else { '<none>' }))
Log ("cpu=" + $cpu.Name.Trim() + " cores=" + $cpu.NumberOfCores + " logical=" + $cpu.NumberOfLogicalProcessors)
Log ("desktop_process_top5=" + ($desktopProcesses -join ', '))
Log ("window_worktree_changes_count=" + $worktree.Count)
Log ("guard_policy=increment_limit_mib=3072 total_half_limit_mib=" + [int]($total / 2) + " sample_interval_s=0.5")
if ($computeLike.Count -gt 0) {
    Log ("window=abort reason=task_related_compute_apps " + ($computeLike -join ' | '))
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
    Log 'step=verify-preconditions'
    $nvrtcPresent = @(Get-ChildItem $cudaBin -Filter 'nvrtc64_*.dll').Count -gt 0
    Log ("nvrtc_present=" + $nvrtcPresent)
    if (-not $nvrtcPresent) { throw "NVRTC DLL 不在 $cudaBin" }
    $testBinary = @(Get-ChildItem (Join-Path $root 'target/debug/deps') -Filter 'g10_p5_nvrtc_launcher-*.exe' |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1)
    Log ("prebuilt_test_binary=" + $(if ($testBinary.Count) { $testBinary[0].FullName } else { '<missing>'}))
    if (-not $testBinary.Count) { throw '窗口外未预构建集成测试（先跑 --no-run）' }
    Assert-Guard 'after-preconditions' $baseline $total

    Log 'step=run-ignored-launcher-test(3 cases)'
    $previousEap = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $result = @(& cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher `
            -- --ignored --test-threads=1 --nocapture 2>&1)
    $code = $LASTEXITCODE
    $ErrorActionPreference = $previousEap
    $header = @(
        "window_head=$head",
        "window_head_short=$headShort",
        "path_prepend=$cudaBin",
        "nvcc=$nvccLine",
        "nvrtc_dlls=$($nvrtcDlls -join ',')",
        "command=cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher -- --ignored --test-threads=1 --nocapture",
        "exit_code=$code"
    )
    [System.IO.File]::WriteAllLines($outputPath, ($header + $result), $logEncoding)
    Log-Raw $result
    Log ("launcher_test_exit_code=" + $code + " (expect 0)")
    Assert-Guard 'after-launcher-test' $baseline $total
    if ($code -ne 0) { throw "launcher 集成测试失败 exit=$code（明细见 window-output.txt）" }

    Log 'step=artifacts-sha256'
    $files = @(
        (Join-Path $root 'src/runtime/collision_gpu.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/source.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/error.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/abi.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/tests.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/nvrtc_smoke.rs'),
        (Join-Path $root 'src/physics/collision/cuda/product.cu'),
        (Join-Path $root 'tests/g10_p5_nvrtc_launcher.rs'),
        (Join-Path $evidence 'run_window.ps1'),
        $outputPath
    )
    $entries = foreach ($path in $files) {
        if (Test-Path $path) {
            $relative = $path.Substring($root.Length + 1).Replace('\', '/')
            "{0}`t{1}" -f $relative, (Sha256 $path)
        }
    }
    [System.IO.File]::WriteAllLines((Join-Path $evidence 'artifacts-sha256.txt'), $entries, $logEncoding)
    Log ("artifacts_sha256_entries=" + @($entries).Count)
} finally {
    Stop-Job $sampler -ErrorAction SilentlyContinue
    Remove-Job $sampler -Force -ErrorAction SilentlyContinue
    Log ("after_memory=" + (Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu'))
    $afterApps = ComputeApps
    Log ("after_compute_apps_count=" + $afterApps.Count)
    Log ("after_compute_apps_task_related=" +
        $(if (@($afterApps | Where-Object { $_ -match '(?i)python|probe|nvcc|cargo|warp|mjwarp|nvrtc' }).Count) {
            (@($afterApps | Where-Object { $_ -match '(?i)python|probe|nvcc|cargo|warp|mjwarp|nvrtc' }) -join ' | ')
        } else { '<none>' }))
    Log ("after_used_mib=" + (UsedMiB))
    $rows = @(Get-Content $samples -ErrorAction SilentlyContinue | Where-Object { $_ -match '^\d+(,\d+)*$' })
    $values = @($rows | ForEach-Object { [int](($_ -split ',')[0]) })
    if ($values.Count -gt 0) {
        $peak = ($values | Measure-Object -Maximum).Maximum
        Log ("peak_memory_used_mib=" + $peak)
        Log ("peak_delta_mib=" + ($peak - $baseline))
        Log ("sample_count=" + $values.Count)
        $utilValues = @($rows | ForEach-Object { $parts = $_ -split ','; if ($parts.Count -ge 2) { [int]$parts[1] } })
        if ($utilValues.Count -gt 0) {
            Log ("util_max_percent=" + (($utilValues | Measure-Object -Maximum).Maximum))
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
