$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$outputPath = "$root/precision-mapping-v3.json"
if (Test-Path -LiteralPath $outputPath) { throw "精度映射已存在：$outputPath" }
$capture = Get-Content -LiteralPath "$root/capture-inventory-v2.json" -Raw | ConvertFrom-Json
$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory-v5.json" -Raw | ConvertFrom-Json
$historyPath = 'docs/development-evidence/layers-m3-rne-bias/precision-revalidation-v2.json'
$history = Get-Content -LiteralPath $historyPath -Raw | ConvertFrom-Json
if ($history.historical_precision.failed_blocks -ne 1042 -or
    $history.historical_precision.repaired_by_this_slice -ne $false) {
    throw '历史精度边界变化'
}
$record = [ordered]@{
    schema=1
    task_id='2026-10-19-layers-m3-rigid-mass-solve'
    source_identity=[ordered]@{
        base_revision='a2d057618d2884234e9b6779e619cce5abfc2adc'
        rust_path='src/physics/mass_solve.rs'
        rust_sha256=$capture.current_source_identity.sha256
        rust_working_blob=$capture.current_source_identity.working_blob
        gpu_validation_rust_sha256=$capture.gpu_validation_source_identity.sha256
        product_source_equal_to_gpu_validation_source=$capture.product_source_equal_to_gpu_validation_source
        full_file_equal_to_gpu_validation_source=$false
        frozen_cuda_unchanged=$capture.frozen_cuda_unchanged_from_base
    }
    abi=[ordered]@{
        entry='rigid_mass_solve'
        pointers=@('blocks:i32:read:address-proof','rhs:f32:read','state:f32:read','output:f32:read-write')
        scalars=@('nb:u32','n:u32','nr:u32','nw:u32')
        status_location='inside output prefix per world'
        thread_ownership='one thread per world'
        block=256
    }
    metadata=[ordered]@{
        contiguous_partition=$true
        positive_sizes=$true
        legal_kinds=@(0,1,2)
        diagonal_requires_size_one=$true
        reverse_ldl_size_two_allowed=$true
        upload_generation_bound=$true
        dirty_write_blocks_old_and_new_proofs=$true
    }
    numerical=[ordered]@{
        cuda_arithmetic_unchanged=$true
        floating_order_unchanged=$true
        fma_contract_unchanged=$true
        existing_tolerances_changed=$false
        raw_and_typed_bit_equal=$true
        routes=@('diagonal-1','cholesky-6','tile-cholesky-7','reverse-ldl-65')
    }
    generated_product=[ordered]@{
        current_ptx_sha256=$capture.ptx.sha256
        first_and_second_ptx_byte_equal=$capture.ptx.first_and_second_byte_equal
        nvrtc_version=$capture.device_result.nvrtc_version
        nvrtc_sha256=$capture.device_result.nvrtc_sha256
    }
    validation=[ordered]@{
        worlds=257
        grid_tail_crossed=$true
        invalid_pivot_checked=$true
        nonfinite_reciprocal_checked=$true
        zero_dof_placeholder_checked=$true
        capacity_session_alias_overflow_and_index_rejections_checked=$true
        stale_old_launch_and_new_preflight_checked=$true
        recovery_checked=$true
        task_gpu_memory=$capture.device_result.task_memory
    }
    inventory=[ordered]@{
        version=$inventory.inventory_version
        typed_entries=$inventory.counts.typed_actual_entry_kinds
        unmigrated_entries=$inventory.counts.unmigrated_actual_entry_kinds
        next_candidate=$inventory.next_candidate.entry
    }
    historical_precision=[ordered]@{
        source=$historyPath
        source_sha256=(Get-FileHash -LiteralPath $historyPath -Algorithm SHA256).Hash.ToLowerInvariant()
        failed_blocks=1042
        repaired_by_this_slice=$false
    }
    acceptance=[ordered]@{np01=$false;np02=$false;np03=$false;np04=$false;performance=$false;linux=$false;release=$false;parent_m3=$false}
}
$json = ($record | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
$outputBytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
$outputStream = [IO.File]::Open(
    (Join-Path (Get-Location) $outputPath),
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::Read
)
try { $outputStream.Write($outputBytes, 0, $outputBytes.Length) } finally { $outputStream.Dispose() }
$outputHash = (Get-FileHash -LiteralPath $outputPath -Algorithm SHA256).Hash.ToLowerInvariant()
$outputBlob = @(& git hash-object --no-filters -- $outputPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or $outputBlob -notmatch '^[0-9a-f]{40}$') { throw '无法登记精度映射对象' }
[ordered]@{
    schema=1
    output_path=$outputPath
    output_sha256=$outputHash
    output_blob=$outputBlob
    record=$record
} | ConvertTo-Json -Depth 12
