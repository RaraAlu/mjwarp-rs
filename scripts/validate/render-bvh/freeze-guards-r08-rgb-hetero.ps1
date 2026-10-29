# R08-RGB validation-window guard freezer (ASCII-only on purpose: PS 5.1 reads .ps1 as ANSI).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r08-rgb-hetero.ps1 -Attempt 1
# It rebuilds nothing by itself except running `cargo test --no-run` to resolve the 9 entity
# executables, then freezes: 9 exe + 68 read-only inputs = 77 entries (same input set as the
# R02 skybox window: 8 hfield capture + 2 hfield xml + 42 legacy (38 + 4 skybox references) + 16 shadow).
# Run this only BEFORE the validation window freeze; never inside the window.
# The R05 guards.json and R02 guards-r02-skybox.json are history and must not be touched.
param(
    [string]$RepoRoot = 'D:\Rust\mjwarp-rs',
    [string]$OutFile = 'scripts\validate\render-bvh\guards-r08-rgb-hetero.json',
    [string]$WorkDir = 'target\render-bvh\r08-rgb-hetero-window',
    [int]$Attempt = 1
)
$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)
$work = Join-Path $RepoRoot $WorkDir
New-Item -ItemType Directory -Force $work | Out-Null
$artifactsPath = Join-Path $work 'cargo-artifacts.jsonl'
$suite = 'render-r08-rgb-hetero'
$taskId = '2026-10-29-render-r08-rgb-hetero-validate'

# Entity order: the R08 camera target first (includes per-camera RGB/depth/seg mask cases and
# the cam1 all-channels-off case), then the 8 existing regressions (entity-table order).
$entities = @(
    @{ entity = 'render_r08_cameras'; source = 'new' },
    @{ entity = 'render_skybox_reference'; source = 'regression' },
    @{ entity = 'render_camera_rays'; source = 'regression' },
    @{ entity = 'render_context_lifecycle'; source = 'regression' },
    @{ entity = 'render_e2e'; source = 'regression' },
    @{ entity = 'render_kernel_depth_seg'; source = 'regression' },
    @{ entity = 'render_kernel_flags'; source = 'regression' },
    @{ entity = 'render_kernel_rgb'; source = 'regression' },
    @{ entity = 'render_r10_multi_world'; source = 'regression' }
)

Write-Output 'freeze-guards-r08-rgb-hetero: resolving entity executables via cargo test --no-run --message-format=json'
$previousErrorAction = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$lines = @(& cargo test --locked --no-run --message-format=json 2>$null)
$cargoCode = $LASTEXITCODE
$ErrorActionPreference = $previousErrorAction
if ($cargoCode -ne 0) { throw "cargo test --no-run failed: $cargoCode" }
[IO.File]::WriteAllText($artifactsPath, (($lines -join "`n") + "`n"), $utf8)
$exeMap = @{}
foreach ($line in $lines) {
    if (-not $line -or -not $line.Trim().StartsWith('{')) { continue }
    $obj = $null
    try { $obj = $line | ConvertFrom-Json } catch { continue }
    if ($obj.reason -ne 'compiler-artifact') { continue }
    if (-not $obj.executable) { continue }
    $name = $obj.target.name
    if ($name) { $exeMap[$name] = $obj.executable }
}

