$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-post-freeze-checker-fix'
$task = 'docs/development-evidence/layers-m3-post-freeze-checker-fix'
$runtime = "target/normify-evidence/$taskId"
$manifestPath = "$task/formal-run/manifest.json"
$mapPath = "$task/source-map.json"
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$map = Get-Content -LiteralPath $mapPath -Raw | ConvertFrom-Json
if ($manifest.member_count -le 0 -or $map.mappings.Count -ne $manifest.member_count) {
    throw '正式成员数量错误'
}
$manifestBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $manifestPath))
$manifestHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($manifestBytes)).ToLowerInvariant()
$computedBlob = @(& git hash-object --no-filters -- $manifestPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or
    $map.manifest_identity.sha256 -cne $manifestHash -or
    $map.manifest_identity.bytes -ne $manifestBytes.Length -or
    $map.manifest_identity.reference_blob -cne $computedBlob) {
    throw '正式清单身份错误'
}

$memberByPath = @{}
foreach ($member in $manifest.members) { $memberByPath[$member.path] = $member }
foreach ($mapping in $map.mappings) {
    $member = $memberByPath[$mapping.path]
    if ($null -eq $member) { throw "映射缺少成员：$($mapping.path)" }
    foreach ($path in @((Join-Path $manifest.root $mapping.path), $mapping.source)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
            (Get-Item -LiteralPath $path).Length -ne $member.bytes -or
            (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $member.sha256) {
            throw "证据原件漂移：$path"
        }
    }
}

$formalRunIds = @(
    'delivery-preflight',
    'static-mapping',
    'prior-commit-objects',
    'static-mapping-retry-01',
    'prior-commit-objects-retry-01',
    'scope'
)
$expectedExit = @{'prior-commit-objects'=1}
foreach ($id in $formalRunIds) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $root = [string]$registration.output_root
    $result = Get-Content -LiteralPath "$root/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$root/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        $result.exit_code -ne $(if ($expectedExit.ContainsKey($id)) { $expectedExit[$id] } else { 0 })) {
        throw "正式运行登记错误：$id"
    }
}
foreach ($id in @('manifest', 'manifest-blob')) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $root = [string]$registration.output_root
    $result = Get-Content -LiteralPath "$root/result.json" -Raw | ConvertFrom-Json
    if ($result.exit_code -ne 0) { throw "基础设施运行失败：$id" }
}
$selfRegistrationPath = "$runtime/registrations/evidence-final.json"
$selfRegistration = Get-Content -LiteralPath $selfRegistrationPath -Raw | ConvertFrom-Json
$selfStarted = Get-Content -LiteralPath "$runtime/check-06-evidence-final/started.json" -Raw | ConvertFrom-Json
$selfHash = (Get-FileHash -LiteralPath $selfRegistrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ([DateTime]$selfRegistration.registered_at_utc -ge [DateTime]$selfStarted.started_at_utc -or
    $selfStarted.registration_sha256 -cne $selfHash) {
    throw '最终证据检查未事前登记'
}

[ordered]@{
    schema = 1
    manifest_sha256 = $manifestHash
    manifest_reference_blob = $computedBlob
    verified_members = $manifest.member_count
    formal_runs_verified = $formalRunIds.Count
    temporary_fix_history_recorded = $true
    result = 'pass'
} | ConvertTo-Json
