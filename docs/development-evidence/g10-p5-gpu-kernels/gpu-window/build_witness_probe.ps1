# G5 witness probe build (authoring-only; no GPU compute).
# Usage (repo root): .\docs\development-evidence\g10-p5-gpu-kernels\gpu-window\build_witness_probe.ps1
# Output: target/g10-p5-g5/witness_probe.exe + build-witness-probe-report.txt
#         (command, source sha256, binary sha256; same device sources, fmad=true, -O3, sm_89).
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root

$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
$nvcc = Join-Path $root 'target/toolchains/cuda-12.8.1/bin/nvcc.exe'
if (-not (Test-Path $nvcc)) { throw "nvcc not found: $nvcc" }

$deviceDir = 'src/physics/collision/cuda'
$evidence = 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window'
$outDir = 'target/g10-p5-g5'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$out = Join-Path $outDir 'witness_probe.exe'
$cudartLib = Get-ChildItem (Join-Path $root 'target/toolchains/cuda-12.8.1') -Recurse -Filter 'cudart.lib' |
    Select-Object -First 1
if (-not $cudartLib) { throw 'cudart.lib not found' }
$cudartLibDir = $cudartLib.DirectoryName
$cudartDll = Get-ChildItem $cudartLibDir -Filter 'cudart64_*.dll' | Select-Object -First 1
if (-not $cudartDll) {
    $cudartDll = Get-ChildItem (Join-Path $root 'target/toolchains/cuda-12.8.1/bin') -Filter 'cudart64_*.dll' |
        Select-Object -First 1
}
Copy-Item $cudartDll.FullName (Join-Path $outDir $cudartDll.Name) -Force

$command = '"' + $nvcc + '" -std=c++17 -O3 -arch=sm_89 --fmad=true -Xcompiler /utf-8' +
    ' -I ' + $deviceDir + ' -L "' + $cudartLibDir + '" -o "' + $out + '" ' + $evidence + '/witness_probe.cu'
cmd /c ('"' + $vcvars + '" >nul 2>&1 && ' + $command)
if ($LASTEXITCODE -ne 0) { throw "nvcc witness_probe build failed exit=$LASTEXITCODE" }

function Sha256([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }

$sources = @('witness_probe.cu') + (Get-ChildItem $deviceDir -Filter '*.cu' | Sort-Object Name | ForEach-Object { $_.Name })
$lines = @(
    "witness_probe_build schema=1",
    "nvcc=$(& $nvcc --version | Select-String -Pattern 'release' | ForEach-Object { $_.Line.Trim() })",
    "nvcc_path=$nvcc",
    "command=$command",
    "fmad=true fast_math=false arch=sm_89 opt=-O3",
    "include_dir=$deviceDir",
    "probe_source=docs/development-evidence/g10-p5-gpu-kernels/gpu-window/witness_probe.cu",
    "probe_source_sha256=$(Sha256 (Join-Path $evidence 'witness_probe.cu'))",
    "probe_binary=$out",
    "probe_binary_sha256=$(Sha256 $out)",
    "probe_binary_bytes=$((Get-Item $out).Length)"
)
foreach ($name in $sources) {
    $path = if ($name -eq 'witness_probe.cu') { Join-Path $evidence $name } else { Join-Path $deviceDir $name }
    $lines += "source_sha256 $name $(Sha256 $path)"
}
$report = Join-Path $evidence 'build-witness-probe-report.txt'
[System.IO.File]::WriteAllLines((Join-Path (Get-Location) $report), $lines, (New-Object System.Text.UTF8Encoding($false)))
Write-Output "BUILD_OK witness_probe_sha256=$(Sha256 $out)"
Get-Content $report | Select-Object -First 5
