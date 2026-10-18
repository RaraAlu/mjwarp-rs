$ErrorActionPreference='Stop'
$task='2026-10-19-layers-m3-rne-bias-clippy-structural-fix'
$root='docs/development-evidence/layers-m3-rne-bias-clippy-structural-fix'
$formal="$root/formal-run"
$runtime="target/normify-evidence/$task"
$baseCommit='d029f4ed5fa0402d637a04cb1aa4d0c5d4159f7b'
$source='src/physics/rne_bias.rs'
if(Test-Path -LiteralPath $formal){throw '正式归档目录已存在'}
New-Item -ItemType Directory -Path $formal|Out-Null
$mappings=[Collections.Generic.List[object]]::new()
function Copy-New([string]$Source,[string]$Destination){
  if(-not(Test-Path -LiteralPath $Source -PathType Leaf)){throw "缺少原件：$Source"}
  $parent=Split-Path -Parent $Destination;if(-not(Test-Path -LiteralPath $parent)){New-Item -ItemType Directory -Path $parent|Out-Null}
  $bytes=[IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source));$stream=[IO.FileStream]::new((Join-Path (Get-Location) $Destination),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
  try{$stream.Write($bytes,0,$bytes.Length)}finally{$stream.Dispose()}
  $relative=$Destination.Substring($formal.Length+1).Replace('\','/');$mappings.Add([ordered]@{path=$relative;source=$Source})
}
function Copy-GitBlob([string]$Commit,[string]$Path,[string]$Destination){
  $parent=Split-Path -Parent $Destination;if(-not(Test-Path -LiteralPath $parent)){New-Item -ItemType Directory -Path $parent|Out-Null}
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName='git';$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.ArgumentList.Add('cat-file');$info.ArgumentList.Add('blob');$info.ArgumentList.Add("${Commit}:$Path")
  $process=[Diagnostics.Process]::new();$process.StartInfo=$info;$stream=[IO.FileStream]::new((Join-Path (Get-Location) $Destination),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
  try{if(-not $process.Start()){throw '无法启动git'};$process.StandardOutput.BaseStream.CopyTo($stream);$process.WaitForExit();$errorText=$process.StandardError.ReadToEnd();if($process.ExitCode -ne 0){throw "git读取失败：$errorText"}}
  finally{$stream.Dispose();$process.Dispose()}
  $relative=$Destination.Substring($formal.Length+1).Replace('\','/');$mappings.Add([ordered]@{path=$relative;source="git:${Commit}:$Path"})
}
$runs=@(
  [ordered]@{id='host';dir='check-01-host'},
  [ordered]@{id='format-first';dir='check-02-format';registration='format'},
  [ordered]@{id='format-apply';dir='check-02-format-apply'},
  [ordered]@{id='format-retry';dir='check-02-format-retry'},
  [ordered]@{id='format-final';dir='check-02-format-final'},
  [ordered]@{id='clippy-first';dir='check-03-clippy';registration='clippy'},
  [ordered]@{id='clippy';dir='check-03-clippy-retry';registration='clippy-retry'},
  [ordered]@{id='inventory-first';dir='check-04-inventory';registration='inventory'},
  [ordered]@{id='inventory';dir='check-04-inventory-retry';registration='inventory-retry'},
  [ordered]@{id='evidence-check';dir='check-05-evidence-check'}
)
foreach($run in $runs){
  $dest="$formal/runs/$($run.id)"
  $registration=if($run.registration){$run.registration}else{$run.id}
  Copy-New "$runtime/registrations/$registration.json" "$dest/registration.json"
  foreach($name in @('started.json','result.json','stdout.log','stderr.log')){Copy-New "$runtime/$($run.dir)/$name" "$dest/$name"}
}
$prior='docs/development-evidence/layers-m3-rne-bias/formal-run/runs'
foreach($generation in @(
  [ordered]@{name='initial-exit-101';prefix='clippy-first'},
  [ordered]@{name='allow-exit-0';prefix='clippy-final'}
)){
  $dest="$formal/historical/$($generation.name)"
  Copy-New "$prior/$($generation.prefix).registration.json" "$dest/registration.json"
  Copy-New "$prior/$($generation.prefix).started.json" "$dest/started.json"
  Copy-New "$prior/$($generation.prefix).run.json" "$dest/result.json"
  Copy-New "$prior/$($generation.prefix).stdout.log" "$dest/stdout.log"
  Copy-New "$prior/$($generation.prefix).stderr.log" "$dest/stderr.log"
}
Copy-GitBlob $baseCommit $source "$formal/frozen/base-rne-bias.rs"
Copy-New $source "$formal/frozen/current-rne-bias.rs"
foreach($name in @('claims.json','execution-plan.json','bias-inventory.json','verification.json')){Copy-New "$root/$name" "$formal/records/$name"}
$sourceMap=[ordered]@{schema=1;archive_root=$formal;mappings=@($mappings);scope='pre-commit core verification only; delivery hooks are excluded'}
[IO.File]::WriteAllText((Join-Path $formal 'records/source-map.json'),((($sourceMap|ConvertTo-Json -Depth 8).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
$mappings.Add([ordered]@{path='records/source-map.json';source='generated from copied original paths'})
$formalPath=(Resolve-Path -LiteralPath $formal).Path
$members=@(Get-ChildItem -LiteralPath $formal -Recurse -File|Where-Object Name -ne 'manifest.json'|Sort-Object FullName|ForEach-Object{[ordered]@{path=$_.FullName.Substring($formalPath.Length+1).Replace('\','/');bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}})
if($members.Count -ne 67){throw "正式成员数错误：$($members.Count)"}
$manifest=[ordered]@{schema=1;root=$formal;self_excluded=$true;member_count=$members.Count;members=$members}
[IO.File]::WriteAllText((Join-Path $formal 'manifest.json'),((($manifest|ConvertTo-Json -Depth 6).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
[ordered]@{schema=1;member_count=$members.Count;mappings=$mappings.Count;result='pass'}|ConvertTo-Json
