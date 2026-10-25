# G10@P5 primitive narrowphase 设备片 · GPU 窗口执行脚本（获批后跑；窗口外先做预构建）。
#
# 窗口外前置（脚本不负责编译，窗口内只跑 --ignored）：
#   1) cargo test --locked --features cuda-probe --lib --no-run
#   2) cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher --no-run
#   3) cargo test --locked --features cuda-probe --test g10_p5_primitive_cuda --no-run
#   4) NVRTC 运行库在 target/toolchains/cuda-12.8.1/bin（脚本前置到 PATH）。
#   5) 上游参考复用 QA 已提交 qa-upstream-gpu.json（两捕获逐字节相同）；窗口不新捕获。
#
# 窗口内步骤：
#   A. CPU 侧 NVRTC 冒烟（不创建 CUDA 上下文）：记录拼装源 bytes/sha256、PTX bytes/entries；
#   B. NVRTC launcher 回归：hfield-sphere / sdf-box / ellipsoid-box 三 case。设备源拼装
#      14→16 段、product.cu/source.rs 都改过，本步防拼装/PTX 变化把既有 3 case 带崩；
#   C. primitive 3 case 对照：sphere-box-shallow / sphere-box-deep / capsule-box，
#      参考 QA 上游 GPU 捕获（测试内部重算 G5 输入 23 项 sha256 守卫）。
# 窗口纪律：0.25 s 采样；守卫 = 增量 ≥3072 MiB **或** >总显存 50%，两者取更严（本机即 3072 MiB，
# 50% 口径同时记录作参考）；跑前跑后 query-compute-apps；每步后守卫断言；任一步非 0 退出立即停止
# 并保留 window-output.txt；primitive 按 case 早停（首个失败 case 打印明细后 panic，后续 case 不跑）。
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Set-Location $root
$evidence = Join-Path $root 'docs/development-evidence/g10-p5-primitive-narrowphase-cuda'
$cudaBin = Join-Path $root 'target/toolchains/cuda-12.8.1/bin'
$log = Join-Path $evidence 'window-log.txt'
$outputPath = Join-Path $evidence 'window-output.txt'
$samples = Join-Path $evidence 'window-memory-samples.txt'
$expectedHead = '7e5f4f4392fc0cbfa768e42300ba79a6690d8e50'
Remove-Item $log, $samples, $outputPath -ErrorAction SilentlyContinue
$started = Get-Date
$utf8 = New-Object System.Text.UTF8Encoding($false)

