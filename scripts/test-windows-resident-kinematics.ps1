[CmdletBinding()]
param([switch]$AllFeatures, [switch]$Release)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
foreach ($name in @('kinematics', 'com-position', 'attached-kinematics', 'mocap-kinematics', 'camlight', 'fixed-tendon', 'spatial-tendon')) {
    $fixtures = Join-Path $root "fixtures\$name"
    $manifest = Get-Content -LiteralPath (Join-Path $fixtures 'manifest.json') -Raw | ConvertFrom-Json
    foreach ($file in $manifest.files) {
        $base = if ($name -eq 'com-position') { $root } else { $fixtures }
        $actual = (Get-FileHash -LiteralPath (Join-Path $base $file.path) -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $file.sha256) { throw "样本哈希不符：$name/$($file.path)" }
    }
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
$directory = Join-Path $root ('target\resident-kinematics\' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $directory | Out-Null
$base = @('+stable-x86_64-pc-windows-msvc', 'test', '--locked')
if ($AllFeatures) { $base += '--all-features' } else { $base += @('--features', 'cuda-probe') }
if ($Release) { $base += '--release' }
function Invoke-CheckedTest {
    param([string]$Name, [string[]]$Arguments, [int]$Expected)
    $output = & cargo @Arguments 2>&1 | Tee-Object -FilePath (Join-Path $directory "$Name.log")
    if ($LASTEXITCODE -ne 0) { throw "$Name 失败" }
    $match = [regex]::Match(($output -join "`n"), 'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
    if (-not $match.Success -or [int]$match.Groups[1].Value -ne $Expected -or
        [int]$match.Groups[2].Value -ne 0 -or [int]$match.Groups[3].Value -ne 0) { throw "$Name 计数不符" }
    return $Expected
}
Push-Location $root
try {
    $references = Invoke-CheckedTest 'references' ($base + @('--test', 'resident_kinematics', 'reference_manifests', '--', '--test-threads=1')) 1
    $resident = Invoke-CheckedTest 'resident' ($base + @('--test', 'resident_kinematics', '--', '--ignored', '--test-threads=1', '--nocapture')) 6
    $parameters = Invoke-CheckedTest 'parameters' ($base + @('--test', 'resident_parameters', '--', '--ignored', '--test-threads=1', '--nocapture')) 3
    $mocapReferences = Invoke-CheckedTest 'mocap-references' ($base + @('--test', 'resident_mocap', 'reference_hashes', '--', '--test-threads=1')) 1
    $mocap = Invoke-CheckedTest 'mocap' ($base + @('--test', 'resident_mocap', '--', '--ignored', '--test-threads=1', '--nocapture')) 4
    $camlightReferences = Invoke-CheckedTest 'camlight-references' ($base + @('--test', 'resident_camlight', 'reference_hashes', '--', '--test-threads=1')) 1
    $camlight = Invoke-CheckedTest 'camlight' ($base + @('--test', 'resident_camlight', '--', '--ignored', '--test-threads=1', '--nocapture')) 5
    $camlightGuards = Invoke-CheckedTest 'camlight-guards' ($base + @('--lib', 'camlight_readback_rejects', '--', '--ignored', '--test-threads=1')) 1
    $tendonReferences = Invoke-CheckedTest 'tendon-references' ($base + @('--test', 'resident_fixed_tendon', 'reference_hashes', '--', '--test-threads=1')) 1
    $tendon = Invoke-CheckedTest 'fixed-tendon' ($base + @('--test', 'resident_fixed_tendon', '--', '--ignored', '--test-threads=1', '--nocapture')) 4
    $tendonGuards = Invoke-CheckedTest 'tendon-guards' ($base + @('--lib', 'fixed_tendon_readback_rejects', '--', '--ignored', '--test-threads=1')) 2
    $spatialReferences = Invoke-CheckedTest 'spatial-references' ($base + @('--test', 'resident_spatial_tendon', 'reference_hashes', '--', '--test-threads=1')) 1
    $spatial = Invoke-CheckedTest 'spatial-tendon' ($base + @('--test', 'resident_spatial_tendon', '--', '--ignored', '--test-threads=1', '--nocapture')) 5
    $spatialGuards = Invoke-CheckedTest 'spatial-guards' ($base + @('--lib', 'spatial_tendon_readback_rejects', '--', '--ignored', '--test-threads=1')) 2
    $staticCache = Invoke-CheckedTest 'static-cache' ($base + @('--lib', 'static_geom_cache', '--', '--ignored', '--test-threads=1')) 1
    $adapter = Invoke-CheckedTest 'adapter' ($base + @('--lib', 'runtime::transfer::kernel::tests', '--', '--ignored', '--test-threads=1')) 3
    $gpu = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>&1
    if ($LASTEXITCODE -ne 0) { throw 'GPU信息查询失败' }
    $largest = 0.0
    $comparisons = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'resident.log')) {
        if ($line -match 'max_abs_error=([0-9.eE+-]+)') {
            $comparisons++
            $number = [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
            $largest = [Math]::Max($largest, $number)
        }
    }
    if ($comparisons -ne 1245) { throw "参考比较计数不符：$comparisons" }
    $parameterLargest = 0.0
    $parameterComparisons = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'parameters.log')) {
        if ($line -match 'G01-field-batches kind=\w+ world=\d+ max_abs_error=([0-9.eE+-]+)') {
            $parameterComparisons++
            $number = [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
            $parameterLargest = [Math]::Max($parameterLargest, $number)
        }
    }
    if ($parameterComparisons -ne 3102) { throw "参数比较计数不符：$parameterComparisons" }
    $mocapLargest = 0.0
    $mocapComparisons = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'mocap.log')) {
        if ($line -match 'G01-mocap kind=native world=\d+ max_abs_error=([0-9.eE+-]+)') {
            $mocapComparisons++
            $number = [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
            $mocapLargest = [Math]::Max($mocapLargest, $number)
        }
    }
    if ($mocapComparisons -ne 2084) { throw "mocap比较计数不符：$mocapComparisons" }
    $camlightLargest = 0.0
    $camlightComparisons = 0
    $camlightParameterLargest = 0.0
    $camlightParameterComparisons = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'camlight.log')) {
        if ($line -match 'G01-camlight kind=(native|parameters) world=\d+ max_abs_error=([0-9.eE+-]+)') {
            $number = [double]::Parse($Matches[2], [Globalization.CultureInfo]::InvariantCulture)
            if ($Matches[1] -eq 'native') {
                $camlightComparisons++
                $camlightLargest = [Math]::Max($camlightLargest, $number)
            } else {
                $camlightParameterComparisons++
                $camlightParameterLargest = [Math]::Max($camlightParameterLargest, $number)
            }
        }
    }
    if ($camlightComparisons -ne 2084) { throw "相机光源比较计数不符：$camlightComparisons" }
    if ($camlightParameterComparisons -ne 1042) { throw "相机光源参数计数不符：$camlightParameterComparisons" }
    $tendonLargest = 0.0
    $tendonComparisons = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'fixed-tendon.log')) {
        if ($line -match 'G01-fixed-tendon kind=native world=\d+ max_abs_error=([0-9.eE+-]+)') {
            $tendonComparisons++
            $number = [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
            $tendonLargest = [Math]::Max($tendonLargest, $number)
        }
    }
    if ($tendonComparisons -ne 2084) { throw "固定肌腱比较计数不符：$tendonComparisons" }
    $spatialLargest = 0.0
    $spatialComparisons = 0
    $spatialParameterLargest = 0.0
    $spatialParameterComparisons = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'spatial-tendon.log')) {
        if ($line -match 'G01-spatial-tendon kind=(native|parameters) world=\d+ max_abs_error=([0-9.eE+-]+)') {
            $number = [double]::Parse($Matches[2], [Globalization.CultureInfo]::InvariantCulture)
            if ($Matches[1] -eq 'native') {
                $spatialComparisons++
                $spatialLargest = [Math]::Max($spatialLargest, $number)
            } else {
                $spatialParameterComparisons++
                $spatialParameterLargest = [Math]::Max($spatialParameterLargest, $number)
            }
        }
    }
    if ($spatialComparisons -ne 2084) { throw "空间肌腱比较计数不符：$spatialComparisons" }
    if ($spatialParameterComparisons -ne 1042) { throw "空间肌腱参数计数不符：$spatialParameterComparisons" }
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
        referencesPassed = $references
        residentPassed = $resident
        parametersPassed = $parameters
        mocapReferencesPassed = $mocapReferences
        mocapPassed = $mocap
        camlightReferencesPassed = $camlightReferences
        camlightPassed = $camlight
        camlightGuardsPassed = $camlightGuards
        fixedTendonReferencesPassed = $tendonReferences
        fixedTendonPassed = $tendon
        fixedTendonGuardsPassed = $tendonGuards
        spatialTendonReferencesPassed = $spatialReferences
        spatialTendonPassed = $spatial
        spatialTendonGuardsPassed = $spatialGuards
        staticCachePassed = $staticCache
        adapterPassed = $adapter
        comparedWorlds = $comparisons
        maxAbsoluteError = $largest
        absoluteTolerance = 0.00002
        relativeTolerance = 0.00002
        fixtureManifests = @('fixtures/kinematics/manifest.json', 'fixtures/com-position/manifest.json', 'fixtures/attached-kinematics/manifest.json', 'fixtures/mocap-kinematics/manifest.json', 'fixtures/camlight/manifest.json', 'fixtures/fixed-tendon/manifest.json', 'fixtures/spatial-tendon/manifest.json')
        sharedReferenceBatchSize = 1
        parameterIndexRule = 'world % B_f'
        parameterComparedWorlds = $parameterComparisons
        parameterMaxAbsoluteError = $parameterLargest
        parameterWorldCounts = @(1, 2, 5, 513)
        analyticFieldPeriods = @(3, 'W', 3, 5, 4, 7, 2, 3, 5, 2, 4, 7, 3)
        emptyFieldPeriod = 2147483647
        mocapComparedWorlds = $mocapComparisons
        mocapMaxAbsoluteError = $mocapLargest
        mocapWorldCounts = @(1, 2, 5, 513)
        mocapDefaultFieldPeriods = @(3, 5, 7)
        mocapIdsReversed = $true
        staticCacheMutationWorlds = 513
        camlightComparedWorlds = $camlightComparisons
        camlightMaxAbsoluteError = $camlightLargest
        camlightParameterComparedWorlds = $camlightParameterComparisons
        camlightParameterMaxAbsoluteError = $camlightParameterLargest
        camlightFieldPeriods = @(2, 3, 5, 7, 11, 3, 5, 7, 2, 11)
        camlightWorldCounts = @(1, 2, 5, 513)
        camlightModes = @(0, 1, 2, 3, 4)
        camlightNegativeTargets = @(-2, -1)
        camlightDegenerateTargetWorlds = 513
        fixedTendonComparedWorlds = $tendonComparisons
        fixedTendonMaxAbsoluteError = $tendonLargest
        fixedTendonWorldCounts = @(1, 2, 5, 513)
        fixedTendonCount = 4
        fixedTendonNnz = 8
        fixedTendonQpos0Period = 3
        fixedTendonSharedCoefficients = $true
        spatialTendonComparedWorlds = $spatialComparisons
        spatialTendonMaxAbsoluteError = $spatialLargest
        spatialTendonParameterComparedWorlds = $spatialParameterComparisons
        spatialTendonParameterMaxAbsoluteError = $spatialParameterLargest
        spatialTendonWorldCounts = @(1, 2, 5, 513)
        spatialTendonFieldPeriods = @(3, 2, 5)
        spatialTendonCount = 5
        spatialTendonNnz = 45
        spatialTendonWrapCount = 19
        spatialTendonWrapTypes = @('site', 'pulley')
        spatialTendonSharedTopologyAndDivisors = $true
        residentSubsets = 6
        residentOutputBuffers = 7
        nvrtcLibraries = @(Get-ChildItem -LiteralPath (Join-Path $env:CUDA_PATH 'bin') -Filter 'nvrtc*.dll' |
            ForEach-Object { [ordered]@{ name = $_.Name; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() } })
        scriptInvokesNativeKinematics = $false
        scriptInvokesModelCompiler = $false
        scriptInvokesPython = $false
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $directory 'report.json') -Encoding utf8
} finally { Pop-Location }
Write-Host "RESIDENT_KINEMATICS_EVIDENCE=$directory"
