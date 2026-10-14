param([Parameter(Mandatory=$true)][ValidateRange(1,3)][int]$Attempt)
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root
$dir=Join-Path $root "target/sparse-solve-reference/attempt-$Attempt"
if(Test-Path -LiteralPath $dir){throw 'Attempt directory already exists'}
[void](New-Item -ItemType Directory -Path $dir)
function HashFile([string]$path){(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
$contract=Get-Content fixtures/sparse-mass-solve/contract.json -Raw | ConvertFrom-Json
foreach($pin in $contract.files){if((HashFile $pin.path) -ne $pin.sha256){throw "Pin mismatch: $($pin.path)"}}
Copy-Item -LiteralPath fixtures/sparse-mass-solve/contract.json -Destination (Join-Path $dir 'contract.json')
$models=(Get-Content fixtures/sparse-mass-factor/native.json -Raw | ConvertFrom-Json).models | Select-Object -First 3
$stream=[IO.File]::Create((Join-Path $dir 'input.bin'))
$writer=[IO.BinaryWriter]::new($stream)
try {
 $writer.Write([Text.Encoding]::ASCII.GetBytes('SMREF001'));$writer.Write([int]3)
 foreach($m in $models){
  $bytes=[Text.Encoding]::ASCII.GetBytes($m.name);$writer.Write([int]$bytes.Length);$writer.Write($bytes)
  $writer.Write([int]$m.nv);$writer.Write([int]$m.nc);$writer.Write([int]$m.cases.Count)
  foreach($key in @('m_rownnz','m_rowadr','m_colind')){foreach($v in $m.$key){$writer.Write([int]$v)}}
  foreach($case in $m.cases){foreach($v in $case.matrix){$writer.Write([single]$v)}}
 }
} finally {$writer.Dispose();$stream.Dispose()}
$vswhere="${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(!$vs){throw 'MSVC unavailable'}
$dev=Join-Path $vs 'Common7/Tools/VsDevCmd.bat'
& cmd /c "call `"$dev`" -arch=x64 -host_arch=x64 >nul && set" | ForEach-Object {
 if($_ -match '^([^=]+)=(.*)$'){[Environment]::SetEnvironmentVariable($matches[1],$matches[2],'Process')}
}
$exe=Join-Path $dir 'reference.exe'
& cl /nologo /EHsc /MD /std:c++17 /fp:strict /W4 /WX fixtures/sparse-mass-solve/reference.cpp "/Fe:$exe" "/Fo:$dir/reference.obj" 2>&1 | Tee-Object (Join-Path $dir 'compile.log')
if($LASTEXITCODE){throw "Compiler exit $LASTEXITCODE"}
& $exe (Join-Path $dir 'input.bin') (Join-Path $dir 'reference.json') 2>&1 | Tee-Object (Join-Path $dir 'author.log')
if($LASTEXITCODE){throw "Author exit $LASTEXITCODE"}
$output='fixtures/sparse-mass-solve/reference.json'
if(Test-Path -LiteralPath $output){throw 'Frozen output already exists'}
Copy-Item -LiteralPath (Join-Path $dir 'reference.json') -Destination $output
$manifest=[ordered]@{schema=1;purpose='Independent f64 partial-pivot LU of immutable native f32 CSR matrices; no MuJoCo, factorM or product code';attempt=$Attempt;frozen=$true;contract_sha256=(HashFile fixtures/sparse-mass-solve/contract.json);binary_input_sha256=(HashFile (Join-Path $dir 'input.bin'));author_exe_sha256=(HashFile $exe);output_sha256=(HashFile $output);reference='not direct mj_solveM or frozen generated sparse solve';rhs_count=3;models=3;cases=6;rhs_columns=18}
$json=($manifest | ConvertTo-Json -Depth 10).Replace("`r`n","`n")+"`n"
[IO.File]::WriteAllText((Join-Path $root 'fixtures/sparse-mass-solve/manifest.json'),$json,[Text.UTF8Encoding]::new($false))
