$ErrorActionPreference = 'Stop'
$base = '0f3f4b30e8771441354123d159f29e79067fc61c'
$root = 'docs/development-evidence/layers-m3-inventory-derived-correction'
$source = 'docs/development-evidence/layers-m3-smooth-force-compute/unmigrated-launch-inventory.json'

& git diff --quiet $base -- $source
if ($LASTEXITCODE -ne 0) { throw '来源快照相对基线发生变化' }
& git diff --quiet -- src
if ($LASTEXITCODE -ne 0) { throw '产品源码存在未暂存改动' }
& git diff --cached --quiet -- src
if ($LASTEXITCODE -ne 0) { throw '产品源码存在暂存改动' }

$inventory = Get-Content -LiteralPath "$root/corrected-inventory.json" -Raw | ConvertFrom-Json
$sourceHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
if ($inventory.correction.source_inventory_sha256 -cne $sourceHash) { throw '来源快照摘要不符' }
if ($inventory.correction.old_snapshot_modified -ne $false -or
    $inventory.correction.product_source_modified -ne $false) {
    throw '纠正边界声明错误'
}
if ($inventory.next_candidate.entry -cne 'sleep_copy' -or $inventory.next_candidate.selected -ne $false) {
    throw 'sleep_copy只读边界错误'
}

[ordered]@{
    schema = 1
    base_revision = $base
    source_snapshot_unchanged = $true
    product_source_unchanged = $true
    source_sha256 = $sourceHash
    next_candidate = 'sleep_copy'
    next_candidate_started = $false
    gpu_tests_run = $false
    product_tests_run = $false
    result = 'pass'
} | ConvertTo-Json
