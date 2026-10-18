$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rigid-mass-solve'
$root = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$sourcePath = 'src/physics/mass_solve.rs'
$priorPath = "$root/unmigrated-launch-inventory-v4.json"
$incorrectPath = "$root/unmigrated-launch-inventory-v3.json"
$outputPath = "$root/unmigrated-launch-inventory-v5.json"
if (Test-Path -LiteralPath $outputPath) { throw "盘点输出已存在：$outputPath" }
if (-not (Test-Path -LiteralPath $incorrectPath -PathType Leaf)) { throw "缺少保留的错误盘点：$incorrectPath" }

. 'docs/development-evidence/layers-m3-inventory-derived-correction/inventory-lib.ps1'
$inventory = Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json -AsHashtable
$priorHash = (Get-FileHash -LiteralPath $priorPath -Algorithm SHA256).Hash.ToLowerInvariant()
$priorBlob = @(& git hash-object --no-filters -- $priorPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or $priorBlob -notmatch '^[0-9a-f]{40}$') { throw '无法计算基线对象' }
$incorrectHash = (Get-FileHash -LiteralPath $incorrectPath -Algorithm SHA256).Hash.ToLowerInvariant()
$incorrectBlob = @(& git hash-object --no-filters -- $incorrectPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or $incorrectBlob -notmatch '^[0-9a-f]{40}$') { throw '无法计算错误盘点对象' }
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

$spec = Find-One $product 'struct RigidMassSolveSpec;' '质量求解规格'
$compile = Find-One $product 'SynchronousKernel::compile\(session, MASS_SOLVE_CUDA, "rigid_mass_solve"\)' '质量求解编译点'
$wrapper = Find-One $product '(?s)impl RigidMassSolveKernel \{(?<body>.*?)\n\}' '质量求解包装实现'
$low = Find-One $wrapper.Groups['body'].Value 'self\.kernel\s*\.launch\s*\(' '质量求解低层调用'
$outer = Find-One $product 'kernel\.launch\(&launch, &metadata, &parameters, state, &mut output\)' '质量求解编排调用'
$rows = [ordered]@{
    spec_line = Get-Line $source $spec.Index
    compile_line = Get-Line $source $compile.Index
    low_line = Get-Line $source ($wrapper.Groups['body'].Index + $low.Index)
    outer_line = Get-Line $source $outer.Index
}

$inventory.compile_sites = @(
    $inventory.compile_sites | Where-Object { $_.entry -cne 'rigid_mass_solve' }
    [ordered]@{file=$sourcePath;line=$rows.compile_line;entry='rigid_mass_solve';expression=$null}
)
$entry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rigid_mass_solve')
if ($entry.Count -ne 1) { throw '质量求解入口不唯一' }
$entry[0].compile_sites = @("$sourcePath`:$($rows.compile_line)")
$entry[0].migration_status = 'typed_safe_wrapper'
$entry[0].kernel_abi_evidence = @("RigidMassSolveSpec in $sourcePath`:$($rows.spec_line)")

$inventory.kernel_abi_families = @(
    $inventory.kernel_abi_families | Where-Object logical_name -CNE 'rigid_mass_solve'
    [ordered]@{
        type='RigidMassSolveSpec'
        logical_name='rigid_mass_solve'
        definition=$sourcePath
        actual_cuda_entries='rigid_mass_solve'
        typed_low_level_calls=@("$sourcePath`:$($rows.low_line)")
    }
)
$inventory.product_low_level_call_sites = @(
    $inventory.product_low_level_call_sites | Where-Object {
        $_.entry -cne 'rigid_mass_solve' -and $_.file -cne $sourcePath
    }
    [ordered]@{
        file=$sourcePath
        line=$rows.low_line
        receiver='self.kernel'
        method='launch'
        classification='typed_wrapper_low_level'
        entry='rigid_mass_solve'
    }
)
$inventory.wrapper_orchestration_call_sites = @(
    $inventory.wrapper_orchestration_call_sites | Where-Object entry -CNE 'rigid_mass_solve'
    [ordered]@{
        file=$sourcePath
        line=$rows.outer_line
        receiver='kernel'
        method='launch'
        classification='wrapper_orchestration'
        entry='rigid_mass_solve'
    }
)

