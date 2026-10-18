$ErrorActionPreference = 'Stop'
$relative = 'docs/development-evidence/2026-10-19-layers-m3-inventory-derived-correction'
$workspace = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath '.').Path).TrimEnd([IO.Path]::DirectorySeparatorChar)
$target = [IO.Path]::GetFullPath((Join-Path $workspace $relative))
$expected = [IO.Path]::GetFullPath("$workspace\docs\development-evidence\2026-10-19-layers-m3-inventory-derived-correction")
if ($target -cne $expected -or -not $target.StartsWith("$workspace\", [StringComparison]::OrdinalIgnoreCase)) {
    throw "错误证据目录越界：$target"
}

& git restore --staged -- $relative
if ($LASTEXITCODE -ne 0) { throw "取消错误暂存失败：$LASTEXITCODE" }
if (Test-Path -LiteralPath $target) {
    Remove-Item -LiteralPath $target -Recurse -Force
}
if (Test-Path -LiteralPath $target) { throw '错误证据目录仍然存在' }
$remaining = @(& git diff --cached --name-only -- $relative)
if ($LASTEXITCODE -ne 0 -or $remaining.Count -ne 0) { throw '错误证据目录仍在暂存区' }
[ordered]@{schema=1;removed=$relative;workspace=$workspace;result='pass'} | ConvertTo-Json
