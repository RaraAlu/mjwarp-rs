param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,
    [Parameter(Mandatory = $true)]
    [string]$Map,
    [Parameter(Mandatory = $true)]
    [string]$Revision
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Sha256 {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyCollection()]
        [byte[]]$Bytes
    )
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function GitText([string[]]$Arguments) {
    $value = @(& git @Arguments 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "git $($Arguments -join ' ')退出码：$LASTEXITCODE；$($value -join ' ')" }
    ($value -join "`n").Trim()
}

function GitBlob([string]$Object) {
    $info = [Diagnostics.ProcessStartInfo]::new('git')
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.ArgumentList.Add('cat-file')
    $info.ArgumentList.Add('blob')
    $info.ArgumentList.Add($Object)
    $process = [Diagnostics.Process]::new()
    $memory = [IO.MemoryStream]::new()
    try {
        $process.StartInfo = $info
        if (-not $process.Start()) { throw 'git cat-file启动失败' }
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.StandardOutput.BaseStream.CopyTo($memory)
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) {
            throw "git cat-file退出码：$($process.ExitCode)；$($stderr.Result)"
        }
        $bytes = $memory.ToArray()
        Write-Output -NoEnumerate $bytes
    }
    finally {
        $memory.Dispose()
        $process.Dispose()
    }
}

$manifestBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Manifest))
$manifestValue = [Text.Encoding]::UTF8.GetString($manifestBytes) | ConvertFrom-Json
$mapValue = Get-Content -LiteralPath $Map -Raw | ConvertFrom-Json
if ($manifestValue.schema -ne 1 -or $mapValue.schema -ne 1) { throw '证据输入版本无效' }
if ($manifestValue.root -cne $mapValue.archive_root) { throw '归档根目录不匹配' }
$manifestHash = Sha256 -Bytes $manifestBytes
$identity = $mapValue.manifest_identity
if (
    $null -eq $identity -or
    $identity.path -cne $Manifest -or
    $identity.bytes -ne $manifestBytes.Length -or
    $identity.sha256 -cne $manifestHash
) {
    throw '旧清单身份不匹配'
}
$identityNames = @($identity.PSObject.Properties.Name)
$hasRevision = 'reference_revision' -in $identityNames
$hasBlob = 'reference_blob' -in $identityNames
if ($hasRevision -eq $hasBlob) { throw '清单须且只能声明一种不可变身份' }
$referenceRevision = $null
$referenceBlob = $null
if ($hasRevision) {
    $referenceRevision = [string]$identity.reference_revision
    if ($referenceRevision -notmatch '^[0-9a-f]{40}$') { throw '旧提交身份无效' }
    $referenceObject = GitText @('rev-parse', "$referenceRevision`:$Manifest")
    $referenceKind = 'revision'
}
else {
    $referenceBlob = [string]$identity.reference_blob
    if ($referenceBlob -notmatch '^[0-9a-f]{40}$') { throw '清单blob身份无效' }
    if ((GitText @('cat-file', '-t', $referenceBlob)) -cne 'blob') { throw '清单对象不是blob' }
    $referenceObject = $referenceBlob
    $referenceKind = 'blob'
}
$referenceBytes = GitBlob $referenceObject
if (
    $referenceBytes.Length -ne $manifestBytes.Length -or
    (Sha256 -Bytes $referenceBytes) -cne $manifestHash
) {
    throw '旧提交清单偏离当前清单'
}
$manifestItems = @($manifestValue.members)
$mappings = @($mapValue.mappings)
if (
    $manifestItems.Count -eq 0 -or
    $mappings.Count -eq 0 -or
    $manifestItems.Count -ne $manifestValue.member_count -or
    $mappings.Count -ne $manifestItems.Count
) {
    throw '证据成员数量不匹配'
}
$manifestPaths = @{}
foreach ($item in $manifestItems) {
    if ([string]::IsNullOrWhiteSpace($item.path)) { throw '旧清单包含空路径' }
    if ($manifestPaths.ContainsKey($item.path)) { throw "旧清单路径重复：$($item.path)" }
    $manifestPaths[$item.path] = $true
}
$mapByPath = @{}
foreach ($mapping in $mappings) {
    if ([string]::IsNullOrWhiteSpace($mapping.path) -or [string]::IsNullOrWhiteSpace($mapping.source)) {
        throw '原件映射包含空路径'
    }
    if ($mapByPath.ContainsKey($mapping.path)) { throw "原件映射重复：$($mapping.path)" }
    $mapByPath[$mapping.path] = $mapping.source
}
foreach ($path in $mapByPath.Keys) {
    if (-not $manifestPaths.ContainsKey($path)) { throw "原件映射多余：$path" }
}
$rows = [Collections.Generic.List[object]]::new()
$mismatches = [Collections.Generic.List[object]]::new()
foreach ($item in $manifestItems) {
    if (-not $mapByPath.ContainsKey($item.path)) { throw "缺少原件映射：$($item.path)" }
    $archivePath = "$($manifestValue.root)/$($item.path)"
    $sourcePath = [string]$mapByPath[$item.path]
    $archive = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $archivePath))
    $source = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $sourcePath))
    $archiveHash = Sha256 -Bytes $archive
    $sourceHash = Sha256 -Bytes $source
    if ($archive.Length -ne $item.bytes -or $archiveHash -cne $item.sha256) {
        throw "归档偏离旧清单：$($item.path)"
    }
    if ($source.Length -ne $archive.Length -or $sourceHash -cne $archiveHash) {
        throw "现存原件偏离归档：$($item.path)"
    }
    $attributes = @((GitText @('check-attr', 'text', 'diff', '--', $archivePath)) -split "`n")
    if ($attributes.Count -ne 2 -or $attributes[0] -notmatch ': text: unset$' -or $attributes[1] -notmatch ': diff: unset$') {
        throw "文件属性未禁用文本转换：$($item.path)"
    }
    $objectSpec = if ($Revision -ceq 'INDEX') { ":$archivePath" } else { "$Revision`:$archivePath" }
    $object = GitText @('rev-parse', $objectSpec)
    if ($object -notmatch '^[0-9a-f]{40}$') { throw "对象身份无效：$($item.path)" }
    $blob = GitBlob $object
    $blobHash = Sha256 -Bytes $blob
    $isMatch = $blob.Length -eq $archive.Length -and $blobHash -ceq $archiveHash
    $row = [ordered]@{
        path = $item.path
        source = $sourcePath
        source_bytes = $source.Length
        source_sha256 = $sourceHash
        archive_bytes = $archive.Length
        archive_sha256 = $archiveHash
        object = $object
        object_bytes = $blob.Length
        object_sha256 = $blobHash
        matches = $isMatch
    }
    $rows.Add($row)
    if (-not $isMatch) { $mismatches.Add($row) }
}
$result = [ordered]@{
    schema = 1
    revision = $Revision
    manifest = $Manifest
    manifest_bytes = $manifestBytes.Length
    manifest_sha256 = $manifestHash
    manifest_reference_kind = $referenceKind
    manifest_reference_revision = $referenceRevision
    manifest_reference_blob = $referenceBlob
    manifest_reference_object = $referenceObject
    map = $Map
    members = $rows.Count
    matches = $rows.Count - $mismatches.Count
    mismatches = $mismatches.Count
    result = if ($mismatches.Count -eq 0) { 'pass' } else { 'fail' }
    rows = @($rows)
}
$result | ConvertTo-Json -Depth 8
if ($mismatches.Count -ne 0) { exit 1 }
