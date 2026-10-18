param([Parameter(Mandatory=$true)][string]$Id)
$ErrorActionPreference='Stop'
$task='2026-10-19-layers-m3-rne-bias-clippy-structural-fix'
$root="target/normify-evidence/$task"
$plan=Get-Content -LiteralPath 'docs/development-evidence/layers-m3-rne-bias-clippy-structural-fix/execution-plan.json' -Raw|ConvertFrom-Json
$entries=@($plan.registered_validations|Where-Object id -CEQ $Id)
if($entries.Count -ne 1){throw "运行计划不唯一：$Id"}
$entry=$entries[0]
if(-not $entry.registered_before_execution){throw "运行未事前登记：$Id"}
if(-not $entry.output_root.StartsWith("$root/",[StringComparison]::Ordinal)){throw '输出目录越界'}
if(Test-Path -LiteralPath $entry.output_root){throw "输出目录已存在：$Id"}
$registry="$root/registrations";New-Item -ItemType Directory -Force -Path $registry|Out-Null
$path="$registry/$Id.json";if(Test-Path -LiteralPath $path){throw "登记已存在：$Id"}
$record=[ordered]@{schema=1;task_id=$task;id=$Id;registered_before_execution=$true;registered_at_utc=[DateTime]::UtcNow.ToString('o');purpose=$entry.purpose;executable=$entry.executable;arguments=@($entry.arguments);environment=$entry.environment;output_root=$entry.output_root;stdout="$($entry.output_root)/stdout.log";stderr="$($entry.output_root)/stderr.log"}
[IO.File]::WriteAllText((Join-Path (Get-Location) $path),((($record|ConvertTo-Json -Depth 8).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
[ordered]@{schema=1;registration=$path;registration_sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant();output_root=$entry.output_root;result='registered'}|ConvertTo-Json
