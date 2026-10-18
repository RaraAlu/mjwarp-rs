$ErrorActionPreference = 'Stop'
$sourcePath = 'src/physics/smooth_force.rs'
$priorPath = 'docs/development-evidence/layers-m3-applied-force-publish/unmigrated-launch-inventory.json'
$outputPath = 'docs/development-evidence/layers-m3-smooth-force-validate/unmigrated-launch-inventory.json'
if (Test-Path -LiteralPath $outputPath) { throw "盘点输出已存在：$outputPath" }
$inventory = Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json -AsHashtable

function Line([string]$needle, [int]$occurrence = 1) {
    $hits = @(Select-String -LiteralPath $sourcePath -SimpleMatch -Pattern $needle)
    if ($hits.Count -lt $occurrence) { throw "找不到源码锚点：$needle#$occurrence" }
    $hits[$occurrence - 1].LineNumber
}

$sourceHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
$validateDefinition = Line 'struct SmoothForceValidateSpec;'
$validateCompile = Line 'SynchronousKernel::compile(session, source, "smooth_force_validate")?'
$computeCompile = Line 'SynchronousKernel::compile(session, &source, "smooth_force")?'
$validateLow = Line 'self.kernel' 1
$computeLow = Line 'self.compute.launch('
$validateOuter = Line 'self.validate.launch('

$inventory.task_id = '2026-10-19-layers-m3-smooth-force-validate'
$inventory.base_commit = 'c2fc21e9569128188b74d0321a992f03a2955da6'
$inventory.source_file_sha256 = $sourceHash
$inventory.scope = 'repository-wide prior inventory with current smooth_force_validate reclassified'
$inventory.methodology.source = 'verified prior inventory plus current-source lexical and receiver-definition cross-check for the only changed product file'
$inventory.methodology.prior_inventory = '../layers-m3-applied-force-publish/unmigrated-launch-inventory.json'
$inventory.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'typed wrapper locations use current-source static evidence',
    'repository-wide unchanged entries inherit the verified prior inventory',
    'this task reclassifies only smooth_force_validate and leaves smooth_force raw'
)
$inventory.counts.kernel_abi_families = 19
$inventory.counts.typed_actual_entry_kinds = 23
$inventory.counts.unmigrated_actual_entry_kinds = 65
$inventory.counts.typed_low_level_call_sites = 19
$inventory.counts.unmigrated_low_level_call_sites = 80
$inventory.counts.wrapper_orchestration_call_sites = 21

$families = [Collections.Generic.List[object]]::new()
foreach ($family in $inventory.kernel_abi_families) { $families.Add($family) }
$families.Add([ordered]@{
    type = 'SmoothForceValidateSpec'
    logical_name = 'smooth_force_validate'
    definition = "$sourcePath`:$validateDefinition"
    actual_cuda_entries = 'smooth_force_validate'
    typed_low_level_calls = @("$sourcePath`:$validateLow")
})
$inventory.kernel_abi_families = @($families)

foreach ($entry in $inventory.actual_cuda_entry_kinds) {
    if ($entry.entry -eq 'smooth_force_validate') {
        $entry.compile_sites = @("$sourcePath`:$validateCompile")
        $entry.migration_status = 'typed_safe_wrapper'
        $entry.kernel_abi_evidence = @("SmoothForceValidateSpec at $sourcePath`:$validateDefinition")
    }
    elseif ($entry.entry -eq 'smooth_force') {
        $entry.compile_sites = @("$sourcePath`:$computeCompile")
        $entry.migration_status = 'raw_unmigrated'
        $entry.kernel_abi_evidence = @()
    }
}
foreach ($site in $inventory.compile_sites) {
    if ($site.file -eq $sourcePath) {
        $site.line = switch ($site.entry) {
            'smooth_force_validate' { $validateCompile }
            'smooth_force' { $computeCompile }
            default { $site.line }
        }
    }
}

$low = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.product_low_level_call_sites) {
    if ($site.file -ne $sourcePath) { $low.Add($site) }
}
$low.Add([ordered]@{file=$sourcePath;line=$validateLow;receiver='self.kernel';method='launch';classification='typed_wrapper_low_level';entry='smooth_force_validate'})
$low.Add([ordered]@{file=$sourcePath;line=$computeLow;receiver='self.compute';method='launch';classification='raw_product_low_level';entry='smooth_force'})
$inventory.product_low_level_call_sites = @($low)

$outer = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.wrapper_orchestration_call_sites) {
    if ($site.file -ne $sourcePath) { $outer.Add($site) }
}
$outer.Add([ordered]@{file=$sourcePath;line=$validateOuter;receiver='self.validate';method='launch';classification='wrapper_orchestration';entry=$null})
$inventory.wrapper_orchestration_call_sites = @($outer)
$inventory.manual_cross_checks = @($inventory.manual_cross_checks | Where-Object { $_ -notmatch '^smooth_force' }) + @(
    'smooth_force_validate uses one typed synchronous wrapper',
    'smooth_force remains one raw synchronous launch'
)
$inventory.acceptance_boundary.smooth_force_validate_raw = $false
$inventory.acceptance_boundary.smooth_force_validate_migrated = $true
$inventory.acceptance_boundary.smooth_force_raw = $true
$inventory.acceptance_boundary.task_gpu_memory_usage = 'unknown'
$inventory.acceptance_boundary.next_candidate = 'smooth_force'

$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema = 1
    source_sha256 = $sourceHash
    validate_definition = $validateDefinition
    validate_compile = $validateCompile
    validate_typed_launch = $validateLow
    validate_orchestration = $validateOuter
    compute_raw_launch = $computeLow
    counts = $inventory.counts
} | ConvertTo-Json -Depth 5
