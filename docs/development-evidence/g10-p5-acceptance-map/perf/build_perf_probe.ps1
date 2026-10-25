# G6-2 性能探针 · nvcc 构建（authoring-only，编译不占 GPU；本任务仅 fmad=true 变体）。
#
# 用法（仓库根目录）：
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-acceptance-map\perf\build_perf_probe.ps1
# 产物：target/g6-perf/probe_perf.exe + perf/build-report.txt（命令、源 sha256、产物 sha256）。
# 口径与冻结 G5 探针一致：-std=c++17 -O3 -arch=sm_89 --fmad=true、禁止 fast-math。
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root

$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
$nvcc = Join-Path $root 'target/toolchains/cuda-12.8.1/bin/nvcc.exe'
if (-not (Test-Path $nvcc)) { throw "nvcc 不存在: $nvcc" }
$deviceDir = 'src/physics/collision/cuda'
$perfDir = 'docs/development-evidence/g10-p5-acceptance-map/perf'
$outDir = Join-Path $root 'target/g6-perf'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$out = Join-Path $outDir 'probe_perf.exe'
$cudartLib = Get-ChildItem (Join-Path $root 'target/toolchains/cuda-12.8.1') -Recurse -Filter 'cudart.lib' |
    Select-Object -First 1
if (-not $cudartLib) { throw 'cudart.lib 不存在' }
$cudartLibDir = $cudartLib.DirectoryName
$cudartDll = Get-ChildItem $cudartLibDir -Filter 'cudart64_*.dll' | Select-Object -First 1
if (-not $cudartDll) {
    $cudartDll = Get-ChildItem (Join-Path $root 'target/toolchains/cuda-12.8.1/bin') -Filter 'cudart64_*.dll' |
        Select-Object -First 1
}
Copy-Item $cudartDll.FullName (Join-Path $outDir $cudartDll.Name) -Force

$command = '"' + $nvcc + '" -std=c++17 -O3 -arch=sm_89 --fmad=true -Xcompiler /utf-8' +
' -I ' + $deviceDir + ' -I ' + $perfDir + ' -L "' + $cudartLibDir + '" -o "' + $out + '" ' +
$perfDir + '/perf_probe.cu'
cmd /c ('"' + $vcvars + '" >nul 2>&1 && ' + $command)
if ($LASTEXITCODE -ne 0) { throw "nvcc 构建失败 exit=$LASTEXITCODE" }

function Sha256([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }
$sources = @('perf_probe.cu', 'perf_probe_common.cu', 'perf_probe_timing.cu') |
    ForEach-Object { Join-Path $perfDir $_ }
$lines = @(
    "perf_probe_build schema=1 task=2026-10-25-g10-p5-acceptance-perf-native",
    "nvcc=$(& $nvcc --version | Select-String -Pattern 'release' | ForEach-Object { $_.Line.Trim() })",
    "nvcc_path=$nvcc",
    "command=$command",
    "fmad=true fast_math=false arch=sm_89 opt=-O3",
    "include_dirs=$deviceDir;$perfDir",
    "probe_binary=$out",
    "probe_binary_sha256=$(Sha256 $out)",
    "probe_binary_bytes=$((Get-Item $out).Length)"
)
foreach ($path in $sources) {
    $lines += ("source_sha256 {0} {1}" -f (Split-Path $path -Leaf), (Sha256 $path))
}
$report = Join-Path $root "$perfDir/build-report.txt"
[System.IO.File]::WriteAllLines($report, $lines, (New-Object System.Text.UTF8Encoding($false)))
Write-Output ("BUILD_OK probe_perf_sha256={0} bytes={1}" -f (Sha256 $out), (Get-Item $out).Length)
Get-Content $report | Select-Object -First 6
