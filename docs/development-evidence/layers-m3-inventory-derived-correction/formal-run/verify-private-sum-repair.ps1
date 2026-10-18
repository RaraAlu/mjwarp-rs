$ErrorActionPreference = 'Stop'
$path = 'docs/development-evidence/layers-m3-inventory-derived-correction/inventory-lib.ps1'
$text = Get-Content -LiteralPath $path -Raw
if ($text -notmatch '\$privateTyped\s*=\s*0' -or
    $text -notmatch '\$privateRaw\s*=\s*0' -or
    $text -notmatch '\$privateOuter\s*=\s*0') {
    throw '显式标量累加修复缺失'
}
if ($text -match 'Measure-Object typed_low_level_calls' -or
    $text -match 'Measure-Object unmigrated_raw_calls' -or
    $text -match 'Measure-Object orchestration_calls') {
    throw '旧数组求和仍然存在'
}
[ordered]@{schema=1;repair='explicit scalar accumulation';result='pass'} | ConvertTo-Json
