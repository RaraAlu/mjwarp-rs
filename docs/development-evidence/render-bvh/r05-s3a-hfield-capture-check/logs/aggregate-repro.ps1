$ErrorActionPreference = 'Stop'
$root = & git rev-parse --show-toplevel
Set-Location $root
$rows = @()
Get-Content 'docs\development-evidence\render-bvh\r05-s3a-hfield-capture\materials\legacy-baseline.txt' -Encoding utf8 |
  Where-Object { $_ -and -not $_.StartsWith('#') } |
  ForEach-Object { $parts = $_ -split ' ', 3; $rows += [pscustomobject]@{ Sha = $parts[0]; Bytes = [int]$parts[1]; Path = $parts[2] } }
Write-Output "rows=$($rows.Count)"

function ShaOf([string]$text) {
  $sha = [System.Security.Cryptography.SHA256]::Create()
  $bytes = [System.Text.Encoding]::UTF8.GetBytes($text)
  return ([System.BitConverter]::ToString($sha.ComputeHash($bytes)) -replace '-', '').ToLowerInvariant()
}

$sorted = $rows | Sort-Object Path
Write-Output ("aggregate_hexconcat " + (ShaOf (($sorted | ForEach-Object { $_.Sha.ToLowerInvariant() }) -join '')))
$lines = ($sorted | ForEach-Object { $_.Sha.ToLowerInvariant() + ' ' + $_.Bytes + ' ' + $_.Path }) -join "`n"
Write-Output ("aggregate_lines " + (ShaOf ($lines + "`n")))
Write-Output ("files " + $rows.Count + " bytes " + (($rows | Measure-Object Bytes -Sum).Sum))
