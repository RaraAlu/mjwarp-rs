[CmdletBinding()]
param(
    [string]$Scene = 'fixtures/render-bvh/scene-a/scene.xml',
    [string]$FixtureDir = 'fixtures/render-bvh/scene-a',
    [string]$EvidenceDir = 'docs/development-evidence/render-bvh/s0a-reference-fixture',
    [string]$MujocoPackage = 'target/toolchains/mujoco-3.12.0/package',
    [string]$Venv = 'target/mjwarp-parallel/venv'
)
# Offline authoring: MuJoCo 3.12 vs 3.15 render-model parity for S0a.
# OFFLINE ONLY: never referenced by product build/tests/gates/scripts; no GPU.
# 1) compiles compile_scene_312.cpp against MuJoCo 3.12 C API and produces the
#    3.12 side (scene-312.mjb + model-312.json);
# 2) runs model_parity.py (pinned venv, MuJoCo 3.15) to produce the 3.15 side
#    (scene-315.mjb + model-315.json) and parity.json.
$ErrorActionPreference = 'Stop'
if (-not ($IsWindows -or $env:OS -eq 'Windows_NT')) { throw '此脚本只验证 Windows' }
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
Set-Location $root
$include = Join-Path $root (Join-Path $MujocoPackage 'include')
$lib = Join-Path $root (Join-Path $MujocoPackage 'lib')
$bin = Join-Path $root (Join-Path $MujocoPackage 'bin')
if (-not (Test-Path (Join-Path $include 'mujoco/mujoco.h'))) { throw "缺少 MuJoCo 3.12 头文件：$include" }
if (-not (Test-Path (Join-Path $lib 'mujoco.lib'))) { throw "缺少 MuJoCo 3.12 导入库：$lib" }
$python = Join-Path (Resolve-Path $Venv).Path 'Scripts/python.exe'
if (-not (Test-Path $python)) { throw "缺少 pinned venv python：$python" }
$buildDir = Join-Path $root 'target/render-bvh/s0a'
New-Item -ItemType Directory -Force $buildDir | Out-Null
New-Item -ItemType Directory -Force (Join-Path $root $EvidenceDir) | Out-Null
$cpp = Join-Path $root 'scripts/authoring/render-bvh/compile_scene_312.cpp'
$exe = Join-Path $buildDir 'compile_scene_312.exe'

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path $vswhere)) { throw '缺少 vswhere' }
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw '需要 MSVC 与 Windows SDK' }
$devcmd = Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
$environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC 环境初始化失败' }
foreach ($line in $environment) {
    if ($line -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}
Push-Location $buildDir
try {
    & cl /nologo /O2 /EHsc /std:c++17 "/I$include" "$cpp" /link "/LIBPATH:$lib" mujoco.lib "/OUT:$exe"
    if ($LASTEXITCODE -ne 0) { throw "3.12 C++ 编译失败（exit=$LASTEXITCODE）" }
}
finally {
    Pop-Location
}
$env:PATH = "$bin;$env:PATH"
& $python -B scripts/authoring/render-bvh/model_parity.py --exe $exe --cpp $cpp --scene $Scene --out $FixtureDir --evidence $EvidenceDir
$code = $LASTEXITCODE
if ($code -ne 0) { throw "model parity 失败（exit=$code），见 $EvidenceDir/parity.json" }
Write-Host "S0a parity 通过：$FixtureDir / $EvidenceDir"
