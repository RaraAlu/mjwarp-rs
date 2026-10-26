# QA: fresh-target fmt/clippy/clippy-cuda runs (real lint, not cached)
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
Set-Location $root
$log = Join-Path $root 'target\qa-stage2\clippy-fresh.log'
"[$(Get-Date -Format o)] start" | Set-Content -LiteralPath $log -Encoding ascii
function Run-Cli([string]$label, [string]$dir, [string[]]$cargoArgs) {
    $env:CARGO_TARGET_DIR = $dir
    if (Test-Path -LiteralPath $dir) { Remove-Item -LiteralPath $dir -Recurse -Force }
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try { $out = & cargo @cargoArgs *>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.Exception.Message } else { [string]$_ } }; $code = $LASTEXITCODE }
    finally { $ErrorActionPreference = $prev }
    $sw.Stop()
    ("## $label exit=$code ms=$($sw.ElapsedMilliseconds)") | Add-Content -LiteralPath $log -Encoding ascii
    $out | Add-Content -LiteralPath $log -Encoding ascii
    Write-Host ("{0,-14} exit={1} ms={2} lines={3}" -f $label, $code, $sw.ElapsedMilliseconds, $out.Count)
    return $code
}
$c1 = Run-Cli 'fmt' (Join-Path $root 'target\qa-stage2\t-fmt') @('fmt','--check')
$c2 = Run-Cli 'clippy-gnu' (Join-Path $root 'target\qa-stage2\t-clippy') @('clippy','--all-targets','--locked','--','-D','warnings')
$c3 = Run-Cli 'clippy-cuda' (Join-Path $root 'target\qa-stage2\t-clippy-cuda') @('clippy','--all-targets','--locked','--features','cuda','--','-D','warnings')
Write-Host "ALL_EXITS=$c1,$c2,$c3"
