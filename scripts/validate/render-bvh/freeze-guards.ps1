# R05-S3b hfield validation-window guard freezer (ASCII-only on purpose: PS 5.1 reads .ps1 as ANSI).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards.ps1
# It rebuilds nothing by itself except running `cargo test --no-run` to resolve the 9 entity
# executables, then freezes: 9 exe + 8 capture files + 2 signed scene XML + 38 legacy reference
# files + 16 scene-shadow capture artifacts = 73 entries.
# Run this only BEFORE the window freeze; never inside the window.
param(
    [string]$RepoRoot = 'D:\Rust\mjwarp-rs',
    [string]$OutFile = 'scripts\validate\render-bvh\guards.json',
    [string]$WorkDir = 'target\render-bvh\r05-s3b-window'
)
$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)
$work = Join-Path $RepoRoot $WorkDir
New-Item -ItemType Directory -Force $work | Out-Null
$artifactsPath = Join-Path $work 'cargo-artifacts.jsonl'

# Entity order: the new hfield target first, then the 8 R09-S3b regressions (entity-table order).
$entities = @(
    @{ entity = 'render_hfield_reference'; source = 'new' },
    @{ entity = 'render_camera_rays'; source = 'regression' },
    @{ entity = 'render_context_lifecycle'; source = 'regression' },
    @{ entity = 'render_e2e'; source = 'regression' },
    @{ entity = 'render_kernel_depth_seg'; source = 'regression' },
    @{ entity = 'render_kernel_flags'; source = 'regression' },
    @{ entity = 'render_kernel_rgb'; source = 'regression' },
    @{ entity = 'render_r08_cameras'; source = 'regression' },
    @{ entity = 'render_r10_multi_world'; source = 'regression' }
)

Write-Output 'freeze-guards: resolving entity executables via cargo test --no-run --message-format=json'
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

$inputs = @()
# 8 capture products (R05-S3a window; repo-relative paths).
$captureDir = Join-Path $RepoRoot 'fixtures/render-bvh/scene-hfield/capture-hfield'
foreach ($file in (Get-ChildItem $captureDir -Recurse -File | Sort-Object { $_.FullName })) {
    $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1)
}
# 2 signed scene XML.
$inputs += File-Entry 'fixtures/render-bvh/scene-hfield/scene.xml'
$inputs += File-Entry 'fixtures/render-bvh/scene-hfield/scene-mixed.xml'
# 38 legacy reference files: fixtures/render-bvh excluding scene-shadow and scene-hfield.
$legacy = @(Get-ChildItem (Join-Path $RepoRoot 'fixtures/render-bvh') -Recurse -File | Where-Object {
    $rel = $_.FullName.Substring($RepoRoot.Length + 1).Replace('\', '/')
    -not ($rel.StartsWith('fixtures/render-bvh/scene-shadow/') -or $rel.StartsWith('fixtures/render-bvh/scene-hfield/'))
} | Sort-Object { $_.FullName })
foreach ($file in $legacy) { $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1) }
# 16 scene-shadow capture artifacts (read-only in this window).
$shadow = @(Get-ChildItem (Join-Path $RepoRoot 'fixtures/render-bvh/scene-shadow') -Recurse -File | Where-Object {
    $rel = $_.FullName.Substring($RepoRoot.Length + 1).Replace('\', '/')
    $rel -match '^fixtures/render-bvh/scene-shadow/capture(-exact)?/'
} | Sort-Object { $_.FullName })
foreach ($file in $shadow) { $inputs += File-Entry $file.FullName.Substring($RepoRoot.Length + 1) }

$total = $frozenExes.Count + $inputs.Count
if ($total -ne 73) { throw "expected 73 frozen entries, got $total ($($frozenExes.Count) exes + $($inputs.Count) inputs)" }
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
    suite = 'r05-s3b-hfield-validate'
    task_id = '2026-10-28-render-bvh-r05-s3b-hfield-validate'
    generated_at = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffK')
    repo = $RepoRoot
    attempt = 1
    counts = [pscustomobject]@{
        frozen_exes = $frozenExes.Count
        guard_inputs = $inputs.Count
        total = $total
    }
    aggregate_algorithm = 'sha256 over the sha256 hex strings of all 73 entries in repo-relative path-sorted order, concatenated with no separator'
    aggregate_sha256 = $aggregate
    entity_order = @($entities | ForEach-Object { $_.entity })
    frozen_exes = $frozenExes
    guard_inputs = $inputs
}
$outPath = Join-Path $RepoRoot $OutFile
[IO.File]::WriteAllText($outPath, (($guards | ConvertTo-Json -Depth 6) + "`n"), $utf8)
$outSha = (Get-FileHash $outPath -Algorithm SHA256).Hash.ToLower()
Write-Output ("GUARDS_FROZEN total={0} exes={1} inputs={2} aggregate_sha256={3} guards_sha256={4} bytes={5}" -f `
    $total, $frozenExes.Count, $inputs.Count, $aggregate, $outSha, (Get-Item $outPath).Length)
exit 0
