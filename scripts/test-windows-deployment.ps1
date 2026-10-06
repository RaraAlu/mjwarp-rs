[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw '此脚本只验证Windows' }
$root = Split-Path -Parent $PSScriptRoot
$cuda = Join-Path $root 'target\toolchains\cuda-12.8.1'
if (-not (Test-Path -LiteralPath (Join-Path $cuda 'components.json'))) {
    throw '请先准备私有CUDA组件'
}
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw '需要MSVC与Windows SDK' }
$devcmd = Join-Path $vs 'Common7\Tools\VsDevCmd.bat'
$environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC环境初始化失败' }
foreach ($line in $environment) {
    if ($line -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}
$env:CUDA_PATH = $cuda
$env:PATH = (Join-Path $cuda 'bin') + ';' + $env:PATH

# 每次保留独立证据，不删除旧运行。
$stage = Join-Path $root ('target\deployment-probe\' + [guid]::NewGuid().ToString('N'))
$producerDir = Join-Path $stage 'producer'
$driverDir = Join-Path $stage 'driver'
$cache = Join-Path $stage 'cache'
New-Item -ItemType Directory -Path $producerDir, $driverDir, $cache | Out-Null
$producer = Join-Path $producerDir 'mjwarp-rs.exe'
$consumer = Join-Path $driverDir 'mjwarp-rs.exe'
$records = [System.Collections.Generic.List[object]]::new()
$runId = 0
$passed = $false

function Invoke-Probe {
    param([string]$Executable, [string[]]$Arguments, [switch]$DriverOnly, [int]$ExpectedExit = 0)
    $script:runId++
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Executable
    $start.WorkingDirectory = Split-Path -Parent $Executable
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardOutputEncoding = [System.Text.Encoding]::UTF8
    $start.StandardErrorEncoding = [System.Text.Encoding]::UTF8
    foreach ($argument in $Arguments) { $start.ArgumentList.Add($argument) }
    if ($DriverOnly) {
        # 仅清理子进程，不改父终端。
        $start.Environment['PATH'] = "$env:SystemRoot\System32;$env:SystemRoot"
        foreach ($name in @($start.Environment.Keys)) {
            if ($name -match '^(CUDA_|CUDAToolkit|LLVM_|TRACEL_)') {
                $start.Environment.Remove($name) | Out-Null
            }
        }
        # 避免把驱动磁盘缓存命中当成证据。
        $start.Environment['CUDA_CACHE_DISABLE'] = '1'
    }
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        if (-not $process.Start()) { throw '子进程启动失败' }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(120000)) {
            $process.Kill($true)
            $process.WaitForExit()
            throw '子进程超过两分钟'
        }
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        $exitCode = $process.ExitCode
    } finally { $process.Dispose() }
    $stdout | Set-Content -LiteralPath (Join-Path $stage "$script:runId.stdout.txt") -Encoding utf8
    $stderr | Set-Content -LiteralPath (Join-Path $stage "$script:runId.stderr.txt") -Encoding utf8
    $records.Add([pscustomobject]@{
        id = $script:runId; executable = $Executable; arguments = $Arguments
        driver_only = [bool]$DriverOnly; exit_code = $exitCode
        stdout = $stdout; stderr = $stderr
    })
    if ($exitCode -ne $ExpectedExit) {
        throw "子进程失败：$exitCode；预期$ExpectedExit`n$stdout`n$stderr"
    }
    return $stdout + $stderr
}

