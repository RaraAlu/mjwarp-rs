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
    throw 'native toolkit checksum failed'
}
$package = Join-Path $destination 'package'
Expand-Archive -LiteralPath $archive -DestinationPath $package -Force
if (-not (Test-Path -LiteralPath (Join-Path $package 'include\mujoco\mujoco.h'))) {
    throw 'native header missing'
}
# Downloads the toolkit only; it does not run the model compiler.
# The user must explicitly trust this toolkit.
Write-Host "MJWARP_MUJOCO_ROOT=$package"
Write-Host "MJWARP_MUJOCO_DLL=$(Join-Path $package 'bin\mujoco.dll')"
Write-Host ("PREPARE_NATIVE_SUMMARY exit=0 archive_sha256={0} sha_verified=1" -f $sha)
