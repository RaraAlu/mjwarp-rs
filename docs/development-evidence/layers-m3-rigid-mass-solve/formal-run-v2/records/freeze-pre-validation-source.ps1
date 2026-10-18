$ErrorActionPreference = 'Stop'
$source = 'src/physics/mass_solve.rs'
$root = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$snapshot = "$root/pre-validation-current-mass-solve.rs"
$identity = "$root/pre-validation-source-identity.json"
foreach ($path in @($snapshot, $identity)) {
    if (Test-Path -LiteralPath $path) { throw "冻结输出已存在：$path" }
}
$bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $source))
[IO.File]::WriteAllBytes((Join-Path (Get-Location) $snapshot), $bytes)
$sha = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
$blob = @(& git hash-object --no-filters -- $source)[0].Trim()
$headBlob = @(& git rev-parse "HEAD`:$source")[0].Trim()
if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$' -or $headBlob -notmatch '^[0-9a-f]{40}$') {
    throw '源码对象身份解析失败'
}
$record = [ordered]@{
    schema = 1
    source = $source
    snapshot = $snapshot
    bytes = $bytes.Length
    sha256 = $sha
    working_blob = $blob
    base_revision = 'a2d057618d2884234e9b6779e619cce5abfc2adc'
    base_blob = $headBlob
    first_gpu_run_source_snapshot_available = $false
    first_gpu_run_limit = '首轮2世界运行早于三项监督修复，且没有独立源码快照'
}
$json = ($record | ConvertTo-Json).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $identity), $json, [Text.UTF8Encoding]::new($false))
$record | ConvertTo-Json
