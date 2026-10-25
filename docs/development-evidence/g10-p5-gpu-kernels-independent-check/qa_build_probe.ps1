# QA G5 independent device probe build (nvcc + MSVC environment).
# Uses the repo-cached CUDA 12.8 toolkit and system Visual Studio; does not call the
# delivery build script. ASCII-only so Windows PowerShell 5.1 / pwsh 7 both parse it.
param(
  [string]$Out = "target\qa-g5\qa_probe.exe",
  [string]$Fmad = "true"
)

$ErrorActionPreference = "Stop"
$root = (Get-Location).Path
$nvcc = Join-Path $root "target\toolchains\cuda-12.8.1\bin\nvcc.exe"
$cudart = Join-Path $root "target\toolchains\cuda-12.8.1\cuda_cudart\cuda_cudart-windows-x86_64-12.8.90-archive\lib\x64"
$include = Join-Path $root "src\physics\collision\cuda"
$source = Join-Path $root "docs\development-evidence\g10-p5-gpu-kernels-independent-check\probe\qa_probe.cu"
$report = Join-Path $root "docs\development-evidence\g10-p5-gpu-kernels-independent-check\qa-gpu\build-recipe.txt"

if (-not (Test-Path $nvcc)) { throw "nvcc not found: $nvcc" }

$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vswhere)) { throw "vswhere not found" }
$vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$vcvars = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"
if (-not (Test-Path $vcvars)) { throw "vcvars64.bat not found: $vcvars" }

New-Item -ItemType Directory -Force -Path (Split-Path $Out) | Out-Null
$command = "call `"$vcvars`" >nul && `"$nvcc`" -std=c++17 -O3 -arch=sm_89 --fmad=$Fmad -Xcompiler /utf-8 -I `"$include`" -L `"$cudart`" -o `"$root\$Out`" `"$source`""
Write-Host "+ cmd /c $command"
& cmd /c $command
if ($LASTEXITCODE -ne 0) { throw "nvcc failed: $LASTEXITCODE" }

$hash = (Get-FileHash -Algorithm SHA256 -Path (Join-Path $root $Out)).Hash.ToLower()
$lines = @(
  "qa_probe_build schema=1",
  "nvcc=$(& $nvcc --version | Select-Object -Last 1)",
  "nvcc_path=$nvcc",
  "command=$command",
  "fmad=$Fmad arch=sm_89 opt=-O3",
  "source=probe/qa_probe.cu",
  "source_sha256=$((Get-FileHash -Algorithm SHA256 -Path $source).Hash.ToLower())",
  "binary=$Out",
  "binary_sha256=$hash",
  "product_sources_sha256:"
)
foreach ($file in Get-ChildItem $include -Filter *.cu | Sort-Object Name) {
  $lines += "  $($file.Name) $((Get-FileHash -Algorithm SHA256 -Path $file.FullName).Hash.ToLower())"
}
Set-Content -Path $report -Value $lines -Encoding utf8
Write-Host "binary $Out sha256=$hash"
