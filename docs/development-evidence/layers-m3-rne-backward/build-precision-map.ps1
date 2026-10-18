$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-rne-backward'
$outputPath = "$root/precision-mapping-v2.json"
$base = '0f54bcf8c36feeb00032a923925433c8c1dfd9ee'
if (Test-Path -LiteralPath $outputPath) { throw "精度映射已存在：$outputPath" }

& git diff --quiet $base -- src/physics/rne.cu
if ($LASTEXITCODE -ne 0) { throw 'RNE CUDA源码发生变化' }
$baseCudaBlob = @(& git rev-parse "$base`:src/physics/rne.cu")
if ($LASTEXITCODE -ne 0 -or $baseCudaBlob.Count -ne 1) { throw '无法解析基线CUDA对象' }
$capture = Get-Content -LiteralPath "$root/capture-inventory.json" -Raw | ConvertFrom-Json
$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory-v2.json" -Raw | ConvertFrom-Json
if ($inventory.inventory_version -ne 2) { throw '权威盘点版本错误' }
$historyPath = 'docs/development-evidence/layers-m3-rne-bias/precision-revalidation-v2.json'
$history = Get-Content -LiteralPath $historyPath -Raw | ConvertFrom-Json
if ($history.historical_precision.failed_blocks -ne 1042 -or
    $history.historical_precision.repaired_by_this_slice -ne $false) {
    throw '历史精度边界变化'
}

$record = [ordered]@{
    schema = 1
    task_id = '2026-10-19-layers-m3-rne-backward'
    source_identity = [ordered]@{
        base_revision = $base
        rust_path = 'src/physics/rne.rs'
        rust_sha256 = (Get-FileHash -LiteralPath 'src/physics/rne.rs' -Algorithm SHA256).Hash.ToLowerInvariant()
        cuda_path = 'src/physics/rne.cu'
        cuda_sha256 = (Get-FileHash -LiteralPath 'src/physics/rne.cu' -Algorithm SHA256).Hash.ToLowerInvariant()
        cuda_base_blob = $baseCudaBlob[0]
        cuda_unchanged = $true
    }
    abi = [ordered]@{
        entry = 'rne_backward'
        pointers = @('metadata:i32:read:address-proof', 'com:f32:read', 'velocity:f32:read', 'output:f32:read-write')
        scalars = @('nbody:u32', 'level:u32', 'zero:u32', 'work:u32')
        scalar_constraints = @('level in 1..=max_depth', 'zero == 0', 'work == worlds*nbody')
        block = 256
    }
    orchestration = [ordered]@{
        all_levels_preflight_before_cacc = $true
        launch_order = 'max_depth down to 1'
        synchronous_level_barriers = $true
        root_only_backward_launches = 0
    }
    numerical = [ordered]@{
        cuda_arithmetic_unchanged = $true
        floating_order_unchanged = $true
        atomic_add_unchanged = $true
        exact_integer_bit_comparison = $true
        general_sibling_atomic_determinism_claimed = $false
        existing_general_tolerance_changed = $false
        existing_general_tolerance_revalidated = $false
    }
    generated_product = [ordered]@{
        current_ptx_sha256 = $capture.ptx.current.sha256
        historical_ptx_sha256 = $capture.ptx.historical_sha256
        byte_equal = $capture.ptx.byte_equal
        nvrtc_version = $capture.device_result.nvrtc_version
        nvrtc_sha256 = $capture.device_result.nvrtc_sha256
    }
    validation = [ordered]@{
        worlds = 43
        work_items = 258
        multi_level_depth = 4
        same_parent_siblings = @(1, 5)
        stale_backward_proof_checked_directly = $true
        foreign_session_rejection_precedes_output_write = $true
        task_gpu_memory = $capture.device_result.task_memory
    }
    inventory = [ordered]@{
        version = $inventory.inventory_version
        supersedes = $inventory.supersedes
        typed_entries = $inventory.counts.typed_actual_entry_kinds
        unmigrated_entries = $inventory.counts.unmigrated_actual_entry_kinds
        next_candidate = $inventory.next_candidate.entry
    }
    historical_precision = [ordered]@{
        source = $historyPath
        source_sha256 = (Get-FileHash -LiteralPath $historyPath -Algorithm SHA256).Hash.ToLowerInvariant()
        failed_blocks = 1042
        repaired_by_this_slice = $false
    }
    acceptance = [ordered]@{
        np01 = $false
        np02 = $false
        np03 = $false
        np04 = $false
        performance = $false
        linux = $false
        release = $false
    }
}
$json = ($record | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
$record | ConvertTo-Json -Depth 12
