$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-backward'
$root = 'docs/development-evidence/layers-m3-rne-backward'
$runtime = "target/normify-evidence/$task"
$base = '0f54bcf8c36feeb00032a923925433c8c1dfd9ee'

function Assert-Run([string]$Id) {
    $registrationPath = "$runtime/registrations/$Id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $output = [string]$registration.output_root
    $result = Get-Content -LiteralPath "$output/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$output/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        -not (Test-Path -LiteralPath $result.stdout) -or
        -not (Test-Path -LiteralPath $result.stderr) -or
        $result.exit_code -ne 0) {
        throw "运行证据错误：$Id"
    }
}

$successfulRuns = @(
    'format-apply',
    'format',
    'host-wrapper',
    'format-after-gpu-code-apply',
    'format-after-gpu-code',
    'gpu-selector-list',
    'clippy-all-targets',
    'device-preflight',
    'gpu-wrapper-captured',
    'inventory',
    'inventory-v2',
    'capture-evidence',
    'precision-map',
    'precision-map-v2',
    'evidence-retry-01'
)
foreach ($id in $successfulRuns) { Assert-Run $id }

$selector = Get-Content -LiteralPath "$runtime/check-04-selector/stdout.log" -Raw
$gpu = Get-Content -LiteralPath "$runtime/run-01-product/stdout.log" -Raw
if ($selector -notmatch '(?m)^1 test, 0 benchmarks$' -or
    $gpu -notmatch '(?m)^running 1 test$' -or
    $gpu -notmatch 'runs_rne_backward_wrapper_in_reverse_levels_and_recovers \.\.\. ok' -or
    $gpu -notmatch '1 passed; 0 failed') {
    throw '设备测试选择或结果不符'
}

$sourcePath = 'src/physics/rne.rs'
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath)).Replace("`r`n", "`n")
$testStart = [regex]::Match($source, '(?m)^#\[cfg\(test\)\]\s*\nmod tests')
if (-not $testStart.Success) { throw '找不到测试模块边界' }
$product = $source.Substring(0, $testStart.Index)
foreach ($token in @(
    'RNE_BACKWARD_METADATA_KIND',
    'RNE_BACKWARD_SLOTS',
    'RneBackwardDims',
    'RneBackwardSpec',
    'RneBackwardMetadata',
    'RneBackwardKernel',
    'backward_proof',
    'RneBackwardRequest'
)) {
    if (-not $product.Contains($token)) { throw "缺少反向包装证据：$token" }
}
if ($product.Contains('self.kernels[0]') -or
    [regex]::Matches($product, 'self\.backward_kernel\.launch\s*\(').Count -ne 1 -or
    [regex]::Matches($product, 'SynchronousKernel::compile\(session, RNE_CUDA, "rne_backward"\)').Count -ne 1) {
    throw '反向包装调用结构不符'
}
$preflightIndex = $product.IndexOf('let mut backward_launches = Vec::with_capacity(self.depth);', [StringComparison]::Ordinal)
$caccIndex = $product.IndexOf('self.cacc_kernel.launch(', [StringComparison]::Ordinal)
if ($preflightIndex -lt 0 -or $caccIndex -lt 0 -or $preflightIndex -ge $caccIndex -or
    -not $product.Contains('for level in (1..=self.depth).rev()')) {
    throw '反向预检或层级顺序不符'
}

& git diff --quiet $base -- src/physics/rne.cu
if ($LASTEXITCODE -ne 0) { throw 'RNE CUDA源码发生变化' }
$diff = @(& git diff -U0 $base -- src/physics/rne.rs)
if ($diff -match '(?m)^\+\s*#\[allow\(') { throw '新增Clippy豁免' }

