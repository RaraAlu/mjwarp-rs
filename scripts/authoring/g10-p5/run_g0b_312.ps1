# G0b：离线构建并运行 MuJoCo 3.12 字段 dump 工具（不占 GPU、不跑产品代码）。
#
# 产物：
#   fixtures/g10-p5/mjb-312/<model>.mjb          3.12 编译产物
#   fixtures/g10-p5/dump-312/<model>/*.bin       原始小端字节 + fields.json
#
# 说明：3.15 不能加载 3.12 MJB（实测 different number of sizes），故 3.12 侧
# 只用 3.12 C API 编译 + 重载校验；3.15 侧由 dump_fields_315.py 独立生成。
[CmdletBinding()]
param(
    [string]$Root = '',
    [string]$BuildDir = 'target/g10-p5',
    [string]$FixtureDir = 'fixtures/g10-p5'
)
$ErrorActionPreference = 'Stop'
if (-not $Root) {
    $Root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
}
Set-Location $Root

$pkg = Join-Path $Root 'target/toolchains/mujoco-3.12.0/package'
$include = Join-Path $pkg 'include'
$lib = Join-Path $pkg 'lib'
$bin = Join-Path $pkg 'bin'
if (-not (Test-Path (Join-Path $include 'mujoco/mujoco.h'))) { throw "缺少 MuJoCo 3.12 头文件：$include" }
if (-not (Test-Path (Join-Path $lib 'mujoco.lib'))) { throw "缺少 MuJoCo 3.12 导入库：$lib" }

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path $vswhere)) { throw '缺少 vswhere' }
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw '需要 MSVC 与 Windows SDK' }
$devcmd = Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
$environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'MSVC 环境初始化失败' }
foreach ($line in $environment) {
    if ($line -match '^([^=]+)=(.*)$') {
        [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
    }
}
$env:PATH = "$bin;$env:PATH"

$build = Join-Path $Root $BuildDir
New-Item -ItemType Directory -Force $build | Out-Null
$cpp = Join-Path $Root 'scripts/authoring/g10-p5/dump_collision_fields_312.cpp'
$exe = Join-Path $build 'dump_collision_fields_312.exe'
Write-Host "[g0b] compiling $cpp"
& cl /nologo /O2 /EHsc /std:c++17 /utf-8 "/Fo$build/" "/I$include" "$cpp" /link "/LIBPATH:$lib" mujoco.lib "/OUT:$exe"
if ($LASTEXITCODE -ne 0) { throw 'cl 编译失败' }
if (-not (Test-Path $exe)) { throw "缺少可执行：$exe" }

$models = @(
    @{ Name = 'hfield-box'; Xml = 'fixtures/mjwarp-complex-physics/hfield-box.xml' },
    @{ Name = 'sdf-cylinder'; Xml = 'fixtures/mjwarp-complex-physics/sdf-cylinder.xml' }
)
$mjbDir = Join-Path $Root "$FixtureDir/mjb-312"
$dumpDir = Join-Path $Root "target/g10-p5/dump-312"
New-Item -ItemType Directory -Force $mjbDir | Out-Null
New-Item -ItemType Directory -Force $dumpDir | Out-Null

$summary = @()
foreach ($model in $models) {
    $xml = Join-Path $Root $model.Xml
    $mjb = Join-Path $mjbDir "$($model.Name).mjb"
    $out = Join-Path $dumpDir $model.Name
    New-Item -ItemType Directory -Force $out | Out-Null
    Write-Host "[g0b] 3.12 compile+dump: $($model.Name)"
    & $exe $xml $mjb $out
    if ($LASTEXITCODE -ne 0) { throw "3.12 工具失败：$($model.Name)" }
    $bytes = (Get-Item $mjb).Length
    $sha = (Get-FileHash $mjb -Algorithm SHA256).Hash.ToLower()
    $summary += [pscustomobject]@{ name = $model.Name; mjb_bytes = $bytes; mjb_sha256 = $sha }
    Write-Host "[g0b]   mjb=$mjb bytes=$bytes sha256=$sha"
}
$summary | ConvertTo-Json -Depth 3 | Set-Content -Path (Join-Path $dumpDir 'summary.json') -Encoding utf8
Write-Host '[g0b] 3.12 侧完成（成功构造 3.12 产物并同版本重载校验）'
