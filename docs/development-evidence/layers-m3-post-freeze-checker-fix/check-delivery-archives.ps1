param(
    [Parameter(Mandatory = $true)]
    [string]$Revision
)

$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-post-freeze-checker-fix'
$task = 'docs/development-evidence/layers-m3-post-freeze-checker-fix'
$runtime = "target/normify-evidence/$taskId"
$mappings = [Collections.Generic.List[object]]::new()

function Add-Mapping([string]$Source, [string]$Archive) {
    $mappings.Add([ordered]@{source=$Source;archive=$Archive})
}

$evidenceRegistration = "$runtime/registrations/evidence-final.json"
Add-Mapping $evidenceRegistration "$task/post-freeze-check/evidence-final.registration.json"
Add-Mapping "$runtime/check-06-evidence-final/started.json" "$task/post-freeze-check/evidence-final.started.json"
Add-Mapping "$runtime/check-06-evidence-final/result.json" "$task/post-freeze-check/evidence-final.result.json"
Add-Mapping "$runtime/check-06-evidence-final/stdout.log" "$task/post-freeze-check/evidence-final.stdout.log"
Add-Mapping "$runtime/check-06-evidence-final/stderr.log" "$task/post-freeze-check/evidence-final.stderr.log"
$indexRegistration = "$runtime/registrations/index-bytes.json"
Add-Mapping $indexRegistration "$task/index-check/registration.json"
Add-Mapping "$runtime/check-07-index-bytes/started.json" "$task/index-check/started.json"
Add-Mapping "$runtime/check-07-index-bytes/result.json" "$task/index-check/result.json"
Add-Mapping "$runtime/check-07-index-bytes/stdout.log" "$task/index-check/stdout.log"
Add-Mapping "$runtime/check-07-index-bytes/stderr.log" "$task/index-check/stderr.log"

foreach ($mapping in $mappings) {
    foreach ($path in @($mapping.source, $mapping.archive)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "缺少收尾文件：$path" }
    }
    if ((Get-Item -LiteralPath $mapping.source).Length -ne (Get-Item -LiteralPath $mapping.archive).Length -or
        (Get-FileHash -LiteralPath $mapping.source -Algorithm SHA256).Hash -cne
        (Get-FileHash -LiteralPath $mapping.archive -Algorithm SHA256).Hash) {
        throw "收尾原件与归档不符：$($mapping.archive)"
    }
    $reference = if ($Revision -ceq 'INDEX') { ":$($mapping.archive)" } else { "$Revision`:$($mapping.archive)" }
    $object = @(& git rev-parse $reference)[0].Trim()
    if ($LASTEXITCODE -ne 0 -or $object -notmatch '^[0-9a-f]{40}$') {
        throw "无法解析收尾对象：$($mapping.archive)"
    }
    $workingObject = @(& git hash-object --no-filters -- $mapping.archive)[0].Trim()
    if ($LASTEXITCODE -ne 0 -or $workingObject -cne $object) {
        throw "收尾对象字节不符：$($mapping.archive)"
    }
}

[ordered]@{
    schema = 1
    revision = $Revision
    original_archive_matches = $mappings.Count
    object_matches = $mappings.Count
    result = 'pass'
} | ConvertTo-Json
