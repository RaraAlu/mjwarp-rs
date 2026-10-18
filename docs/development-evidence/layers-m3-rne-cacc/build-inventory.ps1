$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-cacc'
$root = 'docs/development-evidence/layers-m3-rne-cacc'
$sourcePath = 'src/physics/rne.rs'
$priorPath = 'docs/development-evidence/layers-m3-rne-cfrc/unmigrated-launch-inventory-v2.json'
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

$compilePattern = [regex]::new(
    'SynchronousKernel::compile\s*\((?<args>.*?)\)\?',
    [Text.RegularExpressions.RegexOptions]::Singleline
)
$rneCompile = [Collections.Generic.List[object]]::new()
foreach ($match in $compilePattern.Matches($product)) {
    $literals = [regex]::Matches($match.Groups['args'].Value, '"(?<entry>rne_[A-Za-z0-9_]+)"')
    if ($literals.Count -eq 0) { continue }
    $literal = $literals[$literals.Count - 1]
    $rneCompile.Add([ordered]@{
        file = $sourcePath
        line = Get-Line ($match.Groups['args'].Index + $literal.Index)
        entry = $literal.Groups['entry'].Value
        expression = $null
    })
}
if ($rneCompile.Count -ne 4) { throw "RNE编译点数量错误：$($rneCompile.Count)" }

$compileSites = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.compile_sites) {
    if ($site.file -ne $sourcePath) { $compileSites.Add($site) }
}
foreach ($site in $rneCompile) { $compileSites.Add($site) }
$inventory.compile_sites = @($compileSites)

foreach ($entry in $inventory.actual_cuda_entry_kinds) {
    $other = @($entry.compile_sites | Where-Object { -not $_.StartsWith("$sourcePath`:") })
    $current = @($rneCompile | Where-Object entry -ceq $entry.entry | ForEach-Object { "$sourcePath`:$($_.line)" })
    $entry.compile_sites = @($other + $current)
    if ($entry.entry -ceq 'rne_cacc') {
        $entry.migration_status = 'typed_safe_wrapper'
        $entry.kernel_abi_evidence = @('RneCaccSpec in src/physics/rne.rs')
    }
}

$caccBlock = Find-One '(?s)impl RneCaccKernel \{(?<body>.*?)\n\}\n\n#\[cfg\(feature = "cuda-probe"\)\]\nstruct RneCfrcLaunch' 'rne_cacc实现'
$caccLow = [regex]::Match($caccBlock.Groups['body'].Value, 'self\.kernel\.launch_rne_cacc\s*\(')
if (-not $caccLow.Success) { throw '找不到rne_cacc低层调用' }
$caccLowIndex = $caccBlock.Groups['body'].Index + $caccLow.Index
$backward = Find-One 'self\.kernels\[0\]\s*\.\s*launch\s*\(' 'rne_backward低层调用'
$cfrcBlock = Find-One '(?s)impl RneCfrcKernel \{(?<body>.*?)\n\}\n\n#\[cfg\(feature = "cuda-probe"\)\]\nstruct RneQfrcLaunch' 'rne_cfrc实现'
$cfrcLow = [regex]::Match($cfrcBlock.Groups['body'].Value, 'self\.kernel\s*\.\s*launch\s*\(metadata\.as_buffer\(\)')
if (-not $cfrcLow.Success) { throw '找不到rne_cfrc低层调用' }
$cfrcLowIndex = $cfrcBlock.Groups['body'].Index + $cfrcLow.Index
$qfrcBlock = Find-One '(?s)impl RneQfrcKernel \{(?<body>.*?)\n\}\n\nimpl RnePlan' 'rne_qfrc实现'
$qfrcLow = [regex]::Match($qfrcBlock.Groups['body'].Value, 'self\.kernel\s*\.\s*launch\s*\(metadata\.as_buffer\(\)')
if (-not $qfrcLow.Success) { throw '找不到rne_qfrc低层调用' }
$qfrcLowIndex = $qfrcBlock.Groups['body'].Index + $qfrcLow.Index
$rneLow = @(
    [ordered]@{file=$sourcePath;line=Get-Line $caccLowIndex;receiver='self.kernel';method='launch_rne_cacc';classification='typed_wrapper_low_level';entry='rne_cacc'},
    [ordered]@{file=$sourcePath;line=Get-Line $backward.Index;receiver='self.kernels[0]';method='launch';classification='unmigrated_raw';entry=$null},
    [ordered]@{file=$sourcePath;line=Get-Line $cfrcLowIndex;receiver='self.kernel';method='launch';classification='typed_wrapper_low_level';entry='rne_cfrc'},
    [ordered]@{file=$sourcePath;line=Get-Line $qfrcLowIndex;receiver='self.kernel';method='launch';classification='typed_wrapper_low_level';entry='rne_qfrc'}
)
$low = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.product_low_level_call_sites) {
    if ($site.file -ne $sourcePath) { $low.Add($site) }
}
foreach ($site in $rneLow) { $low.Add($site) }
$inventory.product_low_level_call_sites = @($low)

