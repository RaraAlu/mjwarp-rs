$ErrorActionPreference = 'Stop'
$sourcePath = 'src/physics/applied_force.rs'
$priorPath = 'docs/development-evidence/layers-m3-applied-force-validate/unmigrated-launch-inventory.json'
$outputPath = 'docs/development-evidence/layers-m3-applied-force-publish/unmigrated-launch-inventory.json'
if (Test-Path -LiteralPath $outputPath) { throw "盘点输出已存在：$outputPath" }
$inventory = Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json -AsHashtable

function Line([string]$needle, [int]$occurrence = 1) {
    $hits = @(Select-String -LiteralPath $sourcePath -SimpleMatch -Pattern $needle)
    if ($hits.Count -lt $occurrence) { throw "找不到源码锚点：$needle#$occurrence" }
    $hits[$occurrence - 1].LineNumber
}

$sourceHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
$validateDefinition = Line 'struct AppliedForceValidateSpec;'
$deltaDefinition = Line 'struct AppliedForceDeltaSpec;'
$publishDefinition = Line 'struct AppliedForcePublishSpec;'
$validateCompile = Line 'SynchronousKernel::compile(session, CUDA, "applied_force_validate")?'
$deltaCompile = Line 'SynchronousKernel::compile(session, CUDA, "applied_force_delta")?'
$publishCompile = Line 'SynchronousKernel::compile(session, CUDA, "applied_force_publish")?'
$validateLow = Line 'self.kernel' 1
$deltaLow = Line 'self.kernel' 2
$publishLow = Line 'self.kernel' 3
$validateOuter = Line 'self.validate.launch('
$deltaOuter = Line 'self.delta.launch('
$publishOuter = Line 'self.publish.launch('

$inventory.task_id = '2026-10-19-layers-m3-applied-force-publish'
$inventory.base_commit = '6bcf8496477e154d5f904b4aa1884637ecbe5b9e'
$inventory.source_file_sha256 = $sourceHash
$inventory.scope = 'repository-wide prior inventory with current applied_force.rs publish reclassified'
$inventory.methodology.source = 'verified prior inventory plus current-source lexical and receiver-definition cross-check for the only changed product file'
$inventory.methodology.prior_inventory = '../layers-m3-applied-force-validate/unmigrated-launch-inventory.json'
$inventory.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'typed wrapper locations use current-source static evidence',
    'repository-wide unchanged entries inherit the verified prior inventory',
    'current publish reclassifies only applied_force_publish because this task changes no other product file'
)
$inventory.counts.kernel_abi_families = 18
$inventory.counts.typed_actual_entry_kinds = 22
$inventory.counts.unmigrated_actual_entry_kinds = 66
$inventory.counts.typed_low_level_call_sites = 18
$inventory.counts.unmigrated_low_level_call_sites = 81
$inventory.counts.wrapper_orchestration_call_sites = 20

$families = [Collections.Generic.List[object]]::new()
foreach ($family in $inventory.kernel_abi_families) {
    if ($family.type -eq 'AppliedForceValidateSpec') {
        $family.definition = "$sourcePath`:$validateDefinition"
        $family.typed_low_level_calls = @("$sourcePath`:$validateLow")
    }
    elseif ($family.type -eq 'AppliedForceDeltaSpec') {
        $family.definition = "$sourcePath`:$deltaDefinition"
        $family.typed_low_level_calls = @("$sourcePath`:$deltaLow")
    }
    $families.Add($family)
}
$families.Add([ordered]@{
    type = 'AppliedForcePublishSpec'
    logical_name = 'applied_force_publish'
    definition = "$sourcePath`:$publishDefinition"
    actual_cuda_entries = 'applied_force_publish'
    typed_low_level_calls = @("$sourcePath`:$publishLow")
})
$inventory.kernel_abi_families = @($families)

foreach ($entry in $inventory.actual_cuda_entry_kinds) {
    if ($entry.entry -eq 'applied_force_validate') {
        $entry.compile_sites = @("$sourcePath`:$validateCompile")
        $entry.kernel_abi_evidence = @("AppliedForceValidateSpec at $sourcePath`:$validateDefinition")
    }
    elseif ($entry.entry -eq 'applied_force_delta') {
        $entry.compile_sites = @("$sourcePath`:$deltaCompile")
        $entry.kernel_abi_evidence = @("AppliedForceDeltaSpec at $sourcePath`:$deltaDefinition")
    }
    elseif ($entry.entry -eq 'applied_force_publish') {
        $entry.compile_sites = @("$sourcePath`:$publishCompile")
        $entry.migration_status = 'typed_safe_wrapper'
        $entry.kernel_abi_evidence = @("AppliedForcePublishSpec at $sourcePath`:$publishDefinition")
    }
}
foreach ($site in $inventory.compile_sites) {
    if ($site.file -eq $sourcePath) {
        $site.line = switch ($site.entry) {
            'applied_force_validate' { $validateCompile }
            'applied_force_delta' { $deltaCompile }
            'applied_force_publish' { $publishCompile }
            default { $site.line }
        }
    }
}

$low = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.product_low_level_call_sites) {
    if ($site.file -ne $sourcePath) { $low.Add($site) }
}
$low.Add([ordered]@{file=$sourcePath;line=$validateLow;receiver='self.kernel';method='launch';classification='typed_wrapper_low_level';entry='applied_force_validate'})
$low.Add([ordered]@{file=$sourcePath;line=$deltaLow;receiver='self.kernel';method='launch';classification='typed_wrapper_low_level';entry='applied_force_delta'})
$low.Add([ordered]@{file=$sourcePath;line=$publishLow;receiver='self.kernel';method='launch';classification='typed_wrapper_low_level';entry='applied_force_publish'})
$inventory.product_low_level_call_sites = @($low)

$outer = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.wrapper_orchestration_call_sites) {
    if ($site.file -ne $sourcePath) { $outer.Add($site) }
}
$outer.Add([ordered]@{file=$sourcePath;line=$validateOuter;receiver='self.validate';method='launch';classification='wrapper_orchestration';entry=$null})
$outer.Add([ordered]@{file=$sourcePath;line=$deltaOuter;receiver='self.delta';method='launch';classification='wrapper_orchestration';entry=$null})
$outer.Add([ordered]@{file=$sourcePath;line=$publishOuter;receiver='self.publish';method='launch';classification='wrapper_orchestration';entry=$null})
$inventory.wrapper_orchestration_call_sites = @($outer)
$inventory.manual_cross_checks = @($inventory.manual_cross_checks | Where-Object { $_ -notmatch '^applied_force_' }) + @(
    'applied_force_validate, applied_force_delta and applied_force_publish use distinct typed safe wrappers',
    'applied_force_publish is the only entry reclassified by this task'
)
$inventory.acceptance_boundary.applied_force_delta_migrated = $true
$inventory.acceptance_boundary.applied_force_validate_raw = $false
$inventory.acceptance_boundary.applied_force_validate_migrated = $true
$inventory.acceptance_boundary.applied_force_publish_raw = $false
$inventory.acceptance_boundary.applied_force_publish_migrated = $true
$inventory.acceptance_boundary.task_gpu_memory_usage = 'unknown'
$inventory.acceptance_boundary.next_candidate = 'unselected'

$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema = 1
    source_sha256 = $sourceHash
    validate_definition = $validateDefinition
    delta_definition = $deltaDefinition
    publish_definition = $publishDefinition
    publish_compile = $publishCompile
    publish_typed_launch = $publishLow
    publish_orchestration = $publishOuter
    counts = $inventory.counts
} | ConvertTo-Json -Depth 5
