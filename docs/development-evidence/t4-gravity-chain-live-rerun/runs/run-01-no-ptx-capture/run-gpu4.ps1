$ErrorActionPreference = 'Stop'
Set-Location D:\Rust\mjwarp-rs
$out = 'D:\Rust\mjwarp-rs\target\t4g-run'
$env:CUDA_PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
$evidence = 'D:\Rust\mjwarp-rs\docs\evidence\gravity-nonzero-product-chain\gpu-4'
$env:MJWARP_GRAVITY_NONZERO_CHAIN_EVIDENCE_DIR = $evidence
$args1 = @('+1.99.0-x86_64-pc-windows-msvc', 'test', '--locked', '--features', 'cuda-probe', '--test', 't4_gravity_nonzero_product_chain')
$args2 = @('--', '--ignored', '--test-threads=1')
$head = (git rev-parse HEAD).Trim()
$gpu = (nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader).Trim()

# Build first so the timed run excludes compilation.
$build = Measure-Command { & cargo @args1 --no-run *> "$out\build.log" }
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) { "build exit $buildExit"; exit $buildExit }
if (Test-Path $evidence) { throw 'evidence directory already exists' }

$idle = (nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits).Trim()
$smi = Start-Process -FilePath nvidia-smi -ArgumentList '--query-gpu=timestamp,memory.used,utilization.gpu', '--format=csv,noheader,nounits', '-lms', '50' -RedirectStandardOutput "$out\vram-samples.csv" -NoNewWindow -PassThru
Start-Sleep -Milliseconds 500
$sw = [System.Diagnostics.Stopwatch]::StartNew()
& cargo @args1 @args2 *> "$out\test.log"
$testExit = $LASTEXITCODE
$sw.Stop()
Start-Sleep -Milliseconds 500
Stop-Process -Id $smi.Id
$after = (nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits).Trim()
$apps = (nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader) -join "`n"

$samples = Get-Content "$out\vram-samples.csv" | Where-Object { $_ -match ',' } | ForEach-Object { [int](($_ -split ',')[1].Trim()) }
$peak = ($samples | Measure-Object -Maximum).Maximum
@"
head: $head
gpu: $gpu
build_exit: $buildExit
build_seconds: $([math]::Round($build.TotalSeconds, 2))
test_exit: $testExit
test_wall_ms: $([math]::Round($sw.Elapsed.TotalMilliseconds, 1))
vram_idle_mib: $idle
vram_peak_mib: $peak
vram_after_mib: $after
vram_samples: $($samples.Count)
compute_apps_after: $apps
"@ | Set-Content "$out\summary.txt" -Encoding utf8
Get-Content "$out\summary.txt"
exit $testExit
