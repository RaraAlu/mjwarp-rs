param(
    [Parameter(Mandatory = $true)]
    [string]$Revision
)

$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-inventory-derived-correction'
$evidence = 'docs/development-evidence/layers-m3-inventory-derived-correction'
$runtime = "target/normify-evidence/$task"
$postRoot = "$evidence/post-freeze-check"
$indexRoot = "$evidence/index-check"

function Assert-SameBytes([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf) -or
        -not (Test-Path -LiteralPath $Destination -PathType Leaf)) {
        throw "缺少字节核验文件：$Source -> $Destination"
    }
    if ((Get-Item -LiteralPath $Source).Length -ne (Get-Item -LiteralPath $Destination).Length -or
        (Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash -cne
        (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash) {
        throw "原件与归档不符：$Destination"
    }
}

$mappings = [Collections.Generic.List[object]]::new()
function Join-RunArchivePath(
    [string]$Root,
    [string]$Name,
    [string]$NamedSuffix,
    [string]$EmptySuffix = $NamedSuffix
) {
    $file = if ([string]::IsNullOrEmpty($Name)) { $EmptySuffix } else { "$Name.$NamedSuffix" }
    "$Root/$file"
}

function Add-RunMappings([string]$Id, [string]$Name, [string]$DestinationRoot) {
    $registration = "$runtime/registrations/$Id.json"
    $record = Get-Content -LiteralPath $registration -Raw | ConvertFrom-Json
    $runRoot = [string]$record.output_root
    foreach ($item in @(
        @{source=$registration;destination=(Join-RunArchivePath $DestinationRoot $Name 'registration.json')},
        @{source="$runRoot/started.json";destination=(Join-RunArchivePath $DestinationRoot $Name 'started.json')},
        @{source="$runRoot/result.json";destination=(Join-RunArchivePath $DestinationRoot $Name 'run.json' 'result.json')},
        @{source="$runRoot/stdout.log";destination=(Join-RunArchivePath $DestinationRoot $Name 'stdout.log')},
        @{source="$runRoot/stderr.log";destination=(Join-RunArchivePath $DestinationRoot $Name 'stderr.log')}
    )) { $mappings.Add($item) }
}

Add-RunMappings 'evidence-final' 'evidence-final' $postRoot
Add-RunMappings 'manifest' 'manifest' "$postRoot/infrastructure"
Add-RunMappings 'manifest-blob' 'manifest-blob' "$postRoot/infrastructure"
Add-RunMappings 'delivery-preflight' 'preflight-first' "$postRoot/delivery"
Add-RunMappings 'delivery-preflight-retry-01' 'preflight-final' "$postRoot/delivery"
Add-RunMappings 'receipt' 'receipt-first' "$postRoot/delivery"
Add-RunMappings 'receipt-retry-01' 'receipt-retry-first' "$postRoot/delivery"
Add-RunMappings 'index-bytes' '' $indexRoot
foreach ($mapping in $mappings) {
    Assert-SameBytes $mapping.source $mapping.destination
}

if (Test-Path -LiteralPath "$postRoot/index-extra/result.json") {
    Add-RunMappings 'post-freeze-index-retry-01' '' "$postRoot/index-extra"
    foreach ($mapping in @($mappings | Select-Object -Last 5)) {
        Assert-SameBytes $mapping.source $mapping.destination
    }
}

$postManifest = Get-Content -LiteralPath "$postRoot/manifest.json" -Raw | ConvertFrom-Json
foreach ($member in $postManifest.members) {
    $path = Join-Path $postRoot $member.path
    if ((Get-Item -LiteralPath $path).Length -ne $member.bytes -or
        (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $member.sha256) {
        throw "收尾清单成员漂移：$($member.path)"
    }
}

$tracked = @(
    Get-ChildItem -LiteralPath $postRoot, $indexRoot -Recurse -File |
        Sort-Object FullName
)
foreach ($file in $tracked) {
    $relative = $file.FullName.Substring((Resolve-Path -LiteralPath '.').Path.Length + 1).Replace('\', '/')
    $reference = if ($Revision -ceq 'INDEX') { ":$relative" } else { "$Revision`:$relative" }
    $object = @(& git rev-parse $reference)[0].Trim()
    if ($LASTEXITCODE -ne 0 -or $object -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$relative" }
    $workingObject = @(& git hash-object --no-filters -- $relative)[0].Trim()
    if ($LASTEXITCODE -ne 0 -or $workingObject -cne $object) { throw "对象字节不符：$relative" }
    $objectSize = [int64]@(& git cat-file -s $object)[0]
    if ($LASTEXITCODE -ne 0 -or $objectSize -ne $file.Length) { throw "对象长度不符：$relative" }
}

[ordered]@{
    schema = 1
    revision = $Revision
    original_archive_matches = $mappings.Count
    tracked_files = $tracked.Count
    post_manifest_members = $postManifest.member_count
    result = 'pass'
} | ConvertTo-Json
