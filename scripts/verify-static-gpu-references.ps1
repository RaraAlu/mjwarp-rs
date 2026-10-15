#Requires -Version 7.0
[CmdletBinding()]
param(
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot),
    [switch]$SelfTest,
    [switch]$RequireComplete
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = [IO.Path]::GetFullPath($RepositoryRoot)
$revision = '71da24d956378a87a703b6e1442b13aec0c4ac29'
$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
$checks = 0

function Resolve-ReferencePath([string]$Base, [string]$Relative) {
    if ([IO.Path]::IsPathRooted($Relative)) { throw 'Reference path must be relative' }
    $basePath = [IO.Path]::GetFullPath($Base).TrimEnd([char[]]@('/', '\'))
    $path = [IO.Path]::GetFullPath((Join-Path $basePath $Relative))
    $prefix = $basePath + [IO.Path]::DirectorySeparatorChar
    if (-not $path.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Reference path escapes its declared base'
    }
    return $path
}

function Assert-ReferenceHash([string]$Base, [string]$Relative, [string]$Expected) {
    if ($Expected -notmatch '^[0-9a-fA-F]{64}$') { throw 'Invalid SHA-256 declaration' }
    $path = Resolve-ReferencePath $Base $Relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing reference file: $Relative" }
    $actual = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
    if ($actual -ne $Expected) { throw "Reference hash mismatch: $Relative" }
    [void]$seen.Add($path)
    $script:checks += 1
}

function Assert-ReferenceMap([string]$Base, [Collections.IDictionary]$Map) {
    if ($Map.Count -eq 0) { throw 'Empty reference map cannot pass' }
    foreach ($name in $Map.Keys) { Assert-ReferenceHash $Base $name $Map[$name] }
}

function Assert-ReferenceCoverage([int]$Count) {
    if ($Count -le 0) { throw 'Zero reference coverage cannot pass' }
}

function Assert-CompleteReferenceCatalog($Catalog) {
    if ($Catalog.missing_required_families.Count -ne 0 -or -not $Catalog.complete) {
        throw 'D04 incomplete: required complex-physics GPU references are missing'
    }
}

function Assert-ExpectedFailure([scriptblock]$Action, [string]$Message) {
    try { & $Action }
    catch {
        if ($_.Exception.Message -notmatch $Message) { throw }
        return
    }
    throw "Expected rejection did not occur: $Message"
}

function Assert-ReferenceDeclarations($Mass, $Geom, $MassOutput, $GeomOutput) {
    foreach ($set in @(
        @('mass-input', $Mass.inputs.Count, 18),
        @('mass-source', $Mass.source_hashes.Count, 75),
        @('mass-cache', $Mass.cache_hashes.Count, 35),
        @('geom-input', $Geom.inputs.Count, 3),
        @('geom-source', $Geom.source_hashes.Count, 75),
        @('geom-cache', $Geom.cache_hashes.Count, 3),
        @('mass-model', $MassOutput.models.Count, 6),
        @('geom-row', $GeomOutput.rows.Count, 37)
    )) {
        if ($set[1] -ne $set[2]) { throw "$($set[0]) coverage denominator mismatch" }
    }
    if ($MassOutput.revision -ne $revision -or $GeomOutput.upstream_revision -ne $revision) {
        throw 'Output revision mismatch'
    }
}

function Assert-ComplexReferenceDeclarations($Manifest, $Output) {
    if ($Manifest.upstream_revision -ne $revision -or $Output.upstream_revision -ne $revision) {
        throw 'Complex output revision mismatch'
    }
    if ($Manifest.run_id -ne 'run-5') { throw 'Complex authoring run mismatch' }
    foreach ($set in @(
        @('complex-input', $Manifest.inputs.Count, 14),
        @('complex-source', $Manifest.source_hashes.Count, 75),
        @('complex-installed-source', $Manifest.installed_source_hashes.Count, 75),
        @('complex-cache', $Manifest.cache_hashes.Count, 78),
        @('complex-collision-case', $Output.collision_cases.Count, 12),
        @('complex-flex-case', $Output.flex_cases.Count, 4),
        @('complex-probe-case', $Output.probes.Count, 1)
    )) {
        if ($set[1] -ne $set[2]) { throw "$($set[0]) coverage denominator mismatch" }
    }
    $expected = [ordered]@{ G07 = 4; G08 = 2; G09 = 3; G10 = 3; G11 = 3; G22 = 4 }
    foreach ($family in $expected.Keys) {
        if (-not $Output.coverage_contract.Contains($family)) { throw "Missing complex family: $family" }
        $actual = @($Output.coverage_contract[$family].Values | ForEach-Object { @($_).Count } | Measure-Object -Sum).Sum
        if ($actual -ne $expected[$family]) { throw "$family coverage denominator mismatch" }
    }
    foreach ($name in $Manifest.source_hashes.Keys) {
        if ($Manifest.installed_source_hashes[$name] -ne $Manifest.source_hashes[$name]) {
            throw "Imported source identity mismatch: $name"
        }
    }
}

function Assert-ArrayRecord($Record, [string]$Dtype, [switch]$RequireBits) {
    if ($null -eq $Record -or $Record.dtype -ne $Dtype) { throw "Missing $Dtype array record" }
    $elements = 1
    foreach ($dimension in @($Record.shape)) { $elements *= [int]$dimension }
    if (@($Record.values).Count -ne $elements) { throw "$Dtype array value denominator mismatch" }
    if ($RequireBits -and @($Record.bits).Count -ne $elements) { throw "$Dtype array bit denominator mismatch" }
}

function Assert-ComplexReferenceContent($Output) {
    $contactFloat = @('dist', 'pos', 'frame', 'includemargin', 'friction', 'solref', 'solreffriction', 'solimp', 'adhesion')
    $contactInt = @('dim', 'geom', 'flex', 'elem', 'vert', 'worldid', 'type', 'geomcollisionid')
    $collisionNames = @('plane-sphere-nxn', 'plane-sphere-sap-tile', 'plane-sphere-sap-segmented', 'plane-sphere-sphere-only-precheck', 'sphere-sphere', 'convex-mesh', 'box-box-multiccd', 'degenerate-mesh', 'hfield-box', 'sdf-cylinder', 'flex-geom', 'flex-self')
    foreach ($name in $collisionNames) {
        $case = @($Output.collision_cases | Where-Object name -eq $name)
        if ($case.Count -ne 1) { throw "Missing complex collision case: $name" }
        if ($name -eq 'flex-self') {
            Assert-ArrayRecord $case[0].input.flexvert_xpos_override 'float32' -RequireBits
        } else {
            Assert-ArrayRecord $case[0].input.qpos 'float32' -RequireBits
            Assert-ArrayRecord $case[0].input.qvel 'float32' -RequireBits
        }
        foreach ($field in $contactFloat) { Assert-ArrayRecord $case[0].output.fields[$field] 'float32' }
        foreach ($field in $contactInt) { Assert-ArrayRecord $case[0].output.fields[$field] 'int32' }
    }
    $box = @($Output.collision_cases | Where-Object name -eq 'box-box-multiccd')[0]
    if ($null -eq $box.boundary_evidence -or $null -eq $box.boundary_evidence.contacts_per_world) {
        throw 'Missing multicontact boundary evidence'
    }
    foreach ($count in $box.boundary_evidence.contacts_per_world.Values) {
        if ([int]$count -lt 2) { throw 'Multicontact per-world denominator mismatch' }
    }
    $self = @($Output.collision_cases | Where-Object name -eq 'flex-self')[0]
    if ($null -eq $self.boundary_evidence -or [int]$self.boundary_evidence.same_flex_contacts -le 0 -or $self.boundary_evidence.distinct_element_pairs.Count -le 0) {
        throw 'Missing flex self-collision boundary evidence'
    }
    $flexFields = @('flexnode_xpos', 'flexvert_xpos', 'flexedge_length', 'flexedge_velocity', 'flexedge_J', 'face_xpos', 'face_quat', 'flexvert_hessian', 'flexedge_hessian', 'qfrc_spring', 'qfrc_damper', 'qfrc_passive')
    foreach ($name in @('rope-edge', 'cloth-stretch-bend', 'softbody-3d', 'contact-coupled')) {
        $case = @($Output.flex_cases | Where-Object name -eq $name)
        if ($case.Count -ne 1) { throw "Missing complex flex case: $name" }
        Assert-ArrayRecord $case[0].input.qpos 'float32' -RequireBits
        Assert-ArrayRecord $case[0].input.qvel 'float32' -RequireBits
        foreach ($field in $flexFields) { Assert-ArrayRecord $case[0].output.fields[$field] 'float32' }
        Assert-ArrayRecord $case[0].output.fields.flex_hessian_valid 'bool'
    }
    $probe = @($Output.probes | Where-Object name -eq 'volume-gradient-outside')
    if ($probe.Count -ne 1) { throw 'Missing volume gradient boundary probe' }
    Assert-ArrayRecord $probe[0].input.points 'float32' -RequireBits
    Assert-ArrayRecord $probe[0].output.distance 'float32'
    Assert-ArrayRecord $probe[0].output.gradient 'float32'
}

function Copy-ReferenceDeclaration($Value) {
    return ($Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -AsHashtable)
}

# 只读冻结记录；不执行参考制作脚本。
Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/manifest.json' '08cc1fd3670915c7cdbaf63027bf5c1e8d50b3711896521799994235028e5462'
Assert-ReferenceHash $root 'fixtures/mjwarp-geom-tendon/provenance.json' '5ef29b13c82f43a01cad9e303276cf52bf362d8dca16fe46c79d204d5edd5a6a'
Assert-ReferenceHash $root 'fixtures/mjwarp-complex-physics/manifest.json' 'c0b21c855cda70eb06ee401c8921a5a3d51505fe31f053b7b715dd62fefccf2b'
Assert-ReferenceHash $root 'docs/evidence/d04-complex-physics/archive-manifest.json' 'af57bb59c9c8fb860a45df654b8c6e020463969bea45e7398a23aba00c6fdbef'
$mass = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-mass-solve/manifest.json') -Raw | ConvertFrom-Json -AsHashtable
$geom = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-geom-tendon/provenance.json') -Raw | ConvertFrom-Json -AsHashtable
$complex = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-complex-physics/manifest.json') -Raw | ConvertFrom-Json -AsHashtable
$complexArchiveRoot = Join-Path $root 'docs/evidence/d04-complex-physics'
$complexArchive = Get-Content -LiteralPath (Join-Path $complexArchiveRoot 'archive-manifest.json') -Raw | ConvertFrom-Json -AsHashtable
if ($mass.revision -ne $revision -or $geom.upstream_revision -ne $revision -or $complex.upstream_revision -ne $revision) { throw 'Frozen upstream revision mismatch' }
$upstream = Join-Path $root 'target/mjwarp-parallel/upstream/mujoco_warp'
Assert-ReferenceMap $root $mass.inputs
Assert-ReferenceMap $upstream $mass.source_hashes
# 两个作者工具声明了不同WORK；禁止混用同名缓存。
Assert-ReferenceMap (Join-Path $root 'target/g03-block-reference') $mass.cache_hashes
Assert-ReferenceMap $root $geom.inputs
Assert-ReferenceMap $upstream $geom.source_hashes
Assert-ReferenceMap (Join-Path $root 'target/mjwarp-parallel') $geom.cache_hashes
Assert-ReferenceMap $root $complex.inputs
Assert-ReferenceMap $complexArchiveRoot $complexArchive.files
$complexFinalRun = Join-Path $complexArchiveRoot 'authoring-runs/run-5'
Assert-ReferenceMap (Join-Path $complexFinalRun 'frozen-source') $complex.source_hashes
Assert-ReferenceMap (Join-Path $complexFinalRun 'installed-source') $complex.installed_source_hashes
Assert-ReferenceMap $complexFinalRun $complex.cache_hashes
Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.json' $mass.reference_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.py' $mass.authoring_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-geom-tendon/reference.json' $geom.reference_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-geom-tendon/reference.py' $geom.authoring_script_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-complex-physics/reference.json' $complex.reference_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-complex-physics/reference.py' $complex.authoring_script_sha256
$massOutput = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-mass-solve/reference.json') -Raw | ConvertFrom-Json -AsHashtable
$geomOutput = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-geom-tendon/reference.json') -Raw | ConvertFrom-Json -AsHashtable
$complexOutput = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-complex-physics/reference.json') -Raw | ConvertFrom-Json -AsHashtable
Assert-ReferenceDeclarations $mass $geom $massOutput $geomOutput
Assert-ComplexReferenceDeclarations $complex $complexOutput
Assert-ComplexReferenceContent $complexOutput
if ($massOutput.scalar -ne 'f32') { throw 'Mass reference scalar mismatch' }
Assert-ReferenceCoverage $massOutput.models.Count
Assert-ReferenceCoverage $geomOutput.rows.Count

$catalog = [ordered]@{
    schema = 1
    upstream_revision = $revision
    verified_hash_relations = $checks
    unique_files = $seen.Count
    mass_models = $massOutput.models.Count
    geom_rows = $geomOutput.rows.Count
    reference_versions = @{ mass = $mass.packages; geom = $geom.packages; complex = $complex.packages }
    available_families = @('G03 mass-solve subset', 'G01 geom-tendon diagnostic subset', 'G07-G11 complex contacts', 'G22 complete flex dynamics')
    missing_required_families = @()
    complete = $true
    physical_numerical_acceptance = $false
    fair_performance_measured = $false
    reference_authoring_runs = 5
    admitted_authoring_run = 'run-5'
    failed_or_rejected_authoring_runs = 4
    python_invocations = 5
    gpu_sessions = 5
    self_tests_passed = 0
}

if ($SelfTest) {
    # 实际目录检查计一次；以下反例复用结果，不再扫描全部样本。
    Assert-ExpectedFailure { Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.json' ('0' * 64) } 'hash mismatch'
    Assert-ExpectedFailure { Assert-ReferenceHash $root 'target/d04-intentionally-missing-reference-file' ('0' * 64) } 'Missing reference'
    Assert-ExpectedFailure { Resolve-ReferencePath $root '../outside-reference' } 'escapes'
    Assert-ExpectedFailure { Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.json' 'not-a-hash' } 'Invalid SHA'
    Assert-ExpectedFailure { Assert-ReferenceCoverage 0 } 'Zero reference coverage'
    $incompleteCatalog = Copy-ReferenceDeclaration $catalog
    $incompleteCatalog.complete = $false
    $incompleteCatalog.missing_required_families = @('G22 complete flex dynamics')
    Assert-ExpectedFailure { Assert-CompleteReferenceCatalog $incompleteCatalog } 'D04 incomplete'
    Assert-ExpectedFailure { Assert-ReferenceMap $root @{} } 'Empty reference map'
    $badMass = Copy-ReferenceDeclaration $mass
    [void]$badMass.inputs.Remove(@($badMass.inputs.Keys)[0])
    Assert-ExpectedFailure { Assert-ReferenceDeclarations $badMass $geom $massOutput $geomOutput } 'mass-input coverage'
    $badGeom = Copy-ReferenceDeclaration $geom
    [void]$badGeom.source_hashes.Remove(@($badGeom.source_hashes.Keys)[0])
    Assert-ExpectedFailure { Assert-ReferenceDeclarations $mass $badGeom $massOutput $geomOutput } 'geom-source coverage'
    $badMass = Copy-ReferenceDeclaration $mass
    [void]$badMass.cache_hashes.Remove(@($badMass.cache_hashes.Keys)[0])
    Assert-ExpectedFailure { Assert-ReferenceDeclarations $badMass $geom $massOutput $geomOutput } 'mass-cache coverage'
    $badMassOutput = Copy-ReferenceDeclaration $massOutput
    $badMassOutput.models = @($badMassOutput.models)[0..4]
    Assert-ExpectedFailure { Assert-ReferenceDeclarations $mass $geom $badMassOutput $geomOutput } 'mass-model coverage'
    $badGeomOutput = Copy-ReferenceDeclaration $geomOutput
    $badGeomOutput.rows = @($badGeomOutput.rows)[0..35]
    Assert-ExpectedFailure { Assert-ReferenceDeclarations $mass $geom $massOutput $badGeomOutput } 'geom-row coverage'
    $badMassOutput = Copy-ReferenceDeclaration $massOutput
    $badMassOutput.revision = 'wrong-revision'
    Assert-ExpectedFailure { Assert-ReferenceDeclarations $mass $geom $badMassOutput $geomOutput } 'Output revision'
    $badGeomOutput = Copy-ReferenceDeclaration $geomOutput
    $badGeomOutput.upstream_revision = 'wrong-revision'
    Assert-ExpectedFailure { Assert-ReferenceDeclarations $mass $geom $massOutput $badGeomOutput } 'Output revision'
    $badComplex = Copy-ReferenceDeclaration $complex
    [void]$badComplex.inputs.Remove(@($badComplex.inputs.Keys)[0])
    Assert-ExpectedFailure { Assert-ComplexReferenceDeclarations $badComplex $complexOutput } 'complex-input coverage'
    $badComplex = Copy-ReferenceDeclaration $complex
    [void]$badComplex.cache_hashes.Remove(@($badComplex.cache_hashes.Keys)[0])
    Assert-ExpectedFailure { Assert-ComplexReferenceDeclarations $badComplex $complexOutput } 'complex-cache coverage'
    $badComplexOutput = Copy-ReferenceDeclaration $complexOutput
    $badComplexOutput.flex_cases = @($badComplexOutput.flex_cases)[0..2]
    Assert-ExpectedFailure { Assert-ComplexReferenceDeclarations $complex $badComplexOutput } 'complex-flex-case coverage'
    $badComplexOutput = Copy-ReferenceDeclaration $complexOutput
    $badComplexOutput.upstream_revision = 'wrong-revision'
    Assert-ExpectedFailure { Assert-ComplexReferenceDeclarations $complex $badComplexOutput } 'Complex output revision'
    $badComplexOutput = Copy-ReferenceDeclaration $complexOutput
    $badComplexOutput.collision_cases[0].input.qpos = $null
    Assert-ExpectedFailure { Assert-ComplexReferenceContent $badComplexOutput } 'Missing float32 array record'
    $badComplexOutput = Copy-ReferenceDeclaration $complexOutput
    $boxIndex = 0
    while ($badComplexOutput.collision_cases[$boxIndex].name -ne 'box-box-multiccd') { $boxIndex += 1 }
    $badComplexOutput.collision_cases[$boxIndex].boundary_evidence = $null
    Assert-ExpectedFailure { Assert-ComplexReferenceContent $badComplexOutput } 'Missing multicontact boundary evidence'
    $catalog.self_tests_passed = 20
}

$catalog | ConvertTo-Json -Depth 6
if ($RequireComplete) {
    # 这里只关闭参考目录，不代替产品数值或性能验收。
    if (-not $catalog.complete) { exit 2 }
}
