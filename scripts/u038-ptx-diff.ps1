<#
U038 三维运动学逐表达式 PTX 对照工具（只读、只编译不跑内核）。

一条命令复现：
  pwsh -NoProfile -File scripts/u038-ptx-diff.ps1

前置（本机 pinned 环境，均为仓库 target 下的既有产物）：
  - NVRTC 12.8 工具链：target/toolchains/cuda-12.8.1/bin（只加载 nvrtc64_120_0.dll）
  - pinned Warp 缓存：target/mjwarp-parallel/cache-quarantine/warp-1.15.0-double-version-2026-10-18
  - Wheel native 头文件：target/mjwarp-parallel/venv/Lib/site-packages/warp/native

本脚本不修改产品源码、不启动 GPU 内核；只把产品/上游 CUDA 编译成 PTX 后做逐表达式对照。
#>
[CmdletBinding()]
param(
    [string]$RepoRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)),
    [string]$CacheDir = '',
    [string]$WarpInclude = '',
    [string]$Out = '',
    [string]$CudaBin = ''
)

$ErrorActionPreference = 'Stop'

if (-not $CacheDir) {
    $CacheDir = Join-Path $RepoRoot 'target/mjwarp-parallel/cache-quarantine/warp-1.15.0-double-version-2026-10-18/wp_mujoco_warp._src.smooth_ca013d6'
}
if (-not $WarpInclude) {
    $WarpInclude = Join-Path $RepoRoot 'target/mjwarp-parallel/venv/Lib/site-packages/warp/native'
}
if (-not $Out) {
    $Out = Join-Path $RepoRoot 'target/u038-ptx-diff-tool/out'
}
if (-not $CudaBin) {
    $CudaBin = Join-Path $RepoRoot 'target/toolchains/cuda-12.8.1/bin'
}

function Require-Path([string]$Path, [string]$What) {
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "缺少$What：$Path（需先准备 pinned 环境，见脚本头注释）"
    }
}

Require-Path $RepoRoot '仓库根目录'
Require-Path (Join-Path $RepoRoot 'src/physics/mod.rs') '产品源码'
Require-Path (Join-Path $CacheDir 'wp_mujoco_warp._src.smooth_ca013d6.cu') 'pinned 上游生成 CUDA'
Require-Path (Join-Path $CacheDir 'wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx') 'pinned 上游冻结 PTX'
Require-Path $WarpInclude 'Warp native 头文件目录'
Require-Path (Join-Path $CudaBin 'nvrtc64_120_0.dll') 'NVRTC 12.8 动态库'

$env:PATH = "$CudaBin$([IO.Path]::PathSeparator)$env:PATH"
$manifest = Join-Path $RepoRoot 'scripts/u038-ptx-diff-tool/Cargo.toml'

$arguments = @(
    'run', '--quiet', '--offline', '--manifest-path', $manifest, '--',
    '--repo', $RepoRoot,
    '--cache', $CacheDir,
    '--warp-include', $WarpInclude,
    '--out', $Out,
    '--cuda-bin', $CudaBin
)
& cargo @arguments
if ($LASTEXITCODE -ne 0) {
    throw "PTX 对照工具执行失败（cargo 退出码 $LASTEXITCODE）"
}
Write-Host "PTX 对照完成：$(Join-Path $Out 'ptx-diff.json')"
