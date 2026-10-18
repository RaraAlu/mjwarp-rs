$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-backward'
$root = 'docs/development-evidence/layers-m3-rne-backward'
$sourcePath = 'src/physics/rne.rs'
$priorPath = 'docs/development-evidence/layers-m3-rne-bias/unmigrated-launch-inventory.json'
$overlayPath = 'docs/development-evidence/layers-m3-rne-bias-clippy-structural-fix/bias-inventory.json'
$outputPath = "$root/unmigrated-launch-inventory-v2.json"
if (Test-Path -LiteralPath $outputPath) { throw "盘点输出已存在：$outputPath" }

. 'docs/development-evidence/layers-m3-inventory-derived-correction/inventory-lib.ps1'
$inventory = Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json -AsHashtable
$overlay = Get-Content -LiteralPath $overlayPath -Raw | ConvertFrom-Json
$priorHash = (Get-FileHash -LiteralPath $priorPath -Algorithm SHA256).Hash.ToLowerInvariant()
$priorBlob = @(& git hash-object -- $priorPath)
if ($LASTEXITCODE -ne 0 -or $priorBlob.Count -ne 1) { throw '无法计算基线对象' }
if ($overlay.baseline.path -cne $priorPath -or
    $overlay.baseline.sha256 -cne $priorHash -or
    $overlay.baseline.blob -cne $priorBlob[0]) {
    throw '偏置力覆盖层没有绑定当前基线'
}
$biasHash = (Get-FileHash -LiteralPath $overlay.source.path -Algorithm SHA256).Hash.ToLowerInvariant()
if ($overlay.source.sha256 -cne $biasHash) { throw '偏置力源码身份漂移' }

$biasSource = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $overlay.source.path))
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$testStart = [regex]::Match($source, '(?m)^#\[cfg\(test\)\]\s*\r?\nmod tests')
if (-not $testStart.Success) { throw '找不到测试模块边界' }
$product = $source.Substring(0, $testStart.Index)

function Get-Line([string]$Text, [int]$Index) {
    1 + [regex]::Matches($Text.Substring(0, $Index), "`n").Count
}

function Find-One([string]$Text, [string]$Pattern, [string]$Name) {
    $matches = [regex]::Matches($Text, $Pattern)
    if ($matches.Count -ne 1) { throw "$Name 数量错误：$($matches.Count)" }
    $matches[0]
}

$biasAnchors = [ordered]@{
    spec_definition_line = Get-Line $biasSource (Find-One $biasSource 'struct RneBiasSpec;' '偏置力规格').Index
    input_group_line = Get-Line $biasSource (Find-One $biasSource "struct RneBiasInputs<'a>" '偏置力输入组').Index
    compile_line = Get-Line $biasSource (Find-One $biasSource 'SynchronousKernel::compile\(session, RNE_BIAS_CUDA, "rne_bias"\)' '偏置力编译点').Index
    preflight_line = Get-Line $biasSource (Find-One $biasSource '(?m)^    fn preflight\(' '偏置力预检').Index
    low_level_launch_line = Get-Line $biasSource (Find-One $biasSource 'self\.kernel\.launch_flex_edges\s*\(' '偏置力低层调用').Index
    orchestration_line = Get-Line $biasSource (Find-One $biasSource 'self\.kernel\.launch\(&launch, &inputs, &mut data\.output\)' '偏置力编排调用').Index
    cuda_source_line = Get-Line $biasSource (Find-One $biasSource 'const RNE_BIAS_CUDA: &str' '偏置力源码').Index
}
foreach ($name in $biasAnchors.Keys) {
    if ($overlay.rne_bias.$name -ne $biasAnchors[$name]) { throw "偏置力行号漂移：$name" }
}

foreach ($site in $inventory.compile_sites) {
    if ($site.entry -ceq 'rne_bias') { $site.line = $overlay.rne_bias.compile_line }
}
foreach ($entry in $inventory.actual_cuda_entry_kinds) {
    if ($entry.entry -ceq 'rne_bias') {
        $entry.compile_sites = @("src/physics/rne_bias.rs`:$($overlay.rne_bias.compile_line)")
    }
}
foreach ($site in $inventory.product_low_level_call_sites) {
    if ($site.entry -ceq 'rne_bias') { $site.line = $overlay.rne_bias.low_level_launch_line }
}
foreach ($site in $inventory.wrapper_orchestration_call_sites) {
    if ($site.entry -ceq 'rne_bias') { $site.line = $overlay.rne_bias.orchestration_line }
}
foreach ($family in $inventory.kernel_abi_families) {
    if ($family.logical_name -ceq 'rne_bias') {
        $family.typed_low_level_calls = @("src/physics/rne_bias.rs`:$($overlay.rne_bias.low_level_launch_line)")
    }
}

