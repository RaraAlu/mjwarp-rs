# G5 window 2 (seconds-scale): ellipsoid-box fix retest + 3-case regression + witness comparison.
# authoring-only; run once inside the approved GPU window.
# Usage (repo root): . .\docs\development-evidence\g10-p5-gpu-kernels\gpu-window\run_window_witness.ps1
# Steps: baseline -> build probes (CPU) -> probe 4 contact cases x2 -> witness probe 3 cases x2
#        -> upstream CUDA witness capture -> Rust ignored comparisons -> after baseline + peak.
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root
$evidence = 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window'
# Absolute paths: Start-Job runs in its own session location, relative paths would be lost.
$log = Join-Path $root (Join-Path $evidence 'window-witness-log.txt')
$samples = Join-Path $root (Join-Path $evidence 'window-witness-memory-samples.txt')
Remove-Item $log, $samples -ErrorAction SilentlyContinue
$started = Get-Date
function Log([string]$text) {
    $line = ('[' + (Get-Date -Format 'HH:mm:ss') + '] ' + $text)
    $line | Tee-Object -FilePath $log -Append
}
function Smi([string]$query) {
    (nvidia-smi --query-gpu=$query --format=csv,noheader 2>&1) -join ' | '
}

Log 'window=start'
Log ('baseline_memory=' + (Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu,driver_version'))
Log ('baseline_compute_apps=' + ((nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1) -join ' | '))

$sampler = Start-Job -ScriptBlock {
    param($path)
    while ($true) {
        $value = (nvidia-smi --query-gpu=memory.used --format=csv,noheader 2>$null)
        if ($value) { $value | Out-File -Append -FilePath $path -Encoding ascii }
        Start-Sleep -Seconds 1
    }
} -ArgumentList $samples

try {
    Log 'step=build-probes(CPU, nvcc; device sources unchanged)'
    & (Join-Path $evidence 'build_probe.ps1') 2>&1 | Tee-Object -FilePath $log -Append
    if ($LASTEXITCODE -ne 0) { throw "build_probe 失败 exit=$LASTEXITCODE" }
    & (Join-Path $evidence 'build_witness_probe.ps1') 2>&1 | Tee-Object -FilePath $log -Append
    if ($LASTEXITCODE -ne 0) { throw "build_witness_probe 失败 exit=$LASTEXITCODE" }

    Log 'step=contact-probe(fix retest + 3-case regression)'
    $probe = Join-Path $root 'target/g10-p5-g5/probe.exe'
    foreach ($case in @('ellipsoid-box', 'hfield-sphere', 'hfield-capsule', 'sdf-box')) {
        $caseDir = Join-Path $evidence ('cases/' + $case)
        $output = & $probe $caseDir 2 2>&1
        if ($LASTEXITCODE -ne 0) { throw "probe $case 失败 exit=$LASTEXITCODE" }
        $output | Tee-Object -FilePath $log -Append
    }

    Log 'step=witness-probe(multicontact 3 cases)'
    $witnessProbe = Join-Path $root 'target/g10-p5-g5/witness_probe.exe'
    foreach ($case in @('cylinder-box', 'box-box-margin', 'mesh-mesh')) {
        $caseDir = Join-Path $evidence ('cases/' + $case)
        $output = & $witnessProbe $caseDir 2 2>&1
        if ($LASTEXITCODE -ne 0) { throw "witness_probe $case 失败 exit=$LASTEXITCODE" }
        $output | Tee-Object -FilePath $log -Append
    }

    Log 'step=upstream-cuda-witness-capture(warp cuda:0)'
    $python = 'target/mjwarp-parallel/venv/Scripts/python.exe'
    & $python -B (Join-Path $evidence 'capture_witness_ccd.py') --device cuda 2>&1 |
        Tee-Object -FilePath $log -Append
    if ($LASTEXITCODE -ne 0) { throw "capture_witness_ccd 失败 exit=$LASTEXITCODE" }

    Log 'step=compare(cargo test --ignored)'
    cargo test --locked --features cuda-probe --test g10_p5_gpu_kernels -- --ignored --test-threads=1 2>&1 |
        Tee-Object -FilePath $log -Append
    Log ("compare_exit=$LASTEXITCODE")
} finally {
    Stop-Job $sampler -ErrorAction SilentlyContinue
    Remove-Job $sampler -Force -ErrorAction SilentlyContinue
    Log ('after_memory=' + (Smi 'name,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu'))
    Log ('after_compute_apps=' + ((nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1) -join ' | '))
    $elapsed = (Get-Date) - $started
    Log ('elapsed_seconds=' + [math]::Round($elapsed.TotalSeconds, 1))
    $values = Get-Content $samples -ErrorAction SilentlyContinue |
        Where-Object { $_ -match '\d+' } |
        ForEach-Object { [int](($_ -replace '[^0-9]', '')) }
    if ($values) {
        Log ('peak_memory_used_mib=' + ($values | Measure-Object -Maximum).Maximum)
        Log ('sample_count=' + $values.Count)
    }
}
Log 'window=done'
Get-Content $log | Select-Object -Last 25
