# G5 GPU 窗口执行脚本（authoring-only，窗口内一次性运行）。
# 用法（仓库根目录）：. .\docs\development-evidence\g10-p5-gpu-kernels\gpu-window\run_window.ps1
# 步骤：基线/采样 → CUDA 准备输入与参考绑定 → 构建探针 → 逐 case 探针两次 → Rust 比对 → 收尾基线。
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root
$evidence = 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window'
$log = Join-Path $evidence 'window-log.txt'
$samples = Join-Path $evidence 'window-memory-samples.txt'
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

# 每 2 秒采样显存，用于峰值统计。
$sampler = Start-Job -ScriptBlock {
    param($path)
    while ($true) {
        $value = (nvidia-smi --query-gpu=memory.used --format=csv,noheader 2>$null)
        if ($value) { $value | Out-File -Append -FilePath $path -Encoding ascii }
        Start-Sleep -Seconds 2
    }
} -ArgumentList $samples

try {
    Log 'step=prepare-inputs(device=cuda)'
    $python = 'target/mjwarp-parallel/venv/Scripts/python.exe'
    & $python -B (Join-Path $evidence 'prepare_cases.py') --device cuda 2>&1 |
        Tee-Object -FilePath $log -Append
    if ($LASTEXITCODE -ne 0) { throw "prepare_cases 失败 exit=$LASTEXITCODE" }

    Log 'step=build-probe'
    . (Join-Path $evidence 'build_probe.ps1') 2>&1 | Tee-Object -FilePath $log -Append

    Log 'step=probe-all-cases'
    $probe = Join-Path $root 'target/g10-p5-g5/probe.exe'
    $cases = Get-ChildItem (Join-Path $evidence 'cases') -Directory | Sort-Object Name
    foreach ($case in $cases) {
        $caseDir = $case.FullName
        $output = & $probe $caseDir 2 2>&1
        if ($LASTEXITCODE -ne 0) { throw "probe $($case.Name) 失败 exit=$LASTEXITCODE" }
        $output | Tee-Object -FilePath $log -Append
        Add-Content -Path $samples -Value ((nvidia-smi --query-gpu=memory.used --format=csv,noheader 2>$null))
    }

    Log 'step=compare(cargo test --ignored)'
    cargo test --locked --features cuda-probe --test g10_p5_gpu_kernels -- --ignored --test-threads=1 2>&1 |
        Tee-Object -FilePath $log -Append
    if ($LASTEXITCODE -ne 0) { throw "比对测试失败 exit=$LASTEXITCODE" }
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
Get-Content $log | Select-Object -Last 20
