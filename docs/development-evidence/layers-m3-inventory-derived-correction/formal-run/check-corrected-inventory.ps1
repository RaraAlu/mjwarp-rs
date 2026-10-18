$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-inventory-derived-correction'
. "$root/inventory-lib.ps1"
$inventory = Get-Content -LiteralPath "$root/corrected-inventory.json" -Raw | ConvertFrom-Json -AsHashtable
$claims = Get-Content -LiteralPath "$root/claims.json" -Raw | ConvertFrom-Json -AsHashtable
$derived = Assert-InventoryConsistent -Inventory $inventory -Claims $claims
[ordered]@{
    schema = 1
    counts = $derived.counts
    generic_launch = $derived.generic_launch
    private_launch_methods = $derived.private_launch_methods.Count
    next_candidate = $derived.next_candidate
    result = 'pass'
} | ConvertTo-Json -Depth 8
