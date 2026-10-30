# Offline self-check for the window runner aggregate-suite derivation fix.
# Read-only: parses run-window.ps1 (no execution), parses the existing guards files and
# recomputes the aggregate_sha256 recorded by each guards file. No window action, no GPU.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../../..')).Path
Set-Location $root
$runner = 'scripts/validate/render-bvh/run-window.ps1'
$failures = New-Object System.Collections.Generic.List[string]

# 1) PSParser: 0 parse errors after the edit
$tokens = $null
$parseErrors = $null
[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $root $runner), [ref]$tokens, [ref]$parseErrors) | Out-Null
$parseErrorCount = @($parseErrors).Count
"parser_errors=$parseErrorCount"
if ($parseErrorCount -ne 0) {
    foreach ($e in $parseErrors) { $failures.Add("psparser: $($e.Message)") }
}

# 2) ASCII discipline unchanged
$text = [System.IO.File]::ReadAllText((Join-Path $root $runner), [System.Text.Encoding]::UTF8)
$nonAscii = ([regex]::Matches($text, '[^\x00-\x7F]')).Count
"non_ascii_bytes=$nonAscii"
if ($nonAscii -ne 0) { $failures.Add("non-ascii bytes: $nonAscii") }

# 3) Derivation present and the old hard-coded value only survives as the documented fallback
$deriveHits = ([regex]::Matches($text, '\$aggregateSuite = \$guard\.suite')).Count
$useHits = ([regex]::Matches($text, 'suite = \$aggregateSuite')).Count
$hardHits = ([regex]::Matches($text, "'r05-s3b-hfield-validate'")).Count
"derive_assignment_hits=$deriveHits suite_field_hits=$useHits old_literal_hits=$hardHits"
if ($deriveHits -lt 1) { $failures.Add('missing $aggregateSuite = $guard.suite') }
if ($useHits -lt 1) { $failures.Add('suite field does not use $aggregateSuite') }
if ($hardHits -ne 1) { $failures.Add("old literal must appear exactly once (fallback), got $hardHits") }

# 4) For each existing guards file: suite parses, aggregate recomputes to the recorded value
foreach ($guardsRel in @('scripts/validate/render-bvh/guards.json',
                         'scripts/validate/render-bvh/guards-r02-skybox.json',
                         'scripts/validate/render-bvh/guards-r08-rgb-hetero.json')) {
    $guardsPath = Join-Path $root $guardsRel
    if (-not (Test-Path $guardsPath)) { $failures.Add("missing guards file: $guardsRel"); continue }
    $guard = [System.IO.File]::ReadAllText($guardsPath, [System.Text.Encoding]::UTF8) | ConvertFrom-Json
    $suite = [string]$guard.suite
    $entries = @($guard.frozen_exes) + @($guard.guard_inputs)
    $sorted = @($entries | Sort-Object { $_.path })
    $hex = ($sorted | ForEach-Object { $_.sha256 }) -join ''
    $engine = [System.Security.Cryptography.SHA256]::Create()
    $recomputed = [BitConverter]::ToString(
        $engine.ComputeHash([System.Text.Encoding]::ASCII.GetBytes($hex))).Replace('-', '').ToLower()
    $suiteOk = $suite -eq $guard.suite -and $suite.Length -gt 0
    $aggOk = $recomputed -eq $guard.aggregate_sha256
    $countOk = $sorted.Count -eq [int]$guard.counts.total
    "guards=$(Split-Path $guardsRel -Leaf) suite=$suite entries=$($sorted.Count) count_match=$countOk suite_ok=$suiteOk aggregate_ok=$aggOk"
    if (-not $suiteOk) { $failures.Add("$guardsRel suite did not parse") }
    if (-not $countOk) { $failures.Add("$guardsRel entry count mismatch") }
    if (-not $aggOk) { $failures.Add("$guardsRel aggregate recompute mismatch: $recomputed vs $($guard.aggregate_sha256)") }
}

if ($failures.Count -gt 0) {
    "SELF_CHECK_FAILED"
    $failures | ForEach-Object { " - $_" }
    exit 1
}
"SELF_CHECK_OK"
exit 0
