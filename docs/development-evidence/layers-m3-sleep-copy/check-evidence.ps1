$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-sleep-copy'
$root = "docs/development-evidence/layers-m3-sleep-copy"
$runtime = "target/normify-evidence/$task"
$base = '15e31190182dd7c58312424851d477d9495b24d6'
$sourcePath = 'src/physics/resident.rs'

. 'docs/development-evidence/layers-m3-inventory-derived-correction/inventory-lib.ps1'
$claims = Get-Content -LiteralPath "$root/claims.json" -Raw | ConvertFrom-Json -AsHashtable
$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json -AsHashtable
$precisionV1 = Get-Content -LiteralPath "$root/precision-revalidation.json" -Raw | ConvertFrom-Json
$precision = Get-Content -LiteralPath "$root/precision-revalidation-v2.json" -Raw | ConvertFrom-Json
$derived = Assert-InventoryConsistent -Inventory $inventory -Claims $claims
if ($derived.counts.typed_actual_entry_kinds -ne 25 -or
    $derived.counts.unmigrated_actual_entry_kinds -ne 63 -or
    $derived.counts.typed_low_level_call_sites -ne 21 -or
    $derived.counts.unmigrated_low_level_call_sites -ne 78 -or
    $derived.counts.wrapper_orchestration_call_sites -ne 23) {
    throw '盘点派生计数错误'
}
$sleepEntry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -ceq 'sleep_copy')
$sleepLow = @($inventory.product_low_level_call_sites | Where-Object entry -ceq 'sleep_copy')
$sleepOuter = @($inventory.wrapper_orchestration_call_sites | Where-Object receiver -ceq 'device.copy_kernel')
if ($sleepEntry.Count -ne 1 -or $sleepEntry[0].migration_status -cne 'typed_safe_wrapper' -or
    $sleepLow.Count -ne 1 -or $sleepLow[0].classification -cne 'typed_wrapper_low_level' -or
    $sleepOuter.Count -ne 1) {
    throw 'sleep_copy分类错误'
}

& git diff --quiet $base -- 'src/physics/sleep.rs'
if ($LASTEXITCODE -ne 0) { throw 'CUDA内核源码发生变化' }
foreach ($path in @(
    'docs/evidence/passive-force-sleep/gpu-1/ptx/sleep_copy-0f0740d3df05b413ea24faf778f76e32e651183ade60c45b2dc324f97e50d2ff.cu',
    'docs/evidence/passive-force-sleep/gpu-1/ptx/sleep_copy-0f0740d3df05b413ea24faf778f76e32e651183ade60c45b2dc324f97e50d2ff.ptx',
    'docs/evidence/passive-force-sleep/gpu-1/ptx/sleep_copy-0f0740d3df05b413ea24faf778f76e32e651183ade60c45b2dc324f97e50d2ff.json',
    'docs/evidence/t4-g03-retest/tendon-wake-internal/map.json'
)) {
    & git diff --quiet $base -- $path
    if ($LASTEXITCODE -ne 0) { throw "冻结证据发生变化：$path" }
}
if ($precisionV1.prior_map.groups.Count -ne 18 -or $precisionV1.prior_map.outputs.Count -ne 5 -or
    $precision.prior_map.groups.Count -ne 18 -or $precision.prior_map.outputs.Count -ne 5 -or
    @($precision.prior_map.groups | Where-Object product_checked -ne $true).Count -ne 0 -or
    @($precision.prior_map.outputs | Where-Object checked -ne $true).Count -ne 0 -or
    $precision.generated_product.checks.Count -ne 2 -or
    $precision.resident_identity.base_blob -notmatch '^[0-9a-f]{40}$' -or
    $precision.resident_identity.base_lf_sha256 -cne '6389af92c39b687cdf3cd01fa3855d651bf264d6df7e2ff52fd8f230950718eb' -or
    $precision.resident_identity.old_identity_gate_passed -ne $false -or
    $precision.boundaries.local_slice_evidence -cne 'pass' -or
    $precision.boundaries.np01_complete -ne $false -or
    $precision.boundaries.np02_complete -ne $false -or
    $precision.boundaries.np03_complete -ne $false -or
    $precision.boundaries.np04_complete -ne $false) {
    throw '精度复核边界错误'
}
foreach ($path in @($precision.resident_identity.base_evidence, $precision.resident_identity.current_evidence)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "缺少源码原件：$path" }
}
if ((Get-FileHash -LiteralPath $precision.resident_identity.base_evidence -Algorithm SHA256).Hash.ToLowerInvariant() -cne
        $precision.resident_identity.base_raw_sha256 -or
    (Get-FileHash -LiteralPath $precision.resident_identity.current_evidence -Algorithm SHA256).Hash.ToLowerInvariant() -cne
        $precision.resident_identity.current_raw_sha256) {
    throw '源码原件摘要漂移'
}

