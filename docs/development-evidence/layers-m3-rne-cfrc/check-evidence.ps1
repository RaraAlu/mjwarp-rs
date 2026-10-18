$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-cfrc'
$root = "docs/development-evidence/layers-m3-rne-cfrc"
$runtime = "target/normify-evidence/$task"
$base = '26592d49a386b52a32170360be18880d3a449f4b'
$sourcePath = 'src/physics/rne.rs'

. 'docs/development-evidence/layers-m3-inventory-derived-correction/inventory-lib.ps1'
$claims = Get-Content -LiteralPath "$root/claims.json" -Raw | ConvertFrom-Json -AsHashtable
$initialInventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json -AsHashtable
$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory-v2.json" -Raw | ConvertFrom-Json -AsHashtable
$precision = Get-Content -LiteralPath "$root/precision-revalidation.json" -Raw | ConvertFrom-Json
$derived = Assert-InventoryConsistent -Inventory $inventory -Claims $claims
if ($derived.counts.typed_actual_entry_kinds -ne 27 -or
    $derived.counts.unmigrated_actual_entry_kinds -ne 61 -or
    $derived.counts.typed_low_level_call_sites -ne 23 -or
    $derived.counts.unmigrated_low_level_call_sites -ne 76 -or
    $derived.counts.wrapper_orchestration_call_sites -ne 25 -or
    $derived.counts.kernel_abi_families -ne 23 -or
    $derived.next_candidate.entry -cne 'rne_cacc') {
    throw '盘点派生计数错误'
}
$entry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -ceq 'rne_cfrc')
$low = @($inventory.product_low_level_call_sites | Where-Object entry -ceq 'rne_cfrc')
$outer = @($inventory.wrapper_orchestration_call_sites | Where-Object entry -ceq 'rne_cfrc')
$family = @($inventory.kernel_abi_families | Where-Object type -ceq 'RneCfrcSpec')
$qfrcFamily = @($inventory.kernel_abi_families | Where-Object type -ceq 'RneQfrcSpec')
if ($entry.Count -ne 1 -or $entry[0].migration_status -cne 'typed_safe_wrapper' -or
    $low.Count -ne 1 -or $low[0].classification -cne 'typed_wrapper_low_level' -or
    $outer.Count -ne 1 -or $family.Count -ne 1 -or $qfrcFamily.Count -ne 1) {
    throw 'rne_cfrc分类错误'
}
$qfrcLow = @($inventory.product_low_level_call_sites | Where-Object entry -ceq 'rne_qfrc')
if ($qfrcLow.Count -ne 1 -or
    @($qfrcFamily[0].typed_low_level_calls).Count -ne 1 -or
    $qfrcFamily[0].typed_low_level_calls[0] -cne "$($qfrcLow[0].file):$($qfrcLow[0].line)") {
    throw 'rne_qfrc继承位置错误'
}
$initialQfrcFamily = @($initialInventory.kernel_abi_families | Where-Object type -ceq 'RneQfrcSpec')
$initialQfrcLow = @($initialInventory.product_low_level_call_sites | Where-Object entry -ceq 'rne_qfrc')
if ($initialQfrcFamily.Count -ne 1 -or $initialQfrcLow.Count -ne 1 -or
    $initialQfrcFamily[0].typed_low_level_calls[0] -ceq "$($initialQfrcLow[0].file):$($initialQfrcLow[0].line)") {
    throw '初版盘点漂移没有保留'
}

foreach ($path in @('src/physics/rne.cu', 'src/runtime/transfer/kernel.rs', 'docs/evidence/rne-acceleration/map.json')) {
    & git diff --quiet $base -- $path
    if ($LASTEXITCODE -ne 0) { throw "冻结文件发生变化：$path" }
}
if ($precision.generated_product.artifacts.Count -ne 4 -or
    $precision.generated_product.historical_and_all_current_captures_exact -ne $true -or
    $precision.generated_product.receipt_contract_exact -ne $true -or
    $precision.frozen_product.rne_cuda_unchanged -ne $true -or
    $precision.frozen_product.transfer_kernel_unchanged -ne $true -or
    $precision.validations.corrected_input_tolerance_gate_one_test -ne $true -or
    $precision.validations.raw_vs_typed_gpu_one_test -ne $true -or
    $precision.validations.metadata_and_session_prewrite_rejection -ne $true -or
    $precision.validations.recovery_on_same_plan -ne $true -or
    $precision.validations.zero_dof_product_path -ne $true -or
    $precision.validations.worlds -ne 513 -or
    $precision.historical_precision.flg_acc_false_failed_blocks -ne 1042 -or
    $precision.historical_precision.repaired_by_this_slice -ne $false -or
    $precision.historical_precision.full_operator_compiler_equivalence -ne $false -or
    @($precision.boundaries.PSObject.Properties | Where-Object Value -ne $false).Count -ne 0) {
    throw '精度复核边界错误'
}
foreach ($path in @(
    $precision.source_identity.base_evidence,
    $precision.source_identity.current_evidence,
    $precision.frozen_product.rne_cuda_evidence
)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "缺少冻结原件：$path" }
}
if ((Get-FileHash -LiteralPath $precision.source_identity.base_evidence -Algorithm SHA256).Hash.ToLowerInvariant() -cne
        $precision.source_identity.base_raw_sha256 -or
    (Get-FileHash -LiteralPath $precision.source_identity.current_evidence -Algorithm SHA256).Hash.ToLowerInvariant() -cne
        $precision.source_identity.current_raw_sha256 -or
    (Get-FileHash -LiteralPath $precision.frozen_product.rne_cuda_evidence -Algorithm SHA256).Hash.ToLowerInvariant() -cne
        $precision.frozen_product.rne_cuda_sha256) {
    throw '冻结原件摘要漂移'
}

