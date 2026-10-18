$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-rne-backward'
$task = 'docs/development-evidence/layers-m3-rne-backward'
$runtime = "target/normify-evidence/$taskId"
$archiveRoot = "$task/formal-run"
$frozenRoot = "$task/frozen"
$mapPath = "$task/source-map.json"
foreach ($path in @($archiveRoot, $frozenRoot, $mapPath)) {
    if (Test-Path -LiteralPath $path) { throw "正式输出已存在：$path" }
}
New-Item -ItemType Directory -Path $archiveRoot | Out-Null
New-Item -ItemType Directory -Path $frozenRoot | Out-Null
$mappings = [Collections.Generic.List[object]]::new()

function Copy-New([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少原件：$Source" }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    if (Test-Path -LiteralPath $Destination) { throw "目标重复：$Destination" }
    [IO.File]::WriteAllBytes($Destination, [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source)))
}

function Add-Original([string]$Path, [string]$Source) {
    $destination = Join-Path $archiveRoot $Path
    Copy-New $Source $destination
    $mappings.Add([ordered]@{path=$Path;source=$Source})
}

Copy-New 'src/physics/rne.rs' "$frozenRoot/current-rne.rs"
Copy-New 'src/physics/rne.cu' "$frozenRoot/current-rne.cu"
Copy-New "$task/rne_backward-a5b528ff34301f8e358d13a53b5dfbd69676fcc02987d39a3148d9693b8ebd22.ptx" "$frozenRoot/current-rne-backward.ptx"

$runSets = @(
    @{id='format-apply';name='format-apply'},
    @{id='format';name='format'},
    @{id='host-wrapper';name='host-wrapper'},
    @{id='format-after-gpu-code-apply';name='format-after-gpu-code-apply'},
    @{id='format-after-gpu-code';name='format-after-gpu-code'},
    @{id='gpu-selector-list';name='gpu-selector-list'},
    @{id='clippy-all-targets';name='clippy-all-targets'},
    @{id='device-preflight';name='device-preflight'},
    @{id='gpu-wrapper-captured';name='gpu-wrapper-captured'},
    @{id='inventory';name='inventory-v1'},
    @{id='capture-evidence';name='capture-evidence'},
    @{id='precision-map';name='precision-map-v1'},
    @{id='evidence';name='evidence-v1'},
    @{id='inventory-v2';name='inventory-v2'},
    @{id='precision-map-v2';name='precision-map-v2'},
    @{id='evidence-retry-01';name='evidence-v2'},
    @{id='evidence-retry-02';name='evidence-final'},
    @{id='delivery-preflight';name='delivery-preflight'}
)
foreach ($set in $runSets) {
    $registration = "$runtime/registrations/$($set.id).json"
    $record = Get-Content -LiteralPath $registration -Raw | ConvertFrom-Json
    $runRoot = [string]$record.output_root
    Add-Original "runs/$($set.name).registration.json" $registration
    Add-Original "runs/$($set.name).started.json" "$runRoot/started.json"
    Add-Original "runs/$($set.name).run.json" "$runRoot/result.json"
    Add-Original "runs/$($set.name).stdout.log" "$runRoot/stdout.log"
    Add-Original "runs/$($set.name).stderr.log" "$runRoot/stderr.log"
}

foreach ($path in @(
    'claims.json',
    'execution-plan.json',
    'registration-command-errata.json',
    'inventory-correction.json',
    'capture-inventory.json',
    'unmigrated-launch-inventory.json',
    'unmigrated-launch-inventory-v1.json',
    'unmigrated-launch-inventory-v2.json',
    'precision-mapping.json',
    'precision-mapping-v1.json',
    'precision-mapping-v2.json',
    'rne_backward-a5b528ff34301f8e358d13a53b5dfbd69676fcc02987d39a3148d9693b8ebd22.ptx'
)) {
    Add-Original "records/$path" "$task/$path"
}
foreach ($file in Get-ChildItem -LiteralPath $frozenRoot -Recurse -File | Sort-Object FullName) {
    $relative = [IO.Path]::GetRelativePath((Resolve-Path -LiteralPath $frozenRoot), $file.FullName).Replace('\', '/')
    Add-Original "frozen/$relative" $file.FullName
}
$captureRoot = "$runtime/run-01-product/gpu-capture"
foreach ($file in Get-ChildItem -LiteralPath $captureRoot -Recurse -File | Sort-Object FullName) {
    $relative = [IO.Path]::GetRelativePath((Resolve-Path -LiteralPath $captureRoot), $file.FullName).Replace('\', '/')
    Add-Original "actual-capture/$relative" $file.FullName
}

$members = @($mappings | Sort-Object path | ForEach-Object {
    $path = Join-Path $archiveRoot $_.path
    [ordered]@{
        path=$_.path
        bytes=(Get-Item -LiteralPath $path).Length
        sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$manifest = [ordered]@{
    schema=1
    root=$archiveRoot
    self_excluded=$true
    infrastructure_run_excluded='manifest builder cannot include its own post-exit result'
    member_count=$members.Count
    members=$members
}
$manifestJson = ($manifest | ConvertTo-Json -Depth 8).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) "$archiveRoot/manifest.json"), $manifestJson, [Text.UTF8Encoding]::new($false))
$map = [ordered]@{
    schema=1
    archive_root=$archiveRoot
    mappings=@($mappings | Sort-Object path)
}
$mapJson = ($map | ConvertTo-Json -Depth 8).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $mapPath), $mapJson, [Text.UTF8Encoding]::new($false))
[ordered]@{schema=1;archive_root=$archiveRoot;members=$members.Count;map=$mapPath} | ConvertTo-Json
