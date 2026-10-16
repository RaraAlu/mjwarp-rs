#Requires -Version 7.0
[CmdletBinding()]
param(
    [ValidateSet('Preflight', 'Staged', 'PostCommit', 'Recorded', 'Final', 'Audit')][string]$Mode = 'Staged',
    [string]$RepoRoot = (Split-Path -Parent $PSScriptRoot),
    [string]$NormifyDir,
    [string]$TaskId
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
function ReceiptPath([string]$Id) { return "docs/development-evidence/changes/$Id.json" }
function SafePath([string]$Path) {
    if (-not $Path -or $Path -match '(^/|\\|(^|/)\.\.?(/|$)|[\r\n\t:]|^\.git(/|$))') {
        throw "Unsafe receipt path: $Path"
    }
}
function Receipt([string]$Spec) {
    $r = (Invoke-RepoGit @('show', $Spec)) -join "`n" | ConvertFrom-Json -AsHashtable
    if ($r.schema -ne 1 -or $r.kind -notin @('delivery', 'governance')) { throw 'Invalid delivery receipt' }
    if (@($r.validation).Count -eq 0 -or -not $r.ContainsKey('remaining')) { throw 'Missing verification or boundaries' }
    if (-not $r.ContainsKey('milestones')) { throw 'Missing delivery milestones' }
    if ($r.milestones.implementation -notin @('complete', 'not_applicable')) { throw 'Implementation milestone is incomplete' }
    if ($r.milestones.validation -cne 'passed') { throw 'Validation milestone is incomplete' }
    if ($r.milestones.delivery_record -cne 'pending_commit') { throw 'Delivery record must await the commit identity' }
    foreach ($v in $r.validation) {
        if (-not $v.command -or -not $v.result -or -not $v.evidence) { throw 'Incomplete verification entry' }
        SafePath $v.evidence
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $v.evidence) -PathType Leaf)) { throw 'Missing verification evidence' }
    }
    return $r
}
function Assert-TaskReady($Value, [string]$ReceiptPath) {
    if ($Value.status -cne 'in_progress') { throw 'Delivery task must be running before commit' }
    if (-not $Value.ContainsKey('acceptance') -or @($Value.acceptance).Count -eq 0) { throw 'Delivery task has no acceptance criteria' }
    if (-not $Value.ContainsKey('schedule') -or @($Value.schedule.files).Count -eq 0) { throw 'Delivery task has no file scope' }
    if (-not $Value.note.Contains("delivery-receipt: $ReceiptPath")) { throw 'Normify task has no receipt binding' }
    foreach ($dependency in @($Value.schedule['depends_on']) | Where-Object { $_ }) {
        if ((Task $dependency).status -ne 'verified') { throw 'Unmet task prerequisite' }
    }
}
function Assert-PreviousDeliveryFinalized {
    $previous = @(Invoke-RepoGit @('diff-tree', '--root', '--no-commit-id', '--name-only', '-r', 'HEAD') | Where-Object { $_ -match '^docs/development-evidence/changes/[a-z0-9-]+\.json$' })
    foreach ($path in $previous) {
        $prior = (Invoke-RepoGit @('show', "HEAD:$path")) -join "`n" | ConvertFrom-Json -AsHashtable
        $task = Task $prior.task_id
        if ($task.status -ne 'verified') { throw "Previous delivery needs Normify close: $($task.id)" }
        $head = @(Invoke-RepoGit @('rev-parse', 'HEAD'))[0]
        if (-not $task.note.Contains($head)) { throw 'Previous task lacks its actual commit identity' }
        if ($task.revision.after -cne $head) { throw 'Previous task revision does not match the actual commit' }
    }
}
function Inspect-HeadDelivery([string]$ExpectedTaskId) {
    $head = @(Invoke-RepoGit @('rev-parse', 'HEAD'))[0]
    $commitFiles = @(Invoke-RepoGit @('diff-tree', '--root', '--no-commit-id', '--name-only', '-r', 'HEAD'))
    $deletedFiles = @(Invoke-RepoGit @('diff-tree', '--root', '--no-commit-id', '--name-only', '--diff-filter=D', '-r', 'HEAD'))
    $receiptPaths = @($commitFiles | Where-Object { $_ -match '^docs/development-evidence/changes/[a-z0-9-]+\.json$' })
    $expectedReceiptPath = ReceiptPath $ExpectedTaskId
    if ($expectedReceiptPath -notin $receiptPaths) { throw 'Post-commit receipt is missing from HEAD' }
    $covered = @{}
    $currentReceipt = $null
    foreach ($path in $receiptPaths) {
        $receipt = Receipt "HEAD:$path"
        if ($path -cne (ReceiptPath $receipt.task_id)) { throw 'Post-commit receipt path mismatch' }
        if ($receipt.task_id -ceq $ExpectedTaskId) { $currentReceipt = $receipt }
        if (@($receipt.files).Count -eq 0) { throw 'Empty delivery coverage' }
        foreach ($file in $receipt.files) {
            SafePath $file.path
            if ($file.path -in $receiptPaths -or $file.path -notin $commitFiles -or $covered.ContainsKey($file.path)) {
                throw 'Duplicate or unrelated post-commit coverage'
            }
            if ($file.blob -ceq 'deleted') {
                & git -C $RepoRoot cat-file -e "HEAD:$($file.path)" 2>$null
                if ($LASTEXITCODE -eq 0) { throw "Deleted receipt path still exists: $($file.path)" }
                if ($file.path -notin $deletedFiles) { throw "Deleted receipt path was not deleted: $($file.path)" }
            }
            else {
                if ($file.path -in $deletedFiles) { throw "Post-commit blob mismatch: $($file.path)" }
                $actual = @(Invoke-RepoGit @('rev-parse', "HEAD:$($file.path)"))[0]
                if ($file.blob -cne $actual) { throw "Post-commit blob mismatch: $($file.path)" }
            }
            $covered[$file.path] = $true
        }
        foreach ($validation in $receipt.validation) {
            Invoke-RepoGit @('cat-file', '-e', "HEAD:$($validation.evidence)") | Out-Null
        }
    }
    foreach ($path in $commitFiles) {
        if ($path -notin $receiptPaths -and -not $covered.ContainsKey($path)) {
            throw "Post-commit receipt coverage mismatch: $path"
        }
    }
    return [pscustomobject]@{
        Head = $head
        CurrentReceipt = $currentReceipt
        ReceiptCount = $receiptPaths.Count
        CoveredCount = $covered.Count
    }
}
if ($Mode -eq 'Preflight') {
    if (-not $TaskId) { throw 'Preflight requires TaskId' }
    $path = ReceiptPath $TaskId
    $task = Task $TaskId
    Assert-TaskReady $task $path
    Assert-PreviousDeliveryFinalized
    [ordered]@{
        mode = $Mode
        task_id = $TaskId
        task_status = $task.status
        acceptance = @($task.acceptance).Count
        scoped_paths = @($task.schedule.files).Count
        previous_delivery = 'finalized'
        result = 'pass'
    } | ConvertTo-Json
    return
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
        Assert-TaskReady $t $path
        if ($t.id -match '^2026-10-11-roadmap-' -or $t.id -eq '2026-10-11-mjwarp-parallel-acceptance') { throw 'Use a scoped delivery task, not an umbrella' }
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
    Assert-PreviousDeliveryFinalized
    [ordered]@{ mode = $Mode; receipts = $receipts.Count; covered_files = $covered.Count; result = 'pass' } | ConvertTo-Json
    return
}
if ($Mode -in @('PostCommit', 'Recorded', 'Final')) {
    if (-not $TaskId) { throw "$Mode requires TaskId" }
    $inspection = Inspect-HeadDelivery $TaskId
    $task = Task $TaskId
    if ($Mode -in @('PostCommit', 'Recorded')) {
        if ($task.status -cne 'in_progress') { throw "$Mode requires a writable delivery task" }
    }
    else {
        if ($task.status -cne 'verified') { throw 'Final requires a closed delivery task' }
    }
    if ($Mode -in @('Recorded', 'Final') -and -not $task.note.Contains($inspection.Head)) {
        throw "$Mode task lacks the full commit identity"
    }
    if ($Mode -eq 'Final' -and $task.revision.after -cne $inspection.Head) {
        throw 'Final revision mismatch'
    }
    [ordered]@{
        mode = $Mode
        task_id = $TaskId
        commit = $inspection.Head
        receipts = $inspection.ReceiptCount
        covered_files = $inspection.CoveredCount
        implementation = $inspection.CurrentReceipt.milestones.implementation
        validation = $inspection.CurrentReceipt.milestones.validation
        delivery_record = if ($Mode -eq 'PostCommit') { 'commit_verified' } elseif ($Mode -eq 'Recorded') { 'sha_recorded' } else { 'complete' }
        result = 'pass'
    } | ConvertTo-Json
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
$correctionPath = 'docs/development-evidence/delivery-corrections.json'
$corrections = JsonFile (Join-Path $RepoRoot $correctionPath)
foreach ($r in $ledger.remaining) {
    $t = Task $r.id
    $frozenStatus = if ($r.status) { $r.status } else { 'proposed' }
    $allowed = switch ($frozenStatus) {
        'proposed' { @('proposed', 'in_progress', 'verified') }
        'in_progress' { @('in_progress', 'verified') }
        'verified' { @('verified') }
        default { @() }
    }
    if ($t.status -ceq 'abandoned') {
        $abandonment = @($corrections.abandonments | Where-Object { $_.task_id -ceq $r.id })
        if ($abandonment.Count -ne 1 -or $abandonment[0].status -cne 'abandoned' -or
            $abandonment[0].product_acceptance -ne $false -or -not $abandonment[0].reason) {
            throw "Unrecorded remaining-task abandonment: $($r.id)"
        }
    }
    elseif ($t.status -notin $allowed) { throw "Remaining work state mismatch: $($r.id)" }
    if (-not (EqualJson $t.acceptance $r.acceptance) -or
        -not (EqualJson $t.schedule.depends_on $r.schedule.depends_on)) { throw "Remaining work mismatch: $($r.id)" }
    if ($frozenStatus -cne 'verified' -and $t.status -ceq 'verified') {
        $receiptPath = ReceiptPath $r.id
        Invoke-RepoGit @('cat-file', '-e', "HEAD:$receiptPath") | Out-Null
        $receipt = (Invoke-RepoGit @('show', "HEAD:$receiptPath")) -join "`n" | ConvertFrom-Json -AsHashtable
        if ($receipt.task_id -cne $r.id) { throw "Advanced task lacks a matching delivery receipt: $($r.id)" }
    }
}
if ($corrections.schema -ne 1 -or @($corrections.entries).Count -eq 0) { throw 'Missing delivery correction records' }
foreach ($correction in $corrections.entries) {
    if ($correction.target_commit -notmatch '^[0-9a-f]{40}$' -or $correction.bypass_commit -notmatch '^[0-9a-f]{40}$') {
        throw 'Correction record requires full commit identities'
    }
    Invoke-RepoGit @('cat-file', '-e', "$($correction.target_commit)^{commit}") | Out-Null
    Invoke-RepoGit @('cat-file', '-e', "$($correction.bypass_commit)^{commit}") | Out-Null
    $originalReceipt = (Invoke-RepoGit @('show', "$($correction.target_commit):$($correction.original_receipt)")) -join "`n" | ConvertFrom-Json -AsHashtable
    if ($originalReceipt.task_id -cne $correction.original_task_id) { throw 'Correction target task mismatch' }
    $originalTask = Task $correction.original_task_id
    if ($originalTask.status -cne 'verified') { throw 'Correction target is not closed' }
    if ($originalTask.revision.after -cne $correction.original_revision_after) { throw 'Correction target revision mismatch' }
    if ($correction.ContainsKey('full_sha_present_in_original_note') -and
        $originalTask.note.Contains($correction.target_commit) -ne $correction.full_sha_present_in_original_note) {
        throw 'Correction target note state mismatch'
    }
    foreach ($related in @($correction.related_main_records)) {
        $relatedTask = Task $related.task_id
        if ($relatedTask.status -cne $related.status -or $relatedTask.revision.after -cne $related.revision_after) {
            throw 'Related correction task state mismatch'
        }
    }
    if ($correction.kind -eq 'd04_delivery_state_conflict' -and $correction.normal_delivery_hook -cne 'not_passed') {
        throw 'D04 bypass must not be recorded as a normal hook pass'
    }
    $remediation = Task $correction.remediation_task_id
    if ($remediation.status -cne 'verified' -or -not $remediation.note.Contains($correction.target_commit) -or
        -not $remediation.note.Contains($correction.bypass_commit)) { throw 'Correction task lacks immutable history bindings' }
    $correctionCommit = @(Invoke-RepoGit @('log', '-1', '--format=%H', '--', $correctionPath))[0]
    if (-not $remediation.note.Contains($correctionCommit) -or $remediation.revision.after -cne $correctionCommit) {
        throw 'Correction task lacks its actual delivery commit'
    }
}
[ordered]@{ mode = $Mode; historical_tasks = @($ledger.records).Count; commits = $mapped.Count; evidence_objects = $bindings; corrections = @($corrections.entries).Count; gates_preserved = $true; result = 'pass' } | ConvertTo-Json