$definitions = @(
    [ordered]@{ entry='rne_cacc'; type='RneCaccSpec'; kernel='RneCaccKernel'; low='launch_rne_cacc' },
    [ordered]@{ entry='rne_cfrc'; type='RneCfrcSpec'; kernel='RneCfrcKernel'; low='launch' },
    [ordered]@{ entry='rne_backward'; type='RneBackwardSpec'; kernel='RneBackwardKernel'; low='launch' },
    [ordered]@{ entry='rne_qfrc'; type='RneQfrcSpec'; kernel='RneQfrcKernel'; low='launch' }
)
$rows = @()
foreach ($definition in $definitions) {
    $compile = Find-One $product ("SynchronousKernel::compile\(session,\s*RNE_CUDA,\s*`"{0}`"\)" -f $definition.entry) "$($definition.entry)编译点"
    $impl = Find-One $product ("(?s)impl {0} \{{(?<body>.*?)\n\}}" -f $definition.kernel) "$($definition.entry)包装实现"
    $body = $impl.Groups['body']
    $lowPattern = if ($definition.low -ceq 'launch_rne_cacc') {
        'self\.kernel\.launch_rne_cacc\s*\('
    } else {
        'self\.kernel\s*\.launch\s*\('
    }
    $low = Find-One $body.Value $lowPattern "$($definition.entry)低层调用"
    $outer = Find-One $product ("self\.{0}_kernel\.launch\s*\(" -f ($definition.entry -replace '^rne_', '')) "$($definition.entry)编排调用"
    $rows += [ordered]@{
        entry = $definition.entry
        type = $definition.type
        compile_line = Get-Line $source $compile.Index
        low_line = Get-Line $source ($body.Index + $low.Index)
        outer_line = Get-Line $source $outer.Index
        low_method = $definition.low
    }
}

$compileSites = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.compile_sites) {
    if ($site.file -cne $sourcePath) { $compileSites.Add($site) }
}
foreach ($row in $rows) {
    $compileSites.Add([ordered]@{ file=$sourcePath; line=$row.compile_line; entry=$row.entry; expression=$null })
}
$inventory.compile_sites = @($compileSites)

foreach ($entry in $inventory.actual_cuda_entry_kinds) {
    $row = @($rows | Where-Object entry -CEQ $entry.entry)
    if ($row.Count -eq 1) {
        $entry.compile_sites = @("$sourcePath`:$($row[0].compile_line)")
        $entry.migration_status = 'typed_safe_wrapper'
        $entry.kernel_abi_evidence = @("$($row[0].type) in $sourcePath")
    }
}

$families = [Collections.Generic.List[object]]::new()
foreach ($family in $inventory.kernel_abi_families) {
    if ($family.definition -cne $sourcePath) { $families.Add($family) }
}
foreach ($row in $rows) {
    $families.Add([ordered]@{
        type=$row.type
        logical_name=$row.entry
        definition=$sourcePath
        actual_cuda_entries=$row.entry
        typed_low_level_calls=@("$sourcePath`:$($row.low_line)")
    })
}
$inventory.kernel_abi_families = @($families)

$lowSites = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.product_low_level_call_sites) {
    if ($site.file -cne $sourcePath) { $lowSites.Add($site) }
}
foreach ($row in $rows) {
    $lowSites.Add([ordered]@{
        file=$sourcePath
        line=$row.low_line
        receiver='self.kernel'
        method=$row.low_method
        classification='typed_wrapper_low_level'
        entry=$row.entry
    })
}
$inventory.product_low_level_call_sites = @($lowSites)

$outerSites = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.wrapper_orchestration_call_sites) {
    if ($site.file -cne $sourcePath) { $outerSites.Add($site) }
}
foreach ($row in $rows) {
    $outerSites.Add([ordered]@{
        file=$sourcePath
        line=$row.outer_line
        receiver="self.$(($row.entry -replace '^rne_', ''))_kernel"
        method='launch'
        classification='wrapper_orchestration'
        entry=$row.entry
    })
}
$inventory.wrapper_orchestration_call_sites = @($outerSites)

