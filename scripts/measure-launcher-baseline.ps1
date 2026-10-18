[CmdletBinding()]
param(
    [ValidateRange(1, 10)][int]$Samples = 3,
    [switch]$Release,
    [string]$OutputRoot
)
# M0编译开销基线。
# 只读运行既有GPU套件，不改物理结果或容差。
# 冷模式每次使用空驱动JIT缓存；热模式先预热一次。
# 本脚本不启动Python。
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (-not $OutputRoot) { $OutputRoot = Join-Path $root 'docs\evidence\launcher-baseline' }

# 套件与期望通过数；数量漂移即停止。
$suites = [ordered]@{
    resident_kinematics   = 6
    resident_mass_matrix  = 2
    resident_mass_solve   = 2
    resident_rne_bias     = 7
    resident_com_velocity = 6
}

$env:CUDA_PATH = Join-Path $root 'target\toolchains\cuda-12.8.1'
if (-not (Test-Path -LiteralPath (Join-Path $env:CUDA_PATH 'bin'))) { throw '请先准备CUDA探针工具链' }
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
Remove-Item Env:MJWARP_PHYSICS_PTX_DIR -ErrorAction SilentlyContinue
Remove-Item Env:MJWARP_LAUNCHER_BASELINE_DIR -ErrorAction SilentlyContinue

