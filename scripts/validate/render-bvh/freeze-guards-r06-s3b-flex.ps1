# R06-S3b flex validation-window guard freezer (ASCII-only on purpose: PS 5.1 reads .ps1 as ANSI).
# Usage (real freeze; window card only, Chief GO required):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1 -Attempt 1
# Static structure self-check (authoring card; reads no binaries, runs no cargo):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1 -GuardsOnly
# The suite has 2 exe entries (product binary + render_flex_validate test binary) and 14 read-only
# input entries (6 scene-flex files + 6 reference files + capture script + this freeze script).
# Run this only BEFORE the validation window freeze; never inside the window.
# The R02/R08/SDF guard files are history and must not be touched.
param(
    [string]$RepoRoot = 'D:\Rust\mjwarp-rs',
    [string]$OutFile = 'scripts\validate\render-bvh\guards-r06-s3b-flex.json',
    [string]$WorkDir = 'target\render-bvh\r06-s3b-flex-window',
    [int]$Attempt = 1,
    [switch]$GuardsOnly
)
$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)
$suite = 'render-r06-s3b-flex'
$taskId = '2026-11-04-render-r06-s3b-flex-freeze-guards'

$exeSpecs = @(
    @{ entity = 'mjwarp-rs'; resolve = 'bin' },
    @{ entity = 'render_flex_validate'; resolve = 'test' }
)

$inputPaths = @(
    'fixtures/render-bvh/scene-flex/scene.xml',
    'fixtures/render-bvh/scene-flex/model-312.json',
    'fixtures/render-bvh/scene-flex/model-315.json',
    'fixtures/render-bvh/scene-flex/scene-312.mjb',
    'fixtures/render-bvh/scene-flex/scene-315.mjb',
    'fixtures/render-bvh/scene-flex/provenance.json',
    'fixtures/render-bvh/scene-flex/reference/base/rgb.npz',
    'fixtures/render-bvh/scene-flex/reference/base/depth.npz',
    'fixtures/render-bvh/scene-flex/reference/base/seg.npz',
    'fixtures/render-bvh/scene-flex/reference/base/state.npz',
    'fixtures/render-bvh/scene-flex/reference/base/capture.json',
    'fixtures/render-bvh/scene-flex/reference/base/cam-poses.json',
    'scripts/authoring/capture_render_flex.py',
    'scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1'
)
if ($inputPaths.Count -ne 14) { throw "expected 14 guard inputs, got $($inputPaths.Count)" }

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

function Guard-Aggregate($entries) {
    # Ordinal path sort so the aggregate does not depend on machine culture (matches Python sort).
    $paths = [string[]]@($entries | ForEach-Object { $_.path })
    [Array]::Sort($paths, [System.StringComparer]::Ordinal)
    $hexConcat = ''
    foreach ($path in $paths) {
        $match = @($entries | Where-Object { $_.path -eq $path })[0]
        $hexConcat += $match.sha256
    }
    [BitConverter]::ToString(
        [Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::ASCII.GetBytes($hexConcat))
    ).Replace('-', '').ToLower()
}

function Resolve-Exe([string]$entity, [string]$kind) {
    $prevEap = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    if ($kind -eq 'bin') {
        $raw = @(& cargo build --locked --bin $entity --message-format=json 2>$null)
    } else {
        $raw = @(& cargo test --locked --features cuda-probe --no-run --message-format=json 2>$null)
    }
    if ($LASTEXITCODE -ne 0) { throw "cargo resolve failed for ${entity} (exit $LASTEXITCODE)" }
    $ErrorActionPreference = $prevEap
    foreach ($line in $raw) {
        if (-not $line -or -not $line.Trim().StartsWith('{')) { continue }
        $obj = $null
        try { $obj = $line | ConvertFrom-Json } catch { continue }
        if ($obj.reason -ne 'compiler-artifact') { continue }
        if (-not $obj.executable) { continue }
        if ($obj.target.name -eq $entity) { return $obj.executable }
    }
    throw "entity executable not found: $entity"
}