$inventory.task_id = $task
$inventory.inventory_version = 2
$inventory.supersedes = "$root/unmigrated-launch-inventory-v1.json"
$inventory.base_commit = '0f54bcf8c36feeb00032a923925433c8c1dfd9ee'
$inventory.source_file = $priorPath
$inventory.source_file_sha256 = $priorHash
$inventory.scope = 'full prior inventory with all current rne rows rebuilt and only rne_backward reclassified'
$inventory.methodology.source = 'committed rne_bias full inventory plus current rne source anchors'
$inventory.methodology.raw_scan = 'all current rne compile, low-level and orchestration rows are rebuilt before summaries'
$inventory.methodology.prior_inventory = '../layers-m3-rne-bias/unmigrated-launch-inventory.json'
$inventory.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'unchanged files inherit committed prior detail rows',
    'all summary counts are derived from updated arrays',
    'this task reclassifies only rne_backward'
)

$derived = Get-InventoryDerived -Inventory $inventory
$inventory.counts = $derived.counts
$inventory.private_launch_methods = $derived.private_launch_methods
$inventory.next_candidate = $derived.next_candidate
$inventory.acceptance_boundary.next_candidate = $derived.next_candidate.entry
$inventory.acceptance_boundary.rne_backward_raw = $false
$inventory.acceptance_boundary.rne_backward_migrated = $true
$inventory.acceptance_boundary.task_gpu_memory_usage = 'unknown'
$inventory.manual_cross_checks = @(
    "all $($derived.counts.actual_cuda_entry_kinds) entry rows are unique",
    "typed entry states total $($derived.counts.typed_actual_entry_kinds)",
    "unmigrated entry states total $($derived.counts.unmigrated_actual_entry_kinds)",
    "all $($derived.counts.product_low_level_call_sites) low-level rows are unique",
    "all $($derived.counts.wrapper_orchestration_call_sites) orchestration rows are unique",
    'all four rne entries have rebuilt current rows',
    'bias overlay is applied to every related detail array',
    'bias overlay source identity and seven line anchors match'
)

$biasCompile = @($inventory.compile_sites | Where-Object entry -CEQ 'rne_bias')
$biasEntry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rne_bias')
$biasLow = @($inventory.product_low_level_call_sites | Where-Object entry -CEQ 'rne_bias')
$biasOuter = @($inventory.wrapper_orchestration_call_sites | Where-Object entry -CEQ 'rne_bias')
$biasFamily = @($inventory.kernel_abi_families | Where-Object logical_name -CEQ 'rne_bias')
if ($biasCompile.Count -ne 1 -or $biasCompile[0].line -ne $overlay.rne_bias.compile_line -or
    $biasEntry.Count -ne 1 -or $biasEntry[0].compile_sites[0] -cne "src/physics/rne_bias.rs`:$($overlay.rne_bias.compile_line)" -or
    $biasLow.Count -ne 1 -or $biasLow[0].line -ne $overlay.rne_bias.low_level_launch_line -or
    $biasOuter.Count -ne 1 -or $biasOuter[0].line -ne $overlay.rne_bias.orchestration_line -or
    $biasFamily.Count -ne 1 -or $biasFamily[0].typed_low_level_calls[0] -cne "src/physics/rne_bias.rs`:$($overlay.rne_bias.low_level_launch_line)") {
    throw '偏置力覆盖层没有写入全量盘点'
}

if ($derived.counts.compile_call_sites -ne 90 -or
    $derived.counts.actual_cuda_entry_kinds -ne 88 -or
    $derived.counts.kernel_abi_families -ne 26 -or
    $derived.counts.typed_actual_entry_kinds -ne 30 -or
    $derived.counts.unmigrated_actual_entry_kinds -ne 58 -or
    $derived.counts.product_low_level_call_sites -ne 99 -or
    $derived.counts.typed_low_level_call_sites -ne 26 -or
    $derived.counts.unmigrated_low_level_call_sites -ne 73 -or
    $derived.counts.wrapper_orchestration_call_sites -ne 28 -or
    $derived.next_candidate.entry -ceq 'rne_backward') {
    throw 'RNE反向盘点派生结果不符'
}

$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema=1
    source_sha256=(Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    prior_sha256=$priorHash
    overlay_sha256=(Get-FileHash -LiteralPath $overlayPath -Algorithm SHA256).Hash.ToLowerInvariant()
    rne_rows=$rows
    counts=$derived.counts
    next_candidate=$derived.next_candidate
} | ConvertTo-Json -Depth 12
