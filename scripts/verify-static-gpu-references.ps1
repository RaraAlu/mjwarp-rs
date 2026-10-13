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

function Copy-ReferenceDeclaration($Value) {
    return ($Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -AsHashtable)
}

# 只读冻结记录；不执行参考制作脚本。
Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/manifest.json' '08cc1fd3670915c7cdbaf63027bf5c1e8d50b3711896521799994235028e5462'
Assert-ReferenceHash $root 'fixtures/mjwarp-geom-tendon/provenance.json' '5ef29b13c82f43a01cad9e303276cf52bf362d8dca16fe46c79d204d5edd5a6a'
$mass = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-mass-solve/manifest.json') -Raw | ConvertFrom-Json -AsHashtable
$geom = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-geom-tendon/provenance.json') -Raw | ConvertFrom-Json -AsHashtable
if ($mass.revision -ne $revision -or $geom.upstream_revision -ne $revision) { throw 'Frozen upstream revision mismatch' }
$upstream = Join-Path $root 'target/mjwarp-parallel/upstream/mujoco_warp'
Assert-ReferenceMap $root $mass.inputs
Assert-ReferenceMap $upstream $mass.source_hashes
# 两个作者工具声明了不同WORK；禁止混用同名缓存。
Assert-ReferenceMap (Join-Path $root 'target/g03-block-reference') $mass.cache_hashes
Assert-ReferenceMap $root $geom.inputs
Assert-ReferenceMap $upstream $geom.source_hashes
Assert-ReferenceMap (Join-Path $root 'target/mjwarp-parallel') $geom.cache_hashes
Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.json' $mass.reference_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.py' $mass.authoring_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-geom-tendon/reference.json' $geom.reference_sha256
Assert-ReferenceHash $root 'fixtures/mjwarp-geom-tendon/reference.py' $geom.authoring_script_sha256
$massOutput = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-mass-solve/reference.json') -Raw | ConvertFrom-Json -AsHashtable
$geomOutput = Get-Content -LiteralPath (Join-Path $root 'fixtures/mjwarp-geom-tendon/reference.json') -Raw | ConvertFrom-Json -AsHashtable
Assert-ReferenceDeclarations $mass $geom $massOutput $geomOutput
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
    reference_versions = @{ mass = $mass.packages; geom = $geom.packages }
    available_families = @('G03 mass-solve subset', 'G01 geom-tendon diagnostic subset')
    missing_required_families = @('G07-G11 complex contacts', 'G22 complete flex dynamics')
    complete = $false
    physical_numerical_acceptance = $false
    fair_performance_measured = $false
    python_invocations = 0
    gpu_sessions = 0
    self_tests_passed = 0
}

if ($SelfTest) {
    # 实际目录检查计一次；以下反例复用结果，不再扫描全部样本。
    Assert-ExpectedFailure { Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.json' ('0' * 64) } 'hash mismatch'
    Assert-ExpectedFailure { Assert-ReferenceHash $root 'target/d04-intentionally-missing-reference-file' ('0' * 64) } 'Missing reference'
    Assert-ExpectedFailure { Resolve-ReferencePath $root '../outside-reference' } 'escapes'
    Assert-ExpectedFailure { Assert-ReferenceHash $root 'fixtures/mjwarp-mass-solve/reference.json' 'not-a-hash' } 'Invalid SHA'
    Assert-ExpectedFailure { Assert-ReferenceCoverage 0 } 'Zero reference coverage'
    Assert-ExpectedFailure { Assert-CompleteReferenceCatalog $catalog } 'D04 incomplete'
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
    $catalog.self_tests_passed = 14
}

$catalog | ConvertTo-Json -Depth 6
if ($RequireComplete) {
    # 返回2只表示参考目录不完整，不代替任何物理数值失败。
    if (-not $catalog.complete) { exit 2 }
}
