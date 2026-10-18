param(
    [ValidateSet(
        'independent',
        'independent-attempt-1',
        'independent-attempt-2',
        'independent-attempt-3',
        'nonzero'
    )]
    [string]$SourceId = 'independent',
    [switch]$SelfTest
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$DestinationRoot = Join-Path $Root 'target\mjwarp-parallel\gravity-upstream-authoring'
$LegacyCacheRoot = 'CACHE_ROOT = ROOT / "target" / "mjwarp-parallel" / "warp-cache" / "1.15.0"'
$IsolatedCacheRoot = 'CACHE_ROOT = ROOT / "target" / "mjwarp-parallel" / "gravity-upstream-warp-cache"'
$LegacyProjectRoot = 'ROOT = HERE.parents[2]'
$ArtifactSuffix = '1.15.0/wp_mujoco_warp._src.smooth_ca013d6'

$Sources = @{
    independent = @{
        Path = 'fixtures/gravity-upstream-abi/independent-kinematics/generate.py'
        Sha256 = '7366b81dba2133f1e09eb8c2cbf27121af271edbb8295bc3f3c4b70c6793bd05'
    }
    'independent-attempt-1' = @{
        Path = 'fixtures/gravity-upstream-abi/independent-kinematics/attempt-1/generate.py'
        Sha256 = 'fcff2ce6ad14438cbdea29a1d511e764eda94bd13b514cabf29e98d44d77a589'
    }
    'independent-attempt-2' = @{
        Path = 'fixtures/gravity-upstream-abi/independent-kinematics/attempt-2/generate.py'
        Sha256 = '48a3195bbf4169d46ccae9066c42952e4d78949e22f50ade0341562b8327a20d'
    }
    'independent-attempt-3' = @{
        Path = 'fixtures/gravity-upstream-abi/independent-kinematics/attempt-3/generate.py'
        Sha256 = '7366b81dba2133f1e09eb8c2cbf27121af271edbb8295bc3f3c4b70c6793bd05'
    }
    nonzero = @{
        Path = 'fixtures/gravity-upstream-abi/nonzero-kinematics/generate.py'
        Sha256 = '43436c4d73af31bcfc84d5b3dbe58fc171a0641e82c081f8954dded8801342a2'
    }
}

function Get-RelativePath([string]$Path) {
    [IO.Path]::GetRelativePath($Root, $Path).Replace('\', '/')
}

function Assert-FrozenHash([string]$Path, [string]$ExpectedHash) {
    $ActualHash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ActualHash -cne $ExpectedHash) {
        throw "Frozen generator hash mismatch: $(Get-RelativePath $Path)"
    }
    $ActualHash
}

function Copy-Dependency([string]$Source, [string]$Destination) {
    $Parent = Split-Path -Parent $Destination
    New-Item -ItemType Directory -Path $Parent -Force | Out-Null
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
    $SourceHash = (Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash
    $DestinationHash = (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash
    if ($SourceHash -cne $DestinationHash) {
        throw "Dependency copy mismatch: $(Get-RelativePath $Destination)"
    }
}

function New-Successor([string]$Id, [string]$SourcePathOverride = '') {
    $Spec = $Sources[$Id]
    $SourcePath = if ($SourcePathOverride) { $SourcePathOverride } else { Join-Path $Root $Spec.Path }
    $ActualHash = Assert-FrozenHash $SourcePath $Spec.Sha256
    $SourceDirectory = Split-Path -Parent (Join-Path $Root $Spec.Path)
    $SourceText = [IO.File]::ReadAllText($SourcePath)

    foreach ($ExpectedLine in @($LegacyCacheRoot, $LegacyProjectRoot)) {
        $Matches = [regex]::Matches($SourceText, [regex]::Escape($ExpectedLine))
        if ($Matches.Count -ne 1) {
            throw "Expected one frozen contract line: $($Spec.Path)"
        }
    }

    $ProjectRoot = $Root.Replace('\', '/')
    $SuccessorText = $SourceText.Replace(
        $LegacyProjectRoot,
        "ROOT = pathlib.Path(r`"$ProjectRoot`")"
    ).Replace($LegacyCacheRoot, $IsolatedCacheRoot)

    $SuccessorRoot = Join-Path $DestinationRoot $Id
    $SuccessorPath = Join-Path $SuccessorRoot 'generate.py'
    New-Item -ItemType Directory -Path $SuccessorRoot -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $SuccessorRoot 'attempt-3') -Force | Out-Null
    [IO.File]::WriteAllText($SuccessorPath, $SuccessorText, [Text.UTF8Encoding]::new($false))

    Copy-Dependency (Join-Path $SourceDirectory 'input.json') (Join-Path $SuccessorRoot 'input.json')
    $SharedSource = Join-Path $Root 'fixtures\gravity-upstream-abi\independent-inputs'
    $SharedDestination = Join-Path $DestinationRoot 'independent-inputs'
    Copy-Dependency (Join-Path $SharedSource 'input.json') (Join-Path $SharedDestination 'input.json')
    Copy-Dependency (Join-Path $SharedSource 'output.json') (Join-Path $SharedDestination 'output.json')

    $Config = Get-Content -LiteralPath (Join-Path $SuccessorRoot 'input.json') -Raw | ConvertFrom-Json
    if ($Config.legacy_static_input) {
        $ResolvedLegacy = [IO.Path]::GetFullPath((Join-Path $SuccessorRoot $Config.legacy_static_input))
        $ExpectedLegacy = [IO.Path]::GetFullPath((Join-Path $SharedDestination 'input.json'))
        if ($ResolvedLegacy -cne $ExpectedLegacy) {
            throw "Legacy input path escaped prepared dependencies: $Id"
        }
    }

    $SuccessorHash = (Get-FileHash -LiteralPath $SuccessorPath -Algorithm SHA256).Hash.ToLowerInvariant()
    [ordered]@{
        source_id = $Id
        source_path = $Spec.Path
        source_sha256 = $ActualHash
        successor_path = Get-RelativePath $SuccessorPath
        successor_sha256 = $SuccessorHash
        input_path = Get-RelativePath (Join-Path $SuccessorRoot 'input.json')
        project_root = $ProjectRoot
        cache_root = 'target/mjwarp-parallel/gravity-upstream-warp-cache'
        artifact_path = "target/mjwarp-parallel/gravity-upstream-warp-cache/$ArtifactSuffix"
        executes_python = $false
    }
}

if ($SelfTest) {
    $Before = @{}
    foreach ($Id in $Sources.Keys) {
        $Spec = $Sources[$Id]
        $Before[$Id] = Assert-FrozenHash (Join-Path $Root $Spec.Path) $Spec.Sha256
    }

    $Results = foreach ($Id in ($Sources.Keys | Sort-Object)) {
        New-Successor $Id
    }

    foreach ($Id in $Sources.Keys) {
        $Spec = $Sources[$Id]
        $After = Assert-FrozenHash (Join-Path $Root $Spec.Path) $Spec.Sha256
        if ($After -cne $Before[$Id]) {
            throw "Frozen generator changed during preparation: $Id"
        }
    }

    $Tampered = Join-Path $DestinationRoot 'tampered-generate.py'
    $Bytes = [IO.File]::ReadAllBytes((Join-Path $Root $Sources.independent.Path))
    [IO.File]::WriteAllBytes($Tampered, $Bytes + [byte[]](10))
    $Rejected = $false
    try {
        New-Successor 'independent' $Tampered | Out-Null
    } catch {
        $Rejected = $_.Exception.Message.StartsWith('Frozen generator hash mismatch:')
    }
    if (-not $Rejected) {
        throw 'Tampered frozen generator was not rejected'
    }

    [ordered]@{
        source_count = $Results.Count
        frozen_hashes_preserved = $true
        tampered_source_rejected = $true
        dependency_contract_complete = $true
        single_version_cache_path = $true
        executes_python = $false
        results = $Results
    } | ConvertTo-Json -Depth 5
    exit 0
}

New-Successor $SourceId | ConvertTo-Json -Depth 3
