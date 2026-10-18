param([Parameter(Mandatory=$true)][string]$Registration)
$ErrorActionPreference='Stop'
$task='2026-10-19-layers-m3-rne-bias-clippy-structural-fix'
$root="target/normify-evidence/$task"
if(-not $Registration.StartsWith("$root/registrations/",[StringComparison]::Ordinal)){throw '登记越界'}
$record=Get-Content -LiteralPath $Registration -Raw|ConvertFrom-Json
if($record.task_id -cne $task -or $record.registered_before_execution -ne $true){throw '登记无效'}
$hash=(Get-FileHash -LiteralPath $Registration -Algorithm SHA256).Hash.ToLowerInvariant()
if(Test-Path -LiteralPath $record.output_root){throw '输出目录已存在'}
New-Item -ItemType Directory -Path $record.output_root|Out-Null
[IO.File]::WriteAllBytes($record.stdout,[byte[]]::new(0));[IO.File]::WriteAllBytes($record.stderr,[byte[]]::new(0))
$started=[DateTime]::UtcNow
$start=[ordered]@{schema=1;id=$record.id;registration=$Registration;registration_sha256=$hash;started_at_utc=$started.ToString('o');stdout_precreated=$true;stderr_precreated=$true}
[IO.File]::WriteAllText((Join-Path $record.output_root 'started.json'),((($start|ConvertTo-Json).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
$watch=[Diagnostics.Stopwatch]::StartNew();$exitCode=0
try{$exe=[string]$record.executable;$argsList=@($record.arguments);& $exe @argsList 1>> $record.stdout 2>> $record.stderr;$exitCode=$LASTEXITCODE;if($null -eq $exitCode){$exitCode=0}}
catch{$_|Out-String|Add-Content -LiteralPath $record.stderr -Encoding utf8;$exitCode=9009}
$watch.Stop();$ended=[DateTime]::UtcNow
if((Get-FileHash -LiteralPath $Registration -Algorithm SHA256).Hash.ToLowerInvariant() -cne $hash){throw '登记在运行中变化'}
$result=[ordered]@{schema=1;id=$record.id;registration=$Registration;registration_sha256=$hash;started_at_utc=$started.ToString('o');ended_at_utc=$ended.ToString('o');duration_ms=$watch.ElapsedMilliseconds;exit_code=$exitCode;stdout=$record.stdout;stderr=$record.stderr;result=if($exitCode -eq 0){'pass'}else{'fail'}}
[IO.File]::WriteAllText((Join-Path $record.output_root 'result.json'),((($result|ConvertTo-Json).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
Write-Output '--- stdout ---';Get-Content -LiteralPath $record.stdout
Write-Output '--- stderr ---';Get-Content -LiteralPath $record.stderr
Write-Output '--- result ---';$result|ConvertTo-Json
exit $exitCode
