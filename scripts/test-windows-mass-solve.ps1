[CmdletBinding()]
param([switch]$AllFeatures, [switch]$Release)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$manifestPath = 'fixtures/mass-solve/manifest.json'
$manifest = Get-Content -LiteralPath (Join-Path $root $manifestPath) -Raw | ConvertFrom-Json
if ($manifest.files.Count -ne 19) { throw '求解参考清单数量不符' }
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
$directory = Join-Path $root ('target\mass-solve-probe\' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $directory | Out-Null
$common = @('+stable-x86_64-pc-windows-msvc', 'test', '--locked')
if ($AllFeatures) { $common += '--all-features' } else { $common += @('--features', 'cuda-probe') }
if ($Release) { $common += '--release' }
function Invoke-Test([string[]]$Target, [string]$Log, [int]$Expected) {
    $arguments = $common + $Target + @('--', '--ignored', '--test-threads=1', '--nocapture')
    $lines = & cargo @arguments 2>&1 | Tee-Object -FilePath (Join-Path $directory $Log)
    if ($LASTEXITCODE -ne 0) { throw "求解测试失败：$Log" }
    $match = [regex]::Match(($lines -join "`n"), 'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
    if (-not $match.Success -or [int]$match.Groups[1].Value -ne $Expected -or
        [int]$match.Groups[2].Value -ne 0 -or [int]$match.Groups[3].Value -ne 0) { throw "求解计数不符：$Log" }
    $lines
}
Push-Location $root
try {
    $null = Invoke-Test -Target @('--lib', 'physics::mass_solve::tests::device_') -Log 'boundaries.log' -Expected 2
    $null = Invoke-Test -Target @('--lib', 'runtime::transfer::kernel::tests::preserves_f64_buffer_width') -Log 'scalar-abi.log' -Expected 1
    $expected = if ($AllFeatures) { 6 } else { 5 }
    $output = @(Invoke-Test -Target @('--test', 'mass_solve_probe') -Log 'mass-solve.log' -Expected $expected)
    $largest = 0.0; $residual = 0.0; $reconstruction = 0.0; $comparisons = 0; $analytical = 0; $singular = 0
    foreach ($line in $output) {
        if ($line -match 'max_abs_error=([0-9.eE+-]+) residual=([0-9.eE+-]+) reconstruction=([0-9.eE+-]+)') {
            $comparisons++
            $largest = [Math]::Max($largest, [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture))
            $residual = [Math]::Max($residual, [double]::Parse($Matches[2], [Globalization.CultureInfo]::InvariantCulture))
            $reconstruction = [Math]::Max($reconstruction, [double]::Parse($Matches[3], [Globalization.CultureInfo]::InvariantCulture))
        }
        if ($line -match 'analytical_world=') { $analytical++ }
        if ($line -match 'singular_rejected_world=') { $singular++ }
    }
    $expectedComparisons = if ($AllFeatures) { 1411 } else { 1363 }
    if ($comparisons -ne $expectedComparisons -or $analytical -ne 1 -or $singular -ne 1) { throw '求解参考比较数量不符' }
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
        massSolvePassed = $expected
        factorBoundaryPassed = 2
        scalarAbiPassed = 1
        comparedNativeWorlds = $comparisons
        analyticalChecks = $analytical
        singularRejections = $singular
        internalScalar = 'f64'
        publicScalar = 'f32'
        maxAbsoluteError = $largest
        maxNormalizedResidual = $residual
        maxNormalizedReconstruction = $reconstruction
        absoluteTolerance = 0.0002
        relativeTolerance = 0.0002
        residualTolerance = 0.00002
        reconstructionTolerance = 0.000002
        fixtureManifest = $manifestPath
        fixtureManifestSha256 = (Get-FileHash -LiteralPath (Join-Path $root $manifestPath) -Algorithm SHA256).Hash.ToLowerInvariant()
        nvrtcLibraries = @(Get-ChildItem -LiteralPath (Join-Path $env:CUDA_PATH 'bin') -Filter 'nvrtc*.dll' |
            ForEach-Object { [ordered]@{ name = $_.Name; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() } })
        scriptInvokesNativePhysics = $false
        scriptInvokesModelCompiler = $false
        scriptInvokesPython = $false
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $directory 'report.json') -Encoding utf8
} finally { Pop-Location }
Write-Host "MASS_SOLVE_EVIDENCE=$directory"
