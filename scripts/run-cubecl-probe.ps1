[CmdletBinding()]
param(
    [ValidateSet('cpp', 'llvm')]
    [string]$Backend = 'cpp',
    [ValidateSet('affine', 'atomic-sum', 'block-reduce', 'block-scan', 'control-flow', 'small-solve')]
    [string]$Kernel = 'affine',
    [ValidateRange(1, 1048576)]
    [int]$Elements = 257,
    [switch]$GpuTests
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$cuda = Join-Path $root 'target\toolchains\cuda-12.8.1'
if (-not (Test-Path -LiteralPath (Join-Path $cuda 'components.json'))) {
    throw '请先运行prepare-cuda-probe.ps1'
}

# 仅修改当前脚本进程。
# 子进程继承环境，父终端不受影响。
$env:CUDA_PATH = $cuda
$env:PATH = (Join-Path $cuda 'bin') + ';' + $env:PATH
$cargoArgs = @()
if ($Backend -eq 'llvm') {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) {
        throw 'LLVM路线需要MSVC与Windows SDK'
    }
    $vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $vs) {
        throw '请安装C++生成工具与Windows SDK'
    }
    $devcmd = Join-Path $vs 'Common7\Tools\VsDevCmd.bat'
    if (-not (Test-Path -LiteralPath $devcmd)) {
        throw '找不到VsDevCmd.bat'
    }
    # call避免cmd吞掉路径外层引号。
    $environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
    if ($LASTEXITCODE -ne 0) {
        throw 'MSVC环境初始化失败'
    }
    foreach ($line in $environment) {
        if ($line -match '^([^=]+)=(.*)$') {
            [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
        }
    }
    # 固定调用目标；不设置rustup default。
    $cargoArgs += '+stable-x86_64-pc-windows-msvc'
} else {
    $cargoArgs += '+stable-x86_64-pc-windows-gnu'
    $gnu = Join-Path $root 'target\toolchains\mingw\mingw64\bin'
    if (Test-Path -LiteralPath (Join-Path $gnu 'dlltool.exe')) {
        $env:PATH = $gnu + ';' + $env:PATH
    }
}

Push-Location $root
try {
    $feature = "cubecl-$Backend-probe"
    if ($GpuTests) {
        & cargo @cargoArgs test --locked --features $feature --test cubecl_probe -- --ignored --test-threads=1
    } else {
        & cargo @cargoArgs run --locked --features $feature -- probe --backend "cubecl-$Backend" --kernel $Kernel --elements $Elements
    }
    if ($LASTEXITCODE -ne 0) {
        throw "探针失败，退出码$LASTEXITCODE"
    }
} finally {
    Pop-Location
}
