# Read-only aggregate recompute for R05-S3b hfield guards (same algorithm as freeze-guards.ps1).
# ASCII-only on purpose: Windows PowerShell 5.1 reads .ps1 files as ANSI.
#
# Algorithm (must match freeze-guards.ps1):
#   aggregate = sha256( concat( sha256_hex(entry_i) for entries sorted by repo-relative path ) )
#   The sort is PowerShell `Sort-Object { $_.path }`, i.e. culture/case-insensitive.
#   Do NOT use ordinal sorting (e.g. Python sorted()) as the recompute basis: it can order
#   mixed-case paths differently (README.md vs context.json), giving a different aggregate.
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File recompute-aggregate.ps1
#   powershell -NoProfile -ExecutionPolicy Bypass -File recompute-aggregate.ps1 -GuardsPath <path\to\guards.json>
param(
    [string]$GuardsPath = 'scripts\validate\render-bvh\guards.json'
)
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $GuardsPath)) { throw "missing guards file: $GuardsPath" }
$g = ((Get-Content $GuardsPath -Raw) | ConvertFrom-Json)
$all = @()
foreach ($e in $g.frozen_exes) { $all += [pscustomobject]@{ path = $e.path; sha256 = $e.sha256 } }
foreach ($e in $g.guard_inputs) { $all += [pscustomobject]@{ path = $e.path; sha256 = $e.sha256 } }
$sorted = @($all | Sort-Object { $_.path })
$hexConcat = ($sorted | ForEach-Object { $_.sha256 }) -join ''
$agg = [BitConverter]::ToString(
    [Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::ASCII.GetBytes($hexConcat))
).Replace('-', '').ToLower()
Write-Output ("file={0}" -f $GuardsPath)
Write-Output ("attempt={0}" -f $g.attempt)
Write-Output ("generated_at={0}" -f $g.generated_at)
Write-Output ("entries={0} exes={1} inputs={2}" -f $all.Count, $g.frozen_exes.Count, $g.guard_inputs.Count)
Write-Output ("aggregate_recomputed={0}" -f $agg)
Write-Output ("aggregate_recorded={0}" -f $g.aggregate_sha256)
Write-Output ("matches_recorded={0}" -f ($agg -eq $g.aggregate_sha256))
