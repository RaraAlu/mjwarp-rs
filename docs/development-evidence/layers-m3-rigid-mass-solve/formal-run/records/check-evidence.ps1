$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rigid-mass-solve'
$root = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$runtime = "target/normify-evidence/$task"
$base = 'a2d057618d2884234e9b6779e619cce5abfc2adc'
$sourcePath = 'src/physics/mass_solve.rs'

function Assert-Run([string]$Id, [int]$ExitCode = 0) {
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
        -not (Test-Path -LiteralPath $result.stdout -PathType Leaf) -or
        -not (Test-Path -LiteralPath $result.stderr -PathType Leaf) -or
        $result.exit_code -ne $ExitCode) {
        throw "运行证据错误：$Id"
    }
}

$successfulRuns = @(
    'format-apply-retry-02',
    'freeze-source-v2-before-validation',
    'host-wrapper-current',
    'format-retry-03',
    'clippy-all-targets-retry-03',
    'device-preflight',
    'gpu-wrapper-captured-retry-01',
    'capture-evidence-v2',
    'inventory-v5',
    'precision-map-v3'
)
foreach ($id in $successfulRuns) { Assert-Run $id }
Assert-Run 'host-wrapper' 101
Assert-Run 'host-wrapper-retry-01' 101
Assert-Run 'clippy-all-targets' 101
Assert-Run 'host-layout-boundary' 0

$hostLog = Get-Content -LiteralPath "$runtime/check-03-host-current/stdout.log" -Raw
$gpu = Get-Content -LiteralPath "$runtime/run-02-product/stdout.log" -Raw
$zero = Get-Content -LiteralPath "$runtime/check-03-host-layout-boundary/stdout.log" -Raw
if ($hostLog -notmatch '(?m)^running 2 tests$' -or
    $hostLog -notmatch '2 passed; 0 failed' -or
    $gpu -notmatch '(?m)^running 1 test$' -or
    $gpu -notmatch 'runs_rigid_mass_solve_wrapper_paths_and_recovers \.\.\. ok' -or
    $gpu -notmatch '1 passed; 0 failed' -or
    $zero -notmatch '(?m)^running 0 tests$') {
    throw '宿主、设备或零选择结果不符'
}

$currentBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $sourcePath))
$snapshotBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath "$root/pre-validation-current-mass-solve-v2.rs"))
$currentHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($currentBytes)).ToLowerInvariant()
$snapshotHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($snapshotBytes)).ToLowerInvariant()
if ($currentBytes.Length -ne $snapshotBytes.Length -or $currentHash -cne $snapshotHash) {
    throw '当前源码与最终快照不符'
}
$currentBlob = @(& git hash-object --no-filters -- $sourcePath)[0].Trim()
$identity = Get-Content -LiteralPath "$root/pre-validation-source-identity-v2.json" -Raw | ConvertFrom-Json
if ($currentHash -cne '72ae216d01cdc2832c0970dbc85e78a407ac8c0bc9e7a6299db5ceec01412ecd' -or
    $identity.sha256 -cne $currentHash -or
    $identity.working_blob -cne $currentBlob -or
    $identity.product_prefix_equal_to_gpu_validation_source -ne $true -or
    $identity.full_file_equal_to_gpu_validation_source -ne $false) {
    throw '最终源码身份错误'
}

$source = [Text.Encoding]::UTF8.GetString($currentBytes).Replace("`r`n", "`n")
$gpuSource = [IO.File]::ReadAllText((Resolve-Path -LiteralPath "$root/pre-validation-current-mass-solve.rs")).Replace("`r`n", "`n")
$testStart = [regex]::Match($source, '(?m)^#\[cfg\(test\)\]\s*\nmod tests')
$gpuTestStart = [regex]::Match($gpuSource, '(?m)^#\[cfg\(test\)\]\s*\nmod tests')
if (-not $testStart.Success -or -not $gpuTestStart.Success -or
    $source.Substring(0, $testStart.Index) -cne $gpuSource.Substring(0, $gpuTestStart.Index) -or
    $source -ceq $gpuSource) {
    throw '设备验证源码边界错误'
}
foreach ($token in @(
    'RIGID_MASS_SOLVE_METADATA_KIND',
    'RIGID_MASS_SOLVE_SLOTS',
    'RigidMassSolveDims',
    'RigidMassSolveMetadata',
    'RigidMassSolveBaseMetadata',
    'RigidMassSolveSpec',
    'RigidMassSolveMetadataBuffer',
    'RigidMassSolveKernel'
)) {
    if (-not $source.Substring(0, $testStart.Index).Contains($token)) { throw "缺少包装证据：$token" }
}

