#Requires -Version 7.0
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$sandbox = Join-Path $root ('target/normify-workflow-tests/' + [Guid]::NewGuid().ToString('N'))
$repo = Join-Path $sandbox 'repo'
$normify = Join-Path $sandbox 'normify-fixture'
$checker = Join-Path $PSScriptRoot 'check-normify-delivery.ps1'
New-Item -ItemType Directory -Path $repo, "$normify/changes", "$repo/docs/development-evidence/changes" -Force | Out-Null
function G([string[]]$Arguments) {
    $result = @(& git -C $repo @Arguments 2>&1)
    if ($LASTEXITCODE -ne 0) { throw ($result -join "`n") }
    return $result
}
function SaveJson([string]$Path, $Value) {
    [IO.File]::WriteAllText($Path, (ConvertTo-Json -InputObject $Value -Depth 20), [Text.UTF8Encoding]::new($false))
}
$checks = [Collections.Generic.List[object]]::new()
function Check([string]$Name, [string]$Expected = '', [string]$Mode = 'Staged', [string]$TaskId = '') {
    $errorText = ''
    try { & $checker -RepoRoot $repo -NormifyDir $normify -Mode $Mode -TaskId $TaskId | Out-Null }
    catch { $errorText = $_.Exception.Message }
    if (($Expected -eq '' -and $errorText) -or ($Expected -ne '' -and $errorText -notlike "*$Expected*")) {
        throw "$Name expected '$Expected', got '$errorText'"
    }
    $checks.Add(@{ name = $Name; result = 'pass'; rejection = $errorText })
}
G @('init', '-q') | Out-Null
G @('config', 'user.name', 'Normify Test') | Out-Null
G @('config', 'user.email', 'normify-test@example.invalid') | Out-Null
G @('config', 'core.autocrlf', 'false') | Out-Null
[IO.File]::WriteAllText("$repo/base.txt", 'base')
SaveJson "$repo/evidence.json" @{ purpose = 'isolated guard test fixture'; result = 'pass' }
G @('add', '.') | Out-Null
G @('-c', 'commit.gpgsign=false', 'commit', '-qm', '测试: 初始化隔离夹具') | Out-Null
$receiptPath = 'docs/development-evidence/changes/fixture-delivery.json'
$task = @{ id = 'fixture-delivery'; status = 'in_progress'; acceptance = @('fixture acceptance'); note = "delivery-receipt: $receiptPath"; schedule = @{ depends_on = @(); files = @('code.txt') }; revision = @{ before = $null; after = $null } }
$taskFile = "$normify/changes/fixture-delivery.json"
SaveJson $taskFile $task
[IO.File]::WriteAllText("$repo/code.txt", 'one')
G @('add', 'code.txt') | Out-Null
Check 'valid-preflight' '' 'Preflight' 'fixture-delivery'
Check 'missing-receipt' 'Stage a per-task'
$receipt = @{ schema = 1; kind = 'delivery'; task_id = 'fixture-delivery'; milestones = @{ implementation = 'complete'; validation = 'passed'; delivery_record = 'pending_commit' }; files = @(@{ path = 'code.txt'; blob = @(G @('rev-parse', ':code.txt'))[0] }); validation = @(@{ command = 'isolated test'; result = 'pass'; evidence = 'evidence.json' }); remaining = @('not product acceptance') }
function StageReceipt { SaveJson "$repo/$receiptPath" $receipt; G @('add', $receiptPath) | Out-Null }
StageReceipt
Check 'valid-receipt'
$milestones = $receipt.milestones
$receipt.Remove('milestones'); StageReceipt
Check 'missing-milestones' 'Missing delivery milestones'
$receipt.milestones = @{ implementation = 'pending'; validation = 'passed'; delivery_record = 'pending_commit' }; StageReceipt
Check 'implementation-incomplete' 'Implementation milestone is incomplete'
$receipt.milestones = @{ implementation = 'complete'; validation = 'pending'; delivery_record = 'pending_commit' }; StageReceipt
Check 'validation-incomplete' 'Validation milestone is incomplete'
$receipt.milestones = @{ implementation = 'complete'; validation = 'passed'; delivery_record = 'complete' }; StageReceipt
Check 'premature-delivery-complete' 'must await the commit identity'
$receipt.milestones = $milestones; StageReceipt
$task.schedule.Remove('depends_on'); SaveJson $taskFile $task
Check 'optional-dependency-list'
$task.schedule.depends_on = @()
$task.status = 'proposed'; SaveJson $taskFile $task
Check 'task-not-running' 'must be running'
Check 'preflight-task-not-running' 'must be running' 'Preflight' 'fixture-delivery'
$task.status = 'in_progress'; $task.note = ''; SaveJson $taskFile $task
Check 'missing-task-binding' 'no receipt binding'
$task.note = "delivery-receipt: $receiptPath"; $task.schedule.files = @('other.txt'); SaveJson $taskFile $task
Check 'outside-claims' 'outside scheduled scope'
$task.schedule.files = @('code.txt'); $task.schedule.depends_on = @('fixture-parent'); SaveJson $taskFile $task
SaveJson "$normify/changes/fixture-parent.json" @{ id = 'fixture-parent'; status = 'proposed' }
Check 'unmet-prerequisite' 'Unmet task prerequisite'
$task.schedule.depends_on = @(); SaveJson $taskFile $task
$oldBlob = $receipt.files[0].blob
$receipt.files[0].blob = '0000000000000000000000000000000000000000'; StageReceipt
Check 'stale-staged-blob' 'Staged blob changed'
$receipt.files[0].blob = $oldBlob; StageReceipt
[IO.File]::WriteAllText("$repo/extra.txt", 'unmapped'); G @('add', 'extra.txt') | Out-Null
Check 'unmapped-file' 'Unmapped staged file'
G @('rm', '--cached', 'extra.txt') | Out-Null
$receipt.validation[0].result = ''; StageReceipt
Check 'missing-result' 'Incomplete verification'
$receipt.validation[0].result = 'pass'; $receipt.validation[0].evidence = '../outside.json'; StageReceipt
Check 'unsafe-evidence' 'Unsafe receipt path'
$receipt.validation[0].evidence = 'absent.json'; StageReceipt
Check 'missing-evidence' 'Missing verification evidence'
$receipt.validation[0].evidence = 'evidence.json'; StageReceipt
$receipt.task_id = '2026-10-11-roadmap-p10'
SaveJson "$normify/changes/2026-10-11-roadmap-p10.json" @{
    id = $receipt.task_id
    status = 'in_progress'
    acceptance = @('fixture acceptance')
    note = "delivery-receipt: $receiptPath"
    schedule = @{ depends_on = @(); files = @('code.txt') }
}
StageReceipt
Check 'umbrella-rejected' 'not an umbrella'
$receipt.task_id = 'fixture-delivery'; StageReceipt
Check 'restored-valid-receipt'
G @('-c', 'commit.gpgsign=false', 'commit', '-qm', '测试: 提交隔离交付回执') | Out-Null
$previousHead = @(G @('rev-parse', 'HEAD'))[0]
Check 'post-commit-before-recording' '' 'PostCommit' 'fixture-delivery'
Check 'recorded-requires-full-sha' 'lacks the full commit identity' 'Recorded' 'fixture-delivery'
$receiptPath = 'docs/development-evidence/changes/fixture-next.json'
$next = @{ id = 'fixture-next'; status = 'in_progress'; acceptance = @('fixture acceptance'); note = "delivery-receipt: $receiptPath"; schedule = @{ depends_on = @(); files = @('code.txt') }; revision = @{ before = $null; after = $null } }
SaveJson "$normify/changes/fixture-next.json" $next
[IO.File]::WriteAllText("$repo/code.txt", 'two'); G @('add', 'code.txt') | Out-Null
$receipt.task_id = 'fixture-next'; $receipt.files[0].blob = @(G @('rev-parse', ':code.txt'))[0]; StageReceipt
Check 'previous-not-closed' 'Previous delivery needs'
$task.note += "`n$previousHead"; SaveJson $taskFile $task
Check 'recorded-full-sha' '' 'Recorded' 'fixture-delivery'
Check 'previous-still-open' 'Previous delivery needs'
$task.status = 'verified'; SaveJson $taskFile $task
Check 'previous-no-revision' 'revision does not match'
$task.revision.after = $previousHead; SaveJson $taskFile $task
Check 'previous-closed-and-bound'
Check 'final-closed-and-bound' '' 'Final' 'fixture-delivery'

