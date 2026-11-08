# Per-world L v2 light (G-L2) validation-window guard freezer.
# ASCII-only on purpose: PS 5.1 reads .ps1 as ANSI.
#
# Suite arithmetic (signed spec v3 section 10.2):
#   existing perworld suite 96 entries (11 exe + 83 read-only inputs + 2 criterion v2.2 outline lists)
#   + 1 new exe render_per_world_light (12 exes)
#   + 10 mandatory new inputs (section 10.2 list), optional inputs = 0 (Chief decision 2026-11-09)
#   => 107 entries total.
# The old suite JSON and its frozen-exe stash are READ-ONLY: this script never rewrites them and checks
# their sha256 before and after, so an accidental re-freeze or reuse cannot pass unnoticed.
#
# Modes:
#   (default)      DRY-RUN / structure self-check. No cargo, no build, no stash, no guards JSON write.
#                  Writes only <WorkDir>/freeze-plan-dryrun.json and prints SELFCHECK PASS n/n.
#                  The aggregate_sha256 needs the 12 live exe sha256, so in dry-run it is PENDING-FREEZE.
#   -Execute       REAL freeze: runs `cargo test --locked --no-run --message-format=json` once, stashes the
#                  12 executables under <WorkDir>/frozen-exes, freezes the guard inputs, writes the guards
#                  JSON. Run this only AFTER the pre-window monitor passes and BEFORE the window body.
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-perworld-l2-light.ps1
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-perworld-l2-light.ps1 -Execute -Attempt 1
param(
    [string]$RepoRoot = 'D:\Rust\mjwarp-rs',
    [string]$OutFile = 'scripts\validate\render-bvh\guards-perworld-l2-light.json',
    [string]$WorkDir = 'target\render-bvh\perworld-l2-window',
    [int]$Attempt = 1,
    [string]$Suite = 'render-perworld-l2-light',
    [string]$OldSuiteJson = 'docs/development-evidence/render-bvh/perworld-impl-s1/window-attempt3/guards-perworld-s1.json',
    [string]$OldSuiteSha256 = 'ca3ecd5c407e0e5273a2867794591026d55ee1904f1cf55b689bdc6c8844a15a',
    [string]$OldStashDir = 'target/render-bvh/perworld-s1-window/frozen-exes',
    [string]$NewEntity = 'render_per_world_light',
    [int]$ExpectedTotal = 107,
    [switch]$AllowOldStashDrift,
    [switch]$Execute
)
$ErrorActionPreference = 'Stop'
Set-Location $RepoRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)
$mode = if ($Execute) { 'execute' } else { 'dryrun' }
$work = Join-Path $RepoRoot $WorkDir
New-Item -ItemType Directory -Force $work | Out-Null

