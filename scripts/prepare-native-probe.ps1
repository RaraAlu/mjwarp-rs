[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $root 'target\toolchains\mujoco-3.12.0'
New-Item -ItemType Directory -Path $destination -Force | Out-Null
$archive = Join-Path $destination 'mujoco-3.12.0-windows-x86_64.zip'
$sha = 'ffe071c2747dd9513a1c59e7d2428bb678d887f9edec9eb9674b4288a248a8e9'
if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest -Uri 'https://github.com/google-deepmind/mujoco/releases/download/3.12.0/mujoco-3.12.0-windows-x86_64.zip' -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $sha) {
    throw '原生工具包校验失败'
}
$package = Join-Path $destination 'package'
Expand-Archive -LiteralPath $archive -DestinationPath $package -Force
if (-not (Test-Path -LiteralPath (Join-Path $package 'include\mujoco\mujoco.h'))) {
    throw '原生头文件缺失'
}
# 只下载工具，不运行模型编译器。
# 用户须主动信任此工具包。
Write-Host "MJWARP_MUJOCO_ROOT=$package"
Write-Host "MJWARP_MUJOCO_DLL=$(Join-Path $package 'bin\mujoco.dll')"
