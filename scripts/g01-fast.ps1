# G01 开发快验：只用于开发迭代，不替代 test-windows-g01.ps1。
# 一次构建后直接运行已构建的测试二进制；支持按 target 或测试名过滤、可选 Release。
# 不检查 fixture/样本哈希、不跑排除清单、不归档验收证据、不改断言与容差。
[CmdletBinding()]
param(
    [string[]]$Target = @(),
    [string]$Test = '',
    [switch]$AllFeatures,
    [switch]$Release,
    [switch]$NoBuild,
    [switch]$BuildOnly,
    [switch]$List,
    [switch]$IncludeIgnored,
    [switch]$IgnoredOnly,
    [switch]$Nocapture,
    [int]$TestThreads = 1,
    [int]$Parallel = 1,
    [string]$LogDir = ''
)
$ErrorActionPreference = 'Stop'
# -File 调用时数组参数会退化成一个逗号串，这里兼容拆分。
if ($Target.Count -eq 1 -and $Target[0] -like '*,*') { $Target = @($Target[0] -split ',') }
$toolchain = '1.99.0-x86_64-pc-windows-msvc'
$root = Split-Path -Parent $PSScriptRoot
$mode = if ($Release) { 'release' } else { 'debug' }
$featureTag = if ($AllFeatures) { 'all' } else { 'g01' }
$manifestDir = Join-Path $root 'target\g01-fast'
$manifestPath = Join-Path $manifestDir "manifest-$mode-$featureTag.json"
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
if ([string]::IsNullOrWhiteSpace($LogDir)) {
    $LogDir = Join-Path $manifestDir "$mode-$featureTag-$stamp"
}
New-Item -ItemType Directory -Force $manifestDir, $LogDir | Out-Null

function Format-Seconds([double]$seconds) { return [string]::Format('{0,7:N1}s', $seconds) }

# 与验收脚本相同的工具链环境；不做样本哈希门禁（那是验收脚本的职责）。
$env:MJWARP_MUJOCO_ROOT = Join-Path $root 'target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = Join-Path $env:MJWARP_MUJOCO_ROOT 'bin\mujoco.dll'
$env:CUDA_PATH = Join-Path $root 'target\toolchains\cuda-12.8.1'
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
$featureArgs = if ($AllFeatures) { @('--all-features') } else { @('--features', 'native-model-probe,cuda-probe') }

Write-Host "G01 fast loop（$mode，$(if ($AllFeatures) { 'all-features' } else { 'g01 features' })）"
Write-Host '开发迭代工具：不替代 scripts/test-windows-g01.ps1；不校验样本哈希、不跑排除清单、不记录验收证据。'

function Read-Manifest {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { throw "没有构建清单：$Path。先不带 -NoBuild 运行一次。" }
    return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
}

function Get-NewestSourceTime {
    $paths = @('src', 'tests', 'benches', 'examples') | ForEach-Object { Join-Path $root $_ } |
        Where-Object { Test-Path -LiteralPath $_ }
    $files = @()
    foreach ($path in $paths) { $files += Get-ChildItem -LiteralPath $path -Recurse -File -Filter *.rs }
    foreach ($name in @('Cargo.toml', 'Cargo.lock', 'build.rs', 'rust-toolchain.toml')) {
        $file = Join-Path $root $name
        if (Test-Path -LiteralPath $file) { $files += Get-Item -LiteralPath $file }
    }
    if (-not $files) { return [DateTime]::MinValue }
    return ($files | Sort-Object LastWriteTime -Descending | Select-Object -First 1).LastWriteTime
}

