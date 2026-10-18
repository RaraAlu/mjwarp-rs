$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-rigid-mass-solve'
$task = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$runtime = "target/normify-evidence/$taskId"
$archiveRoot = "$task/formal-run-v5"
$frozenRoot = "$task/frozen-v5"
$mapPath = "$task/source-map-v5.json"
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

Copy-New 'src/physics/mass_solve.rs' "$frozenRoot/current-mass-solve.rs"
Copy-New "$task/rigid_mass_solve-a45cdb75c4087a4e78ae6916b11fdb2c60d807330703f5eafd788da3d2c6e49c.ptx" "$frozenRoot/current-rigid-mass-solve.ptx"

$registrations = @(Get-ChildItem -LiteralPath "$runtime/registrations" -File | Sort-Object Name)
foreach ($file in $registrations) {
    $record = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json
    if ($record.id -ceq 'manifest-v5') { continue }
    $runRoot = [string]$record.output_root
    foreach ($item in @(
        @{name='registration.json';source=$file.FullName},
        @{name='started.json';source="$runRoot/started.json"},
        @{name='run.json';source="$runRoot/result.json"},
        @{name='stdout.log';source="$runRoot/stdout.log"},
        @{name='stderr.log';source="$runRoot/stderr.log"}
    )) {
        Add-Original "runs/$($record.id).$($item.name)" $item.source
    }
}

foreach ($file in Get-ChildItem -LiteralPath $task -File | Sort-Object Name) {
    Add-Original "records/$($file.Name)" $file.FullName
}
foreach ($path in @('fixtures/precision/t4-current-kernels.json','fixtures/precision/t4-kernels.json')) {
    Add-Original "records/frozen-registry/$([IO.Path]::GetFileName($path))" $path
}
foreach ($file in Get-ChildItem -LiteralPath $frozenRoot -Recurse -File | Sort-Object FullName) {
    $relative = [IO.Path]::GetRelativePath((Resolve-Path -LiteralPath $frozenRoot), $file.FullName).Replace('\', '/')
    Add-Original "frozen/$relative" $file.FullName
}
foreach ($runName in @('run-01-product','run-02-product')) {
    $captureRoot = "$runtime/$runName/gpu-capture"
    foreach ($file in Get-ChildItem -LiteralPath $captureRoot -Recurse -File | Sort-Object FullName) {
        $relative = [IO.Path]::GetRelativePath((Resolve-Path -LiteralPath $captureRoot), $file.FullName).Replace('\', '/')
        Add-Original "actual-capture/$runName/$relative" $file.FullName
    }
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
