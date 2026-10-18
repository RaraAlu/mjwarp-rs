$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-rne-bias'
$task = "docs/development-evidence/layers-m3-rne-bias"
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

$runSets = @(
    @{id='host-wrapper';name='host-first'},
    @{id='host-wrapper-retry-01';name='host-final'},
    @{id='format';name='format-first'},
    @{id='format-apply';name='format-apply'},
    @{id='format-retry-01';name='format-retry'},
    @{id='format-after-gpu-code';name='format-after-gpu'},
    @{id='format-final';name='format-final'},
    @{id='default-lib';name='default-lib'},
    @{id='clippy-all-targets';name='clippy-first'},
    @{id='clippy-all-targets-retry-01';name='clippy-final'},
    @{id='input-tolerance-gate';name='input-tolerance-gate'},
    @{id='device-preflight';name='device'},
    @{id='gpu-product-captured';name='gpu-product'},
    @{id='inventory';name='inventory'},
    @{id='precision-map';name='precision-first'},
    @{id='precision-map-retry-01';name='precision-intermediate'},
    @{id='precision-map-retry-02';name='precision-final'},
    @{id='evidence';name='evidence-first'},
    @{id='evidence-retry-01';name='evidence-final'},
    @{id='delivery-preflight';name='delivery-preflight-first'},
    @{id='delivery-preflight-retry-01';name='delivery-preflight-final'}
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
    'precision-revalidation.json',
    'precision-revalidation-v2.json',
    'unmigrated-launch-inventory.json'
)) {
    Add-Original "records/$path" "$task/$path"
}
foreach ($file in Get-ChildItem -LiteralPath "$task/frozen" -Recurse -File | Sort-Object FullName) {
    $relative = $file.FullName.Substring((Resolve-Path -LiteralPath "$task/frozen").Path.Length + 1).Replace('\', '/')
    Add-Original "frozen/$relative" $file.FullName
}
$captureRoot = "$runtime/run-01-product/gpu-capture"
foreach ($file in Get-ChildItem -LiteralPath $captureRoot -Recurse -File | Sort-Object FullName) {
    $relative = $file.FullName.Substring((Resolve-Path -LiteralPath $captureRoot).Path.Length + 1).Replace('\', '/')
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
