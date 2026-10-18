param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('duplicate-entry', 'unknown-classification', 'derived-drift', 'empty-candidate', 'cross-classification', 'cross-set', 'private-summary-drift')]
    [string]$Case
)

$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-inventory-derived-correction'
. "$root/inventory-lib.ps1"
$inventory = Get-Content -LiteralPath "$root/corrected-inventory.json" -Raw | ConvertFrom-Json -AsHashtable
$claims = Get-Content -LiteralPath "$root/claims.json" -Raw | ConvertFrom-Json -AsHashtable

$expected = switch ($Case) {
    'duplicate-entry' {
        $inventory.actual_cuda_entry_kinds = @($inventory.actual_cuda_entry_kinds) + @($inventory.actual_cuda_entry_kinds[0])
        '重复键'
    }
    'unknown-classification' {
        $inventory.actual_cuda_entry_kinds[0].migration_status = 'unknown_state'
        '未知分类'
    }
    'derived-drift' {
        $inventory.counts.typed_actual_entry_kinds = [int]$inventory.counts.typed_actual_entry_kinds + 1
        '派生计数漂移'
    }
    'empty-candidate' {
        foreach ($entry in $inventory.actual_cuda_entry_kinds) {
            if ($entry.migration_status -ceq 'unmigrated_or_unwrapped') {
                $entry.migration_status = 'typed_wrapper'
            }
        }
        '候选为空'
    }
    'cross-classification' {
        $copy = $inventory.product_low_level_call_sites[0] | ConvertTo-Json -Depth 8 | ConvertFrom-Json -AsHashtable
        $copy.classification = if ($copy.classification -ceq 'unmigrated_raw') {
            'typed_wrapper_low_level'
        }
        else {
            'unmigrated_raw'
        }
        $inventory.product_low_level_call_sites = @($inventory.product_low_level_call_sites) + @($copy)
        '重复键'
    }
    'cross-set' {
        $copy = $inventory.product_low_level_call_sites[0] | ConvertTo-Json -Depth 8 | ConvertFrom-Json -AsHashtable
        $copy.classification = 'wrapper_orchestration'
        $inventory.wrapper_orchestration_call_sites = @($inventory.wrapper_orchestration_call_sites) + @($copy)
        '身份交集'
    }
    'private-summary-drift' {
        $launch = @($inventory.private_launch_methods | Where-Object method -ceq 'launch')
        if ($launch.Count -ne 1) { throw '通用launch汇总不唯一' }
        $launch[0].typed_low_level_calls = 10
        $launch[0].unmigrated_raw_calls = 69
        $launch[0].orchestration_calls = 15
        '私有方法汇总漂移'
    }
}

$rejected = $false
$message = $null
try {
    $null = Assert-InventoryConsistent -Inventory $inventory -Claims $claims
}
catch {
    $rejected = $true
    $message = $_.Exception.Message
}
if (-not $rejected) { throw "负例没有被拒绝：$Case" }
if ($message -notmatch [regex]::Escape($expected)) { throw "负例拒绝原因错误：$message" }
[ordered]@{schema=1;case=$Case;rejected=$true;message=$message;result='pass'} | ConvertTo-Json
