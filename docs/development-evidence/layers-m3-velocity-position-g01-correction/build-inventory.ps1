$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-velocity-position-g01-correction'
$root = 'docs/development-evidence/layers-m3-velocity-position-g01-correction'
$base = 'f40a78af3947142bb31d587d097e42312594edf3'
$priorPath = 'docs/development-evidence/layers-m3-rigid-mass-matrix/unmigrated-launch-inventory.json'
$sourcePath = 'src/physics/resident.rs'
$outPath = "$root/unmigrated-launch-inventory.json"
if (Test-Path -LiteralPath $outPath) { throw "清单已存在：$outPath" }

function Get-Counts($inv) {
    $entries = @($inv.actual_cuda_entry_kinds)
    $low = @($inv.product_low_level_call_sites)
    [ordered]@{
        compile_call_sites = @($inv.compile_sites).Count
        actual_cuda_entry_kinds = $entries.Count
        kernel_abi_families = @($inv.kernel_abi_families).Count
        typed_actual_entry_kinds = @($entries | Where-Object migration_status -CNE 'unmigrated_or_unwrapped').Count
        unmigrated_actual_entry_kinds = @($entries | Where-Object migration_status -CEQ 'unmigrated_or_unwrapped').Count
        product_low_level_call_sites = $low.Count
        typed_low_level_call_sites = @($low | Where-Object classification -CEQ 'typed_wrapper_low_level').Count
        unmigrated_low_level_call_sites = @($low | Where-Object classification -CEQ 'unmigrated_raw').Count
        wrapper_orchestration_call_sites = @($inv.wrapper_orchestration_call_sites).Count
        excluded_non_kernel_helpers = @($inv.excluded_non_kernel_helpers).Count
        dynamic_compile_sites = @($inv.dynamic_compile_resolutions).Count
        unresolved_dynamic_compile_sites = 0
    }
}

function Get-LaunchCounts($inv, [string]$method) {
    $low = @($inv.product_low_level_call_sites | Where-Object method -CEQ $method)
    [ordered]@{
        typed = @($low | Where-Object classification -CEQ 'typed_wrapper_low_level').Count
        raw = @($low | Where-Object classification -CEQ 'unmigrated_raw').Count
        orchestration = @($inv.wrapper_orchestration_call_sites | Where-Object method -CEQ $method).Count
    }
}

# 只扫描测试模块之前的产品前缀，并去掉行注释。
function Get-Rows([string[]]$Lines) {
    $end = -1
    for ($i = 0; $i -lt $Lines.Length - 1; $i++) {
        if ($Lines[$i] -ceq '#[cfg(test)]' -and $Lines[$i + 1] -ceq 'mod tests {') { $end = $i; break }
    }
    if ($end -lt 0) { throw '未找到测试模块' }
    $compile = [Collections.Generic.List[object]]::new()
    $launch = [Collections.Generic.List[object]]::new()
    for ($i = 0; $i -lt $end; $i++) {
        $code = $Lines[$i] -replace '//.*$', ''
        if ($code -match 'SynchronousKernel::compile\(') {
            # 多行调用按入口字面量所在行登记。
            $k = $i
            $limit = [Math]::Min($i + 5, $end)
            while ($k -lt $limit -and ($Lines[$k] -replace '//.*$', '') -notmatch '"([a-z0-9_]+)"') { $k++ }
            if ($k -ge $limit) { throw "编译点缺少字面入口：$($i + 1)" }
            $compile.Add([ordered]@{ line = $k + 1; entry = $Matches[1] })
        }
        if ($code -match '^\s*\.(launch(?:_[a-z0-9_]+)?)\(') {
            $j = $i - 1
            while ($j -ge 0 -and $Lines[$j].Trim() -eq '') { $j-- }
            $launch.Add([ordered]@{ line = $i + 1; receiver = $Lines[$j].Trim(); method = $Matches[1] })
        }
        elseif ($code -match '([A-Za-z_][A-Za-z0-9_\.]*)\.(launch(?:_[a-z0-9_]+)?)\(' -and $code -notmatch '\bfn\s+launch') {
            $launch.Add([ordered]@{ line = $i + 1; receiver = $Matches[1]; method = $Matches[2] })
        }
    }
    [ordered]@{ compile = $compile; launch = $launch }
}

$priorBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $priorPath))
$priorHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($priorBytes)).ToLowerInvariant()
$priorBlob = @(& git rev-parse "${base}:$priorPath")[0].Trim()
$inv = [Text.Encoding]::UTF8.GetString($priorBytes) | ConvertFrom-Json -Depth 50
$derived = Get-Counts $inv
foreach ($key in $derived.Keys) {
    if ([int64]$inv.counts.$key -ne [int64]$derived[$key]) { throw "基线计数无法复现：$key" }
}

