# 逐批更新 M3 未迁移发射清单。
# 以上一版清单为基线，应用一个工作批次补丁，重算派生计数并写出新清单。
# 不覆盖旧清单：输出路径由调用方指定，本脚本只写一次。
param(
    [Parameter(Mandatory = $true)][string]$Prior,
    [Parameter(Mandatory = $true)][string]$Patch,
    [Parameter(Mandatory = $true)][string]$Out
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
Push-Location $root
try {
    if (-not (Test-Path -LiteralPath $Prior)) { throw "基线清单不存在：$Prior" }
    if (Test-Path -LiteralPath $Out) { throw "清单已存在：$Out" }
    $inv = Get-Content $Prior -Raw -Encoding utf8 | ConvertFrom-Json
    $p = Get-Content $Patch -Raw -Encoding utf8 | ConvertFrom-Json

    # 1. 移除被替换的底层调用行。
    $remove = @{}
    foreach ($r in @($p.remove_low_level)) { $remove["$($r.file)|$([int]$r.line)"] = $true }
    if ($remove.Count -gt 0) {
        $inv.product_low_level_call_sites = @($inv.product_low_level_call_sites | Where-Object {
            -not $remove.ContainsKey("$($_.file)|$([int]$_.line)")
        })
    }
    # 2. 追加新的底层调用与编排行。
    foreach ($r in @($p.add_low_level)) {
        $inv.product_low_level_call_sites += [pscustomobject][ordered]@{
            file = $r.file; line = [int]$r.line; receiver = $r.receiver; method = $r.method
            classification = $r.classification; entry = $r.entry
        }
    }
    foreach ($r in @($p.add_orchestration)) {
        $inv.wrapper_orchestration_call_sites += [pscustomobject][ordered]@{
            file = $r.file; line = [int]$r.line; receiver = $r.receiver; method = $r.method
            classification = $r.classification; entry = $r.entry
        }
    }
    # 3. 替换编译点位置。
    foreach ($c in @($p.compile_sites_replace)) {
        $hit = @($inv.compile_sites | Where-Object {
            $_.file -ceq $c.old_file -and [int]$_.line -eq [int]$c.old_line
        })
        if ($hit.Count -ne 1) { throw "编译点替换未命中唯一行：$($c.old_file):$($c.old_line)" }
        $hit[0].file = $c.new_file
        $hit[0].line = [int]$c.new_line
    }
    # 4. 更新入口状态与证据。
    foreach ($e in @($p.entry_status)) {
        $hit = @($inv.actual_cuda_entry_kinds | Where-Object { $_.entry -ceq $e.entry })
        if ($hit.Count -ne 1) { throw "入口未命中：$($e.entry)" }
        if ($null -ne $e.migration_status) { $hit[0].migration_status = $e.migration_status }
        if ($null -ne $e.compile_sites) { $hit[0].compile_sites = @($e.compile_sites) }
        if ($null -ne $e.kernel_abi_evidence) { $hit[0].kernel_abi_evidence = @($e.kernel_abi_evidence) }
    }
    # 5. 追加内核ABI家族。
    foreach ($f in @($p.families_add)) {
        $inv.kernel_abi_families += [pscustomobject][ordered]@{
            type = $f.type
            logical_name = $f.logical_name
            definition = $f.definition
            actual_cuda_entries = $f.actual_cuda_entries
            typed_low_level_calls = @($f.typed_low_level_calls)
        }
    }
    # 6. 派生计数。
    $entries = @($inv.actual_cuda_entry_kinds)
    $low = @($inv.product_low_level_call_sites)
    $inv.counts = [pscustomobject][ordered]@{
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
    # 7. 元数据。
    $priorBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Prior))
    $inv.task_id = '2026-10-18-layers-m3'
    $inv.base_commit = (git rev-parse HEAD).Trim()
    $inv.source_file = $Prior
    $inv.source_file_sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($priorBytes)).ToLowerInvariant()
    $inv.source_file_blob = (& git rev-parse "HEAD:$Prior").Trim()
    $inv.scope = 'full prior inventory plus one migration work batch; unchanged rows keep their positions'
    if ($null -ne $p.next_candidate) { $inv.next_candidate = [pscustomobject]$p.next_candidate }
    if ($null -ne $p.manual_cross_checks_add) {
        $inv.manual_cross_checks = @(@($inv.manual_cross_checks) + @($p.manual_cross_checks_add))
    }
    if ($null -ne $p.acceptance_boundary_set) {
        foreach ($prop in $p.acceptance_boundary_set.PSObject.Properties) {
            $inv.acceptance_boundary | Add-Member -NotePropertyName $prop.Name -NotePropertyValue $prop.Value -Force
        }
    }
    $inv.inventory_version = [int]$inv.inventory_version + 1
    $inv.supersedes = $Prior
    $inv.uncertain = @()
    $json = ($inv | ConvertTo-Json -Depth 50).Replace("`r`n", "`n") + "`n"
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
    $stream = [IO.File]::Open($Out, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
    [ordered]@{ inventory = $Out; version = $inv.inventory_version; counts = $inv.counts } | ConvertTo-Json -Depth 5
}
finally {
    Pop-Location
}