$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory-v2.json" -Raw | ConvertFrom-Json
if ($inventory.inventory_version -ne 2 -or
    $inventory.supersedes -cne "$root/unmigrated-launch-inventory-v1.json" -or
    $inventory.base_commit -cne $base -or
    $inventory.counts.compile_call_sites -ne 90 -or
    $inventory.counts.actual_cuda_entry_kinds -ne 88 -or
    $inventory.counts.kernel_abi_families -ne 26 -or
    $inventory.counts.typed_actual_entry_kinds -ne 30 -or
    $inventory.counts.unmigrated_actual_entry_kinds -ne 58 -or
    $inventory.counts.product_low_level_call_sites -ne 99 -or
    $inventory.counts.typed_low_level_call_sites -ne 26 -or
    $inventory.counts.unmigrated_low_level_call_sites -ne 73 -or
    $inventory.counts.wrapper_orchestration_call_sites -ne 28 -or
    $inventory.next_candidate.entry -ceq 'rne_backward') {
    throw '盘点结果错误'
}
$backwardEntry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rne_backward')
if ($backwardEntry.Count -ne 1 -or $backwardEntry[0].migration_status -cne 'typed_safe_wrapper') {
    throw '反向入口没有完成重分类'
}
$biasCompile = @($inventory.compile_sites | Where-Object entry -CEQ 'rne_bias')
$biasEntry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rne_bias')
$biasLow = @($inventory.product_low_level_call_sites | Where-Object entry -CEQ 'rne_bias')
$biasOuter = @($inventory.wrapper_orchestration_call_sites | Where-Object entry -CEQ 'rne_bias')
$biasFamily = @($inventory.kernel_abi_families | Where-Object logical_name -CEQ 'rne_bias')
if ($biasCompile.Count -ne 1 -or $biasCompile[0].line -ne 522 -or
    $biasEntry.Count -ne 1 -or $biasEntry[0].compile_sites[0] -cne 'src/physics/rne_bias.rs:522' -or
    $biasLow.Count -ne 1 -or $biasLow[0].line -ne 588 -or
    $biasOuter.Count -ne 1 -or $biasOuter[0].line -ne 725 -or
    $biasFamily.Count -ne 1 -or $biasFamily[0].typed_low_level_calls[0] -cne 'src/physics/rne_bias.rs:588') {
    throw '偏置力覆盖层没有进入权威盘点'
}

$capture = Get-Content -LiteralPath "$root/capture-inventory.json" -Raw | ConvertFrom-Json
if ($capture.scopes.main.count -ne 27 -or
    $capture.scopes.foreign_session.count -ne 3 -or
    $capture.scopes.root_only.count -ne 27 -or
    $capture.ptx.byte_equal -ne $true -or
    $capture.ptx.current.sha256 -cne 'a5b528ff34301f8e358d13a53b5dfbd69676fcc02987d39a3148d9693b8ebd22' -or
    $capture.device_result.task_memory -cne 'unknown') {
    throw '捕获证据错误'
}

$precision = Get-Content -LiteralPath "$root/precision-mapping-v2.json" -Raw | ConvertFrom-Json
if ($precision.inventory.version -ne 2 -or
    $precision.source_identity.cuda_unchanged -ne $true -or
    $precision.orchestration.all_levels_preflight_before_cacc -ne $true -or
    $precision.orchestration.root_only_backward_launches -ne 0 -or
    $precision.numerical.atomic_add_unchanged -ne $true -or
    $precision.numerical.exact_integer_bit_comparison -ne $true -or
    $precision.numerical.general_sibling_atomic_determinism_claimed -ne $false -or
    $precision.numerical.existing_general_tolerance_revalidated -ne $false -or
    $precision.validation.worlds -ne 43 -or
    $precision.validation.work_items -ne 258 -or
    $precision.validation.task_gpu_memory -cne 'unknown' -or
    $precision.historical_precision.failed_blocks -ne 1042 -or
    @($precision.acceptance.PSObject.Properties | Where-Object Value -ne $false).Count -ne 0) {
    throw '精度映射或边界错误'
}

$plan = Get-Content -LiteralPath "$root/execution-plan.json" -Raw | ConvertFrom-Json
$errata = Get-Content -LiteralPath "$root/registration-command-errata.json" -Raw | ConvertFrom-Json
$correction = Get-Content -LiteralPath "$root/inventory-correction.json" -Raw | ConvertFrom-Json
$inventoryOriginalEqual = (Get-FileHash -LiteralPath "$root/unmigrated-launch-inventory.json" -Algorithm SHA256).Hash -ceq
    (Get-FileHash -LiteralPath "$root/unmigrated-launch-inventory-v1.json" -Algorithm SHA256).Hash
$precisionOriginalEqual = (Get-FileHash -LiteralPath "$root/precision-mapping.json" -Algorithm SHA256).Hash -ceq
    (Get-FileHash -LiteralPath "$root/precision-mapping-v1.json" -Algorithm SHA256).Hash
$commands = @($plan.registered_validations | ForEach-Object { "$($_.executable) $($_.arguments -join ' ')" }) -join "`n"
if ($plan.resource_lock -cne 'gpu:windows-t4' -or
    $plan.device_identity -cne 'NVIDIA GeForce RTX 4070 Ti SUPER' -or
    $errata.generator_fixed_for_future -ne $true -or
    $errata.rerun_required -ne $false -or
    $correction.product_rerun_required -ne $false -or
    $correction.original_paths_restored_byte_equal -ne $true -or
    -not $inventoryOriginalEqual -or
    -not $precisionOriginalEqual -or
    $commands -match '(?i)(^|\s)python(\.exe)?(\s|$)') {
    throw '执行计划边界错误'
}

[ordered]@{
    schema=1
    task_id=$task
    successful_runs=$successfulRuns.Count
    capture_files=57
    ptx_sha256=$capture.ptx.current.sha256
    next_candidate=$inventory.next_candidate.entry
    task_gpu_memory='unknown'
    result='pass'
} | ConvertTo-Json
