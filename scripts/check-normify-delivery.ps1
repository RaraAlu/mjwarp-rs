#Requires -Version 7.0
[CmdletBinding()]
param(
    [ValidateSet('Staged', 'Audit')][string]$Mode = 'Staged',
    [string]$RepoRoot = (Split-Path -Parent $PSScriptRoot),
    [string]$NormifyDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$RepoRoot = [IO.Path]::GetFullPath($RepoRoot)
function Invoke-RepoGit([string[]]$Arguments) {
    $result = @(& git -C $RepoRoot -c core.quotepath=false @Arguments)
    if ($LASTEXITCODE -ne 0) { throw "Invoke-RepoGit failed: $($Arguments -join ' ')" }
    return $result
}
function JsonFile([string]$Path) { Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json -AsHashtable }
function EqualJson($Left, $Right) {
    (ConvertTo-Json -InputObject $Left -Depth 30 -Compress) -ceq (ConvertTo-Json -InputObject $Right -Depth 30 -Compress)
}
if (-not $NormifyDir) {
    $configured = & git -C $RepoRoot config --get normify.directory
    if ($LASTEXITCODE -eq 0) { $NormifyDir = $configured }
    else { $NormifyDir = Join-Path $HOME '.codex/normify/normify-mjwarp-rs' }
}
function Task([string]$Id) {
    if ($Id -notmatch '^[a-z0-9][a-z0-9-]+$') { throw 'Invalid task id' }
    $task = JsonFile (Join-Path $NormifyDir "changes/$Id.json")
    if ($task.id -cne $Id) { throw 'Task identity mismatch' }
    return $task
}
function SafePath([string]$Path) {
    if (-not $Path -or $Path -match '(^/|\\|(^|/)\.\.?(/|$)|[\r\n\t:]|^\.git(/|$))') {
        throw "Unsafe receipt path: $Path"
    }
}
function Receipt([string]$Spec) {
    $r = (Invoke-RepoGit @('show', $Spec)) -join "`n" | ConvertFrom-Json -AsHashtable
    if ($r.schema -ne 1 -or $r.kind -notin @('delivery', 'governance')) { throw 'Invalid delivery receipt' }
    if (@($r.validation).Count -eq 0 -or -not $r.ContainsKey('remaining')) { throw 'Missing verification or boundaries' }
    foreach ($v in $r.validation) {
        if (-not $v.command -or -not $v.result -or -not $v.evidence) { throw 'Incomplete verification entry' }
        SafePath $v.evidence
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $v.evidence) -PathType Leaf)) { throw 'Missing verification evidence' }
    }
    return $r
}
if ($Mode -eq 'Staged') {
    $staged = @(Invoke-RepoGit @('diff', '--cached', '--name-only', '--diff-filter=ACDMRT'))
    if ($staged.Count -eq 0) { throw 'No staged delivery' }
    $receipts = @($staged | Where-Object { $_ -match '^docs/development-evidence/changes/[a-z0-9-]+\.json$' })
    if ($receipts.Count -eq 0) { throw 'Stage a per-task delivery receipt first' }
    $covered = @{}
    $currentIds = @()
    foreach ($path in $receipts) {
        $r = Receipt ":$path"
        $t = Task $r.task_id
        if ($t.status -cne 'in_progress') { throw 'Delivery task must be running before commit' }
        if ($t.id -match '^2026-10-11-roadmap-' -or $t.id -eq '2026-10-11-mjwarp-parallel-acceptance') { throw 'Use a scoped delivery task, not an umbrella' }
        if (-not $t.note.Contains("delivery-receipt: $path")) { throw 'Normify task has no receipt binding' }
        foreach ($dependency in $t.schedule['depends_on']) {
            if ((Task $dependency).status -ne 'verified') { throw 'Unmet task prerequisite' }
        }
        $currentIds += $t.id
        if (@($r.files).Count -eq 0) { throw 'Empty delivery coverage' }
        foreach ($f in $r.files) {
            SafePath $f.path
            if ($f.path -notin $staged -or $f.path -in $receipts -or $covered.ContainsKey($f.path)) { throw 'Duplicate or unrelated receipt coverage' }
            $deleted = $f.path -in @(Invoke-RepoGit @('diff', '--cached', '--name-only', '--diff-filter=D'))
            $claims = @($t.schedule.files | Where-Object { $f.path -ceq $_ -or $f.path.StartsWith($_.TrimEnd('/') + '/', [StringComparison]::Ordinal) })
            if ($claims.Count -eq 0) { throw "File outside scheduled scope: $($f.path)" }
            $actual = if ($deleted) { 'deleted' } else { @(Invoke-RepoGit @('rev-parse', ":$($f.path)"))[0] }
            if ($f.blob -cne $actual) { throw "Staged blob changed: $($f.path)" }
            $covered[$f.path] = $true
        }
        foreach ($v in $r.validation) {
            # 只接受提交内的证据。工作区文件不能冒充提交证据。
            Invoke-RepoGit @('cat-file', '-e', ":$($v.evidence)") | Out-Null
        }
    }
    foreach ($path in $staged) {
        if ($path -notin $receipts -and -not $covered.ContainsKey($path)) { throw "Unmapped staged file: $path" }
    }
    # 下一次提交前核对上次收尾。不能只写备注而长期悬挂。
    $previous = @(Invoke-RepoGit @('diff-tree', '--root', '--no-commit-id', '--name-only', '-r', 'HEAD') | Where-Object { $_ -match '^docs/development-evidence/changes/[a-z0-9-]+\.json$' })
    foreach ($path in $previous) {
        $prior = (Invoke-RepoGit @('show', "HEAD:$path")) -join "`n" | ConvertFrom-Json -AsHashtable
        $t = Task $prior.task_id
        if ($t.status -ne 'verified') { throw "Previous delivery needs Normify close: $($t.id)" }
        $head = @(Invoke-RepoGit @('rev-parse', 'HEAD'))[0]
        if (-not $t.note.Contains($head)) { throw 'Previous task lacks its actual commit identity' }
    }
    [ordered]@{ mode = $Mode; receipts = $receipts.Count; covered_files = $covered.Count; result = 'pass' } | ConvertTo-Json
    return
}
$ledger = JsonFile (Join-Path $RepoRoot 'docs/development-evidence/schedule-reconciliation.json')
$before = JsonFile (Join-Path $RepoRoot 'docs/development-evidence/schedule-before-2026-10-16.json')
$original = Task $ledger.original_gate
$successor = Task $ledger.successor_gate
if ($original.status -ne 'abandoned' -or $successor.status -eq 'verified') { throw 'Gate migration state mismatch' }
if (-not (EqualJson $ledger.original_acceptance $before.task.acceptance) -or
    -not (EqualJson $original.acceptance $successor.acceptance) -or
    -not (EqualJson $ledger.original_acceptance $successor.acceptance)) { throw 'Original acceptance changed' }
