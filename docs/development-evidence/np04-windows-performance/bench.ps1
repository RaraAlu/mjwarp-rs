[CmdletBinding()]
param(
    [ValidateRange(1, 20)][int]$Samples = 5,
    [switch]$Release,
    [string]$OutputRoot,
    [string[]]$Suites,
    [switch]$WarmOnly,
    [switch]$ColdOnly
)
# NP04 Windows 单平台性能与峰值显存采样（只读运行既有 GPU 测试；不改产品/测试/容差）。
# 固定输入：仓库 fixtures（工作区干净时与 HEAD 绑定）；固定精度：产品 f32 既有容差不变。
# 每个套件 = 一个既有 GPU 测试二进制；每样本一个新进程（冷启动口径），
# 冷/热两种驱动缓存分开：driver-cold = 每次全新空 CUDA_CACHE_PATH；driver-warm = 共享预热缓存。
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
if (-not $OutputRoot) { $OutputRoot = Join-Path $root 'docs\development-evidence\np04-windows-performance' }
$expected = [ordered]@{
    resident_kinematics       = 6
    resident_mass_matrix      = 2
    resident_mass_solve       = 2
    resident_rne_bias         = 7
    resident_com_velocity     = 6
    resident_rne_acceleration = 2
    mass_solve_probe          = 6
    g01                       = 7
    g26_energy_chain          = 1
}
$suiteArgs = @{ g01 = @('--include-ignored', '--test-threads=1') }
foreach ($name in $expected.Keys) {
    if (-not $suiteArgs.ContainsKey($name)) { $suiteArgs[$name] = @('--ignored', '--test-threads=1') }
}
if (-not $Suites) { $Suites = @($expected.Keys) }
foreach ($name in $Suites) { if (-not $expected.Contains($name)) { throw "未知套件：$name" } }

$env:CUDA_PATH = Join-Path $root 'target\toolchains\cuda-12.8.1'
if (-not (Test-Path -LiteralPath (Join-Path $env:CUDA_PATH 'bin'))) { throw '请先准备CUDA探针工具链' }
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
$env:MJWARP_CUDA_INCLUDE = Join-Path $env:CUDA_PATH 'include'
$env:MJWARP_MUJOCO_ROOT = Join-Path $root 'target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = Join-Path $env:MJWARP_MUJOCO_ROOT 'bin\mujoco.dll'
Remove-Item Env:MJWARP_PHYSICS_PTX_DIR -ErrorAction SilentlyContinue

