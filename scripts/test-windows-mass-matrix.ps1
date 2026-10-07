[CmdletBinding()]
param([switch]$AllFeatures, [switch]$Release)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$manifestPath = 'fixtures/mass-matrix/manifest.json'
$manifest = Get-Content -LiteralPath (Join-Path $root $manifestPath) -Raw | ConvertFrom-Json
foreach ($file in $manifest.files) {
    $actual = (Get-FileHash -LiteralPath (Join-Path $root $file.path) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $file.sha256) { throw "样本哈希不符：$($file.path)" }
}
$env:CUDA_PATH = Join-Path $root 'target\toolchains\cuda-12.8.1'
if (-not (Test-Path -LiteralPath (Join-Path $env:CUDA_PATH 'bin'))) { throw '请先准备CUDA探针工具链' }
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
if ($AllFeatures) {
    $env:MJWARP_MUJOCO_ROOT = Join-Path $root 'target\toolchains\mujoco-3.12.0\package'
    $env:MJWARP_MUJOCO_DLL = Join-Path $env:MJWARP_MUJOCO_ROOT 'bin\mujoco.dll'
    $actual = (Get-FileHash -LiteralPath $env:MJWARP_MUJOCO_DLL -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $manifest.dll_sha256) { throw '原生DLL哈希不符' }
}
$directory = Join-Path $root ('target\mass-matrix-probe\' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $directory | Out-Null
$arguments = @('+stable-x86_64-pc-windows-msvc','test','--locked')
if ($AllFeatures) { $arguments += '--all-features' } else { $arguments += @('--features','cuda-probe') }
if ($Release) { $arguments += '--release' }
$arguments += @('--test','mass_matrix_probe','--','--ignored','--test-threads=1','--nocapture')
Push-Location $root
try {
    $output = & cargo @arguments 2>&1 | Tee-Object -FilePath (Join-Path $directory 'mass-matrix.log')
    if ($LASTEXITCODE -ne 0) { throw '质量矩阵测试失败' }
    $match = [regex]::Match(($output -join "`n"),'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
    $expected = if ($AllFeatures) {6} else {5}
    if (-not $match.Success -or [int]$match.Groups[1].Value -ne $expected -or
        [int]$match.Groups[2].Value -ne 0 -or [int]$match.Groups[3].Value -ne 0) { throw '质量矩阵计数不符' }
    $largest = 0.0
    $comparisons = 0
    $energy = 0
    $semidefinite = 0
    foreach ($line in $output) {
        if ($line -match 'max_abs_error=([0-9.eE+-]+)') {
            $comparisons++
            $number = [double]::Parse($Matches[1],[Globalization.CultureInfo]::InvariantCulture)
            $largest = [Math]::Max($largest,$number)
        }
        if ($line -match 'analytical_energy_world=') { $energy++ }
        if ($line -match 'analytical_semidefinite_world=') { $semidefinite++ }
    }
    $expectedComparisons = if ($AllFeatures) {1379} else {1331}
    if ($comparisons -ne $expectedComparisons -or $energy -ne 1 -or $semidefinite -ne 1) {
        throw '质量矩阵参考比较数量不符'
    }
    $gpu = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>&1
    if ($LASTEXITCODE -ne 0) { throw 'GPU信息查询失败' }
    [ordered]@{
        platform = 'Windows x86_64 MSVC'
        windowsVersion = [Environment]::OSVersion.Version.ToString()
        gpu = @($gpu)
        rust = (& rustc +stable-x86_64-pc-windows-msvc -V)
        gitRevision = (& git rev-parse HEAD)
        workingTreeDirty = [bool](& git status --porcelain)
        recordedAtUtc = [DateTime]::UtcNow.ToString('o')
        allFeatures = [bool]$AllFeatures
        release = [bool]$Release
        massMatrixPassed = $expected
        comparedNativeWorlds = $comparisons
        analyticalEnergyChecks = $energy
        analyticalSemidefiniteChecks = $semidefinite
        maxAbsoluteError = $largest
        absoluteTolerance = 0.00002
        relativeTolerance = 0.00002
        fixtureManifest = $manifestPath
        fixtureManifestSha256 = (Get-FileHash -LiteralPath (Join-Path $root $manifestPath) -Algorithm SHA256).Hash.ToLowerInvariant()
        nvrtcLibraries = @(Get-ChildItem -LiteralPath (Join-Path $env:CUDA_PATH 'bin') -Filter 'nvrtc*.dll' |
            ForEach-Object { [ordered]@{ name = $_.Name; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() } })
        scriptInvokesNativePhysics = $false
        scriptInvokesModelCompiler = $false
        scriptInvokesPython = $false
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $directory 'report.json') -Encoding utf8
} finally { Pop-Location }
Write-Host "MASS_MATRIX_EVIDENCE=$directory"