$caccOuter = Find-One 'self\.cacc_kernel\s*\.\s*launch\s*\(' 'rne_cacc编排调用'
$cfrcOuter = Find-One 'self\.cfrc_kernel\s*\.\s*launch\s*\(' 'rne_cfrc编排调用'
$qfrcOuter = Find-One 'self\.qfrc_kernel\s*\.\s*launch\s*\(' 'rne_qfrc编排调用'
$outer = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.wrapper_orchestration_call_sites) {
    if ($site.file -ne $sourcePath) { $outer.Add($site) }
}
$outer.Add([ordered]@{
    file=$sourcePath
    line=Get-Line $caccOuter.Index
    receiver='self.cacc_kernel'
    method='launch'
    classification='wrapper_orchestration'
    entry='rne_cacc'
})
$outer.Add([ordered]@{
    file=$sourcePath
    line=Get-Line $cfrcOuter.Index
    receiver='self.cfrc_kernel'
    method='launch'
    classification='wrapper_orchestration'
    entry='rne_cfrc'
})
$outer.Add([ordered]@{
    file=$sourcePath
    line=Get-Line $qfrcOuter.Index
    receiver='self.qfrc_kernel'
    method='launch'
    classification='wrapper_orchestration'
    entry='rne_qfrc'
})
$inventory.wrapper_orchestration_call_sites = @($outer)

$families = [Collections.Generic.List[object]]::new()
foreach ($family in $inventory.kernel_abi_families) {
    if ($family.type -ceq 'RneQfrcSpec') {
        $family.typed_low_level_calls = @("$sourcePath`:$((Get-Line $qfrcLowIndex))")
    }
    if ($family.type -ceq 'RneCfrcSpec') {
        $family.typed_low_level_calls = @("$sourcePath`:$((Get-Line $cfrcLowIndex))")
    }
    $families.Add($family)
}
$families.Add([ordered]@{
    type='RneCaccSpec'
    logical_name='rne_cacc'
    definition='src/physics/rne.rs'
    actual_cuda_entries='rne_cacc'
    typed_low_level_calls=@("$sourcePath`:$((Get-Line $caccLowIndex))")
})
$inventory.kernel_abi_families = @($families)

$inventory.task_id = $task
$inventory.base_commit = '82105430224f30a5880f4ad1c5a465052b97286c'
$inventory.source_file = $priorPath
$inventory.source_file_sha256 = (Get-FileHash -LiteralPath $priorPath -Algorithm SHA256).Hash.ToLowerInvariant()
$inventory.scope = 'prior repository inventory with only rne_cacc arrays reclassified'
$inventory.methodology.source = 'committed prior detail arrays plus current rne source anchors'
$inventory.methodology.raw_scan = 'current rne compile, low-level and orchestration rows are rebuilt before summaries'
$inventory.methodology.prior_inventory = '../layers-m3-rne-cfrc/unmigrated-launch-inventory-v2.json'
$inventory.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'unchanged files inherit committed prior detail rows',
    'all summary counts are derived from updated arrays',
    'this task reclassifies only rne_cacc'
)

$derived = Get-InventoryDerived -Inventory $inventory
$inventory.counts = $derived.counts
$inventory.private_launch_methods = $derived.private_launch_methods
$inventory.next_candidate = $derived.next_candidate
$inventory.acceptance_boundary.next_candidate = $derived.next_candidate.entry
$inventory.acceptance_boundary.rne_cacc_raw = $false
$inventory.acceptance_boundary.rne_cacc_migrated = $true
$inventory.acceptance_boundary.task_gpu_memory_usage = 'unknown'
$inventory.manual_cross_checks = @(
    "all $($derived.counts.actual_cuda_entry_kinds) entry rows are unique",
    "typed entry states total $($derived.counts.typed_actual_entry_kinds)",
    "unmigrated entry states total $($derived.counts.unmigrated_actual_entry_kinds)",
    "all $($derived.counts.product_low_level_call_sites) low-level rows are unique",
    "all $($derived.counts.wrapper_orchestration_call_sites) orchestration rows are unique",
    'rne_cacc has one typed low-level row and one orchestration row'
)

if ($derived.counts.typed_actual_entry_kinds -ne 28 -or
    $derived.counts.unmigrated_actual_entry_kinds -ne 60 -or
    $derived.counts.typed_low_level_call_sites -ne 24 -or
    $derived.counts.unmigrated_low_level_call_sites -ne 75 -or
    $derived.counts.wrapper_orchestration_call_sites -ne 26 -or
    $derived.counts.kernel_abi_families -ne 24 -or
    $derived.next_candidate.entry -cne 'rne_bias') {
    throw 'RNE盘点派生结果不符'
}

$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema=1
    source_sha256=(Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    compile_sites=$rneCompile.Count
    counts=$derived.counts
    next_candidate=$derived.next_candidate
} | ConvertTo-Json -Depth 8
