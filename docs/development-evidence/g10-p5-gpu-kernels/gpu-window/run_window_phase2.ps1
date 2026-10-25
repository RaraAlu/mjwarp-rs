# G5 GPU 窗口 · 阶段 2（短复跑）：`--fmad=false` 探针变体对照。
# 用法（仓库根目录）：.\docs\development-evidence\g10-p5-gpu-kernels\gpu-window\run_window_phase2.ps1
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root
$evidence = 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window'
$log = Join-Path $evidence 'window-log.txt'
$started = Get-Date
function Log([string]$text) {
    (('[' + (Get-Date -Format 'HH:mm:ss') + '] ' + $text)) | Tee-Object -FilePath $log -Append
}
Log 'window=phase2-start(fmad=false)'
Log ('baseline_memory=' + ((nvidia-smi --query-gpu=name,memory.used,memory.free --format=csv,noheader) -join ' '))

$cases = @('sphere-box-shallow','sphere-box-deep','capsule-box','ellipsoid-box','hfield-sphere','hfield-capsule','sdf-box')
$casesRoot = Join-Path $evidence 'cases'
foreach ($case in $cases) {
    $dir = Join-Path $casesRoot $case
    foreach ($suffix in @('1','2')) {
        $src = Join-Path $dir "out$suffix"
        $dst = Join-Path $dir "fmad_true_out$suffix"
        if (Test-Path $src) { Copy-Item $src $dst -Recurse -Force }
    }
}
$probe = Join-Path $root 'target/g10-p5-g5/probe_nofmad.exe'
foreach ($case in $cases) {
    $dir = Join-Path $casesRoot $case
    $output = & $probe $dir 2 2>&1
    if ($LASTEXITCODE -ne 0) { throw "probe_nofmad $case 失败 exit=$LASTEXITCODE" }
    $output | Tee-Object -FilePath $log -Append
}
Log ('after_memory=' + ((nvidia-smi --query-gpu=name,memory.used,memory.free --format=csv,noheader) -join ' '))
Log ('after_compute_apps=' + ((nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1) -join ' | '))
Log ('elapsed_seconds=' + [math]::Round(((Get-Date) - $started).TotalSeconds, 1))
Log 'window=phase2-done'
Get-Content $log | Select-Object -Last 12
