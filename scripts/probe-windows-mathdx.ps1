param([Parameter(Mandatory=$true)][ValidateRange(1,3)][int]$Attempt)
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root
$dir=Join-Path $root "target/native-mathdx7/attempt-$Attempt"
if(Test-Path -LiteralPath $dir){throw 'Attempt directory already exists'}
[void](New-Item -ItemType Directory -Path $dir)
$site=Join-Path $root 'target/mjwarp-parallel/venv/Lib/site-packages'
$dll=Join-Path $site 'warp/bin/warp.dll'
$contract=Get-Content docs/evidence/native-mathdx7/contract.json -Raw|ConvertFrom-Json
foreach($pin in $contract.files){if((Get-FileHash $pin.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pin.sha256){throw "Frozen source mismatch $($pin.path)"}}
$record=Import-Csv (Join-Path $site 'warp_lang-1.15.0.dist-info/RECORD') -Header path,hash,size|Where-Object path -eq 'warp/bin/warp.dll'
$sha=(Get-FileHash $dll -Algorithm SHA256).Hash.ToLowerInvariant()
$hash='sha256='+[Convert]::ToBase64String([Convert]::FromHexString($sha)).TrimEnd('=').Replace('+','-').Replace('/','_')
if($record.hash -ne $hash -or [long]$record.size -ne (Get-Item $dll).Length){throw 'DLL differs from pinned WHEEL RECORD'}
Copy-Item -LiteralPath docs/evidence/native-mathdx7/contract.json -Destination (Join-Path $dir 'contract.json')
$vswhere="${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(!$vs){throw 'MSVC unavailable'}
$dev=Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
& cmd /c "call `"$dev`" -arch=x64 -host_arch=x64 >nul && set" | ForEach-Object {
 if($_ -match '^([^=]+)=(.*)$'){[Environment]::SetEnvironmentVariable($matches[1],$matches[2],'Process')}
}
$exe=Join-Path $dir 'native-mathdx-probe.exe'
& cl /nologo /EHsc /MD /std:c++17 /utf-8 /W4 /WX scripts/native-mathdx-probe.cpp "/Fe:$exe" "/Fo:$dir/probe.obj" 2>&1 | Tee-Object (Join-Path $dir 'compile.log')
if($LASTEXITCODE){throw "MSVC exit $LASTEXITCODE"}
& $exe $dll $dir 2>&1 | Tee-Object (Join-Path $dir 'probe.log')
$exit=$LASTEXITCODE
$receipt=Get-Content (Join-Path $dir 'receipt.json') -Raw|ConvertFrom-Json
$receipt|Add-Member attempt $Attempt
$receipt|Add-Member exit_code $exit
$receipt|Add-Member dll_sha256 $sha
$receipt|Add-Member exe_sha256 (Get-FileHash $exe -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText((Join-Path $dir 'receipt.json'),(($receipt|ConvertTo-Json -Depth 12).Replace("`r`n","`n")+"`n"),[Text.UTF8Encoding]::new($false))
if($exit){throw "Native probe exit $exit; keep receipt and attempt directory"}
