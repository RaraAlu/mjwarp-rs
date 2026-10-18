$ErrorActionPreference = 'Stop'
$task = 'docs/development-evidence/layers-m3-inventory-derived-correction'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-inventory-derived-correction'
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

$runSets = @(
    @{id='build-snapshot';name='build-first'},
    @{id='preserve-build-failure';name='preserve-build-failure'},
    @{id='repair-private-sum';name='repair-private-sum'},
    @{id='check-normal';name='normal'},
    @{id='negative-duplicate';name='negative-duplicate'},
    @{id='negative-unknown';name='negative-unknown'},
    @{id='negative-drift';name='negative-drift'},
    @{id='negative-empty';name='negative-empty'},
    @{id='negative-cross-class';name='negative-cross-class'},
    @{id='negative-cross-set';name='negative-cross-set'},
    @{id='negative-private-summary';name='negative-private-summary'},
    @{id='evidence';name='evidence'}
)
foreach ($set in $runSets) {
    $registration = "$runtime/registrations/$($set.id).json"
    $record = Get-Content -LiteralPath $registration -Raw | ConvertFrom-Json
    $root = [string]$record.output_root
    Add-Original "$($set.name).registration.json" $registration
    Add-Original "$($set.name).started.json" "$root/started.json"
    Add-Original "$($set.name).run.json" "$root/result.json"
    Add-Original "$($set.name).stdout.log" "$root/stdout.log"
    Add-Original "$($set.name).stderr.log" "$root/stderr.log"
}

foreach ($item in @(
    @{path='source-inventory.json';source='docs/development-evidence/layers-m3-smooth-force-compute/unmigrated-launch-inventory.json'},
    @{path='corrected-inventory.json';source="$task/corrected-inventory.json"},
    @{path='claims.json';source="$task/claims.json"},
    @{path='execution-plan.json';source="$task/execution-plan.json"},
    @{path='register-run.ps1';source="$task/register-run.ps1"},
    @{path='invoke-registered-check.ps1';source="$task/invoke-registered-check.ps1"},
    @{path='inventory-lib.ps1';source="$task/inventory-lib.ps1"},
    @{path='build-corrected-inventory.ps1';source="$task/build-corrected-inventory.ps1"},
    @{path='check-corrected-inventory.ps1';source="$task/check-corrected-inventory.ps1"},
    @{path='run-negative-case.ps1';source="$task/run-negative-case.ps1"},
    @{path='check-evidence.ps1';source="$task/check-evidence.ps1"},
    @{path='preserve-failed-snapshot.ps1';source="$task/preserve-failed-snapshot.ps1"},
    @{path='verify-private-sum-repair.ps1';source="$task/verify-private-sum-repair.ps1"},
    @{path='failed-corrected-inventory.json';source="$runtime/check-01-preserve-failure/failed-corrected-inventory.json"}
)) {
    Add-Original $item.path $item.source
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