$secondReceiptPath = 'docs/development-evidence/changes/fixture-second.json'
$secondTask = @{ id = 'fixture-second'; status = 'in_progress'; acceptance = @('fixture acceptance'); note = "delivery-receipt: $secondReceiptPath"; schedule = @{ depends_on = @(); files = @('other.txt') }; revision = @{ before = $null; after = $null } }
$secondTaskFile = "$normify/changes/fixture-second.json"
SaveJson $secondTaskFile $secondTask
[IO.File]::WriteAllText("$repo/other.txt", 'other'); G @('add', 'other.txt') | Out-Null
$secondReceipt = @{
    schema = 1
    kind = 'delivery'
    task_id = 'fixture-second'
    milestones = @{ implementation = 'complete'; validation = 'passed'; delivery_record = 'pending_commit' }
    files = @(@{ path = 'other.txt'; blob = @(G @('rev-parse', ':other.txt'))[0] })
    validation = @(@{ command = 'isolated test'; result = 'pass'; evidence = 'evidence.json' })
    remaining = @('not product acceptance')
}
SaveJson "$repo/$secondReceiptPath" $secondReceipt; G @('add', $secondReceiptPath) | Out-Null
Check 'multiple-receipts-staged'
G @('-c', 'commit.gpgsign=false', 'commit', '-qm', '测试: 提交多回执夹具') | Out-Null
$multipleHead = @(G @('rev-parse', 'HEAD'))[0]
Check 'multiple-receipts-first-post-commit' '' 'PostCommit' 'fixture-next'
Check 'multiple-receipts-second-post-commit' '' 'PostCommit' 'fixture-second'
$next.note += "`n$multipleHead"; SaveJson "$normify/changes/fixture-next.json" $next
$secondTask.note += "`n$multipleHead"; SaveJson $secondTaskFile $secondTask
Check 'multiple-receipts-first-recorded' '' 'Recorded' 'fixture-next'
Check 'multiple-receipts-second-recorded' '' 'Recorded' 'fixture-second'
$next.status = 'verified'; $next.revision.after = $multipleHead; SaveJson "$normify/changes/fixture-next.json" $next
$secondTask.status = 'verified'; $secondTask.revision.after = $multipleHead; SaveJson $secondTaskFile $secondTask
Check 'multiple-receipts-first-final' '' 'Final' 'fixture-next'
Check 'multiple-receipts-second-final' '' 'Final' 'fixture-second'

