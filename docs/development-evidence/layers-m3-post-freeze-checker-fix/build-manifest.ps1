$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-post-freeze-checker-fix'
$task = 'docs/development-evidence/layers-m3-post-freeze-checker-fix'
$runtime = "target/normify-evidence/$taskId"
$archiveRoot = "$task/formal-run"
$mapPath = "$task/source-map.json"
if (Test-Path -LiteralPath $archiveRoot) { throw "正式归档已存在：$archiveRoot" }
if (Test-Path -LiteralPath $mapPath) { throw "原件映射已存在：$mapPath" }
New-Item -ItemType Directory -Path $archiveRoot | Out-Null
$mappings = [Collections.Generic.List[object]]::new()

function Add-Original([string]$Path, [string]$Source) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少原件：$Source" }
    $destination = Join-Path $archiveRoot $Path
    $parent = Split-Path -Parent $destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    if (Test-Path -LiteralPath $destination) { throw "归档目标重复：$Path" }
    [IO.File]::WriteAllBytes($destination, [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source)))
    $mappings.Add([ordered]@{path=$Path;source=$Source})
}

foreach ($set in @(
    @{id='delivery-preflight';name='delivery-preflight'},
    @{id='static-mapping';name='static-mapping'},
    @{id='prior-commit-objects';name='prior-commit-objects'},
    @{id='static-mapping-retry-01';name='static-mapping-final'},
    @{id='prior-commit-objects-retry-01';name='prior-commit-objects-final'},
    @{id='scope';name='scope'}
)) {
    $registration = "$runtime/registrations/$($set.id).json"
    $record = Get-Content -LiteralPath $registration -Raw | ConvertFrom-Json
    $root = [string]$record.output_root
    Add-Original "$($set.name).registration.json" $registration
    Add-Original "$($set.name).started.json" "$root/started.json"
    Add-Original "$($set.name).run.json" "$root/result.json"
    Add-Original "$($set.name).stdout.log" "$root/stdout.log"
    Add-Original "$($set.name).stderr.log" "$root/stderr.log"
}

foreach ($source in @(
    'docs/development-evidence/layers-m3-inventory-derived-correction/check-post-freeze-blobs.ps1',
    "$task/.gitattributes",
    "$task/execution-plan.json",
    "$task/register-run.ps1",
    "$task/register-retry.ps1",
    "$task/invoke-registered-check.ps1",
    "$task/check-static-mapping.ps1",
    "$task/check-scope.ps1",
    "$task/build-manifest.ps1",
    "$task/freeze-manifest.ps1",
    "$task/check-formal-evidence.ps1",
    "$task/check-delivery-archives.ps1",
    "$task/build-delivery-receipt.ps1"
)) {
    $name = if ($source.StartsWith("$task/", [StringComparison]::Ordinal)) {
        $source.Substring($task.Length + 1)
    }
    else { 'fixed-check-post-freeze-blobs.ps1' }
    Add-Original "sources/$name" $source
}

$members = @($mappings | Sort-Object path | ForEach-Object {
    $path = Join-Path $archiveRoot $_.path
    [ordered]@{
        path = $_.path
        bytes = (Get-Item -LiteralPath $path).Length
        sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$manifest = [ordered]@{
    schema = 1
    root = $archiveRoot
    self_excluded = $true
    infrastructure_run_excluded = 'manifest builder cannot include its own post-exit result'
    member_count = $members.Count
    members = $members
}
$manifestJson = ($manifest | ConvertTo-Json -Depth 8).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) "$archiveRoot/manifest.json"), $manifestJson, [Text.UTF8Encoding]::new($false))
$map = [ordered]@{
    schema = 1
    archive_root = $archiveRoot
    mappings = @($mappings | Sort-Object path)
}
$mapJson = ($map | ConvertTo-Json -Depth 8).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $mapPath), $mapJson, [Text.UTF8Encoding]::new($false))
[ordered]@{schema=1;archive_root=$archiveRoot;members=$members.Count;map=$mapPath;result='pass'} | ConvertTo-Json
