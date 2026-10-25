# G5 GPU 窗口 · nvcc 独立 GPU 探针构建（authoring-only，不占 GPU）。
# 用法（仓库根目录）：
#   .\docs\development-evidence\g10-p5-gpu-kernels\gpu-window\build_probe.ps1
# 产物：target/g10-p5-g5/probe.exe + build-probe-report.txt（命令、源 sha256、产物 sha256）。
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root

$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
$nvcc = Join-Path $root 'target/toolchains/cuda-12.8.1/bin/nvcc.exe'
if (-not (Test-Path $nvcc)) { throw "nvcc 不存在: $nvcc" }

$deviceDir = 'src/physics/collision/cuda'
$evidence = 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window'
$outDir = 'target/g10-p5-g5'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$out = Join-Path $outDir 'probe.exe'
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

# 两个构建变体：fmad=true（自动收缩，原判据口径）与 fmad=false（对照上游 plain 代码生成）。
$outNoFmad = Join-Path $outDir 'probe_nofmad.exe'
$commandFmad = '"' + $nvcc + '" -std=c++17 -O3 -arch=sm_89 --fmad=true -Xcompiler /utf-8' +
    ' -I ' + $deviceDir + ' -L "' + $cudartLibDir + '" -o "' + $out + '" ' + $evidence + '/probe.cu'
$commandNoFmad = '"' + $nvcc + '" -std=c++17 -O3 -arch=sm_89 --fmad=false -Xcompiler /utf-8' +
    ' -I ' + $deviceDir + ' -L "' + $cudartLibDir + '" -o "' + $outNoFmad + '" ' + $evidence + '/probe.cu'
cmd /c ('"' + $vcvars + '" >nul 2>&1 && ' + $commandFmad)
if ($LASTEXITCODE -ne 0) { throw "nvcc(fmad=true) 构建失败 exit=$LASTEXITCODE" }
cmd /c ('"' + $vcvars + '" >nul 2>&1 && ' + $commandNoFmad)
if ($LASTEXITCODE -ne 0) { throw "nvcc(fmad=false) 构建失败 exit=$LASTEXITCODE" }

function Sha256([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }

$sources = @('probe.cu') + (Get-ChildItem $deviceDir -Filter '*.cu' | Sort-Object Name | ForEach-Object { $_.Name })
$lines = @(
    "probe_build schema=1",
    "nvcc=$(& $nvcc --version | Select-String -Pattern 'release' | ForEach-Object { $_.Line.Trim() })",
    "nvcc_path=$nvcc",
    "command_fmad_true=$commandFmad",
    "command_fmad_false=$commandNoFmad",
    "probe_nofmad_binary=$outNoFmad",
    "probe_nofmad_sha256=$(Sha256 $outNoFmad)",
    "fmad=true fast_math=false arch=sm_89 opt=-O3",
    "include_dir=$deviceDir",
    "probe_source_sha256=$(Sha256 (Join-Path $evidence 'probe.cu'))",
    "probe_binary=$out",
    "probe_binary_sha256=$(Sha256 $out)",
    "probe_binary_bytes=$((Get-Item $out).Length)"
)
foreach ($name in $sources) {
    $path = if ($name -eq 'probe.cu') { Join-Path $evidence $name } else { Join-Path $deviceDir $name }
    $lines += "source_sha256 $name $(Sha256 $path)"
}
$report = Join-Path $evidence 'build-probe-report.txt'
[System.IO.File]::WriteAllLines((Join-Path (Get-Location) $report), $lines, (New-Object System.Text.UTF8Encoding($false)))
Write-Output "BUILD_OK $probe_binary_sha256=$(Sha256 $out)"
Get-Content $report | Select-Object -First 6