if ($NoBuild) {
    $manifest = Read-Manifest $manifestPath
    $newest = Get-NewestSourceTime
    if ($newest -gt (Get-Item -LiteralPath $manifestPath).LastWriteTime) {
        Write-Warning "源码比构建清单新（$($newest.ToString('HH:mm:ss'))）：产物可能过期，结果仅供粗看。"
    }
    Write-Host "复用构建清单：$manifestPath（$($manifest.builtAt)）"
} else {
    $buildArgs = @("+$toolchain", 'test', '--locked') + $featureArgs
    if ($Release) { $buildArgs += '--release' }
    $buildArgs += @('--no-run', '--message-format=json')
    $build = [Diagnostics.Stopwatch]::StartNew()
    $lines = & cargo @buildArgs 2>&1
    $buildCode = $LASTEXITCODE
    $build.Stop()
    if ($buildCode -ne 0) {
        $lines | Select-Object -Last 40 | Write-Host
        throw "构建失败（exit=$buildCode）"
    }
    $targets = @()
    foreach ($line in $lines) {
        if ($line -isnot [string] -or -not $line.StartsWith('{')) { continue }
        try { $item = $line | ConvertFrom-Json } catch { continue }
        if ($item.reason -ne 'compiler-artifact' -or -not $item.executable -or -not $item.profile.test) { continue }
        $targets += [ordered]@{ name = [string]$item.target.name; kind = @($item.target.kind) -join ','; exe = [string]$item.executable }
    }
    $targets = @($targets | Sort-Object { $_.name })
    $seenNames = @{}
    $uniqueTargets = @()
    foreach ($item in $targets) {
        if ($seenNames.ContainsKey($item.name)) { continue }
        $seenNames[$item.name] = $true
        $uniqueTargets += $item
    }
    $targets = $uniqueTargets
    $manifest = [ordered]@{
        mode = $mode; allFeatures = [bool]$AllFeatures; release = [bool]$Release
        builtAt = (Get-Date).ToString('o'); toolchain = $toolchain; targets = $targets
    }
    $manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $manifestPath -Encoding utf8
    Write-Host ("构建：{0}（{1} 个测试二进制）" -f (Format-Seconds $build.Elapsed.TotalSeconds), $targets.Count)
}

$available = @($manifest.targets)
if (-not $available.Count) { throw "构建清单没有测试二进制：$manifestPath" }
if ($List) {
    $available | Select-Object @{ n = 'target'; e = { $_.name } }, @{ n = 'kind'; e = { $_.kind } },
        @{ n = 'built'; e = { (Get-Item -LiteralPath $_.exe).LastWriteTime.ToString('MM-dd HH:mm:ss') } } |
        Format-Table -AutoSize | Out-String -Width 200 | Write-Host
    Write-Host "共 $($available.Count) 个；清单 $manifestPath"
    return
}

$selected = $available
if ($Target.Count) {
    $selected = @($available | Where-Object { $name = $_.name; @($Target | Where-Object { $name -like $_ }).Count -gt 0 })
    if (-not $selected.Count) {
        throw "没有匹配的 target：$($Target -join ', ')。可用：$((@($available | ForEach-Object { $_.name })) -join ', ')"
    }
}
if ($BuildOnly) {
    Write-Host "构建完成，未运行测试。清单：$manifestPath"
    return
}

$runArgs = @("--test-threads=$TestThreads")
if ($IgnoredOnly) { $runArgs += '--ignored' }
elseif ($IncludeIgnored) { $runArgs += '--include-ignored' }
if ($Nocapture) { $runArgs += '--nocapture' }
if ($Test) { $runArgs += $Test }

# 捕获目录按本轮唯一，避免并行时多个测试进程写同一个 PTX 根（W-D5 已修根+子目录语义，这里再隔离一轮）。
if (-not $env:MJWARP_PHYSICS_PTX_DIR) {
    $ptxRoot = Join-Path $LogDir 'physics-ptx'
    New-Item -ItemType Directory -Force $ptxRoot | Out-Null
    $env:MJWARP_PHYSICS_PTX_DIR = $ptxRoot
}

