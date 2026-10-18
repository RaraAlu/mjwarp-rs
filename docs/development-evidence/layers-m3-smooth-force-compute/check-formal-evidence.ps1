$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-smooth-force-compute'
$task = 'docs/development-evidence/layers-m3-smooth-force-compute'
$runtime = "target/normify-evidence/$taskId"
$manifestPath = "$task/formal-run/manifest.json"
$mapPath = "$task/source-map.json"
$postRoot = "$task/post-freeze-check"
if (Test-Path -LiteralPath $postRoot) { throw "收尾证据目录已存在：$postRoot" }

$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$map = Get-Content -LiteralPath $mapPath -Raw | ConvertFrom-Json
if ($manifest.member_count -ne 64 -or $map.mappings.Count -ne 64) { throw '正式成员数量错误' }
$manifestBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $manifestPath))
$manifestHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($manifestBytes)).ToLowerInvariant()
if ($map.manifest_identity.sha256 -cne $manifestHash -or
    $map.manifest_identity.bytes -ne $manifestBytes.Length -or
    $map.manifest_identity.reference_blob -notmatch '^[0-9a-f]{40}$') {
    throw '正式清单身份错误'
}
$computedBlob = @(& git hash-object --no-filters -- $manifestPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or $computedBlob -cne $map.manifest_identity.reference_blob) {
    throw '正式清单blob不匹配'
}

$memberByPath = @{}
foreach ($member in $manifest.members) { $memberByPath[$member.path] = $member }
foreach ($mapping in $map.mappings) {
    $member = $memberByPath[$mapping.path]
    if ($null -eq $member) { throw "映射缺少成员：$($mapping.path)" }
    $archive = Join-Path $manifest.root $mapping.path
    foreach ($path in @($archive, $mapping.source)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "证据原件缺失：$path" }
        if ((Get-Item -LiteralPath $path).Length -ne $member.bytes -or
            (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $member.sha256) {
            throw "证据原件漂移：$path"
        }
    }
}

$infrastructure = @('manifest', 'manifest-blob')
foreach ($id in $infrastructure) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $root = [string]$registration.output_root
    $result = Get-Content -LiteralPath "$root/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$root/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or $started.registration_sha256 -cne $hash -or
        $result.exit_code -ne 0) {
        throw "基础设施运行证据错误：$id"
    }
}
$selfRegistrationPath = "$runtime/registrations/evidence-final.json"
$selfRegistration = Get-Content -LiteralPath $selfRegistrationPath -Raw | ConvertFrom-Json
$selfStarted = Get-Content -LiteralPath "$runtime/check-06-evidence-final/started.json" -Raw | ConvertFrom-Json
$selfHash = (Get-FileHash -LiteralPath $selfRegistrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ([DateTime]$selfRegistration.registered_at_utc -ge [DateTime]$selfStarted.started_at_utc -or
    $selfStarted.registration_sha256 -cne $selfHash) {
    throw '最终证据检查未事前登记'
}

New-Item -ItemType Directory -Path "$postRoot/infrastructure" | Out-Null
foreach ($id in $infrastructure) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $root = [string]$registration.output_root
    foreach ($item in @(
        @{name="$id.registration.json";source=$registrationPath},
        @{name="$id.started.json";source="$root/started.json"},
        @{name="$id.run.json";source="$root/result.json"},
        @{name="$id.stdout.log";source="$root/stdout.log"},
        @{name="$id.stderr.log";source="$root/stderr.log"}
    )) {
        [IO.File]::WriteAllBytes(
            (Join-Path "$postRoot/infrastructure" $item.name),
            [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $item.source))
        )
    }
}
[IO.File]::WriteAllBytes(
    (Join-Path $postRoot 'evidence-final.registration.json'),
    [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $selfRegistrationPath))
)
[IO.File]::WriteAllBytes(
    (Join-Path $postRoot 'evidence-final.started.json'),
    [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath "$runtime/check-06-evidence-final/started.json"))
)

$result = [ordered]@{
    schema = 1
    manifest = $manifestPath
    manifest_sha256 = $manifestHash
    manifest_reference_blob = $computedBlob
    verified_members = $manifest.member_count
    original_archive_and_runtime_match = $true
    infrastructure_runs_verified = $infrastructure.Count
    evidence_final_registered_before_start = $true
    recursive_boundary = 'self post-exit runner files cannot be inputs to this check'
    result = 'pass'
}
$json = ($result | ConvertTo-Json).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'result.json'), $json, [Text.UTF8Encoding]::new($false))
$postMembers = @(Get-ChildItem -LiteralPath $postRoot -Recurse -File | Sort-Object FullName | ForEach-Object {
    [ordered]@{
        path = $_.FullName.Substring((Resolve-Path -LiteralPath $postRoot).Path.Length + 1).Replace('\', '/')
        bytes = $_.Length
        sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$postManifest = [ordered]@{schema=1;root=$postRoot;member_count=$postMembers.Count;members=$postMembers}
$postJson = ($postManifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'manifest.json'), $postJson, [Text.UTF8Encoding]::new($false))
$json