$deleteReceiptPath = 'docs/development-evidence/changes/fixture-delete.json'
$deleteTask = @{ id = 'fixture-delete'; status = 'in_progress'; acceptance = @('fixture acceptance'); note = "delivery-receipt: $deleteReceiptPath"; schedule = @{ depends_on = @(); files = @('code.txt') }; revision = @{ before = $null; after = $null } }
SaveJson "$normify/changes/fixture-delete.json" $deleteTask
[IO.File]::WriteAllText("$repo/code.txt", 'three'); G @('add', 'code.txt') | Out-Null
$deleteReceipt = @{
    schema = 1
    kind = 'delivery'
    task_id = 'fixture-delete'
    milestones = @{ implementation = 'complete'; validation = 'passed'; delivery_record = 'pending_commit' }
    files = @(@{ path = 'code.txt'; blob = 'deleted' })
    validation = @(@{ command = 'isolated test'; result = 'pass'; evidence = 'evidence.json' })
    remaining = @('not product acceptance')
}
SaveJson "$repo/$deleteReceiptPath" $deleteReceipt; G @('add', $deleteReceiptPath) | Out-Null
G @('-c', 'commit.gpgsign=false', 'commit', '-qm', '测试: 提交错误删除声明') | Out-Null
Check 'false-delete-rejected' 'Deleted receipt path still exists' 'PostCommit' 'fixture-delete'
$report = @{ schema = 1; passed = $checks.Count; failed = 0; checks = $checks; sandbox = $sandbox; product_tests_run = 0 }
SaveJson (Join-Path $sandbox 'report.json') $report
$report | ConvertTo-Json -Depth 5
