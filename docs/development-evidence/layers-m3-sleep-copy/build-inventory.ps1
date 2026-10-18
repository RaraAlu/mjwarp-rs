$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-sleep-copy'
$root = 'docs/development-evidence/layers-m3-sleep-copy'
$sourcePath = 'src/physics/resident.rs'
$priorPath = 'docs/development-evidence/layers-m3-inventory-derived-correction/corrected-inventory.json'
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

function Get-CallLines([string]$Receiver, [string]$Method) {
    $pattern = [regex]::Escape($Receiver) + '\s*\.\s*' + [regex]::Escape($Method) + '\s*\('
    @([regex]::Matches($product, $pattern) | ForEach-Object { Get-Line $_.Index })
}

$compilePattern = [regex]::new(
    'SynchronousKernel::compile\s*\((?<args>.*?)\)\?',
    [Text.RegularExpressions.RegexOptions]::Singleline
)
$residentCompile = [Collections.Generic.List[object]]::new()
foreach ($match in $compilePattern.Matches($product)) {
    $literals = [regex]::Matches($match.Groups['args'].Value, '"(?<entry>[A-Za-z0-9_]+)"')
    if ($literals.Count -eq 0) { continue }
    $literal = $literals[$literals.Count - 1]
    $residentCompile.Add([ordered]@{
        file = $sourcePath
        line = Get-Line ($match.Groups['args'].Index + $literal.Index)
        entry = $literal.Groups['entry'].Value
        expression = $null
    })
}
if ($residentCompile.Count -ne 22) { throw "常驻编译点数量错误：$($residentCompile.Count)" }

$compileSites = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.compile_sites) {
    if ($site.file -ne $sourcePath) { $compileSites.Add($site) }
}
foreach ($site in $residentCompile) { $compileSites.Add($site) }
$inventory.compile_sites = @($compileSites)

foreach ($entry in $inventory.actual_cuda_entry_kinds) {
    $other = @($entry.compile_sites | Where-Object { -not $_.StartsWith("$sourcePath`:") })
    $current = @($residentCompile | Where-Object entry -ceq $entry.entry | ForEach-Object { "$sourcePath`:$($_.line)" })
    $entry.compile_sites = @($other + $current)
    if ($entry.entry -ceq 'sleep_copy') {
        $entry.migration_status = 'typed_safe_wrapper'
        $entry.kernel_abi_evidence = @('SleepCopySpec at src/physics/resident.rs:202')
    }
}

$residentPrior = @($inventory.product_low_level_call_sites | Where-Object {
    $_.file -ceq $sourcePath -and $_.receiver -cne 'device.copy_kernel'
})
$seen = @{}
$residentLow = [Collections.Generic.List[object]]::new()
foreach ($site in $residentPrior) {
    $key = "$($site.receiver)|$($site.method)"
    $occurrence = if ($seen.ContainsKey($key)) { [int]$seen[$key] } else { 0 }
    $lines = @(Get-CallLines $site.receiver $site.method)
    if ($lines.Count -le $occurrence) { throw "找不到调用：$key#$($occurrence + 1)" }
    $seen[$key] = $occurrence + 1
    $residentLow.Add([ordered]@{
        file = $sourcePath
        line = $lines[$occurrence]
        receiver = $site.receiver
        method = $site.method
        classification = $site.classification
        entry = $site.entry
    })
}
$wrapperLow = [regex]::Match(
    $product,
    '\.launch\(metadata\.as_buffer\(\), parameters, state, result, dims\.abi\)'
)
if (-not $wrapperLow.Success) { throw '找不到sleep_copy低层发射' }
$residentLow.Add([ordered]@{
    file = $sourcePath
    line = Get-Line $wrapperLow.Index
    receiver = 'self.kernel'
    method = 'launch'
    classification = 'typed_wrapper_low_level'
    entry = 'sleep_copy'
})
$low = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.product_low_level_call_sites) {
    if ($site.file -ne $sourcePath) { $low.Add($site) }
}
foreach ($site in $residentLow) { $low.Add($site) }
$inventory.product_low_level_call_sites = @($low)

$copyOuter = @(Get-CallLines 'device.copy_kernel' 'launch')
if ($copyOuter.Count -ne 1) { throw "sleep_copy编排调用数量错误：$($copyOuter.Count)" }
$outer = [Collections.Generic.List[object]]::new()
foreach ($site in $inventory.wrapper_orchestration_call_sites) { $outer.Add($site) }
$outer.Add([ordered]@{
    file = $sourcePath
    line = $copyOuter[0]
    receiver = 'device.copy_kernel'
    method = 'launch'
    classification = 'wrapper_orchestration'
    entry = $null
})
$inventory.wrapper_orchestration_call_sites = @($outer)

$families = [Collections.Generic.List[object]]::new()
foreach ($family in $inventory.kernel_abi_families) { $families.Add($family) }
$families.Add([ordered]@{
    type = 'SleepCopySpec'
    logical_name = 'sleep_copy'
    definition = 'src/physics/resident.rs:202'
    actual_cuda_entries = 'sleep_copy'
    typed_low_level_calls = @("$sourcePath`:$((Get-Line $wrapperLow.Index))")
})
$inventory.kernel_abi_families = @($families)

$inventory.task_id = $task
$inventory.base_commit = '15e31190182dd7c58312424851d477d9495b24d6'
$inventory.source_file = $priorPath
$inventory.source_file_sha256 = (Get-FileHash -LiteralPath $priorPath -Algorithm SHA256).Hash.ToLowerInvariant()
$inventory.scope = 'prior repository inventory with current sleep_copy arrays reclassified'
$inventory.methodology.source = 'committed prior detail arrays plus current resident source anchors'
$inventory.methodology.raw_scan = 'current resident compile, low-level and orchestration rows are rebuilt before summaries'
$inventory.methodology.prior_inventory = '../layers-m3-inventory-derived-correction/corrected-inventory.json'
$inventory.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'unchanged files inherit committed prior detail rows',
    'all summary counts are derived from updated arrays',
    'this task reclassifies only sleep_copy'
)

$derived = Get-InventoryDerived -Inventory $inventory
$inventory.counts = $derived.counts
$inventory.private_launch_methods = $derived.private_launch_methods
$inventory.next_candidate = $derived.next_candidate
$inventory.acceptance_boundary.next_candidate = $derived.next_candidate.entry
$inventory.acceptance_boundary.sleep_copy_raw = $false
$inventory.acceptance_boundary.sleep_copy_migrated = $true
$inventory.acceptance_boundary.task_gpu_memory_usage = 'unknown'
$inventory.manual_cross_checks = @(
    "all $($derived.counts.actual_cuda_entry_kinds) entry rows are unique",
    "typed entry states total $($derived.counts.typed_actual_entry_kinds)",
    "unmigrated entry states total $($derived.counts.unmigrated_actual_entry_kinds)",
    "all $($derived.counts.product_low_level_call_sites) low-level rows are unique",
    "all $($derived.counts.wrapper_orchestration_call_sites) orchestration rows are unique",
    'sleep_copy has one typed low-level row and one orchestration row'
)

$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema = 1
    source_sha256 = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    compile_sites = $residentCompile.Count
    sleep_copy_low_level = Get-Line $wrapperLow.Index
    sleep_copy_orchestration = $copyOuter[0]
    counts = $derived.counts
    next_candidate = $derived.next_candidate
} | ConvertTo-Json -Depth 8