$mapped = @{}
$bindings = 0
foreach ($r in $ledger.records) {
    $t = Task $r.id
    if ($t.status -ne 'verified' -or $r.product_acceptance) { throw "Historical record status: $($r.id)" }
    foreach ($commit in $r.commits) {
        Invoke-RepoGit @('merge-base', '--is-ancestor', $commit, $ledger.baseline) | Out-Null
        if ($mapped.ContainsKey($commit)) { throw 'Duplicate historical commit mapping' }
        if (-not $t.note.Contains($commit)) { throw 'Missing visible commit binding' }
        $mapped[$commit] = $r.id
    }
    foreach ($e in $r.evidence_bindings) {
        $actual = @(Invoke-RepoGit @('rev-parse', "$($e.revision):$($e.path)"))[0]
        if ($actual -cne $e.blob) { throw 'Historical evidence object mismatch' }
        $bindings++
    }
}
foreach ($r in $ledger.existing_records) {
    if ((Task $r.id).status -ne 'verified') { throw 'Existing accepted record changed' }
    $mapped[@(Invoke-RepoGit @('rev-parse', $r.commit))[0]] = $r.id
}
$range = "$($ledger.range_start)^..$($ledger.baseline)"
foreach ($sha in (Invoke-RepoGit @('rev-list', $range))) {
    if (-not $mapped.ContainsKey($sha)) { throw "Unmapped historical commit: $sha" }
}
foreach ($old in $before.schedule) {
    $t = Task $old.id
    if ($old.id -eq $ledger.original_gate) { continue }
    foreach ($dep in $old.depends_on) {
        $expected = if ($dep -eq $ledger.original_gate) { $ledger.successor_gate } else { $dep }
        if ($expected -notin $t.schedule.depends_on) { throw "Removed prerequisite: $($old.id)" }
    }
    if ($old.id -match 'p10$' -and $t.status -ne $old.status) { throw 'P10 promotion without evidence' }
}
foreach ($r in $ledger.remaining) {
    $t = Task $r.id
    if ($t.status -ne 'proposed' -or -not (EqualJson $t.acceptance $r.acceptance) -or
        -not (EqualJson $t.schedule.depends_on $r.schedule.depends_on)) { throw "Remaining work mismatch: $($r.id)" }
}
[ordered]@{ mode = $Mode; historical_tasks = @($ledger.records).Count; commits = $mapped.Count; evidence_objects = $bindings; gates_preserved = $true; result = 'pass' } | ConvertTo-Json
