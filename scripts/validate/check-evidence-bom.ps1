# Evidence BOM audit (read-only).
#
# Scans docs/evidence, docs/development-evidence and fixtures for UTF-8 BOM (EF BB BF).
# Policy:
#   * .ps1 with BOM is INTENTIONALLY preserved (Windows PowerShell 5.1 mis-decodes
#     UTF-8-without-BOM CJK scripts; see 2026-11-04-prepare-scripts-bom-hardening-np02).
#     Reported as INFO, never as a defect.
#   * every other text file must be BOM-free. Reported as WARN (baseline: historical files).
#   * manifest.txt must start with '#' at byte 0 (a BOM hides it). Reported as WARN.
# This script never writes, rewrites or normalizes any file.
#
# Usage (the default repo root is derived from scripts/validate/):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/check-evidence-bom.ps1
#   powershell ... -File scripts/validate/check-evidence-bom.ps1 -Strict   # exit 1 when WARN > 0
#
# ASCII-only on purpose (PS 5.1 parse safety).
[CmdletBinding()]
param(
    [string]$Repo = '',
    [switch]$Strict
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($Repo)) {
    $scriptDir = $PSScriptRoot
    if ([string]::IsNullOrWhiteSpace($scriptDir)) { $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path }
    $Repo = Split-Path -Parent (Split-Path -Parent $scriptDir)
}
$Repo = (Resolve-Path -LiteralPath $Repo).Path

$roots = @('docs/evidence', 'docs/development-evidence', 'fixtures')
$BOM_MIN = 0xEF
$BOM_MID = 0xBB
$BOM_MAX = 0xBF
$HASH_BYTE = 0x23

function Get-Head([string]$path, [int]$count) {
    $stream = [System.IO.File]::Open($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
    try {
        $buffer = New-Object byte[] $count
        $read = $stream.Read($buffer, 0, $count)
        if ($read -le 0) { return ,(New-Object byte[] 0) }
        if ($read -lt $count) {
            $trimmed = New-Object byte[] $read
            [array]::Copy($buffer, $trimmed, $read)
            return ,$trimmed
        }
        return ,$buffer
    } finally {
        $stream.Dispose()
    }
}

function Test-Bom([byte[]]$head) {
    return ($head.Length -ge 3 -and $head[0] -eq $BOM_MIN -and $head[1] -eq $BOM_MID -and $head[2] -eq $BOM_MAX)
}

$bomWarn = @()
$bomInfo = @()
$manifestWarn = @()
$byExtension = @{}
$scanned = 0

foreach ($root in $roots) {
    $full = Join-Path $Repo $root
    if (-not (Test-Path -LiteralPath $full)) {
        Write-Output ("SKIP root not present: " + $root)
        continue
    }
    foreach ($file in (Get-ChildItem -LiteralPath $full -Recurse -File -ErrorAction SilentlyContinue)) {
        $scanned++
        try { $head = Get-Head $file.FullName 3 } catch { Write-Output ("SKIP unreadable: " + $file.FullName); continue }
        if ($head.Length -le 0) { continue }
        $rel = $file.FullName.Substring($Repo.Length).Replace('\', '/').TrimStart('/')
        $ext = [System.IO.Path]::GetExtension($file.Name).ToLowerInvariant()
        if ([string]::IsNullOrEmpty($ext)) { $ext = '(none)' }
        if (Test-Bom $head) {
            $byExtension[$ext] = 1 + [int]$byExtension[$ext]
            if ($ext -eq '.ps1') { $bomInfo += $rel } else { $bomWarn += $rel }
        }
        if ($file.Name -eq 'manifest.txt' -and $head[0] -ne $HASH_BYTE) { $manifestWarn += $rel }
    }
}

$bomTotal = $bomWarn.Count + $bomInfo.Count
Write-Output ("evidence BOM audit: repo=" + $Repo)
Write-Output ("roots=" + ($roots -join ', '))
Write-Output ("scanned files=" + $scanned)
Write-Output ("BOM files total=" + $bomTotal + " (WARN=" + $bomWarn.Count + ", INFO=" + $bomInfo.Count + ")")
Write-Output 'BOM by extension:'
foreach ($key in ($byExtension.Keys | Sort-Object)) {
    Write-Output ("  " + $key + " " + $byExtension[$key])
}

if ($bomInfo.Count -gt 0) {
    Write-Output ''
    Write-Output 'INFO (BOM intentionally preserved for Windows PowerShell 5.1):'
    foreach ($path in ($bomInfo | Sort-Object)) { Write-Output ("  INFO  " + $path) }
}

if ($bomWarn.Count -gt 0) {
    Write-Output ''
    Write-Output 'WARN (historical text evidence with BOM; do NOT rewrite history, new files must be BOM-free):'
    foreach ($path in ($bomWarn | Sort-Object)) { Write-Output ("  WARN  " + $path) }
}

if ($manifestWarn.Count -gt 0) {
    Write-Output ''
    Write-Output ("WARN manifest files whose first byte is not '#': " + $manifestWarn.Count)
    foreach ($path in ($manifestWarn | Sort-Object)) { Write-Output ("  WARN  manifest " + $path) }
}

$warnTotal = $bomWarn.Count + $manifestWarn.Count
Write-Output ''
if ($warnTotal -eq 0) {
    Write-Output 'RESULT: PASS (no BOM outside .ps1, every manifest starts with #)'
} else {
    Write-Output ("RESULT: WARN (" + $bomWarn.Count + " non-.ps1 BOM files, " + $manifestWarn.Count + " manifest header violations)")
}

if ($Strict -and $warnTotal -gt 0) { exit 1 }
exit 0

