$ErrorActionPreference = 'Stop'
$task = 'docs/development-evidence/layers-m3-smooth-force-compute'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-smooth-force-compute'
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
    @{id='host-wrapper';name='host'},
    @{id='device-preflight';name='device'},
    @{id='gpu-sleep-off';name='gpu-off'},
    @{id='gpu-sleep-on';name='gpu-on'},
    @{id='format';name='fmt-first'},
    @{id='format-apply';name='fmt-apply'},
    @{id='format-retry-01';name='fmt-final'},
    @{id='clippy';name='clippy'},
    @{id='inventory';name='inventory'},
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
foreach ($report in @(
    @{name='gpu-off.report.json';source="$runtime/run-01-sleep-off/test-report.json"},
    @{name='gpu-on.report.json';source="$runtime/run-02-sleep-on/test-report.json"}
)) {
    Add-Original $report.name $report.source
}
foreach ($captureSet in @(
    @{name='gpu-off-capture';root="$runtime/run-01-sleep-off/gpu-capture"},
    @{name='gpu-on-capture';root="$runtime/run-02-sleep-on/gpu-capture"}
)) {
    $files = @(Get-ChildItem -LiteralPath $captureSet.root -File | Sort-Object Name)
    if ($files.Count -ne 6) { throw "$($captureSet.name)捕获文件数错误" }
    foreach ($file in $files) {
        Add-Original "$($captureSet.name)/$($file.Name)" $file.FullName
    }
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
[ordered]@{schema=1;archive_root=$archiveRoot;members=$members.Count;map=$mapPath} | ConvertTo-Json
