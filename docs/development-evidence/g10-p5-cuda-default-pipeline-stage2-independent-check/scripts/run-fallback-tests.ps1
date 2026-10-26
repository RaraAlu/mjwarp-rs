# QA: independent process runs for three-state fallback (API/CLI) + selection layer + cuda-probe audit gating
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
Set-Location $root
$env:CARGO_TARGET_DIR = Join-Path $root 'target\qa-stage2\t-default'
$log = Join-Path $root 'target\qa-stage2\fallback-independent-runs.log'
'' | Set-Content -LiteralPath $log -Encoding utf8
function Run-Case([string]$label, [string[]]$cargoArgs) {
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try { $out = & cargo @cargoArgs *>&1 | ForEach-Object { if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.Exception.Message } else { [string]$_ } }; $code = $LASTEXITCODE }
    finally { $ErrorActionPreference = $prev }
    $res = @($out | Select-String -Pattern 'test result:|panicked|FAILED')
    Write-Host ("{0,-70} exit={1} {2}" -f $label, $code, ($res -join ' | '))
    ("## $label exit=$code") | Add-Content -LiteralPath $log -Encoding utf8
    $out | Add-Content -LiteralPath $log -Encoding utf8
    return $code
}

$codes = @()
$codes += Run-Case 'API no-driver (fallback_tests::device_context_without_driver_returns_nodevice_without_panic)' @('test','--locked','--lib','runtime::collision_gpu::fallback_tests::device_context_without_driver_returns_nodevice_without_panic','--','--exact')
$codes += Run-Case 'API no-device (fallback_tests::device_context_without_usable_device_returns_nodevice)' @('test','--locked','--lib','runtime::collision_gpu::fallback_tests::device_context_without_usable_device_returns_nodevice','--','--exact')
$codes += Run-Case 'API culib-panic caught (fallback_tests::device_context_initialization_panic_is_caught)' @('test','--locked','--lib','runtime::collision_gpu::fallback_tests::device_context_initialization_panic_is_caught','--','--exact')
$codes += Run-Case 'API no-nvrtc (fallback_tests::constructor_without_nvrtc_returns_compile_without_panic)' @('test','--locked','--lib','runtime::collision_gpu::fallback_tests::constructor_without_nvrtc_returns_compile_without_panic','--','--exact')
$codes += Run-Case 'CLI no-driver (cuda::tests::prepare_device_without_driver_returns_driver_unavailable_without_panic)' @('test','--locked','--lib','runtime::cuda::tests::prepare_device_without_driver_returns_driver_unavailable_without_panic','--','--exact')
$codes += Run-Case 'CLI no-device (cuda::tests::prepare_device_without_devices_returns_invalid_device)' @('test','--locked','--lib','runtime::cuda::tests::prepare_device_without_devices_returns_invalid_device','--','--exact')
$codes += Run-Case 'CLI no-nvrtc (cuda::tests::require_nvrtc_without_library_returns_nvrtc_unavailable)' @('test','--locked','--lib','runtime::cuda::tests::require_nvrtc_without_library_returns_nvrtc_unavailable','--','--exact')
$codes += Run-Case 'selection layer cuda tests (runtime::backend::tests::cuda_probe)' @('test','--locked','--lib','runtime::backend::tests::cuda_probe','--','--test-threads=1')
$codes += Run-Case 'cuda-probe feature no longer runs 7 audits (t4_camlight_precision --list)' @('test','--locked','--features','cuda-probe','--test','t4_camlight_precision','--','--list')
Write-Host ("ALL_EXITS=" + ($codes -join ','))
