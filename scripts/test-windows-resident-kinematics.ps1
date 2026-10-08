[CmdletBinding()]
param([switch]$AllFeatures, [switch]$Release)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
foreach ($name in @('kinematics', 'com-position', 'attached-kinematics', 'mocap-kinematics', 'camlight', 'fixed-tendon', 'spatial-tendon', 'geom-tendon', 'flex-position', 'tendon-wake', 'flex-edge', 'flex-face')) {
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
    $geomReferences = Invoke-CheckedTest 'geom-references' ($base + @('--test', 'resident_geom_tendon', 'reference_hashes', '--', '--test-threads=1')) 1
    $geom = Invoke-CheckedTest 'geom-tendon' ($base + @('--test', 'resident_geom_tendon', '--', '--ignored', '--test-threads=1', '--nocapture')) 6
    $globalReferences = Invoke-CheckedTest 'global-references' ($base + @('--test', 'resident_tendon', 'reference_hashes', '--', '--test-threads=1')) 1
    $globalBoundaries = Invoke-CheckedTest 'global-boundaries' ($base + @('--test', 'resident_tendon', 'rejects_global', '--', '--test-threads=1')) 1
    $globalTendon = Invoke-CheckedTest 'global-tendon' ($base + @('--test', 'resident_tendon', '--', '--ignored', '--test-threads=1', '--nocapture')) 3
    $globalGuards = Invoke-CheckedTest 'global-guards' ($base + @('--lib', 'global_tendon_readback', '--', '--ignored', '--test-threads=1')) 1
    $flexReferences = Invoke-CheckedTest 'flex-references' ($base + @('--test', 'resident_flex_position', 'reference_hashes', '--', '--test-threads=1')) 1
    $flex = Invoke-CheckedTest 'flex-position' ($base + @('--test', 'resident_flex_position', '--', '--ignored', '--test-threads=1', '--nocapture')) 4
    $flexGuards = Invoke-CheckedTest 'flex-guards' ($base + @('--lib', 'flex_position_readback', '--', '--ignored', '--test-threads=1')) 1
    $wakeReferences = Invoke-CheckedTest 'wake-references' ($base + @('--test', 'resident_tendon_wake', 'reference_hashes', '--', '--test-threads=1')) 1
    $wake = Invoke-CheckedTest 'tendon-wake' ($base + @('--test', 'resident_tendon_wake', '--', '--ignored', '--test-threads=1', '--nocapture')) 5
    $wakeGuards = Invoke-CheckedTest 'wake-guards' ($base + @('--lib', 'tendon_wake_readback', '--', '--ignored', '--test-threads=1')) 1
    $edgeReferences = Invoke-CheckedTest 'edge-references' ($base + @('--test', 'resident_flex_edge', 'reference_hashes', '--', '--test-threads=1')) 1
    $edgeBoundaries = Invoke-CheckedTest 'edge-boundaries' ($base + @('--test', 'resident_flex_edge', 'rejects_invalid', '--', '--test-threads=1')) 1
    $edge = Invoke-CheckedTest 'flex-edge' ($base + @('--test', 'resident_flex_edge', '--', '--ignored', '--test-threads=1', '--nocapture')) 4
    $edgeGuards = Invoke-CheckedTest 'edge-guards' ($base + @('--lib', 'flex_edge_readback', '--', '--ignored', '--test-threads=1')) 1
    $edgeFreeBody = Invoke-CheckedTest 'edge-free-body' ($base + @('--lib', 'flex_edge_free_body', '--', '--ignored', '--test-threads=1')) 1
    $faceReferences = Invoke-CheckedTest 'face-references' ($base + @('--test', 'resident_flex_face', 'reference_hashes', '--', '--test-threads=1')) 1
    $faceBoundaries = Invoke-CheckedTest 'face-boundaries' ($base + @('--test', 'resident_flex_face', 'rejects_face', '--', '--test-threads=1')) 1
    $face = Invoke-CheckedTest 'flex-face' ($base + @('--test', 'resident_flex_face', '--', '--ignored', '--test-threads=1', '--nocapture')) 3
    $faceGuards = Invoke-CheckedTest 'face-guards' ($base + @('--lib', 'flex_face_readback', '--', '--ignored', '--test-threads=1')) 1
    $faceAnalytic = Invoke-CheckedTest 'face-analytic' ($base + @('--lib', 'flex_face_checks_analytic', '--', '--ignored', '--test-threads=1')) 1
    $staticCache = Invoke-CheckedTest 'static-cache' ($base + @('--lib', 'static_geom_cache', '--', '--ignored', '--test-threads=1')) 1
    $adapter = Invoke-CheckedTest 'adapter' ($base + @('--lib', 'runtime::transfer::kernel::tests', '--', '--ignored', '--test-threads=1')) 4
    $gpu = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>&1
    if ($LASTEXITCODE -ne 0) { throw 'GPU信息查询失败' }
    $edgeStats = [regex]::Match((Get-Content -LiteralPath (Join-Path $directory 'flex-edge.log') -Raw),
        'G01-flex-edge states=(\d+) scalars=(\d+) max_abs_error=([0-9.eE+-]+)')
    if (-not $edgeStats.Success -or [int]$edgeStats.Groups[1].Value -ne 2084 -or
        [int]$edgeStats.Groups[2].Value -ne 1204552) { throw '柔体边比较计数不符' }
    $edgeLargest = [double]::Parse($edgeStats.Groups[3].Value, [Globalization.CultureInfo]::InvariantCulture)
    $faceStats = [regex]::Match((Get-Content -LiteralPath (Join-Path $directory 'flex-face.log') -Raw),
        'G01-flex-face states=(\d+) scalars=(\d+) max_position_error=([0-9.eE+-]+) max_quaternion_error=([0-9.eE+-]+)')
    if (-not $faceStats.Success -or [int]$faceStats.Groups[1].Value -ne 2084 -or
        [int]$faceStats.Groups[2].Value -ne 1033664) { throw '柔体面比较计数不符' }
    $facePositionLargest = [double]::Parse($faceStats.Groups[3].Value, [Globalization.CultureInfo]::InvariantCulture)
    $faceQuaternionLargest = [double]::Parse($faceStats.Groups[4].Value, [Globalization.CultureInfo]::InvariantCulture)
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
    $geomLargest = 0.0
    $geomComparisons = 0
    $geomAnalyticLargest = 0.0
    $geomAnalyticComparisons = 0
    $geomMotionLargest = 0.0
    $geomMotionComparisons = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'geom-tendon.log')) {
        if ($line -match 'G01-geom-tendon kind=(native|analytic) world=\d+ max_abs_error=([0-9.eE+-]+)') {
            $number = [double]::Parse($Matches[2], [Globalization.CultureInfo]::InvariantCulture)
            if ($Matches[1] -eq 'native') {
                $geomComparisons++
                $geomLargest = [Math]::Max($geomLargest, $number)
            } else {
                $geomAnalyticComparisons++
                $geomAnalyticLargest = [Math]::Max($geomAnalyticLargest, $number)
            }
        }
        if ($line -match 'G01-geom-tendon kind=motion world=\d+ frame=\d+ max_abs_error=([0-9.eE+-]+)') {
            $geomMotionComparisons++
            $number = [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
            $geomMotionLargest = [Math]::Max($geomMotionLargest, $number)
        }
    }
    if ($geomComparisons -ne 2084) { throw "球柱参考计数不符：$geomComparisons" }
    if ($geomAnalyticComparisons -ne 1042) { throw "圆柱公式计数不符：$geomAnalyticComparisons" }
    if ($geomMotionComparisons -ne 9) { throw "内侧运动计数不符：$geomMotionComparisons" }
    $globalLargest = 0.0
    $globalComparisons = 0
    $globalRounds = 0
    foreach ($line in Get-Content -LiteralPath (Join-Path $directory 'global-tendon.log')) {
        if ($line -match 'G01-global-tendon worlds=\d+ round=\d+ comparisons=(\d+) max_abs_error=([0-9.eE+-]+)') {
            $globalRounds++
            $globalComparisons += [int]$Matches[1]
            $number = [double]::Parse($Matches[2], [Globalization.CultureInfo]::InvariantCulture)
            $globalLargest = [Math]::Max($globalLargest, $number)
        }
    }
    if ($globalRounds -ne 16 -or $globalComparisons -ne 2084) { throw '混合肌腱比较计数不符' }
    $flexStats = @([regex]::Matches([string](Get-Content -LiteralPath (Join-Path $directory 'flex-position.log') -Raw), 'G01-flex states=(\d+) scalars=(\d+) max_abs_error=([0-9.eE+-]+)'))
    if ($flexStats.Count -ne 1 -or [int]$flexStats[0].Groups[1].Value -ne 2084 -or [int]$flexStats[0].Groups[2].Value -ne 625200) { throw '柔体位置比较计数不符' }
    $flexLargest = [double]::Parse($flexStats[0].Groups[3].Value, [Globalization.CultureInfo]::InvariantCulture)
    $wakeStats = @([regex]::Matches([string](Get-Content -LiteralPath (Join-Path $directory 'tendon-wake.log') -Raw), 'G01-tendon-wake kind=native round=\d+ world=\d+ exact_tree_state=true'))
    if ($wakeStats.Count -ne 2084) { throw '肌腱唤醒比较计数不符' }
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
        geomTendonReferencesPassed = $geomReferences
        geomTendonPassed = $geom
        globalTendonReferencesPassed = $globalReferences
        globalTendonBoundariesPassed = $globalBoundaries
        globalTendonPassed = $globalTendon
        globalTendonGuardsPassed = $globalGuards
        flexPositionReferencesPassed = $flexReferences
        flexPositionPassed = $flex
        flexPositionGuardsPassed = $flexGuards
        tendonWakeReferencesPassed = $wakeReferences
        tendonWakePassed = $wake
        tendonWakeGuardsPassed = $wakeGuards
        flexEdgeReferencesPassed = $edgeReferences
        flexEdgeBoundariesPassed = $edgeBoundaries
        flexEdgePassed = $edge
        flexEdgeGuardsPassed = $edgeGuards
        flexEdgeFreeBodyPassed = $edgeFreeBody
        flexFaceReferencesPassed = $faceReferences
        flexFaceBoundariesPassed = $faceBoundaries
        flexFacePassed = $face
        flexFaceGuardsPassed = $faceGuards
        flexFaceAnalyticPassed = $faceAnalytic
        staticCachePassed = $staticCache
        adapterPassed = $adapter
        comparedWorlds = $comparisons
        maxAbsoluteError = $largest
        absoluteTolerance = 0.00002
        relativeTolerance = 0.00002
        fixtureManifests = @('fixtures/kinematics/manifest.json', 'fixtures/com-position/manifest.json', 'fixtures/attached-kinematics/manifest.json', 'fixtures/mocap-kinematics/manifest.json', 'fixtures/camlight/manifest.json', 'fixtures/fixed-tendon/manifest.json', 'fixtures/spatial-tendon/manifest.json', 'fixtures/geom-tendon/manifest.json', 'fixtures/mixed-tendon/manifest.json', 'fixtures/flex-position/manifest.json', 'fixtures/tendon-wake/manifest.json', 'fixtures/flex-edge/manifest.json', 'fixtures/flex-face/manifest.json')
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
        geomTendonComparedWorlds = $geomComparisons
        geomTendonMaxAbsoluteError = $geomLargest
        geomTendonAnalyticComparedWorlds = $geomAnalyticComparisons
        geomTendonAnalyticMaxAbsoluteError = $geomAnalyticLargest
        geomTendonMotionComparedWorlds = $geomMotionComparisons
        geomTendonMotionMaxAbsoluteError = $geomMotionLargest
        geomTendonMotionFrames = @(310, 344, 393)
        geomTendonMotionWorldCounts = @(3, 513)
        geomTendonWorldCounts = @(1, 2, 5, 513)
        geomTendonSizePeriod = 3
        geomTendonAnalyticFieldPeriods = @(3, 2, 5, 7)
        geomTendonCount = 10
        geomTendonNnz = 10
        geomTendonWrapCount = 37
        geomTendonWrapTypes = @('sphere', 'cylinder', 'site', 'pulley')
        geomTendonSharedTopologyAndSideIds = $true
        geomTendonInternalScratchValuesPerWrap = 13
        geomTendonInsideNewtonPrecision = 'f64'
        geomTendonInsideDirectionsBeforeContactRounding = $true
        geomTendonIntegerStageReadsFloatState = $true
        residentSubsets = 6
        residentOutputBuffers = 7
        globalTendonComparedWorlds = $globalComparisons
        globalTendonMaxAbsoluteError = $globalLargest
        globalTendonWorldCounts = @(1, 2, 5, 513)
        globalTendonCount = 14
        globalTendonNnz = 16
        globalTendonWrapCount = 43
        globalTendonFixedIds = @(0, 3, 7, 13)
        globalTendonPreservesNativeOrder = $true
        globalTendonAssemblyLaunches = 3
        globalTendonResidentSubsets = 7
        globalTendonOutputBuffers = 9
        flexPositionComparedWorlds = 2084
        flexPositionComparedScalars = 625200
        flexPositionMaxAbsoluteError = $flexLargest
        flexPositionWorldCounts = @(1, 2, 5, 513)
        flexPositionCount = 4
        flexPositionNodeCount = 20
        flexPositionVertexCount = 80
        flexPositionInterpolationModes = @(0, 1)
        flexPositionSharedFields = $true
        flexPositionResidentSubsets = 8
        flexPositionOutputBuffers = 10
        tendonWakeComparedWorlds = $wakeStats.Count
        tendonWakeExactTreeStates = $true
        tendonWakeWorldCounts = @(1, 2, 5, 513)
        tendonWakeFieldPeriods = @(3, 2)
        tendonWakeLightweightBodyDofCounters = 0
        tendonWakeExtraIntegerBuffers = 2
        flexEdgeComparedWorlds = 2084
        flexEdgeComparedScalars = 1204552
        flexEdgeMaxAbsoluteError = $edgeLargest
        flexEdgeWorldCounts = @(1, 2, 5, 513)
        flexEdgeCount = 284
        flexEdgeNnz = 10
        flexEdgeExtraFloatBuffers = 2
        flexEdgeQvelDefault = 0.0
        flexEdgeSharedTopology = $true
        flexEdgeReadsDevicePositionsAndCom = $true
        flexFaceComparedWorlds = 2084
        flexFaceComparedScalars = 1033664
        flexFaceMaxPositionError = $facePositionLargest
        flexFaceMaxQuaternionError = $faceQuaternionLargest
        flexFaceWorldCounts = @(1, 2, 5, 513)
        flexFaceCount = 16
        flexFaceExtraFloatBuffers = 1
        flexFaceQuaternionOrder = 'xyzw'
        flexFaceReferenceUsesEigenPolar = $true
        flexFaceRequiresOnlyNodePositions = $true
        flexFaceHighOrderSupported = $false
        nvrtcLibraries = @(Get-ChildItem -LiteralPath (Join-Path $env:CUDA_PATH 'bin') -Filter 'nvrtc*.dll' |
            ForEach-Object { [ordered]@{ name = $_.Name; sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() } })
        scriptInvokesNativeKinematics = $false
        scriptInvokesModelCompiler = $false
        scriptInvokesPython = $false
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $directory 'report.json') -Encoding utf8
} finally { Pop-Location }
Write-Host "RESIDENT_KINEMATICS_EVIDENCE=$directory"
