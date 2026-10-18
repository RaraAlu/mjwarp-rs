$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-bias'
$root = 'docs/development-evidence/layers-m3-rne-bias'
$sourcePath = 'src/physics/rne_bias.rs'
$priorPath = 'docs/development-evidence/layers-m3-rne-cacc/unmigrated-launch-inventory.json'
$outputPath = "$root/unmigrated-launch-inventory.json"
if (Test-Path -LiteralPath $outputPath) { throw "盘点输出已存在：$outputPath" }

. 'docs/development-evidence/layers-m3-inventory-derived-correction/inventory-lib.ps1'
$inventory = Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json -AsHashtable
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$testStart = [regex]::Match($source, '(?m)^#\[cfg\(test\)\]\s*\r?\nmod tests')
if (-not $testStart.Success) { throw '找不到测试模块边界' }
$product = $source.Substring(0, $testStart.Index)

function Get-Line([int]$Index) {
    1 + [regex]::Matches($source.Substring(0, $Index), "`n").Count
}

function Find-One([string]$Pattern, [string]$Name) {
    $matches = [regex]::Matches($product, $Pattern)
    if ($matches.Count -ne 1) { throw "$Name 数量错误：$($matches.Count)" }
    $matches[0]
}

$compile = Find-One 'SynchronousKernel::compile\s*\(session,\s*RNE_BIAS_CUDA,\s*"rne_bias"\)' 'rne_bias编译点'
$compileSite = [ordered]@{
    file = $sourcePath
    line = Get-Line $compile.Index
    entry = 'rne_bias'
    expression = $null
}
$compileSites = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.compile_sites) {
    if ($site.file -ne $sourcePath) { $compileSites.Add($site) }
}
$compileSites.Add($compileSite)
$inventory.compile_sites = @($compileSites)

foreach ($entry in $inventory.actual_cuda_entry_kinds) {
    if ($entry.entry -ceq 'rne_bias') {
        $entry.compile_sites = @("$sourcePath`:$($compileSite.line)")
        $entry.migration_status = 'typed_safe_wrapper'
        $entry.kernel_abi_evidence = @('RneBiasSpec in src/physics/rne_bias.rs')
    }
}

$kernelBlock = Find-One '(?s)impl RneBiasKernel \{(?<body>.*?)\n\}\n\nimpl RneBiasPlan' 'rne_bias包装实现'
$lowMatch = [regex]::Match($kernelBlock.Groups['body'].Value, 'self\.kernel\.launch_flex_edges\s*\(')
if (-not $lowMatch.Success) { throw '找不到rne_bias低层调用' }
$lowIndex = $kernelBlock.Groups['body'].Index + $lowMatch.Index
$low = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.product_low_level_call_sites) {
    if ($site.file -ne $sourcePath) { $low.Add($site) }
}
$low.Add([ordered]@{
    file=$sourcePath
    line=Get-Line $lowIndex
    receiver='self.kernel'
    method='launch_flex_edges'
    classification='typed_wrapper_low_level'
    entry='rne_bias'
})
$inventory.product_low_level_call_sites = @($low)

$outerMatch = Find-One 'self\.kernel\s*\.\s*launch\s*\(' 'rne_bias编排调用'
$outer = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.wrapper_orchestration_call_sites) {
    if ($site.file -ne $sourcePath) { $outer.Add($site) }
}
$outer.Add([ordered]@{
    file=$sourcePath
    line=Get-Line $outerMatch.Index
    receiver='self.kernel'
    method='launch'
    classification='wrapper_orchestration'
    entry='rne_bias'
})
$inventory.wrapper_orchestration_call_sites = @($outer)

$families = [Collections.Generic.List[object]]::new()
foreach ($family in $inventory.kernel_abi_families) { $families.Add($family) }
$families.Add([ordered]@{
    type='RneBiasSpec'
    logical_name='rne_bias'
    definition=$sourcePath
    actual_cuda_entries='rne_bias'
    typed_low_level_calls=@("$sourcePath`:$((Get-Line $lowIndex))")
})
$inventory.kernel_abi_families = @($families)

$inventory.task_id = $task
$inventory.base_commit = '9f01475c76e47a837f276c429b0e8994b96bb3e6'
$inventory.source_file = $priorPath
$inventory.source_file_sha256 = (Get-FileHash -LiteralPath $priorPath -Algorithm SHA256).Hash.ToLowerInvariant()
$inventory.scope = 'prior repository inventory with only rne_bias arrays reclassified'
$inventory.methodology.source = 'committed rne_cacc detail arrays plus current rne_bias source anchors'
$inventory.methodology.raw_scan = 'current rne_bias compile, low-level and orchestration rows are rebuilt before summaries'
$inventory.methodology.prior_inventory = '../layers-m3-rne-cacc/unmigrated-launch-inventory.json'
$inventory.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'unchanged files inherit committed prior detail rows',
    'all summary counts are derived from updated arrays',
    'this task reclassifies only rne_bias'
)

$derived = Get-InventoryDerived -Inventory $inventory
$inventory.counts = $derived.counts
$inventory.private_launch_methods = $derived.private_launch_methods
$inventory.next_candidate = $derived.next_candidate
$inventory.acceptance_boundary.next_candidate = $derived.next_candidate.entry
$inventory.acceptance_boundary.rne_bias_raw = $false
$inventory.acceptance_boundary.rne_bias_migrated = $true
$inventory.acceptance_boundary.task_gpu_memory_usage = 'unknown'
$inventory.manual_cross_checks = @(
    "all $($derived.counts.actual_cuda_entry_kinds) entry rows are unique",
    "typed entry states total $($derived.counts.typed_actual_entry_kinds)",
    "unmigrated entry states total $($derived.counts.unmigrated_actual_entry_kinds)",
    "all $($derived.counts.product_low_level_call_sites) low-level rows are unique",
    "all $($derived.counts.wrapper_orchestration_call_sites) orchestration rows are unique",
    'rne_bias has one typed low-level row and one orchestration row'
)

if ($derived.counts.actual_cuda_entry_kinds -ne 88 -or
    $derived.counts.typed_actual_entry_kinds -ne 29 -or
    $derived.counts.unmigrated_actual_entry_kinds -ne 59 -or
    $derived.counts.product_low_level_call_sites -ne 99 -or
    $derived.counts.typed_low_level_call_sites -ne 25 -or
    $derived.counts.unmigrated_low_level_call_sites -ne 74 -or
    $derived.counts.wrapper_orchestration_call_sites -ne 27 -or
    $derived.counts.kernel_abi_families -ne 25 -or
    $derived.next_candidate.entry -ceq 'rne_bias') {
    throw 'RNE偏置力盘点派生结果不符'
}

$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema=1
    source_sha256=(Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    compile_sites=1
    counts=$derived.counts
    next_candidate=$derived.next_candidate
} | ConvertTo-Json -Depth 8
