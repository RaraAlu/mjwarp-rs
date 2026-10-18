$ErrorActionPreference = 'Stop'
$source = 'docs/development-evidence/layers-m3-inventory-derived-correction/corrected-inventory.json'
$destination = 'target/normify-evidence/2026-10-19-layers-m3-inventory-derived-correction/check-01-preserve-failure/failed-corrected-inventory.json'
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw '缺少失败快照' }
if (Test-Path -LiteralPath $destination) { throw '失败快照归档已存在' }
[IO.File]::WriteAllBytes($destination, [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $source)))
[ordered]@{
    schema = 1
    source = $source
    destination = $destination
    bytes = (Get-Item -LiteralPath $destination).Length
    sha256 = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
    result = 'pass'
} | ConvertTo-Json