$inventory.task_id = $task
$inventory.inventory_version = 5
$inventory.supersedes = $priorPath
$inventory.base_commit = 'a2d057618d2884234e9b6779e619cce5abfc2adc'
$inventory.source_file = $priorPath
$inventory.source_file_sha256 = $priorHash
$inventory.scope = 'full prior inventory with current rigid_mass_solve rows rebuilt and only that entry reclassified'
$inventory.methodology.source = 'corrected mass-solve v4 inventory plus current mass_solve source anchors'
$inventory.methodology.raw_scan = 'current rigid_mass_solve compile, low-level and orchestration rows rebuilt before summaries'
$inventory.methodology.prior_inventory = 'unmigrated-launch-inventory-v4.json'
$inventory.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'unchanged files inherit committed prior detail rows',
    'all summary counts are derived from updated arrays',
    'this task reclassifies only rigid_mass_solve'
)
$inventory.correction = [ordered]@{
    preserved_incorrect_inventory=$incorrectPath
    preserved_incorrect_sha256=$incorrectHash
    preserved_incorrect_blob=$incorrectBlob
    defect='the v3 entry-only filter retained the prior null-entry raw mass_solve row'
    correction='remove the prior mass_solve file row before adding the current typed row'
}
$derived = Get-InventoryDerived -Inventory $inventory
$inventory.counts = $derived.counts
$inventory.private_launch_methods = $derived.private_launch_methods
$inventory.next_candidate = $derived.next_candidate
$inventory.acceptance_boundary.next_candidate = $derived.next_candidate.entry
$inventory.acceptance_boundary.rigid_mass_solve_raw = $false
$inventory.acceptance_boundary.rigid_mass_solve_migrated = $true
$inventory.acceptance_boundary.task_gpu_memory_usage = 'unknown'
$inventory.manual_cross_checks = @(
    "all $($derived.counts.actual_cuda_entry_kinds) entry rows are unique",
    "typed entry states total $($derived.counts.typed_actual_entry_kinds)",
    "unmigrated entry states total $($derived.counts.unmigrated_actual_entry_kinds)",
    "all $($derived.counts.product_low_level_call_sites) low-level rows are unique",
    "all $($derived.counts.wrapper_orchestration_call_sites) orchestration rows are unique",
    'rigid_mass_solve has rebuilt current rows',
    'prior inventory path, hash and blob are recorded'
)
if ($derived.counts.compile_call_sites -ne 90 -or
    $derived.counts.actual_cuda_entry_kinds -ne 88 -or
    $derived.counts.kernel_abi_families -ne 27 -or
    $derived.counts.typed_actual_entry_kinds -ne 31 -or
    $derived.counts.unmigrated_actual_entry_kinds -ne 57 -or
    $derived.counts.product_low_level_call_sites -ne 99 -or
    $derived.counts.typed_low_level_call_sites -ne 27 -or
    $derived.counts.unmigrated_low_level_call_sites -ne 72 -or
    $derived.counts.wrapper_orchestration_call_sites -ne 29 -or
    $derived.next_candidate.entry -ceq 'rigid_mass_solve') {
    throw '质量求解盘点派生结果不符'
}
$json = ($inventory | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
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
if ($LASTEXITCODE -ne 0 -or $outputBlob -notmatch '^[0-9a-f]{40}$') { throw '无法登记盘点对象' }
[ordered]@{
    schema=1
    output_path=$outputPath
    output_sha256=$outputHash
    output_blob=$outputBlob
    source_sha256=(Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    prior_sha256=$priorHash
    prior_blob=$priorBlob
    preserved_incorrect_sha256=$incorrectHash
    preserved_incorrect_blob=$incorrectBlob
    rows=$rows
    counts=$derived.counts
    next_candidate=$derived.next_candidate
} | ConvertTo-Json -Depth 12
