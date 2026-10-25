# QA independent review: build QA's own MuJoCo 3.12 dump tool with MSVC (no GPU, no product code).
[CmdletBinding()]
param(
    [string]$Root = 'D:\Rust\mjwarp-rs'
)
$ErrorActionPreference = 'Stop'
Set-Location $Root
$qa = Join-Path $Root 'target/qa-g0b'
$pkg = Join-Path $Root 'target/toolchains/mujoco-3.12.0/package'
$include = Join-Path $pkg 'include'
$lib = Join-Path $pkg 'lib'
$bin = Join-Path $pkg 'bin'

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'MSVC required' }
$devcmd = Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
$environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if ($LASTEXITCODE -ne 0) { throw 'VsDevCmd init failed' }
foreach ($line in $environment) {
    if ($line -match '^([^=]+)=(.*)$') {
        [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
    }
}
$env:PATH = "$bin;$env:PATH"

$cpp = Join-Path $qa 'qa_mjb_dump.cpp'
$exe = Join-Path $qa 'qa_mjb_dump.exe'
& cl /nologo /O2 /EHsc /std:c++17 /utf-8 "/Fo$qa/" "/I$include" "$cpp" /link "/LIBPATH:$lib" mujoco.lib "/OUT:$exe"
if ($LASTEXITCODE -ne 0) { throw 'cl failed' }
if (-not (Test-Path $exe)) { throw "missing exe: $exe" }
Write-Host "[qa-g0b] built $exe"

$models = @(
    @{ Name = 'hfield-box'; Xml = 'fixtures/mjwarp-complex-physics/hfield-box.xml' },
    @{ Name = 'sdf-cylinder'; Xml = 'fixtures/mjwarp-complex-physics/sdf-cylinder.xml' }
)
foreach ($model in $models) {
    $xml = Join-Path $Root $model.Xml
    $mjb = Join-Path $qa ("mjb-312/" + $model.Name + '.mjb')
    $dump = Join-Path $qa ("dump-312/" + $model.Name + "/fromxml")
    New-Item -ItemType Directory -Force (Split-Path -Parent $mjb) | Out-Null
    New-Item -ItemType Directory -Force $dump | Out-Null
    Write-Host "[qa-g0b] 3.12 dump $($model.Name)"
    & $exe dump $xml $mjb $dump
    if ($LASTEXITCODE -ne 0) { throw "3.12 dump failed: $($model.Name)" }
    Write-Host "[qa-g0b] 3.12 dumpmjb(fixture) $($model.Name)"
    $fixtureMjb = Join-Path $Root ("fixtures/g10-p5/mjb-312/" + $model.Name + '.mjb')
    & $exe dumpmjb $fixtureMjb (Join-Path $qa ("dump-312/" + $model.Name + "/frommjb"))
    if ($LASTEXITCODE -ne 0) { throw "3.12 dumpmjb failed: $($model.Name)" }
}
Write-Host '[qa-g0b] 3.12 side done'
