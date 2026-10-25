# QA post-fix binary scan: does the *product* binary contain the test-only seam env
# string? Fresh post-fix builds under target/qa-fma-check-postfix.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$repo = 'D:\Rust\mjwarp-rs'
$logs = Join-Path $repo 'docs\development-evidence\exact-f32-fma-relmat-independent-check\logs-postfix'
New-Item -ItemType Directory -Force -Path $logs | Out-Null
$probe = [Text.Encoding]::ASCII.GetBytes('MJWARP_FORCE_SOFT_FMA')
function Test-Contains {
    param([string]$Path)
    $bytes = [IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt $probe.Length) { return $false }
    for ($i = 0; $i -le $bytes.Length - $probe.Length; $i++) {
        if ($bytes[$i] -eq $probe[0]) {
            $match = $true
            for ($j = 1; $j -lt $probe.Length; $j++) {
                if ($bytes[$i + $j] -ne $probe[$j]) { $match = $false; break }
            }
            if ($match) { return $true }
        }
    }
    return $false
}
$targets = @(
    @{ name = 'product debug exe'; path = (Join-Path $repo 'target\qa-fma-check-postfix\prod-debug\debug\mjwarp-rs.exe') },
    @{ name = 'product release exe'; path = (Join-Path $repo 'target\qa-fma-check-postfix\prod-release\release\mjwarp-rs.exe') }
)
$harness = Get-ChildItem -Path (Join-Path $repo 'target\qa-fma-check-postfix\tests\debug\deps') -Filter 'mjwarp_rs-*.exe' -ErrorAction SilentlyContinue |
    Sort-Object Length -Descending | Select-Object -First 1
if ($null -ne $harness) { $targets += @{ name = 'lib test harness'; path = $harness.FullName } }
$lines = @(
    '# post-fix binary scan for the test-only seam env string',
    ('utc=' + (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')),
    ('git_head=' + ((& git -C $repo rev-parse HEAD) -join '')),
    ('needle=' + [Text.Encoding]::ASCII.GetString($probe)),
    ''
)
foreach ($t in $targets) {
    if (-not (Test-Path $t.path)) {
        $lines += ("{0} | MISSING | {1}" -f $t.name, $t.path)
        continue
    }
    $item = Get-Item $t.path
    $contains = Test-Contains $t.path
    $lines += ("{0} | {1} bytes | contains_env={2} | {3}" -f $t.name, $item.Length, $contains, $t.path)
}
$lines | Out-File -Encoding utf8 (Join-Path $logs 'seam-binary-scan-postfix.txt')
Get-Content (Join-Path $logs 'seam-binary-scan-postfix.txt')
Write-Host 'SEAM_SCAN_POSTFIX_DONE'
