$ErrorActionPreference = 'Stop'
$task = 'docs/development-evidence/layers-m3-post-freeze-checker-fix'
$manifestPath = "$task/formal-run/manifest.json"
$mapPath = "$task/source-map.json"
$manifestBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $manifestPath))
$manifestHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($manifestBytes)).ToLowerInvariant()
$blob = @(& git hash-object -w --no-filters -- $manifestPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw '清单对象写入失败' }
if (@(& git cat-file -t $blob)[0].Trim() -cne 'blob') { throw '清单身份不是blob' }
if ([int64]@(& git cat-file -s $blob)[0] -ne $manifestBytes.Length) { throw '清单对象长度错误' }
$map = Get-Content -LiteralPath $mapPath -Raw | ConvertFrom-Json -AsHashtable
if ($map.ContainsKey('manifest_identity')) { throw '清单身份已存在' }
$map.manifest_identity = [ordered]@{
    path = $manifestPath
    bytes = $manifestBytes.Length
    sha256 = $manifestHash
    reference_blob = $blob
}
$mapJson = ($map | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $mapPath), $mapJson, [Text.UTF8Encoding]::new($false))
[ordered]@{schema=1;manifest=$manifestPath;bytes=$manifestBytes.Length;sha256=$manifestHash;reference_blob=$blob;result='pass'} | ConvertTo-Json