$capture = Get-Content -LiteralPath "$root/capture-inventory-v2.json" -Raw | ConvertFrom-Json
$expectedDirs = @('foreign-session','product-invalid','product-nonfinite','product-recovery','typed-main','zero-dof')
foreach ($run in @($capture.first_run, $capture.authoritative_run)) {
    if ($run.count -ne 18 -or @($run.files).Count -ne 18) { throw '捕获文件数量错误' }
    foreach ($dir in $expectedDirs) {
        $files = @($run.files | Where-Object relative -Like "$dir/*")
        $extensions = @($files | ForEach-Object { [IO.Path]::GetExtension($_.relative) } | Sort-Object)
        if ($files.Count -ne 3 -or ($extensions -join ',') -cne '.cu,.json,.ptx') {
            throw "编译捕获不完整：$dir"
        }
    }
}
if ($capture.first_run_transfers_to_current_source -ne $false -or
    $capture.first_run.count -ne 18 -or
    $capture.authoritative_run.count -ne 18 -or
    $capture.authoritative_worlds -ne 257 -or
    $capture.product_source_equal_to_gpu_validation_source -ne $true -or
    $capture.full_rust_file_equal_to_gpu_validation_source -ne $false -or
    $capture.frozen_cuda_unchanged_from_base -ne $true -or
    $capture.ptx.first_and_second_byte_equal -ne $true -or
    $capture.ptx.sha256 -cne 'a45cdb75c4087a4e78ae6916b11fdb2c60d807330703f5eafd788da3d2c6e49c' -or
    $capture.cuda.sha256 -cne 'be4cb52b7ce12f117dfa1fafbd56d4d8a324312df6a82b72dbbe9949f73ec6aa' -or
    $capture.cuda.base_and_current_string_equal -ne $true -or
    $capture.device_result.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or
    $capture.device_result.resource_lock -cne 'gpu:windows-t4' -or
    $capture.device_result.task_memory -cne 'unknown') {
    throw '捕获证据错误'
}

$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory-v5.json" -Raw | ConvertFrom-Json
$massRows = @($inventory.product_low_level_call_sites | Where-Object file -CEQ $sourcePath)
if ($inventory.inventory_version -ne 5 -or
    $inventory.supersedes -cne "$root/unmigrated-launch-inventory-v4.json" -or
    $inventory.counts.compile_call_sites -ne 90 -or
    $inventory.counts.actual_cuda_entry_kinds -ne 88 -or
    $inventory.counts.kernel_abi_families -ne 27 -or
    $inventory.counts.typed_actual_entry_kinds -ne 31 -or
    $inventory.counts.unmigrated_actual_entry_kinds -ne 57 -or
    $inventory.counts.product_low_level_call_sites -ne 99 -or
    $inventory.counts.typed_low_level_call_sites -ne 27 -or
    $inventory.counts.unmigrated_low_level_call_sites -ne 72 -or
    $inventory.counts.wrapper_orchestration_call_sites -ne 29 -or
    $inventory.next_candidate.entry -cne 'rigid_kinematics' -or
    $inventory.next_candidate.selected -ne $false -or
    $massRows.Count -ne 1 -or
    $massRows[0].entry -cne 'rigid_mass_solve' -or
    $massRows[0].classification -cne 'typed_wrapper_low_level' -or
    $inventory.correction.preserved_incorrect_sha256 -cne '57676264b94072568d15e75ec6ec2861f47c99ea473f453d3b7abdd9fffd2a35') {
    throw '修正版盘点错误'
}