# 扫描器须先逐行复现基线提交的全部resident.rs行。
# 管道读取的行号与源码不符；经cmd重定向保留原始字节。
$baseCopy = "target/normify-evidence/$task/base-resident.rs"
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $baseCopy) | Out-Null
& cmd /c "git show ${base}:$sourcePath > $($baseCopy.Replace('/', '\'))"
if ($LASTEXITCODE -ne 0) { throw '无法读取基线源码' }
$baseLines = [IO.File]::ReadAllLines((Resolve-Path -LiteralPath $baseCopy))
$baseRows = Get-Rows $baseLines
$priorCompile = @($inv.compile_sites | Where-Object file -CEQ $sourcePath)
$priorLow = @($inv.product_low_level_call_sites | Where-Object file -CEQ $sourcePath)
$priorOrchestration = @($inv.wrapper_orchestration_call_sites | Where-Object file -CEQ $sourcePath)
$priorLaunch = @(@($priorLow) + @($priorOrchestration) | Sort-Object { [int]$_.line })
if ($baseRows.compile.Count -ne $priorCompile.Count) { throw '基线编译点数量不符' }
for ($i = 0; $i -lt $priorCompile.Count; $i++) {
    $a = $baseRows.compile[$i]; $b = @($priorCompile | Sort-Object { [int]$_.line })[$i]
    if ($a.line -ne $b.line -or $a.entry -cne $b.entry) { throw "基线编译点不符：$($b.line)" }
}
if ($baseRows.launch.Count -ne $priorLaunch.Count) { throw '基线发射点数量不符' }
for ($i = 0; $i -lt $priorLaunch.Count; $i++) {
    $a = $baseRows.launch[$i]; $b = $priorLaunch[$i]
    if ($a.line -ne $b.line -or $a.receiver -cne $b.receiver -or $a.method -cne $b.method) {
        throw "基线发射点不符：$($b.line)"
    }
}

# 当前源码：只允许删除2211编译点，并把2250接收者改为计划内核。
$current = Get-Rows ([IO.File]::ReadAllLines((Resolve-Path -LiteralPath $sourcePath)))
$expectedCompile = @($priorCompile | Sort-Object { [int]$_.line } | Where-Object { -not ($_.line -eq 2211 -and $_.entry -ceq 'rigid_kinematics') })
if ($current.compile.Count -ne $expectedCompile.Count) { throw '当前编译点数量不符' }
for ($i = 0; $i -lt $expectedCompile.Count; $i++) {
    if ($current.compile[$i].entry -cne $expectedCompile[$i].entry) { throw "当前编译点顺序不符：$i" }
}
if ($current.launch.Count -ne $priorLaunch.Count) { throw '当前发射点数量不符' }
$low = @(); $orchestration = @()
for ($i = 0; $i -lt $priorLaunch.Count; $i++) {
    $a = $current.launch[$i]; $b = $priorLaunch[$i]
    $receiver = $b.receiver
    if ($b.line -eq 2250 -and $b.receiver -ceq 'kernels.rigid') { $receiver = 'self.rigid_kernel' }
    if ($a.receiver -cne $receiver -or $a.method -cne $b.method) { throw "当前发射点不符：$($a.line)" }
    $row = [ordered]@{ file = $sourcePath; line = $a.line; receiver = $a.receiver; method = $a.method; classification = $b.classification; entry = $b.entry }
    if ($b.classification -ceq 'wrapper_orchestration') { $orchestration += $row } else { $low += $row }
}
$compile = @($current.compile | ForEach-Object { [ordered]@{ file = $sourcePath; line = $_.line; entry = $_.entry; expression = $null } })