$expectedRuns = [ordered]@{
    'host-wrapper'=0
    'format'=1
    'format-apply'=0
    'format-retry-01'=0
    'default-lib'=0
    'clippy-all-targets'=0
    'input-tolerance-gate'=0
    'device-preflight'=0
    'gpu-product-captured'=0
    'inventory'=0
    'inventory-retry-01'=0
    'precision-map'=0
}
foreach ($id in $expectedRuns.Keys) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $result = Get-Content -LiteralPath "$($registration.output_root)/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$($registration.output_root)/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        $result.exit_code -ne $expectedRuns[$id]) {
        throw "运行记录错误：$id"
    }
}
$device = Get-Content -LiteralPath "$runtime/check-06-device/stdout.log" -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or
    $device.nvrtc_version -cne '12.8' -or
    $device.nvrtc_loaded -ne $true -or
    $device.nvrtc_builtins_loaded -ne $true) {
    throw '显卡或NVRTC预检错误'
}

$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$anchors = @(
    'struct RneCfrcSpec;',
    'access: Access::ReadWrite,',
    'struct RneMetadataBuffer {',
    'struct RneCfrcKernel {',
    'fn runs_rne_cfrc_wrapper_and_recovers_after_real_metadata_write()',
    'fn rne_cfrc_plan_checks_metadata_spans_aliases_and_overflow()',
    'fn rne_cfrc_plan_accepts_zero_dof_boundary()'
)
foreach ($anchor in $anchors) {
    if (-not $source.Contains($anchor)) { throw "源码锚点缺失：$anchor" }
}
$update = $source.Split('fn update_rne(', 2)[1].Split('fn check_values(', 2)[0]
$cfrcPreflight = $update.IndexOf('self.cfrc_kernel.preflight(', [StringComparison]::Ordinal)
$qfrcPreflight = $update.IndexOf('self.qfrc_kernel.preflight(', [StringComparison]::Ordinal)
$cacc = $update.IndexOf('self.kernels[0].launch_rne_cacc(', [StringComparison]::Ordinal)
$cfrc = $update.IndexOf('self.cfrc_kernel.launch(', [StringComparison]::Ordinal)
$backward = $update.IndexOf('self.kernels[1].launch(', [StringComparison]::Ordinal)
$qfrc = $update.IndexOf('self.qfrc_kernel.launch(', [StringComparison]::Ordinal)
if ($cfrcPreflight -lt 0 -or $qfrcPreflight -lt 0 -or $cacc -lt 0 -or
    $cfrc -lt 0 -or $backward -lt 0 -or $qfrc -lt 0 -or
    $cfrcPreflight -ge $qfrcPreflight -or $qfrcPreflight -ge $cacc -or
    $cacc -ge $cfrc -or $cfrc -ge $backward -or $backward -ge $qfrc) {
    throw 'RNE预检或发射顺序错误'
}
if ([regex]::Matches($source, 'RneCfrcKernel::compile\(').Count -ne 2 -or
    [regex]::Matches($source, 'self\.kernels\[[01]\]').Count -lt 2 -or
    $source.Contains('self.kernels[2]')) {
    throw 'RNE迁移范围错误'
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
    schema=1
    task=$task
    device=$device.device
    source_ptx_unchanged=$true
    input_tolerance_gate_one_test=$true
    initial_inventory_drift_preserved=$true
    typed_actual_entry_kinds=$derived.counts.typed_actual_entry_kinds
    unmigrated_actual_entry_kinds=$derived.counts.unmigrated_actual_entry_kinds
    next_candidate=$derived.next_candidate.entry
    historical_failed_blocks=1042
    np01_to_np04_accepted=$false
    performance_accepted=$false
} | ConvertTo-Json
