$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rigid-mass-matrix'
$root = 'docs/development-evidence/layers-m3-rigid-mass-matrix'
$base = '5c05a22b2942b461e8bc8255b0db096a156f3097'
$priorPath = 'docs/development-evidence/layers-m3-rigid-mass-solve/unmigrated-launch-inventory-v5.json'
$sourcePath = 'src/physics/mass_matrix.rs'
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

$priorBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $priorPath))
$priorHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($priorBytes)).ToLowerInvariant()
$priorBlob = @(& git rev-parse "${base}:$priorPath")[0].Trim()
$inv = [Text.Encoding]::UTF8.GetString($priorBytes) | ConvertFrom-Json -Depth 50

# 先核对派生规则能复现基线计数。
$derived = Get-Counts $inv
foreach ($key in $derived.Keys) {
    if ([int64]$inv.counts.$key -ne [int64]$derived[$key]) { throw "基线计数无法复现：$key" }
}
foreach ($method in @($inv.private_launch_methods)) {
    $c = Get-LaunchCounts $inv $method.method
    if ($c.typed -ne $method.typed_low_level_calls -or $c.raw -ne $method.unmigrated_raw_calls -or
        $c.orchestration -ne $method.orchestration_calls) {
        throw "基线发射方法计数无法复现：$($method.method)"
    }
}

# 只扫描测试模块之前的产品前缀。
$lines = [IO.File]::ReadAllLines((Resolve-Path -LiteralPath $sourcePath))
$testLine = -1
for ($i = 0; $i -lt $lines.Length - 1; $i++) {
    if ($lines[$i] -ceq '#[cfg(test)]' -and $lines[$i + 1] -ceq 'mod tests {') { $testLine = $i; break }
}
if ($testLine -lt 0) { throw '未找到测试模块' }
$compile = @(); $low = @(); $orchestration = @(); $spec = $null
for ($i = 0; $i -lt $testLine; $i++) {
    $line = $lines[$i]; $number = $i + 1
    $code = ($line -replace '//.*$', '')
    if ($code -match 'SynchronousKernel::compile\(.*"([a-z0-9_]+)"') {
        $compile += [ordered]@{ file = $sourcePath; line = $number; entry = $Matches[1]; expression = $null }
    }
    if ($code -match 'unsafe impl KernelAbi for RigidMassMatrixSpec') { $spec = $number }
    if ($code -match '^\s*self\.kernel\.launch\(') {
        $low += [ordered]@{ file = $sourcePath; line = $number; receiver = 'self.kernel'; method = 'launch'; classification = 'typed_wrapper_low_level'; entry = 'rigid_mass_matrix' }
    }
    elseif ($code -match '^\s*kernel\.launch\(&launch') {
        $orchestration += [ordered]@{ file = $sourcePath; line = $number; receiver = 'kernel'; method = 'launch'; classification = 'wrapper_orchestration'; entry = 'rigid_mass_matrix' }
    }
    elseif ($code -match '\.launch(_[a-z_]+)?\(' -and $code -notmatch 'fn launch') {
        throw "未分类的发射调用：${sourcePath}:$number"
    }
}
if ($compile.Count -ne 1 -or $compile[0].entry -cne 'rigid_mass_matrix' -or $low.Count -ne 1 -or
    $orchestration.Count -ne 1 -or $null -eq $spec) {
    throw '质量矩阵产品行数不符'
}

# 先删除该文件的全部旧行，再加入当前行。
$inv.compile_sites = @(@($inv.compile_sites | Where-Object file -CNE $sourcePath) + $compile)
$inv.product_low_level_call_sites = @(@($inv.product_low_level_call_sites | Where-Object file -CNE $sourcePath) + $low)
$inv.wrapper_orchestration_call_sites = @(@($inv.wrapper_orchestration_call_sites | Where-Object file -CNE $sourcePath) + $orchestration)
$entry = @($inv.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rigid_mass_matrix')
if ($entry.Count -ne 1 -or $entry[0].migration_status -cne 'unmigrated_or_unwrapped') { throw '基线入口状态不符' }
$entry[0].compile_sites = @("${sourcePath}:$($compile[0].line)")
$entry[0].migration_status = 'typed_safe_wrapper'
$entry[0].kernel_abi_evidence = @("RigidMassMatrixSpec in ${sourcePath}:$spec")
if (@($inv.kernel_abi_families | Where-Object type -CEQ 'RigidMassMatrixSpec').Count -ne 0) { throw '规格已存在' }
$inv.kernel_abi_families = @(@($inv.kernel_abi_families) + [pscustomobject][ordered]@{
    type = 'RigidMassMatrixSpec'
    logical_name = 'rigid_mass_matrix'
    definition = "${sourcePath}:$spec"
    actual_cuda_entries = 'rigid_mass_matrix'
    typed_low_level_calls = @("${sourcePath}:$($low[0].line)")
})
foreach ($method in @($inv.private_launch_methods)) {
    $c = Get-LaunchCounts $inv $method.method
    $method.typed_low_level_calls = $c.typed
    $method.unmigrated_raw_calls = $c.raw
    $method.orchestration_calls = $c.orchestration
}
$inv.counts = [pscustomobject](Get-Counts $inv)
$inv.manual_cross_checks = @(@($inv.manual_cross_checks) + 'rigid_mass_matrix has rebuilt current rows')
$inv.acceptance_boundary | Add-Member -NotePropertyName rigid_mass_matrix_raw -NotePropertyValue $false
$inv.acceptance_boundary | Add-Member -NotePropertyName rigid_mass_matrix_migrated -NotePropertyValue $true
$inv.acceptance_boundary.next_candidate = 'rigid_kinematics'
$inv.task_id = $task
$inv.base_commit = $base
$inv.source_file = $priorPath
$inv.source_file_sha256 = $priorHash
$inv | Add-Member -NotePropertyName source_file_blob -NotePropertyValue $priorBlob -Force
$inv.scope = 'full prior inventory with all current src/physics/mass_matrix.rs rows rebuilt and only rigid_mass_matrix reclassified'
$inv.methodology.source = 'committed rigid_mass_solve v5 inventory plus current mass_matrix product prefix'
$inv.methodology.raw_scan = 'every src/physics/mass_matrix.rs row is removed before current compile, low-level and orchestration rows are added'
$inv.methodology.prior_inventory = '../layers-m3-rigid-mass-solve/unmigrated-launch-inventory-v5.json'
$inv.methodology.limitations = @(
    'lexical scan does not perform Rust type inference',
    'unchanged files inherit committed prior detail rows',
    'all summary counts are derived from updated arrays after the derivation reproduced the prior counts',
    'this task reclassifies only rigid_mass_matrix'
)
$inv.correction = $null
$inv.next_candidate = [pscustomobject][ordered]@{
    selected = $false
    entry = 'rigid_kinematics'
    migration_status = 'unmigrated_or_unwrapped'
    compile_sites = @('src/physics/mod.rs:367', 'src/physics/resident.rs:1590', 'src/physics/resident.rs:2211')
    reason = 'read-only candidate; three compile sites use different preprocessor variants and need separate scoping'
}

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
