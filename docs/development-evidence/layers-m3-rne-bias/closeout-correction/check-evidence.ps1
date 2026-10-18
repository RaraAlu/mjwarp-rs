$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-rne-bias/closeout-correction'
$archive = "$root/archive"
$verificationPath = "$root/verification.json"
$generatorPath = 'docs/development-evidence/layers-m3-rne-bias/build-delivery-receipt.ps1'
$oldReceiptPath = 'docs/development-evidence/changes/2026-10-19-layers-m3-rne-bias.json'
$clippyResultPath = 'docs/development-evidence/layers-m3-rne-bias/formal-run/runs/clippy-first.run.json'
$sourceMapPath = 'docs/development-evidence/layers-m3-rne-bias/source-map.json'

if (Test-Path -LiteralPath $verificationPath) { throw '核验结果已存在' }
$facts = Get-Content -LiteralPath "$root/correction.json" -Raw | ConvertFrom-Json
$manifest = Get-Content -LiteralPath "$archive/manifest.json" -Raw | ConvertFrom-Json
if ($facts.original_commit -cne 'd735b1bfcea2a60668421c197e96a56947c111ec') { throw '原提交绑定错误' }
if ($manifest.member_count -ne 31 -or @($manifest.members).Count -ne 31) { throw '归档成员数错误' }

$archiveRoot = (Resolve-Path -LiteralPath $archive).Path
foreach ($member in $manifest.members) {
    $path = Join-Path $archiveRoot ([string]$member.path)
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "缺少归档成员：$($member.path)" }
    $item = Get-Item -LiteralPath $path
    if ($item.Length -ne $member.bytes) { throw "归档长度漂移：$($member.path)" }
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -cne $member.sha256) { throw "归档摘要漂移：$($member.path)" }
}

$oldReceipt = Get-Content -LiteralPath $oldReceiptPath -Raw | ConvertFrom-Json
$archivedReceipt = Get-Content -LiteralPath "$archive/original-receipt.json" -Raw | ConvertFrom-Json
$oldClippy = @($oldReceipt.preserved_failures | Where-Object gate -CEQ 'clippy')
$archivedClippy = @($archivedReceipt.preserved_failures | Where-Object gate -CEQ 'clippy')
if ($oldClippy.Count -ne 1 -or $oldClippy[0].exit_code -ne 1) { throw '旧回执字节未保留错误值' }
if ($archivedClippy.Count -ne 1 -or $archivedClippy[0].exit_code -ne 1) { throw '旧回执归档未保留错误值' }
$oldHash = (Get-FileHash -LiteralPath $oldReceiptPath -Algorithm SHA256).Hash.ToLowerInvariant()
$archivedHash = (Get-FileHash -LiteralPath "$archive/original-receipt.json" -Algorithm SHA256).Hash.ToLowerInvariant()
if ($oldHash -cne $archivedHash -or $oldHash -cne $facts.original_receipt.sha256) { throw '旧回执归档不一致' }

$clippy = Get-Content -LiteralPath $clippyResultPath -Raw | ConvertFrom-Json
if ($clippy.exit_code -ne 101 -or $facts.corrected_field.authoritative_value -ne 101) { throw '权威Clippy退出码错误' }
$generator = Get-Content -LiteralPath $generatorPath -Raw
if ($generator -notmatch "gate='clippy';exit_code=101;") { throw '未来生成脚本没有修正为101' }
if ($generator -match "gate='clippy';exit_code=1;") { throw '未来生成脚本仍含旧错误值' }

$frozenPlanHash = (Get-FileHash -LiteralPath $facts.plans.frozen.path -Algorithm SHA256).Hash.ToLowerInvariant()
$closeoutPlanHash = (Get-FileHash -LiteralPath $facts.plans.closeout.path -Algorithm SHA256).Hash.ToLowerInvariant()
if ($frozenPlanHash -cne $facts.plans.frozen.sha256) { throw '冻结计划摘要漂移' }
if ($closeoutPlanHash -cne $facts.plans.closeout.sha256) { throw '收尾计划摘要漂移' }
if ($frozenPlanHash -ceq $closeoutPlanHash) { throw '两代计划不应相同' }
$frozenPlan = Get-Content -LiteralPath $facts.plans.frozen.path -Raw | ConvertFrom-Json
$closeoutPlan = Get-Content -LiteralPath $facts.plans.closeout.path -Raw | ConvertFrom-Json
$appended = @($facts.plans.closeout.appended_commands)
foreach ($id in $appended) {
    if (@($frozenPlan.registered_validations | Where-Object id -CEQ $id).Count -ne 0) { throw "冻结计划意外包含：$id" }
    if (@($closeoutPlan.registered_validations | Where-Object id -CEQ $id).Count -ne 1) { throw "收尾计划缺少：$id" }
}
$sourceMap = Get-Content -LiteralPath $sourceMapPath -Raw | ConvertFrom-Json
$mapping = @($sourceMap.mappings | Where-Object path -CEQ 'records/execution-plan.json')
if ($mapping.Count -ne 1 -or $mapping[0].source -cne 'docs/development-evidence/layers-m3-rne-bias/execution-plan.json') {
    throw '旧source-map计划映射不符'
}
if ($facts.plans.policy -notmatch 'frozen snapshot') { throw '计划代际范围说明缺失' }

$runs = @('delivery-stage', 'staged-whitespace', 'delivery-staged', 'delivery-postcommit', 'delivery-recorded', 'delivery-final')
foreach ($id in $runs) {
    foreach ($name in @('registration.json', 'started.json', 'result.json', 'stdout.log', 'stderr.log')) {
        if (-not (Test-Path -LiteralPath "$archive/$id/$name" -PathType Leaf)) { throw "收尾原件缺失：$id/$name" }
    }
    $result = Get-Content -LiteralPath "$archive/$id/result.json" -Raw | ConvertFrom-Json
    if ($result.id -cne $id -or $result.exit_code -ne 0) { throw "收尾结果无效：$id" }
}

$verification = [ordered]@{
    schema = 1
    original_commit = $facts.original_commit
    original_receipt_preserved = $true
    original_receipt_clippy_value = 1
    authoritative_clippy_value = 101
    archive_member_count = $manifest.member_count
    closeout_run_count = $runs.Count
    frozen_plan_sha256 = $frozenPlanHash
    closeout_plan_sha256 = $closeoutPlanHash
    frozen_manifest_preserved = $true
    source_map_scope = 'frozen snapshot only'
    gpu_rerun = $false
    product_tests_rerun = $false
    result = 'pass'
}
$json = ($verification | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $verificationPath), $json, [Text.UTF8Encoding]::new($false))
$verification | ConvertTo-Json -Depth 6
