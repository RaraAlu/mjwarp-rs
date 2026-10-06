[CmdletBinding()]
param([switch]$Release)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path -LiteralPath $vswhere)) { throw '请安装MSVC与Windows SDK' }
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw '请安装C++生成工具' }
$devcmd = Join-Path $vs 'Common7\Tools\VsDevCmd.bat'
$environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC环境初始化失败' }
foreach ($line in $environment) {
    if ($line -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}
$cuda = Join-Path $root 'target\toolchains\cuda-12.8.1'
if (-not (Test-Path -LiteralPath (Join-Path $cuda 'components.json'))) {
    throw '请先准备私有CUDA工具链'
}
$env:CUDA_PATH = $cuda
$env:PATH = (Join-Path $cuda 'bin') + ';' + $env:PATH
$native = Join-Path $root 'target\external-probe-native'
New-Item -ItemType Directory -Force -Path $native | Out-Null
Push-Location $native
try {
    # 只核对数据布局；不启动Python。
    & cl /nologo /std:c++17 /EHsc /W4 /WX "/I$(Join-Path $root 'include')" (Join-Path $root 'tests\native\external_descriptor.cpp') /Fe:external-descriptor.exe
    if ($LASTEXITCODE -ne 0) { throw 'C++描述符编译失败' }
    & (Join-Path $native 'external-descriptor.exe')
    if ($LASTEXITCODE -ne 0) { throw 'C++描述符核对失败' }
} finally { Pop-Location }
Push-Location $root
try {
    $arguments = @('+stable-x86_64-pc-windows-msvc', 'test', '--locked', '--all-features', '--test', 'external_resources')
    if ($Release) { $arguments += '--release' }
    & cargo @arguments -- --ignored --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw '外部资源GPU探针失败' }
} finally { Pop-Location }
