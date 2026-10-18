$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-inventory-derived-correction'
$root = 'docs/development-evidence/layers-m3-inventory-derived-correction'
$sourcePath = 'docs/development-evidence/layers-m3-smooth-force-compute/unmigrated-launch-inventory.json'
$outputPath = "$root/corrected-inventory.json"
$claimsPath = "$root/claims.json"
if (Test-Path -LiteralPath $outputPath) { throw "纠正快照已存在：$outputPath" }

. "$root/inventory-lib.ps1"
$inventory = Get-Content -LiteralPath $sourcePath -Raw | ConvertFrom-Json -AsHashtable
$claims = Get-Content -LiteralPath $claimsPath -Raw | ConvertFrom-Json -AsHashtable
$sourceHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
$sourceBlob = @(& git rev-parse "HEAD`:$sourcePath")[0].Trim()
if ($LASTEXITCODE -ne 0 -or $sourceBlob -notmatch '^[0-9a-f]{40}$') { throw '无法解析来源清单对象' }

$inventory.task_id = $task
$inventory.base_commit = '0f3f4b30e8771441354123d159f29e79067fc61c'
$inventory.source_file = $sourcePath
$inventory.source_file_sha256 = $sourceHash
$inventory.scope = 'derived-field correction from the committed call-site detail snapshot; no product source change'
$inventory.methodology.source = 'committed current call-site detail snapshot with derived summaries rebuilt from arrays'
$inventory.methodology.raw_scan = 'product_low_level_call_sites and wrapper_orchestration_call_sites from the committed source inventory'
$inventory.methodology.prior_inventory = '../layers-m3-smooth-force-compute/unmigrated-launch-inventory.json'
$inventory.methodology.limitations = @(
    'this correction does not rescan or modify product source',
    'detail rows remain byte-derived from the committed source inventory',
    'sleep_copy is identified read-only and no product task is started'
)

$derived = Get-InventoryDerived -Inventory $inventory
$inventory.counts = $derived.counts
$inventory.private_launch_methods = $derived.private_launch_methods
$inventory.manual_cross_checks = @(
    "all $($derived.counts.actual_cuda_entry_kinds) actual CUDA entry keys are unique",
    "typed entry states total $($derived.counts.typed_actual_entry_kinds) across typed_wrapper and typed_safe_wrapper",
    "unmigrated_or_unwrapped entry states total $($derived.counts.unmigrated_actual_entry_kinds)",
    "all $($derived.counts.product_low_level_call_sites) low-level call keys are unique",
    "generic launch derives $($derived.generic_launch.typed_low_level_calls) typed and $($derived.generic_launch.unmigrated_raw_calls) raw calls",
    "all $($derived.counts.wrapper_orchestration_call_sites) wrapper orchestration keys are unique",
    "all $($derived.counts.kernel_abi_families) KernelAbi family keys are unique",
    'applied_force and smooth_force migrations are included in the current detail rows',
    'private launch method totals are rebuilt from low-level and orchestration detail rows'
)
$inventory.next_candidate = $derived.next_candidate
$inventory.acceptance_boundary.next_candidate = $derived.next_candidate.entry
$inventory.correction = [ordered]@{
    source_inventory = $sourcePath
    source_inventory_sha256 = $sourceHash
    source_inventory_blob = $sourceBlob
    derived_fields = @('counts', 'private_launch_methods', 'manual_cross_checks', 'methodology.raw_scan', 'next_candidate')
    old_snapshot_modified = $false
    product_source_modified = $false
}

$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
$written = Get-Content -LiteralPath $outputPath -Raw | ConvertFrom-Json -AsHashtable
$verified = Assert-InventoryConsistent -Inventory $written -Claims $claims
[ordered]@{
    schema = 1
    task = $task
    source_sha256 = $sourceHash
    source_blob = $sourceBlob
    output = $outputPath
    counts = $verified.counts
    generic_launch = $verified.generic_launch
    next_candidate = $verified.next_candidate
    result = 'pass'
} | ConvertTo-Json -Depth 8
