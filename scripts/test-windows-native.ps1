[CmdletBinding()]
param([switch]$Gpu, [switch]$Release, [switch]$AllFeatures, [switch]$RestrictedRuntime)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$package = Join-Path $root 'target\toolchains\mujoco-3.12.0\package'
if (-not (Test-Path -LiteralPath (Join-Path $package 'bin\mujoco.dll'))) {
    throw '请先运行prepare-native-probe.ps1'
}
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw '请安装MSVC与Windows SDK' }
$devcmd = Join-Path $vs 'Common7\Tools\VsDevCmd.bat'
$lines = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC初始化失败' }
foreach ($line in $lines) {
    if ($line -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}
$env:MJWARP_MUJOCO_ROOT = $package
$env:MJWARP_MUJOCO_DLL = Join-Path $package 'bin\mujoco.dll'
$env:MJWARP_NATIVE_MOCKS = Join-Path $root ('target\native-probe\' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $env:MJWARP_NATIVE_MOCKS -Force | Out-Null
$env:MJWARP_NATIVE_LIFETIME_LOG = Join-Path $env:MJWARP_NATIVE_MOCKS 'lifetime.log'
$fixtures = Join-Path $root 'fixtures\native-probe'
$manifest = Get-Content -LiteralPath (Join-Path $fixtures 'manifest.json') -Raw | ConvertFrom-Json
foreach ($file in $manifest.files) {
    $hash = (Get-FileHash -LiteralPath (Join-Path $fixtures $file.path) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $file.sha256) { throw "样本校验失败：$($file.path)" }
}
Push-Location $env:MJWARP_NATIVE_MOCKS
try {
    foreach ($case in @(
        @{ Name = 'wrong-version'; Define = '/DMOCK_VERSION=123' },
        @{ Name = 'missing-load'; Define = '/DMOCK_MISSING_LOAD' },
        @{ Name = 'missing-version'; Define = '/DMOCK_MISSING_VERSION' },
        @{ Name = 'missing-delete'; Define = '/DMOCK_MISSING_DELETE' },
        @{ Name = 'tracking'; Define = '/DMOCK_TRACKING' },
        @{ Name = 'invalid-counts'; Define = @('/DMOCK_TRACKING', '/DMOCK_INVALID_COUNTS') }
    )) {
        & cl.exe /nologo /LD /MD /std:c++17 "/I$package\include" $case.Define `
            (Join-Path $root 'tests\native\mock_mujoco.cpp') "/Fe$($case.Name).dll"
        if ($LASTEXITCODE -ne 0) { throw "测试DLL编译失败：$($case.Name)" }
    }
} finally { Pop-Location }
$feature = if ($Gpu) { 'native-model-probe,cuda-probe' } else { 'native-model-probe' }
$cargoArgs = @('+stable-x86_64-pc-windows-msvc', 'test', '--locked')
if ($AllFeatures) {
    $env:CUDA_PATH = Join-Path $root 'target\toolchains\cuda-12.8.1'
    $env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
    $cargoArgs += '--all-features'
} else { $cargoArgs += @('--features', $feature) }
if ($Release) { $cargoArgs += '--release' }
$cargoArgs += @('--test', 'native_model_probe', '--', '--ignored', '--test-threads=1')
Push-Location $root
try {
    $output = & cargo @cargoArgs 2>&1 | Tee-Object -FilePath (Join-Path $env:MJWARP_NATIVE_MOCKS 'tests.log')
    if ($LASTEXITCODE -ne 0) { throw '原生模型探针失败' }
    $match = [regex]::Match(($output -join "`n"), 'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
    $expected = if ($Gpu -or $AllFeatures) { 8 } else { 7 }
    if (-not $match.Success -or [int]$match.Groups[1].Value -ne $expected -or
        [int]$match.Groups[2].Value -ne 0 -or [int]$match.Groups[3].Value -ne 0) {
        throw '测试计数无效'
    }
    $restrictedCount = 0
    if ($RestrictedRuntime) {
        # 获取当前feature对应产物。
        # 不从旧产物中猜测程序路径。
        $buildArgs = @('+stable-x86_64-pc-windows-msvc', 'test', '--locked', '--no-run', '--message-format=json', '--test', 'native_model_probe')
        if ($AllFeatures) { $buildArgs += '--all-features' } else { $buildArgs += @('--features', $feature) }
        if ($Release) { $buildArgs += '--release' }
        $messages = & cargo @buildArgs
        if ($LASTEXITCODE -ne 0) { throw '测试产物定位失败' }
        $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object {
            $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'native_model_probe' -and $_.executable
        } | Select-Object -Last 1
        if (-not $artifact) { throw '测试产物缺失' }
        $oldPath = $env:PATH
        try {
            $env:PATH = "$env:SystemRoot\System32;$env:SystemRoot"
            $restricted = & $artifact.executable --ignored --test-threads=1 2>&1 |
                Tee-Object -FilePath (Join-Path $env:MJWARP_NATIVE_MOCKS 'restricted.log')
            if ($LASTEXITCODE -ne 0) { throw '受限运行失败' }
            $result = [regex]::Match(($restricted -join "`n"), 'test result: ok\. (\d+) passed; 0 failed; 0 ignored;')
            if (-not $result.Success) { throw '受限测试计数无效' }
            $restrictedCount = [int]$result.Groups[1].Value
            if ($restrictedCount -ne $expected) { throw '受限测试数量缺失' }
        } finally { $env:PATH = $oldPath }
    }
    [ordered]@{
        platform = 'Windows x86_64 MSVC'
        candidateVersion = $manifest.candidate_version
        gitRevision = (& git rev-parse HEAD)
        workingTreeDirty = [bool](& git status --porcelain)
        recordedAtUtc = [DateTime]::UtcNow.ToString('o')
        feature = $(if ($AllFeatures) { 'all-features' } else { $feature })
        release = [bool]$Release
        passed = [int]$match.Groups[1].Value
        failed = [int]$match.Groups[2].Value
        ignored = [int]$match.Groups[3].Value
        restrictedPassed = $restrictedCount
        dllSha256 = (Get-FileHash -LiteralPath $env:MJWARP_MUJOCO_DLL -Algorithm SHA256).Hash.ToLowerInvariant()
        fixtureManifest = 'fixtures/native-probe/manifest.json'
        scriptInvokesModelCompiler = $false
        scriptInvokesPython = $false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $env:MJWARP_NATIVE_MOCKS 'report.json') -Encoding utf8
    $output | ForEach-Object { Write-Host $_ }
} finally { Pop-Location }
Write-Host "NATIVE_EVIDENCE=$env:MJWARP_NATIVE_MOCKS"
