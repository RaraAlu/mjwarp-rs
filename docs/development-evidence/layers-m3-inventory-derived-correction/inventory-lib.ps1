function Get-RowKey {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Row
    )

    @(
        [string]$Row.file,
        [string]$Row.line,
        [string]$Row.receiver,
        [string]$Row.method
    ) -join '|'
}

function Assert-UniqueValues {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [object[]]$Values
    )

    $duplicates = @($Values | Group-Object | Where-Object Count -gt 1)
    if ($duplicates.Count -ne 0) {
        throw "$Name 存在重复键：$($duplicates[0].Name)"
    }
}

function Assert-KnownValues {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [object[]]$Values,
        [Parameter(Mandatory = $true)]
        [string[]]$Allowed
    )

    $unknown = @($Values | Where-Object { $_ -notin $Allowed } | Select-Object -Unique)
    if ($unknown.Count -ne 0) {
        throw "$Name 存在未知分类：$($unknown -join ',')"
    }
}

function Get-InventoryDerived {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Inventory
    )

    $entries = @($Inventory.actual_cuda_entry_kinds)
    $low = @($Inventory.product_low_level_call_sites)
    $outer = @($Inventory.wrapper_orchestration_call_sites)
    $families = @($Inventory.kernel_abi_families)
    $compileSites = @($Inventory.compile_sites)
    $dynamic = @($Inventory.dynamic_compile_resolutions)
    $excluded = @($Inventory.excluded_non_kernel_helpers)

    Assert-UniqueValues -Name '实际入口' -Values @($entries | ForEach-Object { [string]$_.entry })
    $lowKeys = @($low | ForEach-Object { Get-RowKey $_ })
    $outerKeys = @($outer | ForEach-Object { Get-RowKey $_ })
    Assert-UniqueValues -Name '低层调用' -Values $lowKeys
    Assert-UniqueValues -Name '编排调用' -Values $outerKeys
    Assert-UniqueValues -Name 'ABI族' -Values @($families | ForEach-Object { [string]$_.type })
    $outerSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($key in $outerKeys) { $null = $outerSet.Add($key) }
    $intersection = @($lowKeys | Where-Object { $outerSet.Contains($_) })
    if ($intersection.Count -ne 0) {
        throw "低层调用与编排调用存在身份交集：$($intersection[0])"
    }

    Assert-KnownValues -Name '入口' `
        -Values @($entries | ForEach-Object { [string]$_.migration_status }) `
        -Allowed @('typed_wrapper', 'typed_safe_wrapper', 'unmigrated_or_unwrapped')
    Assert-KnownValues -Name '低层调用' `
        -Values @($low | ForEach-Object { [string]$_.classification }) `
        -Allowed @('typed_wrapper_low_level', 'unmigrated_raw')
    Assert-KnownValues -Name '编排调用' `
        -Values @($outer | ForEach-Object { [string]$_.classification }) `
        -Allowed @('wrapper_orchestration')

    $typedEntries = @($entries | Where-Object migration_status -in @('typed_wrapper', 'typed_safe_wrapper'))
    $rawEntries = @($entries | Where-Object migration_status -ceq 'unmigrated_or_unwrapped')
    if ($rawEntries.Count -eq 0) { throw '未迁移候选为空，不能解释为没有剩余任务' }

    $typedLow = @($low | Where-Object classification -ceq 'typed_wrapper_low_level')
    $rawLow = @($low | Where-Object classification -ceq 'unmigrated_raw')
    $genericTyped = @($typedLow | Where-Object method -ceq 'launch')
    $genericRaw = @($rawLow | Where-Object method -ceq 'launch')
    $unresolvedDynamic = @($dynamic | Where-Object { [string]$_.certainty -notmatch '^exhaustive' })

    $definitionByMethod = @{}
    foreach ($row in @($Inventory.private_launch_methods)) {
        $method = [string]$row.method
        if ($definitionByMethod.ContainsKey($method)) { throw "私有方法定义重复：$method" }
        $definitionByMethod[$method] = [string]$row.definition
    }
    $methodNames = @(
        @($low | ForEach-Object { [string]$_.method }) +
        @($outer | ForEach-Object { [string]$_.method }) |
        Sort-Object -Unique
    )
    $private = @($methodNames | ForEach-Object {
        $method = $_
        if (-not $definitionByMethod.ContainsKey($method)) { throw "缺少私有方法定义：$method" }
        [ordered]@{
            method = $method
            definition = $definitionByMethod[$method]
            typed_low_level_calls = @($typedLow | Where-Object method -ceq $method).Count
            unmigrated_raw_calls = @($rawLow | Where-Object method -ceq $method).Count
            orchestration_calls = @($outer | Where-Object method -ceq $method).Count
        }
    })

    [ordered]@{
        counts = [ordered]@{
            compile_call_sites = $compileSites.Count
            actual_cuda_entry_kinds = $entries.Count
            kernel_abi_families = $families.Count
            typed_actual_entry_kinds = $typedEntries.Count
            unmigrated_actual_entry_kinds = $rawEntries.Count
            product_low_level_call_sites = $low.Count
            typed_low_level_call_sites = $typedLow.Count
            unmigrated_low_level_call_sites = $rawLow.Count
            wrapper_orchestration_call_sites = $outer.Count
            excluded_non_kernel_helpers = $excluded.Count
            dynamic_compile_sites = $dynamic.Count
            unresolved_dynamic_compile_sites = $unresolvedDynamic.Count
        }
        generic_launch = [ordered]@{
            typed_low_level_calls = $genericTyped.Count
            unmigrated_raw_calls = $genericRaw.Count
            orchestration_calls = @($outer | Where-Object method -ceq 'launch').Count
        }
        private_launch_methods = $private
        next_candidate = [ordered]@{
            selected = $false
            entry = [string]$rawEntries[0].entry
            migration_status = [string]$rawEntries[0].migration_status
            compile_sites = @($rawEntries[0].compile_sites)
            reason = 'read-only candidate; no product task started'
        }
    }
}