$results = @()
if ($Parallel -gt 1 -and $selected.Count -gt 1) {
    if ($PSVersionTable.PSVersion.Major -lt 7) { throw '多 target 并行需要 PowerShell 7+；请用 target\toolchains\powershell-7.6.7\bin\pwsh.exe' }
    Write-Host "并行模式：$($selected.Count) 个 target，每进程 --test-threads=$TestThreads，ThrottleLimit=$Parallel"
    $results = @($selected | ForEach-Object -Parallel {
        $item = $_
        $runArgsLocal = $using:runArgs
        $log = Join-Path $using:LogDir ($item.name + '.log')
        $stopwatch = [Diagnostics.Stopwatch]::StartNew()
        try {
            $output = & $item.exe @runArgsLocal 2>&1
            $code = $LASTEXITCODE
        } catch {
            $output = @($_.Exception.Message)
            $code = -1
        }
        $stopwatch.Stop()
        $output | Set-Content -LiteralPath $log -Encoding utf8
        $text = $output -join "`n"
        $counts = @([regex]::Matches($text, 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;') | ForEach-Object {
            [pscustomobject]@{ passed = [int]$_.Groups[1].Value; failed = [int]$_.Groups[2].Value; ignored = [int]$_.Groups[3].Value }
        })
        $passed = [int](($counts | ForEach-Object { $_.passed } | Measure-Object -Sum).Sum)
        $failed = [int](($counts | ForEach-Object { $_.failed } | Measure-Object -Sum).Sum)
        [pscustomobject]@{ target = $item.name; exitCode = $code; passed = $passed; failed = $failed
            seconds = [math]::Round($stopwatch.Elapsed.TotalSeconds, 2)
            ok = ($code -eq 0 -and $failed -eq 0); log = $log }
    } -ThrottleLimit $Parallel)
    foreach ($result in $results) {
        $mark = if ($result.ok) { 'ok  ' } else { 'FAIL' }
        Write-Host ("[{0}] {1,-28} passed={2,-4} failed={3,-3} {4}" -f $mark, $result.target, $result.passed, $result.failed, (Format-Seconds $result.seconds))
        if (-not $result.ok) { Write-Host "  失败明细见日志：$($result.log)" }
    }
} else {
    foreach ($item in $selected) {
        $log = Join-Path $LogDir ($item.name + '.log')
        $stopwatch = [Diagnostics.Stopwatch]::StartNew()
        try {
            $output = & $item.exe @runArgs 2>&1
            $code = $LASTEXITCODE
        } catch {
            $output = @($_.Exception.Message)
            $code = -1
        }
        $stopwatch.Stop()
        $output | Set-Content -LiteralPath $log -Encoding utf8
        $text = $output -join "`n"
        $counts = @([regex]::Matches($text, 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;') | ForEach-Object {
            [ordered]@{ passed = [int]$_.Groups[1].Value; failed = [int]$_.Groups[2].Value; ignored = [int]$_.Groups[3].Value }
        })
        $passed = [int](($counts | ForEach-Object { $_.passed } | Measure-Object -Sum).Sum)
        $failed = [int](($counts | ForEach-Object { $_.failed } | Measure-Object -Sum).Sum)
        $ok = $code -eq 0 -and $failed -eq 0
        $results += [ordered]@{
            target = $item.name; exitCode = $code; passed = $passed; failed = $failed
            seconds = [math]::Round($stopwatch.Elapsed.TotalSeconds, 2); ok = $ok; log = $log
        }
        $mark = if ($ok) { 'ok  ' } else { 'FAIL' }
        Write-Host ("[{0}] {1,-28} passed={2,-4} failed={3,-3} {4}" -f $mark, $item.name, $passed, $failed, (Format-Seconds $stopwatch.Elapsed.TotalSeconds))
        if (-not $ok) {
            $output | Select-Object -Last 40 | Write-Host
            if ($counts.Count -eq 0) { Write-Host "（没有 test result 行：可能崩溃或提前退出，exit=$code）" }
        }
    }
}

$totalPassed = [int](($results | ForEach-Object { $_.passed } | Measure-Object -Sum).Sum)
$totalFailed = [int](($results | ForEach-Object { $_.failed } | Measure-Object -Sum).Sum)
$totalSeconds = [math]::Round((($results | ForEach-Object { $_.seconds } | Measure-Object -Sum).Sum), 2)
$summary = [ordered]@{
    mode = $mode; allFeatures = [bool]$AllFeatures; release = [bool]$Release
    targets = $results; totalPassed = $totalPassed; totalFailed = $totalFailed; totalSeconds = $totalSeconds
    fastTool = 'scripts/g01-fast.ps1'; acceptanceTool = 'scripts/test-windows-g01.ps1（未执行）'
}
$summaryPath = Join-Path $LogDir 'summary.json'
$summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $summaryPath -Encoding utf8
Write-Host ''
$results | Sort-Object seconds -Descending | ForEach-Object {
    Write-Host ("  {0,-28} {1}" -f $_.target, (Format-Seconds $_.seconds))
}
Write-Host ("汇总：{0} 个 target，passed={1}，failed={2}，测试耗时 {3}" -f $results.Count, $totalPassed, $totalFailed, (Format-Seconds $totalSeconds))
Write-Host "明细：$summaryPath"
Write-Host "G01_FAST_SUMMARY=$summaryPath"
if ($totalFailed -gt 0 -or ($results | Where-Object { -not $_.ok }).Count -gt 0) { exit 1 }
exit 0