$precision = Get-Content -LiteralPath "$root/precision-mapping-v3.json" -Raw | ConvertFrom-Json
if ($precision.inventory.version -ne 5 -or
    $precision.source_identity.frozen_cuda_unchanged -ne $true -or
    $precision.numerical.cuda_arithmetic_unchanged -ne $true -or
    $precision.numerical.floating_order_unchanged -ne $true -or
    $precision.numerical.fma_contract_unchanged -ne $true -or
    $precision.numerical.existing_tolerances_changed -ne $false -or
    $precision.numerical.raw_and_typed_bit_equal -ne $true -or
    $precision.validation.worlds -ne 257 -or
    $precision.validation.grid_tail_crossed -ne $true -or
    $precision.validation.task_gpu_memory -cne 'unknown' -or
    $precision.historical_precision.failed_blocks -ne 1042 -or
    @($precision.acceptance.PSObject.Properties | Where-Object Value -ne $false).Count -ne 0) {
    throw '精度映射或验收边界错误'
}

$derived = @(
    @{id='inventory-v5';path="$root/unmigrated-launch-inventory-v5.json"},
    @{id='capture-evidence-v2';path="$root/capture-inventory-v2.json"},
    @{id='precision-map-v3';path="$root/precision-mapping-v3.json"}
)
foreach ($item in $derived) {
    $registration = Get-Content -LiteralPath "$runtime/registrations/$($item.id).json" -Raw | ConvertFrom-Json
    $identity = Get-Content -LiteralPath "$($registration.output_root)/stdout.log" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $item.path -Algorithm SHA256).Hash.ToLowerInvariant()
    $blob = @(& git hash-object --no-filters -- $item.path)[0].Trim()
    if ($identity.output_path -cne $item.path -or
        $identity.output_sha256 -cne $hash -or
        $identity.output_blob -cne $blob) {
        throw "首次生成身份错误：$($item.id)"
    }
}

$registryIdentities = @(
    @{path='fixtures/precision/t4-current-kernels.json';sha256='de6b5c5dd043426ee388f6cc4f193d6c50e89ab37a7baf85be03dc2a8b86a0fc';blob='3f90f50ffe6380763d88aa6129bb0ab8307fd5ba'},
    @{path='fixtures/precision/t4-kernels.json';sha256='bcdb009d1d21eec89341a6eeeacc26851cf16d1bfba1b5c373c3bca2a5a3cf36';blob='bb1b7cce905001f7cc1eb04950c77cae70e4bbf0'}
)
foreach ($registryIdentity in $registryIdentities) {
    $registry = Get-Content -LiteralPath $registryIdentity.path -Raw | ConvertFrom-Json
    $artifacts = @($registry.artifacts | Where-Object entry -CEQ 'rigid_mass_solve')
    $hash = (Get-FileHash -LiteralPath $registryIdentity.path -Algorithm SHA256).Hash.ToLowerInvariant()
    $blob = @(& git hash-object --no-filters -- $registryIdentity.path)[0].Trim()
    if ($hash -cne $registryIdentity.sha256 -or
        $blob -cne $registryIdentity.blob -or
        $artifacts.Count -ne 1 -or
        $artifacts[0].source_sha256 -cne $capture.cuda.sha256 -or
        $artifacts[0].ptx_sha256 -cne $capture.ptx.sha256 -or
        $artifacts[0].receipt_sha256 -cne '31862fbcc6a4801b7c84c8d414197c615ffaa13f390aecff736f207718d21a7d') {
        throw "冻结PTX登记不符：$($registryIdentity.path)"
    }
}

$plan = Get-Content -LiteralPath "$root/execution-plan.json" -Raw | ConvertFrom-Json
$commands = @($plan.registered_validations | ForEach-Object { "$($_.executable) $($_.arguments -join ' ')" }) -join "`n"
if ($plan.resource_lock -cne 'gpu:windows-t4' -or
    $plan.device_identity -cne 'NVIDIA GeForce RTX 4070 Ti SUPER' -or
    $commands -match '(?i)(^|\s)python(\.exe)?(\s|$)') {
    throw '执行计划边界错误'
}
$diff = @(& git diff -U0 $base -- $sourcePath)
if ($diff -match '(?m)^\+\s*#\[allow\(') { throw '新增Clippy豁免' }

[ordered]@{
    schema=1
    task_id=$task
    successful_authoritative_runs=$successfulRuns.Count
    preserved_failed_runs=3
    preserved_zero_selection_runs=1
    capture_files=36
    worlds=257
    ptx_sha256=$capture.ptx.sha256
    frozen_registry=@($registryIdentities)
    next_candidate=$inventory.next_candidate.entry
    next_candidate_started=$false
    task_gpu_memory='unknown'
    result='pass'
} | ConvertTo-Json
