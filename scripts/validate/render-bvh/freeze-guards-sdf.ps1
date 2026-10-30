# SDF validation-window guard freezer (ASCII-only on purpose: PS 5.1 reads .ps1 as ANSI).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-sdf.ps1 -Attempt 1
# It rebuilds nothing by itself except running `cargo test --no-run` to resolve the 9 entity
# executables, then freezes: 9 exe + 83 read-only inputs = 92 entries.
# Inputs: scene-sdf 10 (2 XML + 8 capture products) + scene-hfield 10 (2 XML + 8 capture)
#         + legacy 47 (38 base + 4 skybox + 5 perworld) + scene-shadow capture 16.
# Run this only BEFORE the validation-window freeze; never inside the window body.
# The R05 guards.json, R02 guards-r02-skybox.json and R08 guards-r08-rgb-hetero.json are history
# and must not be touched or reused (U1/U1b changed product code, so every exe hash differs).
param(
    [string]$RepoRoot = 'D:\Rust\mjwarp-rs',
    [string]$OutFile = 'scripts\validate\render-bvh\guards-sdf-validate.json',
    [string]$WorkDir = 'target\render-bvh\sdf-validate-window',
    [int]$Attempt = 1
)
$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)
$work = Join-Path $RepoRoot $WorkDir
New-Item -ItemType Directory -Force $work | Out-Null
$artifactsPath = Join-Path $work 'cargo-artifacts.jsonl'
$suite = 'render-sdf-validate'
$taskId = '2026-10-31-render-sdf-validate'

# Entity order: the SDF target first, then the 8 regressions (Chief final selection 2026-10-31).
$entities = @(
    @{ entity = 'render_sdf_reference'; source = 'new' },
    @{ entity = 'render_e2e'; source = 'regression' },
    @{ entity = 'render_kernel_rgb'; source = 'regression' },
    @{ entity = 'render_kernel_depth_seg'; source = 'regression' },
    @{ entity = 'render_kernel_flags'; source = 'regression' },
    @{ entity = 'render_r10_multi_world'; source = 'regression' },
    @{ entity = 'render_skybox_reference'; source = 'regression' },
    @{ entity = 'render_context_lifecycle'; source = 'regression' },
    @{ entity = 'render_mesh_bvh'; source = 'regression' }
)

Write-Output 'freeze-guards-sdf: resolving entity executables via cargo test --no-run --message-format=json'
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
# scene-sdf: 2 XML + 8 capture products (the references U4 compares against).
$sdfDir = Join-Path $RepoRoot 'fixtures/render-bvh/scene-sdf'
foreach ($file in (Get-ChildItem $sdfDir -Recurse -File | Sort-Object { $_.FullName })) {
    $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1)
}
# scene-hfield: 2 XML + 8 capture products (existing evidence, protected read-only).
$hfieldDir = Join-Path $RepoRoot 'fixtures/render-bvh/scene-hfield'
foreach ($file in (Get-ChildItem $hfieldDir -Recurse -File | Sort-Object { $_.FullName })) {
    $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1)
}
# legacy: fixtures/render-bvh excluding scene-shadow, scene-hfield and scene-sdf
# (includes the 4 R02 skybox references and the 5 perworld capture products).
$legacy = @(Get-ChildItem (Join-Path $RepoRoot 'fixtures/render-bvh') -Recurse -File | Where-Object {
    $rel = $_.FullName.Substring($RepoRoot.Length + 1).Replace('\', '/')
    -not ($rel.StartsWith('fixtures/render-bvh/scene-shadow/') -or
          $rel.StartsWith('fixtures/render-bvh/scene-hfield/') -or
          $rel.StartsWith('fixtures/render-bvh/scene-sdf/'))
} | Sort-Object { $_.FullName })
foreach ($file in $legacy) { $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1) }
# scene-shadow capture artifacts (read-only regression inputs; same subset as R08).
$shadow = @(Get-ChildItem (Join-Path $RepoRoot 'fixtures/render-bvh/scene-shadow') -Recurse -File | Where-Object {
    $rel = $_.FullName.Substring($RepoRoot.Length + 1).Replace('\', '/')
    $rel -match '^fixtures/render-bvh/scene-shadow/capture(-exact)?/'
} | Sort-Object { $_.FullName })
foreach ($file in $shadow) { $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1) }

$total = $frozenExes.Count + $inputs.Count
# 9 exes + 83 inputs = 92: 10 scene-sdf + 10 scene-hfield + 47 legacy + 16 scene-shadow.
if ($frozenExes.Count -ne 9) { throw "expected 9 frozen exes, got $($frozenExes.Count)" }
if ($inputs.Count -ne 83) { throw "expected 83 read-only inputs, got $($inputs.Count)" }
if ($total -ne 92) { throw "expected 92 frozen entries, got $total" }
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
    aggregate_algorithm = 'sha256 over the sha256 hex strings of all 92 entries in repo-relative path-sorted order, concatenated with no separator'
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