Push-Location $root
try {
    & cargo +stable-x86_64-pc-windows-msvc build --locked --offline --release --all-features
    if ($LASTEXITCODE -ne 0) { throw '产物生产程序构建失败' }
    Copy-Item -LiteralPath (Join-Path $root 'target\release\mjwarp-rs.exe') -Destination $producer
    # 使用独立目录，避免覆盖生产程序。
    $consumerTarget = Join-Path $root 'target\deployment-build'
    & cargo +stable-x86_64-pc-windows-msvc build --locked --offline --release --no-default-features --features cuda-probe --target-dir $consumerTarget
    if ($LASTEXITCODE -ne 0) { throw '仅驱动程序构建失败' }
    Copy-Item -LiteralPath (Join-Path $consumerTarget 'release\mjwarp-rs.exe') -Destination $consumer
    $tree = & cargo +stable-x86_64-pc-windows-msvc tree --locked --offline --no-default-features --features cuda-probe
    if ($LASTEXITCODE -ne 0) { throw '依赖树检查失败' }
    $tree | Set-Content -LiteralPath (Join-Path $stage 'consumer-dependencies.txt') -Encoding utf8
    if (($tree -join "`n") -match '(cubecl|pliron|tracel|llvm)') { throw '仅驱动程序含前端依赖' }
    $imports = & dumpbin /dependents $consumer
    if ($LASTEXITCODE -ne 0) { throw 'PE依赖检查失败' }
    $imports | Set-Content -LiteralPath (Join-Path $stage 'consumer-imports.txt') -Encoding utf8
    if (($imports -join "`n") -match '(nvrtc|LLVM|cudart)') { throw 'PE导入含编译库' }

    $kernels = @('affine', 'atomic-sum', 'float-atomic-sum', 'block-reduce', 'block-scan', 'global-scan', 'control-flow', 'small-solve')
    $configurations = @(@{ backend = 'native-ptx'; kernel = 'affine' })
    foreach ($backend in @('cubecl-cpp', 'cubecl-llvm')) {
        foreach ($kernel in $kernels) { $configurations += @{ backend = $backend; kernel = $kernel } }
    }
    foreach ($config in $configurations) {
        $selection = @('--backend', $config.backend, '--kernel', $config.kernel)
        $built = Invoke-Probe $producer (@('cache-build', '--cache', $cache) + $selection)
        if ($built -notmatch 'cache_status=Compiled') { throw '新目录没有生成产物' }
        # 禁用前端时，构建命令只允许命中。
        $hit = Invoke-Probe $consumer (@('cache-build', '--cache', $cache) + $selection) -DriverOnly
        if ($hit -notmatch 'cache_status=Hit') { throw '仅驱动程序没有命中缓存' }
        foreach ($elements in @(1, 129, 16385, 1048576)) {
            $output = Invoke-Probe $consumer (@('cache-run', '--cache', $cache, '--trust-cache', '--require-no-nvrtc', '--elements', "$elements") + $selection) -DriverOnly
            if ($output -notmatch 'nvrtc_available=false' -or $output -notmatch 'node_updates=' -or $output -notmatch 'inactive_output=pass') {
                throw '仅驱动进程缺少验证记录'
            }
        }
    }
    foreach ($backend in @('cubecl-cpp', 'cubecl-llvm')) {
        $disabled = Invoke-Probe $consumer @('probe', '--backend', $backend) -DriverOnly -ExpectedExit 1
        if ($disabled -notmatch "请启用$backend-probe") { throw '关闭前端后出现路线替换' }
    }
    $missing = Join-Path $stage 'missing'
    $failureOutput = Invoke-Probe $consumer @('cache-run', '--cache', $missing, '--trust-cache', '--require-no-nvrtc', '--backend', 'cubecl-cpp') -DriverOnly -ExpectedExit 1
    if ($failureOutput -notmatch 'missing') { throw '缺失缓存诊断不符' }
    # 保留原产物；只损坏独立副本。
    $corrupt = Join-Path $stage 'corrupt'
    New-Item -ItemType Directory -Path $corrupt | Out-Null
    foreach ($file in Get-ChildItem -LiteralPath $cache -Filter '*.json' -File) {
        'corrupted' | Set-Content -LiteralPath (Join-Path $corrupt $file.Name) -Encoding utf8
    }
    $failureOutput = Invoke-Probe $consumer @('cache-run', '--cache', $corrupt, '--trust-cache', '--require-no-nvrtc') -DriverOnly -ExpectedExit 1
    if ($failureOutput -notmatch 'archive-json') { throw '损坏缓存诊断不符' }
    $failureOutput = Invoke-Probe $producer @('cache-run', '--cache', $cache, '--trust-cache', '--require-no-nvrtc') -ExpectedExit 1
    if ($failureOutput -notmatch '拒绝可见NVRTC') { throw 'NVRTC可见检查没有失败' }
    $passed = $true
    Write-Host "部署烟测通过：17份产物，68组GPU配置。"
} finally {
    $driver = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader
    $compilerDll = Join-Path $cuda 'bin\nvrtc64_120_0.dll'
    [pscustomobject]@{
        date = (Get-Date).ToString('o'); os = [Environment]::OSVersion.VersionString
        process_arch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString()
        gpu_driver = $driver; stage = $stage
        passed = $passed
        nvrtc_dll_sha256 = (Get-FileHash -LiteralPath $compilerDll -Algorithm SHA256).Hash
        cargo_lock_sha256 = (Get-FileHash -LiteralPath (Join-Path $root 'Cargo.lock') -Algorithm SHA256).Hash
        producer_sha256 = if (Test-Path -LiteralPath $producer) { (Get-FileHash -LiteralPath $producer -Algorithm SHA256).Hash } else { $null }
        consumer_sha256 = if (Test-Path -LiteralPath $consumer) { (Get-FileHash -LiteralPath $consumer -Algorithm SHA256).Hash } else { $null }
        records = @($records.ToArray())
        limitation = '受限PATH子进程烟测；不代表清洁部署验收'
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'report.json') -Encoding utf8
    Pop-Location
    Write-Host "证据：$stage"
}
