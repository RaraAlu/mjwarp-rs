$ErrorActionPreference = 'Stop'
$source = 'src/physics/mass_solve.rs'
$root = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$v1 = "$root/pre-validation-current-mass-solve.rs"
$snapshot = "$root/pre-validation-current-mass-solve-v2.rs"
$identity = "$root/pre-validation-source-identity-v2.json"
foreach ($path in @($snapshot, $identity)) {
    if (Test-Path -LiteralPath $path) { throw "冻结输出已存在：$path" }
}
$v1Text = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $v1))
$currentText = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $source))
$marker = "#[cfg(test)]`nmod tests"
$v1Normalized = $v1Text.Replace("`r`n", "`n")
$currentNormalized = $currentText.Replace("`r`n", "`n")
$v1Index = $v1Normalized.IndexOf($marker, [StringComparison]::Ordinal)
$currentIndex = $currentNormalized.IndexOf($marker, [StringComparison]::Ordinal)
if ($v1Index -lt 0 -or $currentIndex -lt 0) { throw '找不到测试模块边界' }
$v1Product = $v1Normalized.Substring(0, $v1Index)
$currentProduct = $currentNormalized.Substring(0, $currentIndex)
if ($v1Product -cne $currentProduct) { throw 'GPU复验后产品源码发生变化' }
$bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $source))
[IO.File]::WriteAllBytes((Join-Path (Get-Location) $snapshot), $bytes)
$sha = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
$blob = @(& git hash-object --no-filters -- $source)[0].Trim()
$productBytes = [Text.UTF8Encoding]::new($false).GetBytes($currentProduct)
$productSha = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($productBytes)).ToLowerInvariant()
$v1Identity = Get-Content -LiteralPath "$root/pre-validation-source-identity.json" -Raw | ConvertFrom-Json
$record = [ordered]@{
    schema=1
    source=$source
    snapshot=$snapshot
    bytes=$bytes.Length
    sha256=$sha
    working_blob=$blob
    gpu_validation_source_identity="$root/pre-validation-source-identity.json"
    gpu_validation_source_sha256=$v1Identity.sha256
    full_file_equal_to_gpu_validation_source=$false
    product_prefix_sha256=$productSha
    product_prefix_equal_to_gpu_validation_source=$true
    difference_scope='tests only after cfg(test) marker'
}
$json = ($record | ConvertTo-Json).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $identity), $json, [Text.UTF8Encoding]::new($false))
$record | ConvertTo-Json
