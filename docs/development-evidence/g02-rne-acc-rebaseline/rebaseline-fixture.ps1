# 离线证据工具：把 rne-acceleration 期望值重基线到冻结上游 MJWarp f32 参考。
# 产品链（构建/测试/脚本）不读取本工具；仅证据生成与复核使用。
# 用法：
#   pwsh -NoProfile -File rebaseline-fixture.ps1                 # 预演，写到 target/rebaseline/out
#   pwsh -NoProfile -File rebaseline-fixture.ps1 -Apply          # 落盘 fixtures + manifest + 证据
#   pwsh -NoProfile -File rebaseline-fixture.ps1 -Apply -VerifyTwice
#     # -VerifyTwice：连续生成两次并逐字节比对（证明确定性），第二次不写盘
param(
    [string]$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path,
    [switch]$Apply,
    [switch]$VerifyTwice,
    [string]$OutputDirectory = 'target/rebaseline/out'
)
$ErrorActionPreference = 'Stop'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$invariant = [System.Globalization.CultureInfo]::InvariantCulture
$models = @('mixed-joints', 'inertial-tree', 'rotated-tree', 'zero-dof', 'massless-tree')
$upstreamEvidence = 'docs/development-evidence/g02-rne-authoring-diff/upstream-rne.json'
$upstreamProvenance = 'docs/development-evidence/g02-rne-authoring-diff/provenance.json'
$upstreamSha256 = '42F7613021CFBFD8B34D271B7A3135CA7DDFD702A3C5511B5749D38577E21AB7'
$evidenceDirectory = 'docs/development-evidence/g02-rne-acc-rebaseline'
$historicalDirectory = "$evidenceDirectory/historical-f64"
$toolPath = "$evidenceDirectory/rebaseline-fixture.ps1"
$tolerances = [ordered]@{ absolute = 2e-05; relative = 2e-05 }

function Read-Json([string]$path) {
    return (Get-Content -LiteralPath $path -Raw -Encoding utf8 | ConvertFrom-Json)
}
function Hash-File([string]$path) {
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}
function Format-F32([string]$hex) {
    $bits = [Convert]::ToInt32($hex, 16)
    $single = [BitConverter]::Int32BitsToSingle($bits)
    # 期望值按 f32 位串原样存档：转成 double 后取最短往返表示，解析回 f64 即等于该 f32 值。
    return ([double]$single).ToString('R', $invariant)
}
function New-QfrcText($recordTable, [int]$caseIndex, [int]$nv) {
    $flags = New-Object System.Collections.Generic.List[string]
    foreach ($flag in @($false, $true)) {
        $gravities = New-Object System.Collections.Generic.List[string]
        for ($g = 0; $g -lt 3; $g++) {
            $key = "$caseIndex|$g|$flag"
            if (-not $recordTable.ContainsKey($key)) { throw "missing upstream record $key" }
            $hex = @($recordTable[$key])
            if ($hex.Count -ne $nv) { throw "record $key has $($hex.Count) values, fixture nv=$nv" }
            $values = New-Object System.Collections.Generic.List[string]
            foreach ($h in $hex) { $values.Add((Format-F32 $h)) }
            $gravities.Add('[' + ($values -join ',') + ']')
        }
        $flags.Add('[' + ($gravities -join ',') + ']')
    }
    return '[' + ($flags -join ',') + ']'
}
function Replace-QfrcValues([string]$text, [string[]]$values) {
    $builder = New-Object System.Text.StringBuilder
    $position = 0
    $replaced = 0
    while ($true) {
        $index = $text.IndexOf('"qfrc":', $position, [StringComparison]::Ordinal)
        if ($index -lt 0) { break }
        $valueStart = $index + 7
        if ($text[$valueStart] -ne '[') { throw "qfrc value at $valueStart does not start with [" }
        $depth = 0
        $valueEnd = -1
        for ($i = $valueStart; $i -lt $text.Length; $i++) {
            $c = $text[$i]
            if ($c -eq '[') { $depth++ }
            elseif ($c -eq ']') {
                $depth--
                if ($depth -eq 0) { $valueEnd = $i; break }
            }
        }
        if ($valueEnd -lt 0) { throw "unbalanced qfrc value at $valueStart" }
        [void]$builder.Append($text.Substring($position, $valueStart - $position))
        [void]$builder.Append($values[$replaced])
        $position = $valueEnd + 1
        $replaced++
    }
    if ($replaced -ne $values.Count) { throw "replaced $replaced qfrc values, expected $($values.Count)" }
    [void]$builder.Append($text.Substring($position))
    return $builder.ToString()
}
function Compare-Inputs($before, $after, [string]$name) {
    foreach ($caseIndex in 0..($before.cases.Count - 1)) {
        foreach ($key in @('id', 'qpos', 'qvel', 'qacc')) {
            $a = $before.cases[$caseIndex].$key | ConvertTo-Json -Depth 5 -Compress
            $b = $after.cases[$caseIndex].$key | ConvertTo-Json -Depth 5 -Compress
            if ($a -ne $b) { throw "$name case $caseIndex input $key changed by rebaseline" }
        }
    }
    foreach ($key in @('native_version', 'seed', 'absolute_tolerance', 'relative_tolerance', 'nq', 'nv', 'nbody', 'gravities')) {
        $a = $before.$key | ConvertTo-Json -Depth 5 -Compress
        $b = $after.$key | ConvertTo-Json -Depth 5 -Compress
        if ($a -ne $b) { throw "$name header $key changed by rebaseline" }
    }
}