function File-Entry([string]$path) {
    $full = if ([IO.Path]::IsPathRooted($path)) { $path } else { Join-Path $RepoRoot $path }
    if (-not (Test-Path $full)) { throw "missing frozen object: $path" }
    $rel = (Resolve-Path $full).Path.Substring($RepoRoot.Length + 1).Replace('\', '/')
    [pscustomobject]@{
        path = $rel
        bytes = (Get-Item $full).Length
        sha256 = (Get-FileHash $full -Algorithm SHA256).Hash.ToLower()
    }
}

$frozenExes = @()
foreach ($item in $entities) {
    $name = $item.entity
    if (-not $exeMap.ContainsKey($name)) { throw "entity executable not found for: $name" }
    $entry = File-Entry $exeMap[$name]
    $frozenExes += [pscustomobject]@{
        entity = $name
        source = $item.source
        path = $entry.path
        bytes = $entry.bytes
        sha256 = $entry.sha256
    }
}

# Stash copies of the frozen executables: a later rebuild in target/debug/deps must not
# destroy the signed bytes. The stash is part of the freeze record (path/bytes/sha256).
$stashDirRel = "$WorkDir/frozen-exes".Replace('\', '/')
$stashDir = Join-Path $RepoRoot $stashDirRel
Remove-Item -Recurse -Force $stashDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $stashDir | Out-Null
foreach ($exe in $frozenExes) {
    $stashRel = "$stashDirRel/$($exe.entity).exe"
    $stashFull = Join-Path $RepoRoot $stashRel
    Copy-Item (Join-Path $RepoRoot $exe.path) $stashFull -Force
    $stashBytes = (Get-Item $stashFull).Length
    $stashSha = (Get-FileHash $stashFull -Algorithm SHA256).Hash.ToLower()
    if (($stashBytes -ne $exe.bytes) -or ($stashSha -ne $exe.sha256)) {
        throw "stash copy mismatch for $($exe.entity)"
    }
    $exe | Add-Member -NotePropertyName stash_path -NotePropertyValue $stashRel -Force
    $exe | Add-Member -NotePropertyName stash_bytes -NotePropertyValue $stashBytes -Force
    $exe | Add-Member -NotePropertyName stash_sha256 -NotePropertyValue $stashSha -Force
}

$inputs = @()
# 8 hfield capture products (R05 window history; only read as regression inputs here).
$captureDir = Join-Path $RepoRoot 'fixtures/render-bvh/scene-hfield/capture-hfield'
foreach ($file in (Get-ChildItem $captureDir -Recurse -File | Sort-Object { $_.FullName })) {
    $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1)
}
# 2 hfield scene XML.
$inputs += File-Entry 'fixtures/render-bvh/scene-hfield/scene.xml'
$inputs += File-Entry 'fixtures/render-bvh/scene-hfield/scene-mixed.xml'
# legacy reference files: fixtures/render-bvh excluding scene-shadow and scene-hfield
# (includes the 4 R02-S1 skybox capture files under scene-a/reference-skybox/).
$legacy = @(Get-ChildItem (Join-Path $RepoRoot 'fixtures/render-bvh') -Recurse -File | Where-Object {
    $rel = $_.FullName.Substring($RepoRoot.Length + 1).Replace('\', '/')
    -not ($rel.StartsWith('fixtures/render-bvh/scene-shadow/') -or $rel.StartsWith('fixtures/render-bvh/scene-hfield/'))
} | Sort-Object { $_.FullName })
foreach ($file in $legacy) { $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1) }
# scene-shadow capture artifacts (read-only regression inputs).
$shadow = @(Get-ChildItem (Join-Path $RepoRoot 'fixtures/render-bvh/scene-shadow') -Recurse -File | Where-Object {
    $rel = $_.FullName.Substring($RepoRoot.Length + 1).Replace('\', '/')
    $rel -match '^fixtures/render-bvh/scene-shadow/capture(-exact)?/'
} | Sort-Object { $_.FullName })
foreach ($file in $shadow) { $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1) }

$skyboxRefs = @(Get-ChildItem (Join-Path $RepoRoot 'fixtures/render-bvh/scene-a/reference-skybox') -File -ErrorAction SilentlyContinue | Sort-Object { $_.FullName })
if ($skyboxRefs.Count -ne 4) { throw "expected 4 skybox reference files (rgb/depth/seg npz + capture.json), got $($skyboxRefs.Count)" }

$total = $frozenExes.Count + $inputs.Count
# 9 exes + 68 inputs = 77: 8 hfield capture + 2 hfield xml + 42 legacy (38 + 4 skybox) + 16 shadow.
if ($frozenExes.Count -ne 9) { throw "expected 9 frozen exes, got $($frozenExes.Count)" }
if ($inputs.Count -ne 68) { throw "expected 68 read-only inputs, got $($inputs.Count)" }
if ($total -ne 77) { throw "expected 77 frozen entries, got $total" }
$all = @()
foreach ($entry in $frozenExes) { $all += [pscustomobject]@{ path = $entry.path; sha256 = $entry.sha256 } }
foreach ($entry in $inputs) { $all += [pscustomobject]@{ path = $entry.path; sha256 = $entry.sha256 } }
$sorted = @($all | Sort-Object { $_.path })
$hexConcat = ($sorted | ForEach-Object { $_.sha256 }) -join ''
$aggregate = [BitConverter]::ToString(
    [Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::ASCII.GetBytes($hexConcat))
).Replace('-', '').ToLower()

$guards = [pscustomobject]@{
    schema = 1
    suite = $suite
    task_id = $taskId
    generated_at = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffK')
    repo = $RepoRoot
    attempt = $Attempt
    counts = [pscustomobject]@{
        frozen_exes = $frozenExes.Count
        guard_inputs = $inputs.Count
        total = $total
    }
    aggregate_algorithm = 'sha256 over the sha256 hex strings of all 77 entries in repo-relative path-sorted order, concatenated with no separator'
    aggregate_sha256 = $aggregate
    frozen_exe_stash_dir = $stashDirRel
    entity_order = @($entities | ForEach-Object { $_.entity })
    frozen_exes = $frozenExes
    guard_inputs = $inputs
}
$outPath = Join-Path $RepoRoot $OutFile
[IO.File]::WriteAllText($outPath, (($guards | ConvertTo-Json -Depth 6) + "`n"), $utf8)
$outSha = (Get-FileHash $outPath -Algorithm SHA256).Hash.ToLower()
Write-Output ("GUARDS_FROZEN suite={0} total={1} exes={2} inputs={3} aggregate_sha256={4} guards_sha256={5} bytes={6} stash_dir={7}" -f `
    $suite, $total, $frozenExes.Count, $inputs.Count, $aggregate, $outSha, (Get-Item $outPath).Length, $stashDirRel)
exit 0
