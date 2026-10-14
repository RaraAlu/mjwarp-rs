#Requires -Version 7.0
[CmdletBinding()]
param([ValidateRange(1,3)][int]$Attempt=1)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$root=[IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$output=Join-Path $root "target/native-sparse-mass/attempt-$Attempt"
if(Test-Path -LiteralPath $output){throw 'Refuse to overwrite an existing new-purpose run'}
New-Item -ItemType Directory -Path $output | Out-Null
function Sha([string]$Path){(Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()}
$package=Join-Path $root 'target/toolchains/mujoco-3.12.0/package'
$library=Join-Path $package 'bin/mujoco.dll'
if((Sha $library) -ne '79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b'){throw 'Native DLL identity mismatch'}
# 只核对已有MJB，不生成模型或参考。
$groups=@('fixtures/native-probe','fixtures/kinematics','fixtures/com-position','fixtures/mass-matrix')
$paths=@('fixtures/native-probe/mixed-joints.mjb','fixtures/native-probe/inertial-tree.mjb','fixtures/kinematics/rotated-tree.mjb',
    'fixtures/native-probe/zero-dof.mjb','fixtures/com-position/massless-tree.mjb','fixtures/mass-matrix/armature-chain.mjb')
foreach($path in $paths){
    $directory=(Split-Path $path -Parent).Replace('\','/')
    if($directory -notin $groups){throw 'Unexpected fixture directory'}
    $manifest=Get-Content (Join-Path $root "$directory/manifest.json") -Raw | ConvertFrom-Json
    $name=Split-Path $path -Leaf
    $pins=@($manifest.files|Where-Object { $_.path -eq $name -or $_.path -eq $path })
    if($pins.Count -ne 1 -or (Sha (Join-Path $root $path)) -ne $pins[0].sha256){throw "Frozen MJB mismatch: $path"}
}
$massManifest=Get-Content (Join-Path $root 'fixtures/mass-matrix/manifest.json') -Raw | ConvertFrom-Json
foreach($name in @('mixed-joints','inertial-tree','rotated-tree','zero-dof','massless-tree','armature-chain')){
    $path="fixtures/mass-matrix/$name.json"
    $pins=@($massManifest.files|Where-Object {$_.path -eq $path -or $_.path -eq "$name.json"})
    if($pins.Count -ne 1 -or (Sha (Join-Path $root $path)) -ne $pins[0].sha256){throw "Frozen model JSON mismatch: $name"}
}
$solveManifest=Get-Content (Join-Path $root 'fixtures/mjwarp-mass-solve/manifest.json') -Raw | ConvertFrom-Json
if((Sha (Join-Path $root 'fixtures/mjwarp-mass-solve/reference.json')) -ne $solveManifest.reference_sha256){throw 'Frozen block reference mismatch'}
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if($LASTEXITCODE -ne 0 -or -not $vs){throw 'MSVC unavailable'}
$devcmd=Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
$lines=& $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 && set"
if($LASTEXITCODE -ne 0){throw 'MSVC environment failed'}
foreach($line in $lines){$i=$line.IndexOf('=');if($i -gt 0){[Environment]::SetEnvironmentVariable($line.Substring(0,$i),$line.Substring($i+1),'Process')}}
$exe=Join-Path $output 'sparse-preflight.exe'
$compileLog=Join-Path $output 'preflight-build.log'
& cl /nologo /EHsc /MD /std:c++17 /utf-8 /W4 /WX "/I$(Join-Path $package 'include')" "/I$(Join-Path $root 'include')" `
    (Join-Path $root 'tests/native/sparse_preflight.cpp') "/Fe:$exe" "/Fo:$(Join-Path $output 'preflight.obj')" *> $compileLog
if($LASTEXITCODE -ne 0){Get-Content $compileLog;throw 'New sparse ABI preflight build failed'}
$env:MJWARP_MUJOCO_ROOT=$package
$env:MJWARP_MUJOCO_DLL=$library
$env:MJWARP_SPARSE_PREFLIGHT_EXE=$exe
$env:MJWARP_NATIVE_SPARSE_OUTPUT=$output
$watch=[Diagnostics.Stopwatch]::StartNew()
Push-Location $root
try{
    & cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe --test native_sparse_mass -- --ignored --test-threads=1 --nocapture *> (Join-Path $output 'tests.log')
    $exit=$LASTEXITCODE
}finally{Pop-Location}
$watch.Stop()
$log=Get-Content (Join-Path $output 'tests.log') -Raw
Write-Host $log
if($exit -ne 0 -or $log -notmatch 'test result: ok\. 3 passed; 0 failed; 0 ignored;'){throw 'New sparse capture tests failed'}
$receipt=[ordered]@{schema=1;attempt=$Attempt;passed=3;failed=0;ignored=0;test_wall_ms=$watch.ElapsedMilliseconds;
    command='cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe --test native_sparse_mass -- --ignored --test-threads=1 --nocapture';
    preflight_exe_sha256=(Sha $exe);native_dll_sha256=(Sha $library);test_log_sha256=(Sha (Join-Path $output 'tests.log'));
    cpu_physics_calls=0;gpu_sessions=0;python_calls=0;old_target_calls=0;reference_author_calls=0;full_G03_NP_accepted=$false}
[IO.File]::WriteAllText((Join-Path $output 'receipt.json'),(($receipt|ConvertTo-Json -Depth 6).Replace("`r`n","`n")+"`n"),[Text.UTF8Encoding]::new($false))
