# Clean-clone portability check: fresh worktree created with core.autocrlf=false (LF bytes), no target/ artifacts
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
$base = Join-Path $root 'target\qa-stage2'
$wt = Join-Path $base 'wt-lf'
if (Test-Path -LiteralPath $wt) {
    $prevEA = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
    $rm = git -C $root worktree remove --force $wt 2>&1
    $ErrorActionPreference = $prevEA
    if (Test-Path -LiteralPath $wt) { Remove-Item -LiteralPath $wt -Recurse -Force }
}
$prevEA = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
$add = git -c core.autocrlf=false -C $root worktree add --detach $wt HEAD 2>&1
$ErrorActionPreference = $prevEA
$add | Select-Object -Last 2 | Write-Host
Set-Location $wt
$script = Join-Path $wt 'scripts\authoring\g10-p5\capture_sdf_rotation_references.py'
$sha = (Get-FileHash -LiteralPath $script -Algorithm SHA256).Hash.ToLower()
Write-Host "fixture_script_sha=$sha"
Write-Host "worktree_has_target_dir=$(Test-Path (Join-Path $wt 'target'))"
$env:CARGO_TARGET_DIR = Join-Path $base 't-wt-lf'
if (Test-Path -LiteralPath $env:CARGO_TARGET_DIR) { Remove-Item -LiteralPath $env:CARGO_TARGET_DIR -Recurse -Force }
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$prev = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$out = & cargo test --locked *>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.Exception.Message } else { [string]$_ } }
$code = $LASTEXITCODE
$ErrorActionPreference = $prev
$sw.Stop()
$out | Set-Content -LiteralPath (Join-Path $base 'logs\cleanclone-lf-test.log') -Encoding utf8
$p = 0; $f = 0; $ig = 0; $b = 0
foreach ($m in [regex]::Matches(($out -join "`n"), 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;')) { $b++; $p += [int]$m.Groups[1].Value; $f += [int]$m.Groups[2].Value; $ig += [int]$m.Groups[3].Value }
Write-Host "CLEANCLONE-LF exit=$code ms=$($sw.ElapsedMilliseconds) passed=$p failed=$f ignored=$ig binaries=$b"
if ($code -ne 0) { $out | Select-String -Pattern 'FAILED|panicked at' | Select-Object -First 10 | ForEach-Object { Write-Host "  $($_.Line)" } }
foreach ($name in 't4_camlight_precision','t4_face_fusion_repair','t4_face_normalization_control','t4_hessian_generation','t4_spatial_tendon_precision','t4_tendon_wake_precision','t4_trig_control_domain') {
    $idx = ($out | Select-String -Pattern ([regex]::Escape("Running tests\$name.rs")) | Select-Object -First 1).LineNumber
    $res = if ($null -ne $idx) { $out[$idx] } else { 'NOT FOUND' }
    Write-Host "AUDIT $name -> $res"
}
Write-Host 'CLEANCLONE-LF DONE'
