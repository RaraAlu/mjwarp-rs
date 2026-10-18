$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$task = '2026-10-19-layers-m3-applied-force-validate-byte-correction'
$evidenceDir = 'layers-m3-applied-force-validate-byte-correction'
$base = '25cc639be64b404f80be6edd3e0db178362e03b6'
$manifest = 'docs/development-evidence/layers-m3-applied-force-validate/formal-run/manifest.json'
$map = "docs/development-evidence/$evidenceDir/source-map.json"
$driftPath = "docs/development-evidence/$evidenceDir/drift-map.json"
$summaryPath = "docs/development-evidence/$evidenceDir/validation-summary.json"
$formal = "docs/development-evidence/$evidenceDir/formal-run"
$archiveRoot = 'docs/development-evidence/layers-m3-applied-force-validate/formal-run'

function Sha256 {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyCollection()]
        [byte[]]$Bytes
    )
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function SameBytes([string]$Left, [string]$Right) {
    $leftBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Left))
    $rightBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Right))
    $leftBytes.Length -eq $rightBytes.Length -and
        (Sha256 -Bytes $leftBytes) -ceq (Sha256 -Bytes $rightBytes)
}

function RunVerifier([string]$Revision) {
    $output = @(
        & pwsh -NoProfile -File scripts/check-evidence-blobs.ps1 `
            -Manifest $manifest -Map $map -Revision $Revision 2>&1
    )
    $exitCode = $LASTEXITCODE
    $json = ($output -join "`n") | ConvertFrom-Json
    [pscustomobject]@{ ExitCode = $exitCode; Json = $json }
}

if ((& git rev-parse HEAD).Trim() -cne $base) { throw '基线提交已变化' }
$drift = Get-Content -LiteralPath $driftPath -Raw | ConvertFrom-Json
$summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
if ($drift.base_revision -cne $base -or $summary.base_revision -cne $base) {
    throw '纠正基线不一致'
}
if ($summary.task -cne $task -or $summary.product_tests -cne 'not_rerun') {
    throw '纠正摘要范围不一致'
}

$old = RunVerifier $base
$index = RunVerifier 'INDEX'
if ($old.ExitCode -ne 1 -or $old.Json.members -ne 49 -or $old.Json.mismatches -ne 24) {
    throw '旧提交检出结果错误'
}
if ($index.ExitCode -ne 0 -or $index.Json.members -ne 49 -or $index.Json.mismatches -ne 0) {
    throw '暂存对象核验结果错误'
}
if ($old.Json.manifest_sha256 -cne $drift.manifest.sha256) { throw '旧清单摘要漂移' }

$oldByPath = @{}
$indexByPath = @{}
foreach ($row in $old.Json.rows) { $oldByPath[$row.path] = $row }
foreach ($row in $index.Json.rows) { $indexByPath[$row.path] = $row }
$beforeRecorded = Get-Content -LiteralPath "$formal/old-commit-retry.stdout.txt" -Raw | ConvertFrom-Json
if ($beforeRecorded.members -ne 49 -or $beforeRecorded.mismatches -ne 24) {
    throw '暂存前记录不完整'
}
$beforeByPath = @{}
foreach ($row in $beforeRecorded.rows) { $beforeByPath[$row.path] = $row }
$entries = @($drift.entries)
if ($entries.Count -ne 24) { throw '漂移映射数量错误' }
$entryPaths = @{}
foreach ($entry in $entries) {
    if ($entryPaths.ContainsKey($entry.path)) { throw "漂移映射重复：$($entry.path)" }
    $entryPaths[$entry.path] = $true
    $before = $oldByPath[$entry.path]
    $after = $indexByPath[$entry.path]
    $recordedBefore = $beforeByPath[$entry.path]
    if ($null -eq $before -or $null -eq $after -or $before.matches -or -not $after.matches) {
        throw "漂移状态错误：$($entry.path)"
    }
    if ($null -eq $recordedBefore -or $recordedBefore.matches) {
        throw "暂存前漂移状态错误：$($entry.path)"
    }
    if (
        $entry.old_blob -cne $before.object -or
        $entry.old_blob_sha256 -cne $before.object_sha256 -or
        $entry.new_staged_blob -cne $after.object -or
        $entry.new_staged_blob_sha256 -cne $after.object_sha256
    ) {
        throw "对象映射错误：$($entry.path)"
    }
    if (
        $entry.original_before_evidence -cne 'formal-run/old-commit-retry.stdout.txt' -or
        $entry.original_before_sha256 -cne $recordedBefore.archive_sha256 -or
        $entry.original_before_bytes -ne $recordedBefore.archive_bytes -or
        $entry.original_before_sha256 -cne $entry.original_after_sha256 -or
        $entry.original_after_sha256 -cne $entry.new_staged_blob_sha256 -or
        $entry.original_before_bytes -ne $entry.original_after_bytes -or
        $entry.original_after_bytes -ne $entry.new_staged_blob_bytes
    ) {
        throw "原件变化或暂存不符：$($entry.path)"
    }
}
$oldMismatchPaths = @($old.Json.rows | Where-Object { -not $_.matches } | ForEach-Object path)
if (@($oldMismatchPaths | Where-Object { -not $entryPaths.ContainsKey($_) }).Count -ne 0) {
    throw '漂移映射遗漏旧对象'
}

$logSources = [ordered]@{
    'old-commit-first' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-01-old-commit'
    'old-commit-retry' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-01b-old-commit-retry'
    'old-commit-final' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-01c-old-commit-final'
    'stage-original-bytes' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-02-stage'
    'index-first' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-03-index'
    'index-final' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-03b-index-final'
    'evidence-first' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-04-evidence'
    'evidence-retry' = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate-byte-correction/check-04b-evidence-retry'
}
$archivedLogs = 0
foreach ($prefix in $logSources.Keys) {
    foreach ($name in @('run.json', 'stdout.txt', 'stderr.txt')) {
        $archived = "$formal/$prefix.$name"
        $source = "$($logSources[$prefix])/$name"
        if (-not (SameBytes $archived $source)) { throw "归档日志变化：$archived" }
        $attributes = @((& git check-attr text diff -- $archived) -split "`n")
        if ($attributes.Count -ne 2 -or $attributes[0] -notmatch ': text: unset$' -or $attributes[1] -notmatch ': diff: unset$') {
            throw "归档日志属性错误：$archived"
        }
        $archivedLogs++
    }
}

$unstagedArchive = @(& git diff --name-only -- $archiveRoot)
if ($unstagedArchive.Count -ne 0) { throw '磁盘归档出现未暂存变化' }
$staged = @(& git diff --cached --name-only)
$stagedOld = @($staged | Where-Object { $_ -like "$archiveRoot/*" })
if ($stagedOld.Count -ne 24) { throw "旧归档暂存数量错误：$($stagedOld.Count)" }
foreach ($path in $stagedOld) {
    $relative = $path.Substring($archiveRoot.Length + 1)
    if (-not $entryPaths.ContainsKey($relative)) { throw "旧归档暂存越界：$path" }
}
$allowedReceipt = "docs/development-evidence/changes/$task.json"
foreach ($path in $staged) {
    if (
        $path -notlike "$archiveRoot/*" -and
        $path -cne 'scripts/check-evidence-blobs.ps1' -and
        $path -notlike "docs/development-evidence/$evidenceDir/*" -and
        $path -cne $allowedReceipt
    ) {
        throw "暂存范围越界：$path"
    }
}

[ordered]@{
    schema = 1
    task = $task
    result = 'pass'
    base_revision = $base
    manifest_members = $index.Json.members
    old_mismatches = $old.Json.mismatches
    index_mismatches = $index.Json.mismatches
    drift_entries = $entries.Count
    archived_logs = $archivedLogs
    staged_files = $staged.Count
    product_tests = 'not_rerun'
} | ConvertTo-Json -Depth 4
