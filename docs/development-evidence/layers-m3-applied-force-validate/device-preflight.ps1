$ErrorActionPreference = 'Stop'
$cudaRoot = 'target/toolchains/cuda-12.8.1'
$nvrtc = Join-Path $cudaRoot 'bin/nvrtc64_120_0.dll'
$builtins = Join-Path $cudaRoot 'bin/nvrtc-builtins64_128.dll'
$raw = @(& nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader,nounits)
if ($LASTEXITCODE -ne 0) { throw "nvidia-smi退出码：$LASTEXITCODE" }
$device = @($raw | Where-Object { $_ -match '^NVIDIA GeForce RTX 4070 Ti SUPER,' })
if ($device.Count -ne 1) { throw '本机T4设备不唯一' }
foreach ($path in @($cudaRoot, $nvrtc, $builtins)) {
    if (-not (Test-Path -LiteralPath $path)) { throw "缺少CUDA文件：$path" }
}
[ordered]@{
    schema = 1
    device = $device[0]
    resource_lock = 'gpu:windows-t4'
    cuda_root = $cudaRoot
    nvrtc_sha256 = (Get-FileHash -LiteralPath $nvrtc -Algorithm SHA256).Hash.ToLowerInvariant()
    nvrtc_builtins_sha256 = (Get-FileHash -LiteralPath $builtins -Algorithm SHA256).Hash.ToLowerInvariant()
    task_memory = 'unknown'
} | ConvertTo-Json