function Assert-InventoryConsistent {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Inventory,
        [Parameter(Mandatory = $true)]
        [object]$Claims
    )

    $derived = Get-InventoryDerived -Inventory $Inventory
    foreach ($name in $derived.counts.Keys) {
        if ([int64]$Inventory.counts[$name] -ne [int64]$derived.counts[$name]) {
            throw "派生计数漂移：$name"
        }
    }
    $expectedMap = [ordered]@{
        actual_cuda_entry_kinds = 'actual_cuda_entry_kinds'
        typed_actual_entry_kinds = 'typed_actual_entry_kinds'
        unmigrated_actual_entry_kinds = 'unmigrated_actual_entry_kinds'
        product_low_level_call_sites = 'product_low_level_call_sites'
        typed_low_level_call_sites = 'typed_low_level_call_sites'
        unmigrated_low_level_call_sites = 'unmigrated_low_level_call_sites'
        wrapper_orchestration_call_sites = 'wrapper_orchestration_call_sites'
        kernel_abi_families = 'kernel_abi_families'
    }
    foreach ($name in $expectedMap.Keys) {
        $claimName = $expectedMap[$name]
        if ([int64]$derived.counts[$name] -ne [int64]$Claims.expected[$claimName]) {
            throw "预期计数不符：$name"
        }
    }
    if ([int64]$derived.generic_launch.typed_low_level_calls -ne [int64]$Claims.expected.generic_launch_typed_calls -or
        [int64]$derived.generic_launch.unmigrated_raw_calls -ne [int64]$Claims.expected.generic_launch_unmigrated_calls) {
        throw '通用launch汇总不符'
    }
    if ([string]$derived.next_candidate.entry -cne [string]$Claims.expected.next_candidate) {
        throw '下一候选不符'
    }

    $actualPrivate = $Inventory.private_launch_methods | ConvertTo-Json -Depth 8 -Compress
    $derivedPrivate = $derived.private_launch_methods | ConvertTo-Json -Depth 8 -Compress
    if ($actualPrivate -cne $derivedPrivate) { throw '私有方法汇总漂移' }

    [int64]$privateTyped = 0
    [int64]$privateRaw = 0
    [int64]$privateOuter = 0
    foreach ($row in $derived.private_launch_methods) {
        $privateTyped += [int64]$row['typed_low_level_calls']
        $privateRaw += [int64]$row['unmigrated_raw_calls']
        $privateOuter += [int64]$row['orchestration_calls']
    }
    if ($privateTyped -ne $derived.counts.typed_low_level_call_sites -or
        $privateRaw -ne $derived.counts.unmigrated_low_level_call_sites -or
        $privateOuter -ne $derived.counts.wrapper_orchestration_call_sites) {
        throw '私有方法分组未覆盖顶层汇总'
    }

    if ([string]$Inventory.methodology.raw_scan -match 'applied_force\.rs') {
        throw '扫描引用仍指向applied_force旧文件'
    }
    $manual = @($Inventory.manual_cross_checks) -join "`n"
    if ($manual -match '(?i)all fifteen|15 typed|publish is the only|only entry reclassified') {
        throw '交叉核验仍含历史残留'
    }
    if ([string]$Inventory.next_candidate.entry -cne [string]$derived.next_candidate.entry -or
        [string]$Inventory.next_candidate.migration_status -cne 'unmigrated_or_unwrapped') {
        throw '候选筛选未使用实际状态'
    }

    $derived
}