function Log([string]$text) {
    $line = '[' + (Get-Date -Format 'HH:mm:ss.fff') + '] ' + $text
    Write-Output $line
    [System.IO.File]::AppendAllText($log, $line + "`r`n", $utf8)
}
function Log-Raw([string[]]$lines) {
    foreach ($line in $lines) {
        Write-Output $line
        [System.IO.File]::AppendAllText($log, [string]$line + "`r`n", $utf8)
    }
}
function Append-Section([string]$title, [string]$command, [string[]]$lines) {
    $section = @('===== ' + $title + ' =====', 'command=' + $command) + $lines
    [System.IO.File]::AppendAllText($outputPath, (($section -join "`r`n") + "`r`n"), $utf8)
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
    $incrementLimit = 3072
    $halfLimit = [int]($total / 2)
    $overIncrement = $delta -ge $incrementLimit
    $overHalf = $delta -gt $halfLimit
    Log ("guard phase={0} used_mib={1} delta_mib={2} increment_limit_mib={3} half_limit_mib={4}" -f $phase, $used, $delta, $incrementLimit, $halfLimit)
    if ($overIncrement -or $overHalf) {
        Log ("guard_stop phase={0} delta_mib={1} increment_limit_mib={2} half_limit_mib={3}" -f $phase, $delta, $incrementLimit, $halfLimit)
        throw "守卫触发：$phase delta=${delta}MiB（增量 ≥${incrementLimit}MiB 或 >50% 设备 ${halfLimit}MiB）"
    }
}
function Sha256([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }
function NewestExe([string]$pattern) {
    @(Get-ChildItem (Join-Path $root 'target/debug/deps') -Filter $pattern |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1)
}

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
$primitiveExe = NewestExe 'g10_p5_primitive_cuda-*.exe'
$launcherExe = NewestExe 'g10_p5_nvrtc_launcher-*.exe'
$libExe = NewestExe 'mjwarp_rs-*.exe'
$header = @(
    'window=start',
    'task=2026-10-25-g10-p5-primitive-narrowphase-cuda',
    "window_start_head=$headShort full=$head",
    "head_expected=$expectedHead head_match=" + ($head -eq $expectedHead),
    "path_prepend=$cudaBin",
    "nvcc=$nvccLine",
    "nvrtc_dlls=$($nvrtcDlls -join ',')",
    "reference=docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json（QA 已提交、两捕获逐字节相同；窗口内重算 G5 输入 sha256 守卫）",
    "steplist=A:nvrtc-cpu-smoke B:nvrtc-launcher-regression C:primitive-3case"
)
[System.IO.File]::WriteAllText($outputPath, (($header -join "`r`n") + "`r`n"), $utf8)

Log 'window=start'
Log 'task=2026-10-25-g10-p5-primitive-narrowphase-cuda'
Log ("window_start_head=" + $headShort + " full=" + $head)
Log ("head_expected=" + $expectedHead + " head_match=" + ($head -eq $expectedHead))
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
Log ("guard_policy=increment_limit_mib=3072 total_half_limit_mib=" + [int]($total / 2) + " effective_stop_mib=3072 strict=increment sample_interval_s=0.25")
Log ("prebuilt_primitive_test=" + $(if ($primitiveExe.Count) { $primitiveExe[0].FullName + " sha256=" + (Sha256 $primitiveExe[0].FullName) + " mtime=" + $primitiveExe[0].LastWriteTime.ToString('HH:mm:ss') } else { '<missing>' }))
Log ("prebuilt_nvrtc_launcher_test=" + $(if ($launcherExe.Count) { $launcherExe[0].FullName + " sha256=" + (Sha256 $launcherExe[0].FullName) + " mtime=" + $launcherExe[0].LastWriteTime.ToString('HH:mm:ss') } else { '<missing>' }))
Log ("prebuilt_lib_test=" + $(if ($libExe.Count) { $libExe[0].FullName + " sha256=" + (Sha256 $libExe[0].FullName) + " mtime=" + $libExe[0].LastWriteTime.ToString('HH:mm:ss') } else { '<missing>' }))
if ($computeLike.Count -gt 0) {
    Log ("window=abort reason=task_related_compute_apps " + ($computeLike -join ' | '))
    throw '开窗前存在本任务相关的 GPU 计算进程，放弃窗口'
}

$sampler = Start-Job -ScriptBlock {
    param($path)
    while ($true) {
        $value = (nvidia-smi --query-gpu=memory.used,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null)
        if ($value) { (($value -join '') -replace '\s*,\s*', ',') | Out-File -Append -FilePath $path -Encoding ascii }
        Start-Sleep -Milliseconds 250
    }
} -ArgumentList $samples

try {
    Log 'step=verify-preconditions'
    $nvrtcPresent = @(Get-ChildItem $cudaBin -Filter 'nvrtc64_*.dll').Count -gt 0
    Log ("nvrtc_present=" + $nvrtcPresent)
    if (-not $nvrtcPresent) { throw "NVRTC DLL 不在 $cudaBin" }
    if (-not $primitiveExe.Count) { throw '窗口外未预构建 g10_p5_primitive_cuda（先跑 --no-run）' }
    if (-not $launcherExe.Count) { throw '窗口外未预构建 g10_p5_nvrtc_launcher（先跑 --no-run）' }
    if (-not $libExe.Count) { throw '窗口外未预构建 lib 测试（先跑 --no-run）' }
    Assert-Guard 'after-preconditions' $baseline $total

    Log 'step=A-nvrtc-cpu-smoke'
    $smokeCommand = 'cargo test --locked --features cuda-probe --lib nvrtc_smoke -- --include-ignored --nocapture'
    $smoke = @(& cargo test --locked --features cuda-probe --lib nvrtc_smoke -- --include-ignored --nocapture 2>&1)
    $smokeCode = $LASTEXITCODE
    Append-Section 'step=A-nvrtc-cpu-smoke' $smokeCommand $smoke
    Log-Raw $smoke
    Log ("step_a_exit_code=" + $smokeCode + " (expect 0)")
    if ($smokeCode -ne 0) { throw "NVRTC 冒烟失败 exit=$smokeCode（明细见 window-output.txt）" }
    $smokeText = $smoke -join "`n"
    if ($smokeText -match 'nvrtc_smoke_ok ptx_bytes=(\d+) entries=(\d+) source_bytes=(\d+) source_sha256=([0-9a-f]{64})') {
        Log ("device_source_bytes=" + $Matches[3] + " device_source_sha256=" + $Matches[4])
        Log ("ptx_bytes=" + $Matches[1] + " ptx_entries=" + $Matches[2])
        if ($Matches[2] -ne '4') { throw ("PTX 入口数不是 4：" + $Matches[2]) }
    } else {
        Log 'nvrtc_smoke_parse=failed'
        throw 'NVRTC 冒烟输出里找不到 nvrtc_smoke_ok 摘要行'
    }
    Log-Raw @($smoke | Where-Object { $_ -match 'test result:' })
    Assert-Guard 'after-nvrtc-smoke' $baseline $total

    Log 'step=B-nvrtc-launcher-regression(3 cases vs G5 probe out1)'
    $launcherCommand = 'cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher -- --ignored --test-threads=1 --nocapture'
    $launcher = @(& cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher -- --ignored --test-threads=1 --nocapture 2>&1)
    $launcherCode = $LASTEXITCODE
    Append-Section 'step=B-nvrtc-launcher-regression' $launcherCommand $launcher
    Log-Raw $launcher
    Log ("step_b_exit_code=" + $launcherCode + " (expect 0)")
    Assert-Guard 'after-nvrtc-launcher-regression' $baseline $total
    if ($launcherCode -ne 0) { throw "NVRTC launcher 回归失败 exit=$launcherCode（明细见 window-output.txt）" }
    Log-Raw @($launcher | Where-Object { $_ -match 'test result:' })

    Log 'step=C-primitive-3case(vs QA upstream GPU ref; per-case early stop)'
    $primitiveCommand = 'cargo test --locked --features cuda-probe --test g10_p5_primitive_cuda -- --ignored --test-threads=1 --nocapture'
    $primitive = @(& cargo test --locked --features cuda-probe --test g10_p5_primitive_cuda -- --ignored --test-threads=1 --nocapture 2>&1)
    $primitiveCode = $LASTEXITCODE
    Append-Section 'step=C-primitive-3case' $primitiveCommand $primitive
    Log-Raw $primitive
    Log ("step_c_exit_code=" + $primitiveCode + " (expect 0)")
    Assert-Guard 'after-primitive-test' $baseline $total
    if ($primitiveCode -ne 0) { throw "primitive 集成测试失败 exit=$primitiveCode（明细见 window-output.txt）" }
    Log-Raw @($primitive | Where-Object { $_ -match 'test result:' })

    Log 'step=artifacts-sha256'
    $files = @(
        (Join-Path $root 'src/physics/collision/cuda/primitive.cu'),
        (Join-Path $root 'src/physics/collision/cuda/primitive_wrapper.cu'),
        (Join-Path $root 'src/physics/collision/cuda/product.cu'),
        (Join-Path $root 'src/physics/collision/cuda/mod.rs'),
        (Join-Path $root 'src/physics/mod.rs'),
        (Join-Path $root 'src/runtime/collision_gpu.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/types.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/source.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/abi.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/nvrtc_smoke.rs'),
        (Join-Path $root 'src/runtime/collision_gpu/primitive_tests.rs'),
        (Join-Path $root 'tests/g10_p5_primitive_cuda.rs'),
        (Join-Path $root 'tests/g10_p5_nvrtc_launcher.rs'),
        (Join-Path $root 'tests/support/primitive_qa_support.rs'),
        (Join-Path $root 'tests/support/primitive_compare_support.rs'),
        (Join-Path $evidence 'run_window.ps1'),
        $outputPath
    )
    $entries = foreach ($path in $files) {
        if (Test-Path $path) {
            $relative = $path.Substring($root.Length + 1).Replace('\', '/')
            "{0}`t{1}" -f $relative, (Sha256 $path)
        }
    }
    [System.IO.File]::WriteAllLines((Join-Path $evidence 'artifacts-sha256.txt'), $entries, $utf8)
    Log ("artifacts_sha256_entries=" + @($entries).Count)
} finally {
    Stop-Job $sampler -ErrorAction SilentlyContinue
    Remove-Job $sampler -Force -ErrorAction SilentlyContinue
    Log ("after_memory=" + (Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu'))
    $afterApps = ComputeApps
    $afterLike = @($afterApps | Where-Object { $_ -match '(?i)python|probe|nvcc|cargo|warp|mjwarp|nvrtc' })
    Log ("after_compute_apps_count=" + $afterApps.Count)
    Log ("after_compute_apps_task_related=" + $(if ($afterLike.Count) { $afterLike -join ' | ' } else { '<none>' }))
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
        $peakDelta = $peak - $baseline
        $peakHalfLimit = [int]($total / 2)
        if ($peakDelta -ge 3072 -or $peakDelta -gt $peakHalfLimit) {
            Log ("guard_stop_peak=triggered delta_mib=" + $peakDelta + " increment_limit_mib=3072 half_limit_mib=" + $peakHalfLimit)
        } else {
            Log ("guard_peak_ok=delta_mib=" + $peakDelta + " increment_limit_mib=3072 half_limit_mib=" + $peakHalfLimit)
        }
    }
    Log ("elapsed_seconds=" + [math]::Round(((Get-Date) - $started).TotalSeconds, 1))
    Log 'window=done'
}
Get-Content $log | Select-Object -Last 25
