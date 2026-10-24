[CmdletBinding()]
param(
    [string]$Venv = 'target/mjwarp-parallel/venv',
    [string]$CudaPath = 'target/toolchains/cuda-12.8.1',
    [string]$Log = 'docs/development-evidence/render-bvh/s0b-reference-capture/run.log'
)
# Offline authoring wrapper: S0b frozen render reference capture.
# OFFLINE ONLY: never referenced by product build/tests/gates/scripts.
# GPU is used exclusively for the capture; the script fails loudly on any
# environment mismatch. Two consecutive captures must have identical sha256.
# The log is written as UTF-8 with LF endings (no PowerShell object formatting),
# so the committed evidence stays text-only and reproducible.
$ErrorActionPreference = 'Stop'
if (-not ($IsWindows -or $env:OS -eq 'Windows_NT')) { throw '此脚本只验证 Windows' }
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
Set-Location $root
$python = Join-Path (Resolve-Path $Venv).Path 'Scripts/python.exe'
if (-not (Test-Path $python)) { throw "缺少 pinned venv python：$python" }
$cudaBin = Join-Path (Resolve-Path $CudaPath).Path 'bin'
$env:CUDA_PATH = (Resolve-Path $CudaPath).Path
$env:PATH = "$cudaBin;$env:PATH"
$before = (& nvidia-smi --query-gpu=name,driver_version,memory.used,memory.total --format=csv,noheader) -join '; '
$computeBefore = @(& nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>&1 | ForEach-Object { "$_".Trim() })
$header = @(
    'S0b frozen reference capture (offline authoring, GPU exclusive)',
    "GPU before: $before",
    "compute apps before: $($computeBefore -join '; ')",
    "command: $python -B scripts/authoring/render-bvh/capture_reference.py"
)
$body = & $python -B scripts/authoring/render-bvh/capture_reference.py 2>&1 | ForEach-Object { "$_" }
$code = $LASTEXITCODE
$after = (& nvidia-smi --query-gpu=name,driver_version,memory.used,memory.total --format=csv,noheader) -join '; '
$lines = $header + $body + @("GPU after: $after", "exit: $code")
$LogDir = Split-Path -Parent $Log
if ($LogDir -and -not (Test-Path $LogDir)) { New-Item -ItemType Directory -Force $LogDir | Out-Null }
((($lines -join "`n") + "`n")) | Set-Content -Path $Log -Encoding utf8 -NoNewline
$lines | ForEach-Object { Write-Host $_ }
if ($code -ne 0) { throw "S0b reference capture failed (exit=$code)" }
Write-Host 'S0b reference capture OK'
