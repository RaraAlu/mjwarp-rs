$ErrorActionPreference = 'Stop'
$scriptPath = 'docs/development-evidence/layers-m3-inventory-derived-correction/check-post-freeze-blobs.ps1'
$evidenceRoot = 'docs/development-evidence/layers-m3-post-freeze-checker-fix/'
$receiptPath = 'docs/development-evidence/changes/2026-10-19-layers-m3-post-freeze-checker-fix.json'
$allowed = @($scriptPath, $receiptPath)

$rows = @(& git status --porcelain=v1 --untracked-files=all)
if ($LASTEXITCODE -ne 0) { throw '无法读取工作区状态' }
foreach ($row in $rows) {
    if ($row.Length -lt 4) { throw "状态行非法：$row" }
    $path = $row.Substring(3).Replace('\', '/')
    if ($path.Contains(' -> ')) { $path = $path.Split(' -> ')[-1] }
    if ($path -notin $allowed -and -not $path.StartsWith($evidenceRoot, [StringComparison]::Ordinal)) {
        throw "范围外变更：$path"
    }
}

$protected = @(
    'docs/development-evidence/layers-m3-inventory-derived-correction/formal-run',
    'docs/development-evidence/layers-m3-inventory-derived-correction/post-freeze-check',
    'docs/development-evidence/layers-m3-inventory-derived-correction/index-check',
    'docs/development-evidence/layers-m3-inventory-derived-correction/corrected-inventory.json',
    'docs/development-evidence/layers-m3-inventory-derived-correction/source-map.json',
    'src',
    'tests',
    'fixtures'
)
foreach ($path in $protected) {
    & git diff --quiet HEAD -- $path
    if ($LASTEXITCODE -eq 1) { throw "受保护范围发生变化：$path" }
    if ($LASTEXITCODE -ne 0) { throw "无法核对受保护范围：$path" }
    $untracked = @(& git ls-files --others --exclude-standard -- $path)
    if ($LASTEXITCODE -ne 0 -or $untracked.Count -ne 0) { throw "受保护范围出现未跟踪文件：$path" }
}

[ordered]@{
    schema = 1
    changed_paths = $rows.Count
    protected_paths = $protected.Count
    product_source_changed = $false
    old_archives_changed = $false
    result = 'pass'
} | ConvertTo-Json
