# QA independent checks in a clean git worktree (no target/ artifacts):
#  (1) clean-clone default cargo test portability; (2) pre-fix red reproduction for the culib panic guard
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
$base = Join-Path $root 'target\qa-stage2'
$wt = Join-Path $base 'wt-clean'
$env:CARGO_TARGET_DIR = Join-Path $base 't-wt-clean'
if (Test-Path -LiteralPath $wt) { git -C $root worktree remove --force $wt 2>$null; if (Test-Path -LiteralPath $wt) { Remove-Item -LiteralPath $wt -Recurse -Force } }
if (Test-Path -LiteralPath $env:CARGO_TARGET_DIR) { Remove-Item -LiteralPath $env:CARGO_TARGET_DIR -Recurse -Force }
$prevEA = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; $wtOut = git -C $root worktree add --detach $wt HEAD 2>&1; $ErrorActionPreference = $prevEA; $wtOut | Write-Host
if (-not (Test-Path (Join-Path $wt 'Cargo.toml'))) { throw 'worktree create failed' }
Set-Location $wt

$logDir = Join-Path $base 'logs'
function Invoke-Capture {
    param([scriptblock]$Block)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $out = & $Block *>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.Exception.Message } else { [string]$_ } }
        $code = $LASTEXITCODE
    } finally { $ErrorActionPreference = $prev }
    return @{ out = @($out); code = $code }
}
function Counts([string[]]$lines) {
    $p = 0; $f = 0; $ig = 0; $b = 0
    foreach ($m in [regex]::Matches(($lines -join "`n"), 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;')) {
        $b++; $p += [int]$m.Groups[1].Value; $f += [int]$m.Groups[2].Value; $ig += [int]$m.Groups[3].Value
    }
    return "$p/$f/$ig/$b"
}
Write-Host "worktree=$wt target=$env:CARGO_TARGET_DIR"
Write-Host "worktree_has_target_dir=$(Test-Path (Join-Path $wt 'target'))"

# (1) clean-clone default full test
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$t = Invoke-Capture { cargo test --locked }
$sw.Stop()
$t.out | Set-Content -LiteralPath (Join-Path $logDir 'cleanclone-default-test.log') -Encoding utf8
Write-Host "CLEANCLONE test_exit=$($t.code) ms=$($sw.ElapsedMilliseconds) counts=$(Counts $t.out)"
$fail = $t.out | Select-String -Pattern 'FAILED|error\[|panicked' | Select-Object -First 5
if ($fail) { $fail | ForEach-Object { Write-Host "  $($_.Line)" } }
# count the 7 historical audit targets in this clean clone default run
foreach ($name in 't4_camlight_precision','t4_face_fusion_repair','t4_face_normalization_control','t4_hessian_generation','t4_spatial_tendon_precision','t4_tendon_wake_precision','t4_trig_control_domain') {
    $idx = ($t.out | Select-String -Pattern ([regex]::Escape("Running tests\$name.rs")) | Select-Object -First 1).LineNumber
    $res = if ($null -ne $idx) { $t.out[$idx] } else { 'NOT FOUND' }
    Write-Host "AUDIT $name -> $res"
}

# (2) pre-fix red: remove driver precheck + catch_unwind in this scratch worktree (keep test seam injection)
$ctx = Join-Path $wt 'src\runtime\collision_gpu\context.rs'
$new = @'
//! scratch pre-fix reproduction: guards removed (QA independent check only)
use std::sync::Arc;

use cudarc::driver::CudaContext;

use super::CollisionGpuError;
use crate::runtime::presence;

pub(crate) fn device_context(device: usize) -> Result<Arc<CudaContext>, CollisionGpuError> {
    #[cfg(test)]
    match presence::test_context_mode() {
        presence::CONTEXT_PANIC => panic!("injected cudarc culib panic"),
        presence::CONTEXT_ERROR => {
            return Err(CollisionGpuError::NoDevice(format!(
                "CudaContext::new({device}): injected no device"
            )));
        }
        _ => {}
    }
    CudaContext::new(device)
        .map_err(|error| CollisionGpuError::NoDevice(format!("CudaContext::new({device}): {error:?}")))
}
'@
$new | Set-Content -LiteralPath $ctx -Encoding utf8
$env:CARGO_TARGET_DIR = Join-Path $base 't-wt-prefix'
if (Test-Path -LiteralPath $env:CARGO_TARGET_DIR) { Remove-Item -LiteralPath $env:CARGO_TARGET_DIR -Recurse -Force }
$pf = Invoke-Capture { cargo test --locked --lib runtime::collision_gpu::fallback_tests::device_context_initialization_panic_is_caught -- --exact --nocapture }
$pf.out | Set-Content -LiteralPath (Join-Path $logDir 'prefx-red-scratch.log') -Encoding utf8
Write-Host "PRE-FIX test_exit=$($pf.code) (expect nonzero, panic escapes)"
$pf.out | Select-Object -Last 25 | ForEach-Object { Write-Host "  $_" }

# restore scratch file, rebuild and confirm green
git -C $wt checkout -- src/runtime/collision_gpu/context.rs
$green = Invoke-Capture { cargo test --locked --lib runtime::collision_gpu::fallback_tests:: -- --test-threads=1 }
$green.out | Set-Content -LiteralPath (Join-Path $logDir 'prefx-green-restored.log') -Encoding utf8
Write-Host "RESTORED fallback_tests exit=$($green.code) counts=$(Counts $green.out)"
$green.out | Select-String -Pattern 'test result' | ForEach-Object { Write-Host "  $($_.Line)" }
Write-Host 'WORKTREE DONE'