$stamp = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')
$profile = if ($Release) { 'release' } else { 'debug' }
$runRoot = Join-Path $OutputRoot "raw\$profile\$stamp"
$logRoot = Join-Path $root "target\np04-logs\$stamp-$profile"
$scratch = Join-Path $root ("target\np04-scratch\" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $runRoot, $logRoot, $scratch | Out-Null

function Get-Median([double[]]$Values) {
    if ($Values.Count -eq 0) { return $null }
    $sorted = @($Values | Sort-Object)
    $mid = [int][Math]::Floor($sorted.Count / 2)
    if ($sorted.Count % 2 -eq 1) { return [double]$sorted[$mid] }
    return ([double]$sorted[$mid - 1] + [double]$sorted[$mid]) / 2
}

function Get-GpuUsedMiB {
    $value = (& nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits 2>$null | Select-Object -First 1)
    if ($null -eq $value) { return $null }
    return [int]("$value".Trim())
}

Push-Location $root
try {
    $cargo = @('+1.99.0-x86_64-pc-windows-msvc', 'test', '--locked', '--all-features', '--no-run', '--message-format=json')
    if ($Release) { $cargo += '--release' }
    $buildWatch = [Diagnostics.Stopwatch]::StartNew()
    $messages = & cargo @cargo 2>$null
    if ($LASTEXITCODE -ne 0) { throw '测试构建失败' }
    $buildSeconds = $buildWatch.Elapsed.TotalSeconds
    $executables = @{}
    foreach ($line in $messages) {
        if (-not "$line".StartsWith('{')) { continue }
        $message = $line | ConvertFrom-Json
        if ($message.reason -eq 'compiler-artifact' -and $message.executable -and $message.profile.test -and
            $expected.Contains($message.target.name)) {
            $executables[$message.target.name] = $message.executable
        }
    }
    foreach ($name in $Suites) { if (-not $executables.ContainsKey($name)) { throw "缺少测试程序：$name" } }

    $gpu = (& nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader 2>&1 | Out-String).Trim()
    $computeApps = @(& nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1 | ForEach-Object { "$_".Trim() })
    $head = (& git rev-parse HEAD).Trim()
    $dirty = @(& git status --porcelain) -join "`n"
    $fixturesTree = (& git rev-parse 'HEAD:fixtures').Trim()
    $testsTree = (& git rev-parse 'HEAD:tests').Trim()
    $srcTree = (& git rev-parse 'HEAD:src').Trim()
    $rustc = (& rustc +1.99.0-x86_64-pc-windows-msvc --version).Trim()
    $nvcc = (& nvcc --version 2>&1 | Select-String 'release' | Select-Object -First 1)
    $os = (& Get-CimInstance Win32_OperatingSystem | ForEach-Object { "$($_.Caption) $($_.Version) build $($_.BuildNumber)" })
    $cpu = (& Get-CimInstance Win32_Processor | Select-Object -First 1).Name
    $ramMiB = [int]((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1MB)
    $components = Join-Path $env:CUDA_PATH 'components.json'

    $warmCache = Join-Path $scratch 'warm-cache'
    New-Item -ItemType Directory -Force $warmCache | Out-Null

    function Invoke-Suite([string]$Name, [string]$CachePath, [string]$Label) {
        $exe = $executables[$Name]
        $outLog = Join-Path $logRoot "$Label.out.log"
        $errLog = Join-Path $logRoot "$Label.err.log"
        $env:CUDA_CACHE_PATH = $CachePath
        $idleMiB = Get-GpuUsedMiB
        $watch = [Diagnostics.Stopwatch]::StartNew()
        $process = Start-Process -FilePath $exe -ArgumentList $suiteArgs[$Name] -NoNewWindow -PassThru `
            -RedirectStandardOutput $outLog -RedirectStandardError $errLog
        $peakGpuMiB = $idleMiB
        $peakHostMiB = 0.0
        $gpuSamples = 0
        $hostSamples = 0
        while (-not $process.HasExited) {
            $process.Refresh()
            $hostMiB = [double]$process.WorkingSet64 / 1MB
            if ($hostMiB -gt $peakHostMiB) { $peakHostMiB = $hostMiB }
            $hostSamples++
            $used = Get-GpuUsedMiB
            if ($null -ne $used) {
                if ($used -gt $peakGpuMiB) { $peakGpuMiB = $used }
                $gpuSamples++
            }
            Start-Sleep -Milliseconds 100
        }
        $seconds = $watch.Elapsed.TotalSeconds
        $exit = $process.ExitCode
        $process.Refresh()
        $hostMiB = [double]$process.WorkingSet64 / 1MB
        if ($hostMiB -gt $peakHostMiB) { $peakHostMiB = $hostMiB }
        Remove-Item Env:CUDA_CACHE_PATH -ErrorAction SilentlyContinue
        $text = @()
        foreach ($path in @($outLog, $errLog)) {
            if (Test-Path -LiteralPath $path) { $text += Get-Content -LiteralPath $path -Encoding utf8 }
        }
        $joined = $text -join "`n"
        $match = [regex]::Match($joined, 'test result: ok\. (\d+) passed; (\d+) failed;')
        $passed = if ($match.Success) { [int]$match.Groups[1].Value } else { -1 }
        $failed = if ($match.Success) { [int]$match.Groups[2].Value } else { -1 }
        if ($exit -ne 0 -or -not $match.Success -or $passed -ne $expected[$Name] -or $failed -ne 0) {
            throw "$Label 运行失败或计数不符（exit=$exit passed=$passed failed=$failed）日志：$outLog"
        }
        return [ordered]@{
            label = $Label; suite = $Name; seconds = [math]::Round($seconds, 3)
            passed = $passed; failed = $failed; exitCode = $exit
            gpuUsedIdleMiB = $idleMiB; gpuUsedPeakMiB = $peakGpuMiB
            gpuPeakDeltaMiB = if ($null -ne $idleMiB -and $peakGpuMiB -gt $idleMiB) { $peakGpuMiB - $idleMiB } else { 0 }
            gpuSamples = $gpuSamples; hostPeakWorkingSetMiB = [math]::Round($peakHostMiB, 1)
            hostSamples = $hostSamples; cacheMode = if ($CachePath -eq $warmCache) { 'driver-warm' } else { 'driver-cold' }
        }
    }

    # 预热（共享热缓存；结果不记录，但计数仍校验）。
    if (-not $ColdOnly) {
        foreach ($name in $Suites) { [void](Invoke-Suite $name $warmCache "prewarm-$name") }
    }

    $runs = @()
    $modes = if ($WarmOnly) { @('driver-warm') } elseif ($ColdOnly) { @('driver-cold') } else { @('driver-cold', 'driver-warm') }
    foreach ($sample in 1..$Samples) {
        foreach ($mode in $modes) {
            foreach ($name in $Suites) {
                if ($mode -eq 'driver-cold') {
                    $cache = Join-Path $scratch ("cold-$sample-$name")
                    New-Item -ItemType Directory -Force $cache | Out-Null
                }
                else { $cache = $warmCache }
                $label = "$mode-$sample-$name"
                $result = Invoke-Suite $name $cache $label
                $runs += $result
                Write-Host ("{0} {1} {2:N3}s peakGpu+{3}MiB host {4}MiB" -f $mode, $name,
                    $result.seconds, $result.gpuPeakDeltaMiB, $result.hostPeakWorkingSetMiB)
            }
        }
    }

    $summary = @()
    foreach ($mode in $modes) {
        foreach ($name in $Suites) {
            $group = @($runs | Where-Object { $_.cacheMode -eq $mode -and $_.suite -eq $name })
            $row = [ordered]@{ cacheMode = $mode; suite = $name; samples = $group.Count; expectedPassed = $expected[$name] }
            foreach ($key in @('seconds', 'gpuPeakDeltaMiB', 'hostPeakWorkingSetMiB')) {
                $values = @($group | ForEach-Object { [double]$_[$key] })
                $row["median_$key"] = [math]::Round((Get-Median $values), 3)
                $row["min_$key"] = [math]::Round((($values | Measure-Object -Minimum).Minimum), 3)
                $row["max_$key"] = [math]::Round((($values | Measure-Object -Maximum).Maximum), 3)
            }
            $row.samplesSeconds = @($group | ForEach-Object { [math]::Round($_.seconds, 3) })
            $row.samplesGpuPeakDeltaMiB = @($group | ForEach-Object { $_.gpuPeakDeltaMiB })
            $summary += $row
        }
    }

    $environment = [ordered]@{
        schema = 1
        task = '2026-10-23-np04-windows-performance'
        stampUtc = $stamp
        profile = $profile
        commit = $head
        workingTreePorcelain = $dirty
        fixturesTree = $fixturesTree
        testsTree = $testsTree
        srcTree = $srcTree
        rustc = $rustc
        nvcc = "$nvcc".Trim()
        os = $os
        cpu = $cpu
        ramMiB = $ramMiB
        gpu = $gpu
        computeAppsAtStart = $computeApps
        cudaComponentsSha256 = if (Test-Path -LiteralPath $components) { (Get-FileHash -LiteralPath $components -Algorithm SHA256).Hash.ToLowerInvariant() } else { $null }
        buildSeconds = [math]::Round($buildSeconds, 3)
        samplesPerMode = $Samples
        suites = $Suites
        expectedPassed = $expected
        modes = [ordered]@{
            'driver-cold' = '每个样本进程使用全新空 CUDA_CACHE_PATH，含驱动 JIT 冷启动'
            'driver-warm' = '共享已预热 CUDA_CACHE_PATH（预热运行不记录），代表稳态口径'
        }
        elapsedBoundary = '进程墙钟：含进程启动、PTX JIT/加载与测试内全部 GPU 批量；构建不计入'
        memoryBoundary = '峰值显存为 nvidia-smi memory.used 采样（约100ms）相对样本前空闲值的增量，含驱动上下文分配；桌面图形进程仍在运行，非独占GPU'
        hostMemoryBoundary = '宿主峰值取进程 WorkingSet64 采样最大值'
        precision = '产品 f32；既有容差与断言未改动'
    }
    $environment | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $runRoot 'environment.json') -Encoding utf8
    $runs | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $runRoot 'runs.json') -Encoding utf8
    $summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $runRoot 'summary.json') -Encoding utf8
    Write-Host "np04-output=$runRoot"
    foreach ($row in $summary) {
        Write-Host ("{0} {1}: median={2:N2}s min={3:N2}s max={4:N2}s peakGpu+{5}MiB host={6}MiB" -f $row.cacheMode, $row.suite,
            $row.median_seconds, $row.min_seconds, $row.max_seconds, $row.median_gpuPeakDeltaMiB, $row.median_hostPeakWorkingSetMiB)
    }
}
finally {
    Pop-Location
    Remove-Item -LiteralPath $scratch -Recurse -Force -ErrorAction SilentlyContinue
}