$repo = (Resolve-Path -LiteralPath $Repo).Path
$upstreamPath = Join-Path $repo $upstreamEvidence
$actualUpstreamSha = Hash-File $upstreamPath
if ($actualUpstreamSha -ne $upstreamSha256.ToLowerInvariant()) {
    throw "upstream evidence hash mismatch: $actualUpstreamSha != $upstreamSha256 (refuse to rebaseline)"
}
$upstream = Read-Json $upstreamPath
$provenance = Read-Json (Join-Path $repo $upstreamProvenance)
$recordTables = @{}
foreach ($name in $models) {
    $records = @($upstream.records.$name)
    if ($records.Count -ne 48) { throw "$name upstream records=$($records.Count), expected 48" }
    $table = @{}
    foreach ($record in $records) {
        if ($record.model -ne $name) { throw "record model mismatch $($record.model) != $name" }
        $key = "$($record.state)|$($record.gravity)|$($record.flag)"
        if ($table.ContainsKey($key)) { throw "duplicate upstream record $name $key" }
        $table[$key] = @($record.qfrc_bias_hex)
    }
    $recordTables[$name] = $table
}

$outDirectory = if ($Apply) { $null } else { Join-Path $repo $OutputDirectory }
if ($outDirectory) { New-Item -ItemType Directory -Force -Path $outDirectory | Out-Null }
$fixtures = New-Object System.Collections.Generic.List[object]
$newHashes = @{}
foreach ($name in $models) {
    $relative = "fixtures/rne-acceleration/$name.json"
    $path = Join-Path $repo $relative
    $originalText = [System.IO.File]::ReadAllText($path, $utf8)
    $original = $originalText | ConvertFrom-Json
    $cases = @($original.cases)
    if ($cases.Count -ne 8) { throw "$name cases=$($cases.Count), expected 8" }
    $nv = [int]$original.nv
    $replacement = New-Object System.Collections.Generic.List[string]
    for ($caseIndex = 0; $caseIndex -lt 8; $caseIndex++) {
        $replacement.Add((New-QfrcText $recordTables[$name] $caseIndex $nv))
    }
    $newText = Replace-QfrcValues $originalText $replacement
    $newObject = $newText | ConvertFrom-Json
    Compare-Inputs $original $newObject $name
    # 逐条核对：新期望值必须与上游 f32 位串精确一致。
    for ($caseIndex = 0; $caseIndex -lt 8; $caseIndex++) {
        for ($flag = 0; $flag -lt 2; $flag++) {
            for ($g = 0; $g -lt 3; $g++) {
                $expected = @($newObject.cases[$caseIndex].qfrc[$flag][$g])
                $hex = @($recordTables[$name]["$caseIndex|$g|$([bool]$flag)"])
                if ($expected.Count -ne $hex.Count) { throw "$name qfrc [$flag][$g] size mismatch" }
                for ($d = 0; $d -lt $hex.Count; $d++) {
                    $single = [BitConverter]::Int32BitsToSingle([Convert]::ToInt32($hex[$d], 16))
                    if ([double]$expected[$d] -ne [double]$single) {
                        throw "$name case $caseIndex flag $flag gravity $g dof ${d}: $($expected[$d]) != $single"
                    }
                }
            }
        }
    }
    $oldHash = Hash-File $path
    $target = if ($Apply) { $path } else { Join-Path $outDirectory "$name.json" }
    if ($Apply -and -not $script:archived) {
        New-Item -ItemType Directory -Force -Path (Join-Path $repo $historicalDirectory) | Out-Null
        foreach ($archiveName in $models) {
            Copy-Item -LiteralPath (Join-Path $repo "fixtures/rne-acceleration/$archiveName.json") `
                -Destination (Join-Path $repo "$historicalDirectory/$archiveName.json") -Force
        }
        $script:archived = $true
    }
    [System.IO.File]::WriteAllText($target, $newText, $utf8)
    $newHash = Hash-File $target
    $newHashes[$relative] = $newHash
    $fixtures.Add([pscustomobject]@{ path = $relative; old_sha256 = $oldHash; new_sha256 = $newHash; changed = ($oldHash -ne $newHash) })
}

$manifestRelative = 'fixtures/rne-acceleration/manifest.json'
$manifestPath = Join-Path $repo $manifestRelative
$manifestText = [System.IO.File]::ReadAllText($manifestPath, $utf8)
$manifest = $manifestText | ConvertFrom-Json
$manifestChanges = New-Object System.Collections.Generic.List[object]
foreach ($row in @($manifest.files)) {
    if (-not $newHashes.ContainsKey($row.path)) { continue }
    $oldHash = $row.sha256
    $newHash = $newHashes[$row.path]
    if ($oldHash -eq $newHash) { continue }
    $occurrences = ([regex]::Matches($manifestText, [regex]::Escape($oldHash))).Count
    if ($occurrences -ne 1) { throw "manifest hash $oldHash occurs $occurrences times, expected 1" }
    $manifestText = $manifestText.Replace($oldHash, $newHash)
    $manifestChanges.Add([pscustomobject]@{ path = $row.path; old_sha256 = $oldHash; new_sha256 = $newHash })
}
$newScope = 'new qacc RNE auxiliary; qfrc expected values rebaselined to frozen upstream MJWarp f32 reference (71da24d…); historical native f64 reference archived, not an expectation; not full G02 acceptance'
$oldScopeMatch = [regex]::Match($manifestText, '"scope": "[^"]*"')
if (-not $oldScopeMatch.Success) { throw 'manifest scope field not found' }
$manifestText = $manifestText.Replace($oldScopeMatch.Value, '"scope": ' + (ConvertTo-Json $newScope -Compress))
$provenanceBlock = [ordered]@{
    expected_reference = [ordered]@{
        kind = 'frozen-upstream-mjwarp-f32-rne-acceleration'
        upstream_commit = $provenance.upstream_revision
        evidence = $upstreamEvidence
        evidence_sha256 = $actualUpstreamSha
        authoring_script_py_sha256 = $provenance.script_sha256
        authoring_script_ps1_sha256 = (Hash-File (Join-Path $repo 'scripts/authoring/author-g02-rne-acceleration.ps1'))
        authoring_environment = $provenance.versions
        deterministic_runs = $provenance.deterministic_runs
        tolerance_unchanged = $true
        decision = 'user 2026-10-23 option A: rebaseline expected values to frozen upstream MJWarp f32 reference; tolerance unchanged; product unchanged'
        rebaseline_tool = $toolPath
    }
    historical_native_f64_reference = [ordered]@{
        path = $historicalDirectory
        status = 'historical native MuJoCo 3.12 f64 reference only; no longer the expectation'
        source = 'fixtures/rne-acceleration/reference.cpp (native MuJoCo 3.12 f64)'
        superseded_by = 'frozen upstream MJWarp f32 reference'
    }
}
$blockJson = '  "expected_reference": ' + (ConvertTo-Json $provenanceBlock.expected_reference -Depth 6 -Compress) + ",`n" +
    '  "historical_native_f64_reference": ' + (ConvertTo-Json $provenanceBlock.historical_native_f64_reference -Depth 6 -Compress) + ','
$anchor = '"native_version": 3012000,'
$anchorIndex = $manifestText.IndexOf($anchor, [StringComparison]::Ordinal)
if ($anchorIndex -lt 0) { throw 'manifest native_version anchor not found' }
$insertAt = $anchorIndex + $anchor.Length
$manifestText = $manifestText.Insert($insertAt, "`n" + $blockJson)
$manifestTarget = if ($Apply) { $manifestPath } else { Join-Path $outDirectory 'manifest.json' }
[System.IO.File]::WriteAllText($manifestTarget, $manifestText, $utf8)

$report = [ordered]@{
    schema = 1
    kind = 'rne-acceleration-fixture-rebaseline'
    decision = 'user 2026-10-23 option A (rebaseline expected values to frozen upstream MJWarp f32 reference)'
    upstream_commit = $provenance.upstream_revision
    upstream_evidence = $upstreamEvidence
    upstream_evidence_sha256 = $actualUpstreamSha
    rebaseline_tool = $toolPath
    rebaseline_tool_sha256 = (Hash-File $PSCommandPath)
    tolerance_unchanged = $tolerances
    product_unchanged = $true
    assertions_unchanged = $true
    inputs_unchanged = $true
    models = $models
    fixtures = [object[]]$fixtures
    fixture_changes = [object[]]$manifestChanges
    historical_native_f64_reference = $historicalDirectory
    generated_at_utc = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
}
$reportText = $report | ConvertTo-Json -Depth 8
if ($Apply) {
    $reportRelative = "$evidenceDirectory/rebaseline.json"
    [System.IO.File]::WriteAllText((Join-Path $repo $reportRelative), $reportText + "`n", $utf8)
}

if ($VerifyTwice) {
    if ($Apply) { throw '-VerifyTwice 只用于预演模式：先两次预演逐字节一致，再单独 -Apply 落盘' }
    $second = Join-Path $repo 'target/rebaseline/out-second'
    New-Item -ItemType Directory -Force -Path $second | Out-Null
    & $PSCommandPath -Repo $repo -OutputDirectory 'target/rebaseline/out-second' | Out-Null
    foreach ($file in Get-ChildItem -LiteralPath $second -File) {
        $first = Join-Path $outDirectory $file.Name
        if (-not (Test-Path -LiteralPath $first)) { throw "second run produced unexpected file $($file.Name)" }
        if ((Hash-File $first) -ne (Hash-File $file.FullName)) { throw "non-deterministic output $($file.Name)" }
    }
    $firstFiles = (Get-ChildItem -LiteralPath $outDirectory -File).Count
    $secondFiles = (Get-ChildItem -LiteralPath $second -File).Count
    if ($firstFiles -ne $secondFiles) { throw 'second run file count mismatch' }
    Write-Output 'verify-twice: byte-identical'
}
Write-Output ($report | ConvertTo-Json -Depth 6 -Compress)
