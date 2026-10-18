$ErrorActionPreference = 'Stop'
$task = 'docs/development-evidence/layers-m3-smooth-force-validate'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-smooth-force-validate'
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
    @{name='host-first';root="$runtime/check-01-host"},
    @{name='host-retry';root="$runtime/check-01-host-retry-01"},
    @{name='host-final';root="$runtime/check-01-host-retry-02"},
    @{name='device';root="$runtime/check-02-device"},
    @{name='gpu-off-first';root="$runtime/run-01-sleep-off"},
    @{name='gpu-off-final';root="$runtime/run-01-sleep-off-retry-01"},
    @{name='gpu-on-first';root="$runtime/run-02-sleep-on"},
    @{name='gpu-on-final';root="$runtime/run-02-sleep-on-retry-01"},
    @{name='fmt-first';root="$runtime/check-03-format"},
    @{name='fmt-final';root="$runtime/check-03-format-final"},
    @{name='clippy-first';root="$runtime/check-04-clippy"},
    @{name='clippy-final';root="$runtime/check-04-clippy-retry-01"},
    @{name='inventory';root="$runtime/check-05-inventory"},
    @{name='evidence';root="$runtime/check-06-evidence"}
)
foreach ($set in $runSets) {
    Add-Original "$($set.name).run.json" "$($set.root)/result.json"
    Add-Original "$($set.name).stdout.log" "$($set.root)/stdout.log"
    Add-Original "$($set.name).stderr.log" "$($set.root)/stderr.log"
}
foreach ($report in @(
    @{name='gpu-off-first.report.json';source="$runtime/run-01-sleep-off/test-report.json"},
    @{name='gpu-off-final.report.json';source="$runtime/run-01-sleep-off-retry-01/test-report.json"},
    @{name='gpu-on-first.report.json';source="$runtime/run-02-sleep-on/test-report.json"},
    @{name='gpu-on-final.report.json';source="$runtime/run-02-sleep-on-retry-01/test-report.json"}
)) {
    Add-Original $report.name $report.source
}
foreach ($captureSet in @(
    @{name='gpu-off-first-capture';root="$runtime/run-01-sleep-off/gpu-capture"},
    @{name='gpu-off-capture';root="$runtime/run-01-sleep-off-retry-01/gpu-capture"},
    @{name='gpu-on-first-capture';root="$runtime/run-02-sleep-on/gpu-capture"},
    @{name='gpu-on-capture';root="$runtime/run-02-sleep-on-retry-01/gpu-capture"}
)) {
    foreach ($file in @(Get-ChildItem -LiteralPath $captureSet.root -File | Sort-Object Name)) {
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
