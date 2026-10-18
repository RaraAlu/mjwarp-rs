$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-rigid-mass-solve'
$task = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$runtime = "target/normify-evidence/$taskId"
$manifestPath = "$task/formal-run-v3/manifest.json"
$mapPath = "$task/source-map-v3.json"
$postRoot = "$task/post-freeze-check-v3"
if (Test-Path -LiteralPath $postRoot) { throw "收尾证据目录已存在：$postRoot" }

$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$map = Get-Content -LiteralPath $mapPath -Raw | ConvertFrom-Json
if ($manifest.member_count -le 0 -or $map.mappings.Count -ne $manifest.member_count) {
    throw '正式成员数量错误'
}
$manifestBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $manifestPath))
$manifestHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($manifestBytes)).ToLowerInvariant()
if ($map.manifest_identity.sha256 -cne $manifestHash -or
    $map.manifest_identity.bytes -ne $manifestBytes.Length -or
    $map.manifest_identity.reference_blob -notmatch '^[0-9a-f]{40}$') {
    throw '正式清单身份错误'
}
$computedBlob = @(& git hash-object --no-filters -- $manifestPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or $computedBlob -cne $map.manifest_identity.reference_blob) {
    throw '正式清单对象不匹配'
}

$memberByPath = @{}
foreach ($member in $manifest.members) { $memberByPath[$member.path] = $member }
foreach ($mapping in $map.mappings) {
    $member = $memberByPath[$mapping.path]
    if ($null -eq $member) { throw "映射缺少成员：$($mapping.path)" }
    foreach ($path in @((Join-Path $manifest.root $mapping.path), $mapping.source)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "证据原件缺失：$path" }
        if ((Get-Item -LiteralPath $path).Length -ne $member.bytes -or
            (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $member.sha256) {
            throw "证据原件漂移：$path"
        }
    }
}

$infrastructure = @('manifest-v3','manifest-blob-v3')
foreach ($id in $infrastructure) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $runRoot = [string]$registration.output_root
    $result = Get-Content -LiteralPath "$runRoot/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$runRoot/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $result.exit_code -ne 0) {
        throw "基础设施运行证据错误：$id"
    }
}
$selfRegistrationPath = "$runtime/registrations/formal-evidence-retry-02.json"
$selfRegistration = Get-Content -LiteralPath $selfRegistrationPath -Raw | ConvertFrom-Json
$selfStarted = Get-Content -LiteralPath "$runtime/check-13-formal-evidence-retry-02/started.json" -Raw | ConvertFrom-Json
$selfHash = (Get-FileHash -LiteralPath $selfRegistrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ([DateTime]$selfRegistration.registered_at_utc -ge [DateTime]$selfStarted.started_at_utc -or
    $selfStarted.registration_sha256 -cne $selfHash) {
    throw '正式证据检查未事前登记'
}

New-Item -ItemType Directory -Path "$postRoot/infrastructure" | Out-Null
foreach ($id in $infrastructure) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $runRoot = [string]$registration.output_root
    foreach ($item in @(
        @{name="$id.registration.json";source=$registrationPath},
        @{name="$id.started.json";source="$runRoot/started.json"},
        @{name="$id.run.json";source="$runRoot/result.json"},
        @{name="$id.stdout.log";source="$runRoot/stdout.log"},
        @{name="$id.stderr.log";source="$runRoot/stderr.log"}
    )) {
        [IO.File]::WriteAllBytes((Join-Path "$postRoot/infrastructure" $item.name), [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $item.source)))
    }
}
[IO.File]::WriteAllBytes((Join-Path $postRoot 'formal-evidence.registration.json'), [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $selfRegistrationPath)))
[IO.File]::WriteAllBytes((Join-Path $postRoot 'formal-evidence.started.json'), [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath "$runtime/check-13-formal-evidence-retry-02/started.json")))
$result = [ordered]@{
    schema=1
    manifest=$manifestPath
    manifest_sha256=$manifestHash
    manifest_reference_blob=$computedBlob
    verified_members=$manifest.member_count
    original_archive_and_runtime_match=$true
    infrastructure_runs_verified=$infrastructure.Count
    formal_evidence_registered_before_start=$true
    result='pass'
}
$json = ($result | ConvertTo-Json).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'result.json'), $json, [Text.UTF8Encoding]::new($false))
$json