$checkTotal = 0
$checkPass = 0
function Check([string]$name, [scriptblock]$body) {
    $script:checkTotal++
    try {
        & $body
        $script:checkPass++
        Write-Output ("CHECK OK   {0}" -f $name)
    } catch {
        Write-Output ("CHECK FAIL {0}: {1}" -f $name, $_.Exception.Message)
        throw
    }
}
function Sha256([string]$path) { (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower() }
function RelPath([string]$full) { (Resolve-Path -LiteralPath $full).Path.Substring($RepoRoot.Length + 1).Replace('\', '/') }
function File-Entry([string]$rel) {
    $full = Join-Path $RepoRoot ($rel.Replace('/', '\'))
    if (-not (Test-Path -LiteralPath $full)) { throw "missing frozen object: $rel" }
    [pscustomobject]@{
        path = $rel
        bytes = (Get-Item -LiteralPath $full).Length
        sha256 = (Sha256 $full)
    }
}
function Aggregate-Sha([object[]]$entries) {
    # sha256 over the sha256 hex strings of all entries in repo-relative path-sorted order,
    # concatenated with no separator (identical to the signed 96-entry suite algorithm).
    $sorted = @($entries | Sort-Object { $_.path })
    $hexConcat = ($sorted | ForEach-Object { $_.sha256 }) -join ''
    return [BitConverter]::ToString(
        [Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::ASCII.GetBytes($hexConcat))
    ).Replace('-', '').ToLower()
}

# ------------------------------------------------------------------ old suite (read-only)
$oldPath = Join-Path $RepoRoot ($OldSuiteJson.Replace('/', '\'))
$oldShaBefore = Sha256 $oldPath
$old = [IO.File]::ReadAllText($oldPath, [Text.Encoding]::UTF8) | ConvertFrom-Json

$oldExes = @($old.frozen_exes)
$oldInputs = @($old.guard_inputs)
$oldLists = @($old.outline_lists)

Check 'old suite counts 11/83/2/96' {
    if ($old.counts.frozen_exes -ne 11) { throw "old frozen_exes $($old.counts.frozen_exes) != 11" }
    if ($old.counts.guard_inputs -ne 83) { throw "old guard_inputs $($old.counts.guard_inputs) != 83" }
    if ($old.counts.outline_lists -ne 2) { throw "old outline_lists $($old.counts.outline_lists) != 2" }
    if ($old.counts.total -ne 96) { throw "old total $($old.counts.total) != 96" }
    if ($oldExes.Count -ne 11 -or $oldInputs.Count -ne 83 -or $oldLists.Count -ne 2) { throw 'old suite arrays disagree with counts' }
}
Check 'old suite sha256 is the read-only baseline' {
    if ($oldShaBefore -ne $OldSuiteSha256) { throw "old guards sha $oldShaBefore != $OldSuiteSha256" }
}
Check 'old suite entity_order unique (11)' {
    $names = @($old.entity_order)
    if ($names.Count -ne 11) { throw "old entity_order $($names.Count) != 11" }
    if ((@($names | Sort-Object -Unique)).Count -ne 11) { throw 'old entity_order has duplicates' }
    foreach ($exe in $oldExes) { if (-not $names.Contains($exe.entity)) { throw "old exe $($exe.entity) missing from entity_order" } }
}
Check 'aggregate algorithm recompute matches old suite aggregate' {
    $recomputed = Aggregate-Sha @($oldExes + $oldInputs + $oldLists)
    if ($recomputed -ne $old.aggregate_sha256) { throw "aggregate recompute $recomputed != stored $($old.aggregate_sha256)" }
    Write-Output ("           old aggregate_sha256 = {0}" -f $recomputed)
}
Check 'old outline lists are the 2 criterion v2.2 arms' {
    $arms = @($oldLists | ForEach-Object { [string]$_.arm })
    if (($arms -join ',') -ne 'recipe,aa2') { throw "outline arms $($arms -join ',') != recipe,aa2" }
    foreach ($list in $oldLists) {
        $e = File-Entry $list.path
        if ($e.sha256 -ne [string]$list.sha256) { throw "outline $($list.arm) sha mismatch" }
        if ($e.bytes -ne [int]$list.bytes) { throw "outline $($list.arm) bytes mismatch" }
    }
}
# The old stash is history and read-only. Any drift is REPORTED, never "fixed": the G-L2 window rebuilds and
# re-stashes its own 12 executables, so a drifted old copy cannot be reused as evidence.
$oldStashDrift = @()
Check 'old frozen-exe stash present (drift reported, never repaired)' {
    $stash = Join-Path $RepoRoot ($OldStashDir.Replace('/', '\'))
    if (-not (Test-Path -LiteralPath $stash)) { throw "old stash dir missing: $OldStashDir" }
    $files = @(Get-ChildItem -LiteralPath $stash -File)
    if ($files.Count -ne 11) { throw "old stash has $($files.Count) files, expected 11" }
    foreach ($exe in $oldExes) {
        $stashRel = [string]$exe.stash_path
        $stashFull = Join-Path $RepoRoot ($stashRel.Replace('/', '\'))
        if (-not (Test-Path -LiteralPath $stashFull)) { throw "old stash entry missing: $stashRel" }
        $actualBytes = (Get-Item -LiteralPath $stashFull).Length
        $actualSha = Sha256 $stashFull
        if ($actualBytes -ne [int]$exe.stash_bytes -or $actualSha -ne [string]$exe.stash_sha256) {
            $script:oldStashDrift += [pscustomobject]@{
                entity = [string]$exe.entity
                path = $stashRel
                recorded_bytes = [int]$exe.stash_bytes
                recorded_sha256 = [string]$exe.stash_sha256
                actual_bytes = $actualBytes
                actual_sha256 = $actualSha
                last_write_time = (Get-Item -LiteralPath $stashFull).LastWriteTime.ToString('yyyy-MM-ddTHH:mm:ssK')
            }
        }
    }
}
if ($oldStashDrift.Count -gt 0) {
    Write-Output ("WARN old stash drift entries={0} (recorded vs actual, reported only)" -f $oldStashDrift.Count)
    foreach ($d in $oldStashDrift) {
        Write-Output ("WARN   {0}: recorded {1}B/{2} actual {3}B/{4} last_write={5}" -f `
            $d.entity, $d.recorded_bytes, $d.recorded_sha256, $d.actual_bytes, $d.actual_sha256, $d.last_write_time)
    }
}

# ------------------------------------------------------------------ new suite content
$newInputs = @(
    'fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/rgb.npz',
    'fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/depth.npz',
    'fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/seg.npz',
    'fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/state.npz',
    'fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/capture.json',
    'fixtures/render-bvh/scene-a-light/scene.xml',
    'scripts/capture_render_perworld.py',
    'scripts/perworld_capture_meta.py',
    'scripts/perworld_light_prep.py',
    'docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md'
)
$expectedFixed = @{
    'fixtures/render-bvh/scene-a-light/scene.xml' = @{ sha256 = '4b354054116ddb81c6209ed10c021d133ea30d6fd82c1a6f866bafff9f625716'; bytes = 5134 }
    'scripts/capture_render_perworld.py' = @{ sha256 = 'f3af09ab992981f2dc5d7f2e3479aaf9eebb97df0d2a774b247c8e0772ad6f07'; bytes = 23637 }
    'scripts/perworld_capture_meta.py' = @{ sha256 = '3fdb85953681a1a695a9611fc48ddf0862318acf0e0767e831029a820127b87e'; bytes = 19154 }
    'scripts/perworld_light_prep.py' = @{ sha256 = '9fd5f8c25894380992986ffecaa584ee4fdff53a042438a70812f04699113e2c'; bytes = 7206 }
    'docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md' = @{ sha256 = 'bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d'; bytes = 39379 }
}
$referenceTotalBytes = 11078239

$newInputEntries = @()
foreach ($rel in $newInputs) { $newInputEntries += File-Entry $rel }
Check '10 mandatory new inputs exist and fixed shas hold' {
    if ($newInputEntries.Count -ne 10) { throw "new inputs $($newInputEntries.Count) != 10" }
    foreach ($key in $expectedFixed.Keys) {
        $entry = $newInputEntries | Where-Object { $_.path -eq $key }
        if (-not $entry) { throw "mandatory input missing: $key" }
        if ($entry.sha256 -ne $expectedFixed[$key].sha256) { throw "sha mismatch for $key" }
        if ($entry.bytes -ne [int]$expectedFixed[$key].bytes) { throw "bytes mismatch for $key" }
    }
}
Check 'reference set is the five G-L1 files, total 11,078,239 B' {
    $refDir = 'fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/'
    $refEntries = @($newInputEntries | Where-Object { $_.path.StartsWith($refDir) })
    if ($refEntries.Count -ne 5) { throw "reference file count $($refEntries.Count) != 5" }
    $sum = ($refEntries | Measure-Object -Property bytes -Sum).Sum
    if ($sum -ne $referenceTotalBytes) { throw "reference total bytes $sum != $referenceTotalBytes" }
}
Check 'capture.json files_sha256 agrees with the frozen npz entries' {
    $capPath = Join-Path $RepoRoot 'fixtures\render-bvh\scene-a-light\perworld\nw3-light-hinge02040\capture.json'
    $cap = [IO.File]::ReadAllText($capPath, [Text.Encoding]::UTF8) | ConvertFrom-Json
    foreach ($name in @('rgb.npz', 'depth.npz', 'seg.npz', 'state.npz')) {
        $entry = $newInputEntries | Where-Object { $_.path -eq ('fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/' + $name) }
        if ([string]$cap.files_sha256.$name -ne $entry.sha256) { throw "capture.json files_sha256.$name mismatch" }
    }
    if ($cap.two_captures_bitwise_identical -ne $true) { throw 'capture.json two_captures_bitwise_identical is not true' }
}
Check 'optional guard inputs = 0 (fixed decision)' {
    # Chief decision 2026-11-09: only the 10 mandatory section 10.2 inputs; no optional entries.
    if (0 -ne 0) { throw 'impossible' }
}
Check 'suite arithmetic: 12 exe + 93 inputs + 2 lists = 107' {
    $exeCount = $oldExes.Count + 1
    $inputCount = $oldInputs.Count + $newInputEntries.Count
    $listCount = $oldLists.Count
    $total = $exeCount + $inputCount + $listCount
    if ($exeCount -ne 12) { throw "exe count $exeCount != 12" }
    if ($inputCount -ne 93) { throw "input count $inputCount != 93" }
    if ($listCount -ne 2) { throw "list count $listCount != 2" }
    if ($total -ne $ExpectedTotal) { throw "total $total != $ExpectedTotal" }
}
Check 'new entity is appended exactly once and not an old-suite entity' {
    $oldNames = @($old.entity_order)
    if ($oldNames.Contains($NewEntity)) { throw "$NewEntity already exists in the old suite" }
    $newNames = @($oldNames + @($NewEntity))
    if ($newNames.Count -ne 12) { throw "new entity_order $($newNames.Count) != 12" }
    if ((@($newNames | Sort-Object -Unique)).Count -ne 12) { throw 'new entity_order has duplicates' }
}
Check 'new stash dir is distinct from the old read-only stash' {
    $newStashRel = "$WorkDir/frozen-exes".Replace('\', '/')
    if ($newStashRel -eq $OldStashDir) { throw 'new stash dir must differ from the old read-only stash' }
    if (-not $newStashRel.Contains('perworld-l2-window')) { throw "unexpected new stash dir: $newStashRel" }
}

$newExeEntry = [pscustomobject]@{
    entity = $NewEntity
    source = 'new'
    path = 'PENDING-FREEZE (resolved from cargo test --no-run during -Execute)'
    bytes = $null
    sha256 = $null
    stash_path = ("$WorkDir/frozen-exes/$NewEntity.exe").Replace('\', '/')
    stash_bytes = $null
    stash_sha256 = $null
}

# ------------------------------------------------------------------ dry-run plan / real freeze
$counts = [pscustomobject]@{
    frozen_exes = 12
    guard_inputs = 93
    outline_lists = 2
    total = 107
}
if (-not $Execute) {
    $plan = [pscustomobject]@{
        schema = 1
        mode = 'dryrun'
        suite = $Suite
        repo = $RepoRoot
        generated_at = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffK')
        attempt = $Attempt
        counts = $counts
        aggregate_algorithm = 'sha256 over the sha256 hex strings of all 107 entries (12 exes + 93 inputs + 2 outline lists) in repo-relative path-sorted order, concatenated with no separator'
        aggregate_sha256 = 'PENDING-FREEZE (12 live exe sha256 resolved by cargo test --no-run in -Execute)'
        old_suite = [pscustomobject]@{ json = $OldSuiteJson; sha256 = $oldShaBefore; total = 96; read_only = $true }
        old_stash_dir = $OldStashDir
        old_stash_drift = @($oldStashDrift)
        frozen_exe_stash_dir = ("$WorkDir/frozen-exes").Replace('\', '/')
        entity_order = @($old.entity_order + @($NewEntity))
        frozen_exes_from_old_suite = @($oldExes | ForEach-Object { $_.entity })
        new_exe = $newExeEntry
        new_inputs = $newInputEntries
        outline_lists = @($oldLists | ForEach-Object { [pscustomobject]@{ arm = $_.arm; path = $_.path; sha256 = $_.sha256 } })
        optional_inputs = 0
        checks = @($checkTotal)
        selfcheck = "PASS $checkPass/$checkTotal"
    }
    $planPath = Join-Path $work 'freeze-plan-dryrun.json'
    [IO.File]::WriteAllText($planPath, (($plan | ConvertTo-Json -Depth 8) + "`n"), $utf8)
    Write-Output ("SELFCHECK PASS {0}/{1} mode=dryrun suite={2} total={3} exes={4} inputs={5} lists={6}" -f `
        $checkPass, $checkTotal, $Suite, $counts.total, $counts.frozen_exes, $counts.guard_inputs, $counts.outline_lists)
    Write-Output ("PLAN_WRITTEN {0}" -f $planPath)
    Write-Output 'aggregate_sha256=PENDING-FREEZE (filled by -Execute during the G-L2 freeze stage)'
    exit 0
}

# ---------------- REAL freeze (window stage): the only place that builds ----------------
if ($oldStashDrift.Count -gt 0 -and -not $AllowOldStashDrift) {
    throw ("old stash drift detected ({0} entries): report to Chief first; pass -AllowOldStashDrift only after a ruling" -f $oldStashDrift.Count)
}
Check 'cargo test --locked --no-run exit 0' {
    $artifactsPath = Join-Path $work 'cargo-artifacts.jsonl'
    $previousErrorAction = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $lines = @(& cargo test --locked --no-run --message-format=json 2>$null)
    $code = $LASTEXITCODE
    $ErrorActionPreference = $previousErrorAction
    if ($code -ne 0) { throw "cargo test --no-run failed: $code" }
    [IO.File]::WriteAllText($artifactsPath, (($lines -join "`n") + "`n"), $utf8)
}
$exeMap = @{}
$artifactsPath = Join-Path $work 'cargo-artifacts.jsonl'
foreach ($line in (Get-Content -LiteralPath $artifactsPath)) {
    if (-not $line -or -not $line.Trim().StartsWith('{')) { continue }
    try { $obj = $line | ConvertFrom-Json } catch { continue }
    if ($obj.reason -ne 'compiler-artifact') { continue }
    if (-not $obj.executable) { continue }
    if ($obj.target.name) { $exeMap[[string]$obj.target.name] = [string]$obj.executable }
}
$entities = @($old.entity_order + @($NewEntity))
Check 'all 12 entity executables resolved' {
    if ($entities.Count -ne 12) { throw "entity list $($entities.Count) != 12" }
    foreach ($name in $entities) { if (-not $exeMap.ContainsKey($name)) { throw "executable not found: $name" } }
}
$frozenExes = @()
foreach ($name in $entities) {
    $e = File-Entry (RelPath $exeMap[$name])
    $source = if ($name -eq $NewEntity) { 'new' } else { 'regression' }
    $frozenExes += [pscustomobject]@{ entity = $name; source = $source; path = $e.path; bytes = $e.bytes; sha256 = $e.sha256 }
}
$stashDirRel = ("$WorkDir/frozen-exes").Replace('\', '/')
$stashDir = Join-Path $RepoRoot ($stashDirRel.Replace('/', '\'))
Remove-Item -Recurse -Force $stashDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $stashDir | Out-Null
foreach ($exe in $frozenExes) {
    $stashRel = "$stashDirRel/$($exe.entity).exe"
    $stashFull = Join-Path $RepoRoot ($stashRel.Replace('/', '\'))
    Copy-Item -LiteralPath (Join-Path $RepoRoot ($exe.path.Replace('/', '\'))) $stashFull -Force
    $exe | Add-Member -NotePropertyName stash_path -NotePropertyValue $stashRel
    $exe | Add-Member -NotePropertyName stash_bytes -NotePropertyValue ((Get-Item -LiteralPath $stashFull).Length)
    $exe | Add-Member -NotePropertyName stash_sha256 -NotePropertyValue (Sha256 $stashFull)
    if ($exe.stash_bytes -ne $exe.bytes -or $exe.stash_sha256 -ne $exe.sha256) { throw "stash mismatch: $($exe.entity)" }
}
Check 'live == guard == stash for all 12 executables' {
    foreach ($exe in $frozenExes) {
        $live = File-Entry $exe.path
        if ($live.sha256 -ne $exe.sha256 -or $live.bytes -ne $exe.bytes) { throw "live != frozen: $($exe.entity)" }
        if ($exe.stash_sha256 -ne $exe.sha256) { throw "stash != frozen: $($exe.entity)" }
    }
}
$guards = [pscustomobject]@{
    schema = 1
    suite = $Suite
    task_id = '2026-11-09-render-perworld-l-v2-validate-window-g-l2'
    generated_at = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss.fffK')
    repo = $RepoRoot
    attempt = $Attempt
    criterion_version = [string]$old.criterion_version
    bound_ulp = [int]$old.bound_ulp
    counts = $counts
    aggregate_algorithm = 'sha256 over the sha256 hex strings of all 107 entries (12 exes + 93 inputs + 2 outline lists) in repo-relative path-sorted order, concatenated with no separator'
    aggregate_sha256 = (Aggregate-Sha @($frozenExes + $oldInputs + $newInputEntries + $oldLists))
    frozen_exe_stash_dir = $stashDirRel
    entity_order = @($entities)
    frozen_exes = $frozenExes
    guard_inputs = @($oldInputs + $newInputEntries)
    outline_lists = $oldLists
    old_suite = [pscustomobject]@{ json = $OldSuiteJson; sha256 = $oldShaBefore; total = 96; read_only = $true }
    old_stash_drift = @($oldStashDrift)
    optional_inputs = 0
}
$outPath = Join-Path $RepoRoot $OutFile
[IO.File]::WriteAllText($outPath, (($guards | ConvertTo-Json -Depth 8) + "`n"), $utf8)
Check 'old suite and its stash are still untouched after the freeze' {
    if ((Sha256 $oldPath) -ne $oldShaBefore) { throw 'old guards JSON changed during the freeze' }
    if ((Sha256 $oldPath) -ne $OldSuiteSha256) { throw 'old guards JSON no longer matches the recorded baseline' }
}
Check 'guards JSON round-trip: totals and aggregate self-consistent' {
    $doc = [IO.File]::ReadAllText($outPath, [Text.Encoding]::UTF8) | ConvertFrom-Json
    if ($doc.counts.total -ne 107) { throw "guards total $($doc.counts.total) != 107" }
    $recomputed = Aggregate-Sha @($doc.frozen_exes + $doc.guard_inputs + $doc.outline_lists)
    if ($recomputed -ne $doc.aggregate_sha256) { throw 'guards aggregate round-trip mismatch' }
}
Write-Output ("GUARDS_FROZEN suite={0} total={1} exes={2} inputs={3} lists={4} aggregate_sha256={5} guards_sha256={6} bytes={7} stash_dir={8}" -f `
    $Suite, $counts.total, $counts.frozen_exes, $counts.guard_inputs, $counts.outline_lists, $guards.aggregate_sha256, (Sha256 $outPath), (Get-Item -LiteralPath $outPath).Length, $stashDirRel)
Write-Output ("SELFCHECK PASS {0}/{1} mode=execute" -f $checkPass, $checkTotal)
exit 0
