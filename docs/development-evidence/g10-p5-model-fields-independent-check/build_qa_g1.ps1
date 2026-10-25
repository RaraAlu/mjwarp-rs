# QA G1 independent ABI probe build/run: compile QA's probe against pre-G1 and G1 snapshots,
# dump every G01 field's raw bytes from both 3.12 MJBs, and dump the reference fields with QA's
# own MuJoCo 3.12 C tool. No product Rust reader logic is used for the independent export.
[CmdletBinding()]
param(
    [string]$Root = 'D:\Rust\mjwarp-rs'
)
$ErrorActionPreference = 'Stop'
Set-Location $Root
$qa = Join-Path $Root 'target/qa-g1'
$pkg = Join-Path $Root 'target/toolchains/mujoco-3.12.0/package'
$include = Join-Path $pkg 'include'

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

$dll = Join-Path $pkg 'bin/mujoco.dll'
$models = @('hfield-box', 'sdf-cylinder')
$probe = Join-Path $qa 'qa_abi_probe.cpp'
foreach ($side in @('pre', 'post')) {
    $snap = Join-Path $qa $side
    $exe = Join-Path $qa ("qa_abi_probe-" + $side + '.exe')
    & cl /nologo /O2 /EHsc /std:c++17 /utf-8 "/Fo$qa\\" "/I$include" "/I$snap" "/I$snap\\include" "$probe" "/Fe:$exe"
    if ($LASTEXITCODE -ne 0) { throw "cl failed for $side" }
    $maxid = if ($side -eq 'pre') { '44' } else { '58' }
    foreach ($model in $models) {
        $mjb = Join-Path $Root ("fixtures/g10-p5/mjb-312/" + $model + '.mjb')
        $out = Join-Path $qa ("abi-" + $side + "/" + $model)
        New-Item -ItemType Directory -Force $out | Out-Null
        Write-Host "[qa-g1] $side $model"
        & $exe $dll $mjb $out $maxid
        if ($LASTEXITCODE -ne 0) { throw "probe failed: $side $model" }
    }
}

# QA reference dump with QA's own MuJoCo 3.12 C++ tool (G0b tool, QA-written).
$refTool = Join-Path $Root 'target/qa-g0b/qa_mjb_dump.exe'
if (-not (Test-Path $refTool)) { throw 'missing QA reference dumper target/qa-g0b/qa_mjb_dump.exe' }
$env:PATH = "$(Join-Path $pkg 'bin');$env:PATH"
foreach ($model in $models) {
    $mjb = Join-Path $Root ("fixtures/g10-p5/mjb-312/" + $model + '.mjb')
    $out = Join-Path $qa ("ref-312/" + $model)
    New-Item -ItemType Directory -Force $out | Out-Null
    & $refTool dumpmjb $mjb $out
    if ($LASTEXITCODE -ne 0) { throw "reference dump failed: $model" }
}
Write-Host '[qa-g1] done'