$stamp = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')
$profileName = if ($Release) { 'release' } else { 'debug' }
$runRoot = Join-Path $OutputRoot "$stamp-$profileName"
$scratch = Join-Path $root ("target\launcher-baseline\" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $runRoot, $scratch | Out-Null

function Get-Median([double[]]$Values) {
    if ($Values.Count -eq 0) { return $null }
    $sorted = $Values | Sort-Object
    $mid = [int][Math]::Floor($sorted.Count / 2)
    if ($sorted.Count % 2 -eq 1) { return [double]$sorted[$mid] }
    return ([double]$sorted[$mid - 1] + [double]$sorted[$mid]) / 2
}

function Get-Sum($Items, [string]$Property) {
    $measure = @($Items) | Measure-Object -Property $Property -Sum
    if ($null -eq $measure.Sum) { return 0.0 }
    return [double]$measure.Sum
}

Push-Location $root
try {
    # 构建不计入测量。
    $cargo = @('+1.99.0-x86_64-pc-windows-msvc', 'test', '--locked', '--features', 'cuda-probe', '--no-run', '--message-format=json')
    if ($Release) { $cargo += '--release' }
    $build = [Diagnostics.Stopwatch]::StartNew()
    $messages = & cargo @cargo 2>$null
    if ($LASTEXITCODE -ne 0) { throw '测试构建失败' }
    $buildSeconds = $build.Elapsed.TotalSeconds
    $executables = @{}
    foreach ($line in $messages) {
        if (-not $line.StartsWith('{')) { continue }
        $message = $line | ConvertFrom-Json
        if ($message.reason -eq 'compiler-artifact' -and $message.executable -and $message.profile.test -and
            $suites.Contains($message.target.name)) {
            $executables[$message.target.name] = $message.executable
        }
    }
    foreach ($name in $suites.Keys) {
        if (-not $executables.ContainsKey($name)) { throw "缺少测试程序：$name" }
    }

    $gpu = & nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader 2>&1
    if ($LASTEXITCODE -ne 0) { throw 'GPU信息查询失败' }
    $apps = & nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader 2>&1
    $head = (& git rev-parse HEAD).Trim()
    $dirty = @(& git status --porcelain).Count -gt 0
    $rustc = (& rustc +1.99.0-x86_64-pc-windows-msvc --version).Trim()
    $components = Join-Path $env:CUDA_PATH 'components.json'

    function Invoke-Suite([string]$Name, [string]$Cache, [string]$Records) {
        $env:CUDA_CACHE_PATH = $Cache
        if ($Records) { $env:MJWARP_LAUNCHER_BASELINE_DIR = $Records }
        else { Remove-Item Env:MJWARP_LAUNCHER_BASELINE_DIR -ErrorAction SilentlyContinue }
        $used = (& nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits).Trim()
        $watch = [Diagnostics.Stopwatch]::StartNew()
        $output = & $executables[$Name] --ignored --test-threads=1 2>&1
        $seconds = $watch.Elapsed.TotalSeconds
        $exit = $LASTEXITCODE
        Remove-Item Env:MJWARP_LAUNCHER_BASELINE_DIR -ErrorAction SilentlyContinue
        $match = [regex]::Match(($output -join "`n"), 'test result: ok\. (\d+) passed; (\d+) failed;')
        if ($exit -ne 0 -or -not $match.Success -or [int]$match.Groups[1].Value -ne $suites[$Name] -or
            [int]$match.Groups[2].Value -ne 0) {
            throw "$Name 运行失败或计数不符"
        }
        return [ordered]@{ seconds = $seconds; passed = [int]$match.Groups[1].Value; gpuMemoryUsedMiBBefore = [int]$used }
    }

    # 热模式预热：共享驱动缓存，不记录。
    $warmCache = Join-Path $scratch 'warm-cache'
    New-Item -ItemType Directory -Force $warmCache | Out-Null
    foreach ($name in $suites.Keys) { [void](Invoke-Suite $name $warmCache $null) }

    $runs = @()
    foreach ($sample in 1..$Samples) {
        foreach ($mode in @('driver-cold', 'driver-warm')) {
            foreach ($name in $suites.Keys) {
                $cache = if ($mode -eq 'driver-cold') {
                    $fresh = Join-Path $scratch "cold-$sample-$name"
                    New-Item -ItemType Directory -Force $fresh | Out-Null
                    $fresh
                } else { $warmCache }
                $label = "$mode-$sample-$name"
                $records = Join-Path $runRoot "records\$label"
                $result = Invoke-Suite $name $cache $records
                $lines = @(Get-ChildItem -LiteralPath $records -Filter '*.jsonl' |
                    ForEach-Object { Get-Content -LiteralPath $_.FullName -Encoding utf8 } |
                    ForEach-Object { $_ | ConvertFrom-Json })
                if ($lines.Count -eq 0) { throw "$label 没有编译记录" }
                $first = @($lines | Where-Object { $_.occurrence -eq 1 })
                $repeat = @($lines | Where-Object { $_.occurrence -gt 1 })
                $loadDelta = @($lines | Where-Object { $null -ne $_.deviceFreeBeforeLoad } |
                    ForEach-Object { [double]$_.deviceFreeBeforeLoad - [double]$_.deviceFreeAfterLoad })
                $minFree = (@($lines | ForEach-Object { [double]$_.deviceFreeAfterLoad }) | Measure-Object -Minimum).Minimum
                $compileMs = (Get-Sum $lines 'compileNs') / 1e6
                $loadMs = (Get-Sum $lines 'loadNs') / 1e6
                $checkMs = (Get-Sum $lines 'checkNs') / 1e6
                $runs += [ordered]@{
                    sample = $sample; mode = $mode; suite = $name
                    endToEndSeconds = $result.seconds; passed = $result.passed
                    gpuMemoryUsedMiBBefore = $result.gpuMemoryUsedMiBBefore
                    compiles = $lines.Count
                    uniqueSources = @($lines | Select-Object -ExpandProperty sourceSha256 -Unique).Count
                    uniquePtx = @($lines | Select-Object -ExpandProperty ptxSha256 -Unique).Count
                    repeats = $repeat.Count
                    compileMs = $compileMs; checkMs = $checkMs; loadMs = $loadMs
                    firstCompileMs = (Get-Sum $first 'compileNs') / 1e6
                    repeatCompileMs = (Get-Sum $repeat 'compileNs') / 1e6
                    firstLoadMs = (Get-Sum $first 'loadNs') / 1e6
                    repeatLoadMs = (Get-Sum $repeat 'loadNs') / 1e6
                    compileLoadShare = ($compileMs + $loadMs + $checkMs) / ($result.seconds * 1000)
                    maxLoadDeviceBytes = ($loadDelta | Measure-Object -Maximum).Maximum
                    sumLoadDeviceBytes = ($loadDelta | Measure-Object -Sum).Sum
                    minDeviceFreeBytes = $minFree
                    records = "records/$label"
                }
            }
        }
    }

    # 逐模式与套件取中位数。
    $summary = @()
    foreach ($mode in @('driver-cold', 'driver-warm')) {
        foreach ($name in $suites.Keys) {
            $group = @($runs | Where-Object { $_.mode -eq $mode -and $_.suite -eq $name })
            $row = [ordered]@{ mode = $mode; suite = $name; samples = $group.Count }
            foreach ($key in @('endToEndSeconds', 'compiles', 'uniqueSources', 'repeats', 'compileMs', 'checkMs', 'loadMs',
                    'firstCompileMs', 'repeatCompileMs', 'firstLoadMs', 'repeatLoadMs', 'compileLoadShare', 'maxLoadDeviceBytes')) {
                $row["median_$key"] = Get-Median @($group | ForEach-Object { [double]$_[$key] })
                $row["min_$key"] = (@($group | ForEach-Object { [double]$_[$key] }) | Measure-Object -Minimum).Minimum
                $row["max_$key"] = (@($group | ForEach-Object { [double]$_[$key] }) | Measure-Object -Maximum).Maximum
            }
            $summary += $row
        }
    }

    # 逐入口统计，跨全部运行。
    $all = @(Get-ChildItem -LiteralPath (Join-Path $runRoot 'records') -Recurse -Filter '*.jsonl' | ForEach-Object {
            $mode = if ($_.Directory.Name.StartsWith('driver-cold')) { 'driver-cold' } else { 'driver-warm' }
            Get-Content -LiteralPath $_.FullName -Encoding utf8 | ForEach-Object {
                $record = $_ | ConvertFrom-Json
                $record | Add-Member -NotePropertyName mode -NotePropertyValue $mode -PassThru
            }
        })
    $entries = @()
    foreach ($group in ($all | Group-Object entry | Sort-Object Name)) {
        $items = @($group.Group)
        $cold = @($items | Where-Object { $_.mode -eq 'driver-cold' })
        $entries += [ordered]@{
            entry = $group.Name
            compiles = $items.Count
            sourceSha256 = @($items | Select-Object -ExpandProperty sourceSha256 -Unique)
            ptxSha256 = @($items | Select-Object -ExpandProperty ptxSha256 -Unique)
            median_firstCompileMs = (Get-Median @($items | Where-Object { $_.occurrence -eq 1 } | ForEach-Object { $_.compileNs / 1e6 }))
            median_repeatCompileMs = (Get-Median @($items | Where-Object { $_.occurrence -gt 1 } | ForEach-Object { $_.compileNs / 1e6 }))
            median_coldFirstLoadMs = (Get-Median @($cold | Where-Object { $_.occurrence -eq 1 } | ForEach-Object { $_.loadNs / 1e6 }))
            median_warmFirstLoadMs = (Get-Median @($items | Where-Object { $_.mode -eq 'driver-warm' -and $_.occurrence -eq 1 } | ForEach-Object { $_.loadNs / 1e6 }))
            ptxBytes = ($items | Measure-Object ptxBytes -Maximum).Maximum
        }
    }

    $nvrtc = @($all | ForEach-Object { ($_.nvrtcVersion -join '.') } | Select-Object -Unique)
    $options = @($all | ForEach-Object { ($_.options -join ' ') + " fmad=$($_.fmad) fastMath=$($_.useFastMath)" } | Select-Object -Unique)
    $environment = [ordered]@{
        schema = 1
        task = '2026-10-18-layers-m0-w16'
        stampUtc = $stamp
        profile = $profileName
        commit = $head
        workingTreeDirty = $dirty
        rustc = $rustc
        gpu = "$gpu".Trim()
        computeAppsAtStart = @($apps)
        nvrtcVersions = $nvrtc
        compileOptions = $options
        cudaComponentsSha256 = (Get-FileHash -LiteralPath $components -Algorithm SHA256).Hash.ToLowerInvariant()
        buildSeconds = $buildSeconds
        samplesPerMode = $Samples
        suites = $suites
        modes = [ordered]@{
            'driver-cold' = '每个进程使用空CUDA_CACHE_PATH；驱动需重新JIT'
            'driver-warm' = '共享已预热CUDA_CACHE_PATH；预热运行不记录'
        }
        occurrence = '进程内同一源码摘要的第几次编译；1为进程内首次'
        exclusivity = '未独占GPU：桌面图形进程仍在运行，见computeAppsAtStart'
    }
    $environment | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $runRoot 'environment.json') -Encoding utf8
    $runs | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $runRoot 'runs.json') -Encoding utf8
    $summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $runRoot 'summary.json') -Encoding utf8
    $entries | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $runRoot 'entries.json') -Encoding utf8
    Write-Host "launcher-baseline=$runRoot"
    foreach ($row in $summary) {
        Write-Host ("{0} {1}: e2e={2:N2}s compile={3:N0}ms load={4:N0}ms share={5:P1} compiles={6} unique={7}" -f
            $row.mode, $row.suite, $row.median_endToEndSeconds, $row.median_compileMs, $row.median_loadMs,
            $row.median_compileLoadShare, $row.median_compiles, $row.median_uniqueSources)
    }
}
finally {
    Pop-Location
    Remove-Item Env:CUDA_CACHE_PATH -ErrorAction SilentlyContinue
    Remove-Item Env:MJWARP_LAUNCHER_BASELINE_DIR -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $scratch -Recurse -Force -ErrorAction SilentlyContinue
}
