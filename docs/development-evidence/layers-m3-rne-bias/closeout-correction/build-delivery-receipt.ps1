$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-bias-record-correction'
$originalCommit = 'd735b1bfcea2a60668421c197e96a56947c111ec'
$generator = 'docs/development-evidence/layers-m3-rne-bias/build-delivery-receipt.ps1'
$evidence = 'docs/development-evidence/layers-m3-rne-bias/closeout-correction'
$oldReceipt = 'docs/development-evidence/changes/2026-10-19-layers-m3-rne-bias.json'
$receiptPath = "docs/development-evidence/changes/$task.json"

& git diff --quiet -- $oldReceipt
if ($LASTEXITCODE -ne 0) { throw '旧回执发生工作树修改' }
$verification = Get-Content -LiteralPath "$evidence/verification.json" -Raw | ConvertFrom-Json
if ($verification.result -cne 'pass' -or $verification.authoritative_clippy_value -ne 101) {
    throw '权威勘误核验没有通过'
}

& git add -- $generator $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0 -or $staged.Count -eq 0) { throw '暂存区读取失败或为空' }
foreach ($path in $staged) {
    if ($path -ne $generator -and -not $path.StartsWith("$evidence/", [StringComparison]::Ordinal)) {
        throw "暂存范围外文件：$path"
    }
}
if ($oldReceipt -in $staged) { throw '旧回执不得进入纠正提交' }
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})
$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $task
    milestones = [ordered]@{
        implementation = 'complete'
        validation = 'passed'
        delivery_record = 'pending_commit'
    }
    files = $files
    validation = @(
        [ordered]@{
            command = 'pwsh -NoProfile -File docs/development-evidence/layers-m3-rne-bias/closeout-correction/check-evidence.ps1'
            result = 'pass: 31 archived originals, 6 closeout runs, 2 plan generations'
            evidence = "$evidence/verification.json"
        },
        [ordered]@{
            command = 'scripts/check-normify-delivery.ps1 -Mode Preflight -TaskId 2026-10-19-layers-m3-rne-bias-record-correction'
            result = 'pass'
            evidence = "$evidence/correction.json"
        }
    )
    correction = [ordered]@{
        original_task = '2026-10-19-layers-m3-rne-bias'
        original_commit = $originalCommit
        original_receipt = $oldReceipt
        original_receipt_preserved = $true
        authoritative_erratum = "$evidence/correction.json"
        corrected_field = 'preserved_failures[gate=clippy].exit_code'
        old_value = 1
        authoritative_value = 101
    }
    remaining = @(
        'the original receipt keeps the historical mistaken value for auditability',
        'the old source-map applies only to the frozen execution-plan snapshot',
        '59 actual CUDA entry kinds remain unmigrated',
        '74 product low-level call sites remain unmigrated',
        'historical flg_acc=false still has 1042 failed blocks',
        'NP01 through NP04 remain unaccepted',
        'performance remains unaccepted',
        'release and Linux were not run',
        'parent M3 remains incomplete'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{
    schema = 1
    task = $task
    covered_files = $files.Count
    receipt = $receiptPath
    original_receipt_preserved = $true
    result = 'staged'
} | ConvertTo-Json