$expectedRuns = [ordered]@{
    'host-wrapper' = 0
    'device-preflight' = 0
    'format' = 1
    'format-apply' = 0
    'format-retry-01' = 0
    'clippy-lib' = 0
    'precision-static' = 0
    'precision-static-retry-01' = 101
    'gpu-product-captured' = 0
    'gpu-zero-tree-captured' = 0
    'inventory' = 0
    'precision-map' = 0
    'precision-map-v2' = 0
}
foreach ($id in $expectedRuns.Keys) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $result = Get-Content -LiteralPath "$($registration.output_root)/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$($registration.output_root)/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or $started.stderr_precreated -ne $true -or
        $result.exit_code -ne $expectedRuns[$id]) {
        throw "运行记录错误：$id"
    }
}
$unused = @('gpu-product', 'gpu-zero-tree')
foreach ($id in $unused) {
    $registration = Get-Content -LiteralPath "$runtime/registrations/$id.json" -Raw | ConvertFrom-Json
    if (Test-Path -LiteralPath $registration.output_root) { throw "被取代运行已执行：$id" }
}
$device = Get-Content -LiteralPath "$runtime/check-02-device/stdout.log" -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or
    $device.nvrtc_version -cne '12.8' -or $device.nvrtc_loaded -ne $true -or
    $device.nvrtc_builtins_loaded -ne $true) {
    throw '显卡或NVRTC预检错误'
}
$report = Get-Content -LiteralPath "$runtime/run-01-product-captured/report.json" -Raw | ConvertFrom-Json
foreach ($name in @(
    'valid_rows_exact', 'guards_preserved', 'real_resident_call',
    'stale_real_buffer_write_rejected_without_output_write',
    'foreign_session_rejected_without_output_write', 'same_plan_recovery'
)) {
    if ($report.$name -ne $true) { throw "GPU报告缺少通过项：$name" }
}
if ($report.gpu -notmatch 'RTX 4070 Ti SUPER' -or
    $report.np01_to_np04_accepted -ne $false -or $report.performance_accepted -ne $false) {
    throw 'GPU报告边界错误'
}

$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$copy = $source.IndexOf('device.copy_kernel.launch(', [StringComparison]::Ordinal)
$wake = $source.IndexOf('device.wake_kernel.launch(', [StringComparison]::Ordinal)
$swap = $source.IndexOf('std::mem::swap(&mut out.state, &mut out.scratch);', [StringComparison]::Ordinal)
if ($copy -lt 0 -or $copy -ge $wake -or $wake -ge $swap) { throw '事务顺序错误' }
$testStart = $source.IndexOf('fn gpu_m3_sleep_copy_uses_real_resident_buffers_and_recovers()', [StringComparison]::Ordinal)
if ($testStart -lt 0) { throw '找不到GPU恢复测试' }
$testBody = $source.Substring($testStart)
if ([regex]::Matches($testBody, 'assert!\(!data\.ready\.sleep\);').Count -lt 2 -or
    [regex]::Matches($testBody, 'stage:\s*"tendon_wake"').Count -lt 2) {
    throw '失败状态或读回拒绝断言缺失'
}

$status = @(& git status --porcelain=v1 --untracked-files=all)
foreach ($line in $status) {
    $path = $line.Substring(3).Replace('\', '/')
    if ($path -ne $sourcePath -and -not $path.StartsWith("$root/", [StringComparison]::Ordinal)) {
        throw "范围外工作区文件：$path"
    }
}
& git diff --check
if ($LASTEXITCODE -ne 0) { throw "git diff --check退出码：$LASTEXITCODE" }
[ordered]@{
    schema = 1
    task = $task
    device = $device.device
    inherited_groups = 18
    inherited_outputs = 5
    capture_matches_frozen_bytes = $true
    old_identity_gate_passed = $false
    local_slice_evidence = 'pass'
    typed_actual_entry_kinds = $derived.counts.typed_actual_entry_kinds
    unmigrated_actual_entry_kinds = $derived.counts.unmigrated_actual_entry_kinds
    np01_to_np04_accepted = $false
    performance_accepted = $false
} | ConvertTo-Json