$inv.compile_sites = @(@($inv.compile_sites | Where-Object file -CNE $sourcePath) + $compile)
$inv.product_low_level_call_sites = @(@($inv.product_low_level_call_sites | Where-Object file -CNE $sourcePath) + $low)
$inv.wrapper_orchestration_call_sites = @(@($inv.wrapper_orchestration_call_sites | Where-Object file -CNE $sourcePath) + $orchestration)
foreach ($entry in @($inv.actual_cuda_entry_kinds)) {
    $sites = @($entry.compile_sites | Where-Object { -not $_.StartsWith("${sourcePath}:", [StringComparison]::Ordinal) })
    $sites += @($compile | Where-Object entry -CEQ $entry.entry | ForEach-Object { "${sourcePath}:$($_.line)" })
    if (@($entry.compile_sites).Count -ne $sites.Count -and $entry.entry -cne 'rigid_kinematics') {
        throw "入口编译点数量变化：$($entry.entry)"
    }
    $entry.compile_sites = @($sites)
}
$rigid = @($inv.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rigid_kinematics')[0]
if (@($rigid.compile_sites).Count -ne 2 -or $rigid.migration_status -cne 'unmigrated_or_unwrapped') { throw 'rigid_kinematics行不符' }
foreach ($family in @($inv.kernel_abi_families)) {
    $family.typed_low_level_calls = @($family.typed_low_level_calls | ForEach-Object {
        if ($_.StartsWith("${sourcePath}:", [StringComparison]::Ordinal)) {
            $old = [int]($_.Split(':')[1])
            $index = [Array]::IndexOf(@($priorLaunch | ForEach-Object { [int]$_.line }), $old)
            if ($index -lt 0) { throw "规格调用行缺失：$_" }
            "${sourcePath}:$($current.launch[$index].line)"
        }
        else { $_ }
    })
}
# 规格定义按struct声明行登记；基线行须先复现。
$currentLines = [IO.File]::ReadAllLines((Resolve-Path -LiteralPath $sourcePath))
function Find-Struct([string[]]$Lines, [string]$Type) {
    $hits = @(for ($i = 0; $i -lt $Lines.Length; $i++) { if ($Lines[$i] -ceq "struct $Type;") { $i + 1 } })
    if ($hits.Count -ne 1) { throw "规格声明不唯一：$Type" }
    $hits[0]
}
foreach ($family in @($inv.kernel_abi_families | Where-Object { $_.definition.StartsWith("${sourcePath}:", [StringComparison]::Ordinal) })) {
    $old = "${sourcePath}:$(Find-Struct $baseLines $family.type)"
    if ($family.definition -cne $old) { throw "基线规格行不符：$($family.type)" }
    $new = "${sourcePath}:$(Find-Struct $currentLines $family.type)"
    $family.definition = $new
    foreach ($entry in @($inv.actual_cuda_entry_kinds)) {
        $entry.kernel_abi_evidence = @($entry.kernel_abi_evidence | ForEach-Object { $_.Replace("$($family.type) at $old", "$($family.type) at $new") })
    }
}
foreach ($method in @($inv.private_launch_methods)) {
    $c = Get-LaunchCounts $inv $method.method
    $method.typed_low_level_calls = $c.typed
    $method.unmigrated_raw_calls = $c.raw
    $method.orchestration_calls = $c.orchestration
}
$inv.counts = [pscustomobject](Get-Counts $inv)
$inv.manual_cross_checks = @(@($inv.manual_cross_checks) + 'resident.rs rows rebuilt by a scanner that first reproduced every base row')
$inv.acceptance_boundary | Add-Member -NotePropertyName velocity_position_strict_compile_removed -NotePropertyValue $true
$inv.acceptance_boundary | Add-Member -NotePropertyName velocity_position_launch_raw -NotePropertyValue $true
$inv.task_id = $task
$inv.base_commit = $base
$inv.source_file = $priorPath
$inv.source_file_sha256 = $priorHash
$inv.source_file_blob = $priorBlob
$inv.scope = 'full prior inventory with all src/physics/resident.rs rows rebuilt; only the strict rigid_kinematics compile site removed'
$inv.methodology.source = 'committed rigid_mass_matrix inventory plus current resident.rs product prefix'
$inv.methodology.raw_scan = 'the same scanner reproduced every base resident.rs row before rebuilding current rows'
$inv.methodology.prior_inventory = '../layers-m3-rigid-mass-matrix/unmigrated-launch-inventory.json'
$inv.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'unchanged files inherit committed prior detail rows',
    'all summary counts are derived from updated arrays after the derivation reproduced the prior counts',
    'this task reclassifies no entry; it removes one compile site and renames one raw launch receiver'
)
$inv.next_candidate.compile_sites = @($rigid.compile_sites)
$inv.next_candidate.reason = 'read-only candidate; remaining sites are the plain probe in mod.rs and the plan kernel in resident.rs, which now also serves velocity-position'

$json = ($inv | ConvertTo-Json -Depth 50).Replace("`r`n", "`n") + "`n"
$bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
$stream = [IO.File]::Open($outPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
[ordered]@{
    inventory = $outPath
    sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
    prior_sha256 = $priorHash
    counts = $inv.counts
} | ConvertTo-Json -Depth 5