if ($GuardsOnly) {
    $path = Join-Path $RepoRoot $OutFile
    if (-not (Test-Path $path)) { throw "guards file missing: $OutFile" }
    $guards = Get-Content $path -Raw | ConvertFrom-Json
    if ($guards.schema -ne 1) { throw "guards schema mismatch" }
    if ($guards.suite -ne $suite) { throw "guards suite mismatch" }
    if ($guards.guard_inputs.Count -ne 14) { throw "guards input count mismatch" }
    if ($guards.frozen_exes.Count -ne 2) { throw "guards exe count mismatch" }
    $bad = @()
    foreach ($entry in $guards.guard_inputs) {
        $full = Join-Path $RepoRoot $entry.path
        if (-not (Test-Path $full)) { $bad += "missing:$($entry.path)"; continue }
        if ((Get-Item $full).Length -ne $entry.bytes) { $bad += "bytes-mismatch:$($entry.path)" }
        if ((Get-FileHash $full -Algorithm SHA256).Hash.ToLower() -ne $entry.sha256) {
            $bad += "sha-mismatch:$($entry.path)"
        }
    }
    if ($bad.Count -gt 0) { throw ("guards structure check failed: " + ($bad -join ', ')) }
    foreach ($entry in $guards.frozen_exes) {
        if ($entry.pending_freeze -ne $true) { throw "exe entry not marked pending: $($entry.entity)" }
        if ($entry.sha256) { throw "exe entry already frozen: $($entry.entity)" }
    }
    $aggregate = Guard-Aggregate $guards.guard_inputs
    if ($aggregate -ne $guards.aggregate_inputs_sha256) { throw "guards input aggregate mismatch" }
    $classified = @($guards.guard_inputs | Group-Object { $_.kind } | ForEach-Object { "$($_.Name)=$($_.Count)" })
    Write-Output ("GUARDS_STRUCTURE_OK suite={0} exes_pending={1} inputs={2} inputs_sha256_ok=true aggregate_inputs_sha256={3} classification={4}" -f `
        $suite, $guards.frozen_exes.Count, $guards.guard_inputs.Count, $aggregate, ($classified -join ','))
    exit 0
}

$work = Join-Path $RepoRoot $WorkDir
New-Item -ItemType Directory -Force $work | Out-Null
$utf8 = New-Object System.Text.UTF8Encoding($false)
$frozenExes = @()
foreach ($spec in $exeSpecs) {
    $exePath = Resolve-Exe $spec.entity $spec.resolve
    $entry = File-Entry $exePath
    $frozenExes += [pscustomobject]@{
        entity = $spec.entity
        resolve = $spec.resolve
        path = $entry.path
        bytes = $entry.bytes
        sha256 = $entry.sha256
        pending_freeze = $false
    }
}
$stashDirRel = "$WorkDir/frozen-exes".Replace('\', '/')
$stashDir = Join-Path $RepoRoot $stashDirRel
Remove-Item -Recurse -Force $stashDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $stashDir | Out-Null
foreach ($exe in $frozenExes) {
    $stashRel = "$stashDirRel/$($exe.entity).exe"
    Copy-Item (Join-Path $RepoRoot $exe.path) (Join-Path $RepoRoot $stashRel) -Force
    $bytes = (Get-Item (Join-Path $RepoRoot $stashRel)).Length
    $sha = (Get-FileHash (Join-Path $RepoRoot $stashRel) -Algorithm SHA256).Hash.ToLower()
    if (($bytes -ne $exe.bytes) -or ($sha -ne $exe.sha256)) { throw "stash copy mismatch: $($exe.entity)" }
    $exe | Add-Member -NotePropertyName stash_path -NotePropertyValue $stashRel -Force
    $exe | Add-Member -NotePropertyName stash_bytes -NotePropertyValue $bytes -Force
    $exe | Add-Member -NotePropertyName stash_sha256 -NotePropertyValue $sha -Force
}
$inputs = @()
foreach ($path in $inputPaths) {
    if ($path -ne 'scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1') {
        $entry = File-Entry $path
        $entry | Add-Member -NotePropertyName kind -NotePropertyValue 'input' -Force
        $inputs += $entry
    }
}
$scriptEntry = File-Entry 'scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1'
$scriptEntry | Add-Member -NotePropertyName kind -NotePropertyValue 'guard_script' -Force
$inputs += $scriptEntry
$total = $frozenExes.Count + $inputs.Count
if ($total -ne 16) { throw "expected 16 frozen entries, got $total" }
$guards = [pscustomobject]@{
    schema = 1
    suite = $suite
    task_id = $taskId
    generated_at = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffK')
    repo = $RepoRoot
    attempt = $Attempt
    status = 'frozen'
    counts = [pscustomobject]@{
        frozen_exes = $frozenExes.Count
        guard_inputs = $inputs.Count
        total = $total
    }
    aggregate_algorithm = 'sha256 over the sha256 hex strings of all 16 entries in repo-relative path-sorted order, concatenated with no separator'
    aggregate_inputs_sha256 = (Guard-Aggregate $inputs)
    aggregate_sha256 = (Guard-Aggregate (@($frozenExes) + @($inputs)))
    frozen_exe_stash_dir = $stashDirRel
    entity_order = @($exeSpecs | ForEach-Object { $_.entity })
    frozen_exes = $frozenExes
    guard_inputs = $inputs
}
$outPath = Join-Path $RepoRoot $OutFile
[IO.File]::WriteAllText($outPath, (($guards | ConvertTo-Json -Depth 6) + "`n"), $utf8)
$outSha = (Get-FileHash $outPath -Algorithm SHA256).Hash.ToLower()
Write-Output ("GUARDS_OK={0}/{1} suite={2} exes={3} inputs={4} stash={5} aggregate_sha256={6} guards_sha256={7} bytes={8}" -f `
    $total, $total, $suite, $frozenExes.Count, $inputs.Count, $stashDirRel, `
    $guards.aggregate_sha256, $outSha, (Get-Item $outPath).Length)
exit 0
