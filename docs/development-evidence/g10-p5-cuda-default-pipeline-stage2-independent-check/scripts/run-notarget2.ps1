# Portability check v2: tracked-file-only tree (exact on-disk bytes, no target/ artifacts) at a SHORT path for MAX_PATH
$ErrorActionPreference = 'Stop'
$src = 'D:\Rust\mjwarp-rs'
$dst = 'D:\qa-nt'
$logDir = Join-Path $src 'target\qa-stage2\logs'
$partial = Join-Path $src 'target\qa-stage2\tree-no-target'
if (Test-Path -LiteralPath $partial) { Remove-Item -LiteralPath $partial -Recurse -Force }
if (Test-Path -LiteralPath $dst) { Remove-Item -LiteralPath $dst -Recurse -Force }
[System.IO.Directory]::CreateDirectory($dst) | Out-Null
Push-Location $src
$files = @(git ls-files)
Pop-Location
Write-Host "tracked_files=$($files.Count)"
$swCopy = [System.Diagnostics.Stopwatch]::StartNew()
$missing = 0
foreach ($f in $files) {
    $s = Join-Path $src $f
    $d = Join-Path $dst $f
    $parent = Split-Path -Parent $d
    [System.IO.Directory]::CreateDirectory($parent) | Out-Null
    if (Test-Path -LiteralPath $s) {
        [System.IO.File]::Copy($s, $d, $true)
    } else {
        # directory entries in git ls-files cannot happen; count anything unreadable
        $missing++
    }
}
$swCopy.Stop()
Write-Host "copied_ms=$($swCopy.ElapsedMilliseconds) missing=$missing has_target=$((Test-Path (Join-Path $dst 'target'))) has_git=$((Test-Path (Join-Path $dst '.git')))"
Write-Host "copied_files=$((Get-ChildItem -LiteralPath $dst -Recurse -File | Measure-Object).Count)"
Set-Location $dst
$env:CARGO_TARGET_DIR = Join-Path $src 'target\qa-stage2\t-no-target'
if (Test-Path -LiteralPath $env:CARGO_TARGET_DIR) { Remove-Item -LiteralPath $env:CARGO_TARGET_DIR -Recurse -Force }
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$prev = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
$out = & cargo test --locked *>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.Exception.Message } else { [string]$_ } }
$code = $LASTEXITCODE
$ErrorActionPreference = $prev
$sw.Stop()
$out | Set-Content -LiteralPath (Join-Path $logDir 'notarget-default-test.log') -Encoding utf8
$p = 0; $f = 0; $ig = 0; $b = 0
foreach ($m in [regex]::Matches(($out -join "`n"), 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;')) { $b++; $p += [int]$m.Groups[1].Value; $f += [int]$m.Groups[2].Value; $ig += [int]$m.Groups[3].Value }
Write-Host "NOTARGET exit=$code ms=$($sw.ElapsedMilliseconds) passed=$p failed=$f ignored=$ig binaries=$b"
if ($code -ne 0) { $out | Select-String -Pattern 'test result: FAILED|panicked at' | Select-Object -First 8 | ForEach-Object { Write-Host "  $($_.Line)" } }
Write-Host 'NOTARGET DONE'
