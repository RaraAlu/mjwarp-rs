param([Parameter(Mandatory=$true)][ValidateRange(1,3)][int]$Attempt)
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root
$dir=Join-Path $root "target/native-mathdx7-link/attempt-$Attempt"
if(Test-Path -LiteralPath $dir){throw 'Attempt directory already exists'}
[void](New-Item -ItemType Directory -Path $dir)
$contractPath='docs/evidence/native-mathdx7-link/contract.json'
$contract=Get-Content $contractPath -Raw|ConvertFrom-Json
Copy-Item -LiteralPath $contractPath -Destination (Join-Path $dir 'contract.json')
foreach($pin in $contract.files){if((Get-FileHash $pin.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pin.sha256){throw "Frozen source mismatch $($pin.path)"}}
$site=Join-Path $root 'target/mjwarp-parallel/venv/Lib/site-packages'
$dll=Join-Path $site 'warp/bin/warp.dll'
$record=Import-Csv (Join-Path $site 'warp_lang-1.15.0.dist-info/RECORD') -Header path,hash,size|Where-Object path -eq 'warp/bin/warp.dll'
$sha=(Get-FileHash $dll -Algorithm SHA256).Hash.ToLowerInvariant()
$hash='sha256='+[Convert]::ToBase64String([Convert]::FromHexString($sha)).TrimEnd('=').Replace('+','-').Replace('/','_')
if($record.hash -ne $hash -or [long]$record.size -ne (Get-Item $dll).Length){throw 'DLL differs from pinned WHEEL RECORD'}
$vswhere="${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(!$vs){throw 'MSVC unavailable'}
$dev=Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
& cmd /c "call `"$dev`" -arch=x64 -host_arch=x64 >nul && set" | ForEach-Object {
 if($_ -match '^([^=]+)=(.*)$'){[Environment]::SetEnvironmentVariable($matches[1],$matches[2],'Process')}
}
$exe=Join-Path $dir 'native-mathdx7-link.exe'
$cuda=Join-Path $root 'target/toolchains/cuda-12.8.1'
& cl /nologo /EHsc /MD /std:c++17 /utf-8 /W4 /WX "/I$cuda/include" scripts/native-mathdx7-link.cpp "/Fe:$exe" "/Fo:$dir/probe.obj" 2>&1 | Tee-Object (Join-Path $dir 'compile.log')
if($LASTEXITCODE){throw "MSVC exit $LASTEXITCODE"}
$env:CUDA_PATH=$cuda
$env:PATH=(Join-Path $cuda 'bin')+';'+$env:PATH
$env:WARP_DEBUG='1'
& $exe $dll (Join-Path $site 'warp/native') (Join-Path $root 'target/native-mathdx7/attempt-1/potrf7.lto') (Join-Path $root 'target/native-mathdx7/attempt-1/potrf7_fatbin.lto') (Join-Path $root 'scripts/native-mathdx7-call.cu') $dir 2>&1 | Tee-Object (Join-Path $dir 'probe.log')
$exit=$LASTEXITCODE
$receipt=Get-Content (Join-Path $dir 'receipt.json') -Raw|ConvertFrom-Json
$receipt|Add-Member attempt $Attempt
$receipt|Add-Member exit_code $exit
$receipt|Add-Member dll_sha256 $sha
$receipt|Add-Member exe_sha256 (Get-FileHash $exe -Algorithm SHA256).Hash.ToLowerInvariant()
foreach($name in @('mathdx7.cubin','device-output.bin')){
 $file=Join-Path $dir $name
 if(Test-Path -LiteralPath $file){$receipt|Add-Member ($name.Replace('.','_')+'_sha256') (Get-FileHash $file -Algorithm SHA256).Hash.ToLowerInvariant()}
}
[IO.File]::WriteAllText((Join-Path $dir 'receipt.json'),(($receipt|ConvertTo-Json -Depth 12).Replace("`r`n","`n")+"`n"),[Text.UTF8Encoding]::new($false))
if($exit){throw "Native link/device exit $exit; keep receipt and attempt directory"}
