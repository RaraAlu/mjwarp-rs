$ErrorActionPreference='Stop'
$root='docs/development-evidence/layers-m3-rne-bias-clippy-structural-fix'
$formal="$root/formal-run"
$output="$root/formal-verification.json"
$source='src/physics/rne_bias.rs'
$baseCommit='d029f4ed5fa0402d637a04cb1aa4d0c5d4159f7b'
if(Test-Path -LiteralPath $output){throw '正式核验结果已存在'}
$manifest=Get-Content -LiteralPath "$formal/manifest.json" -Raw|ConvertFrom-Json
if($manifest.member_count -ne 67 -or @($manifest.members).Count -ne 67){throw '正式清单成员数错误'}
$formalPath=(Resolve-Path -LiteralPath $formal).Path
foreach($member in $manifest.members){
  $path=Join-Path $formalPath ([string]$member.path);if(-not(Test-Path -LiteralPath $path -PathType Leaf)){throw "缺少成员：$($member.path)"}
  $item=Get-Item -LiteralPath $path;if($item.Length -ne $member.bytes){throw "成员长度漂移：$($member.path)"}
  $hash=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant();if($hash -cne $member.sha256){throw "成员摘要漂移：$($member.path)"}
}
$currentHash=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
$frozenHash=(Get-FileHash -LiteralPath "$formal/frozen/current-rne-bias.rs" -Algorithm SHA256).Hash.ToLowerInvariant()
if($currentHash -cne $frozenHash){throw '当前源码与成功检查归档不一致'}
$baseBlob=@(& git rev-parse "${baseCommit}:$source")[0]
$frozenBaseBlob=@(& git hash-object "$formal/frozen/base-rne-bias.rs")[0]
if($LASTEXITCODE -ne 0 -or $baseBlob -cne $frozenBaseBlob){throw '基线源码对象不一致'}
$initial=Get-Content -LiteralPath "$formal/historical/initial-exit-101/result.json" -Raw|ConvertFrom-Json
$allow=Get-Content -LiteralPath "$formal/historical/allow-exit-0/result.json" -Raw|ConvertFrom-Json
$structural=Get-Content -LiteralPath "$formal/runs/clippy/result.json" -Raw|ConvertFrom-Json
if($initial.exit_code -ne 101 -or $allow.exit_code -ne 0 -or $structural.exit_code -ne 0){throw '三代Clippy证据不完整'}
$currentText=Get-Content -LiteralPath "$formal/frozen/current-rne-bias.rs" -Raw
if($currentText -match '#\[allow\(clippy::too_many_arguments\)\]'){throw '归档源码仍含参数数量豁免'}
$sourceMap=Get-Content -LiteralPath "$formal/records/source-map.json" -Raw|ConvertFrom-Json
if(@($sourceMap.mappings).Count -ne 66){throw '索引映射数量错误'}
$manifestBlob=@(& git hash-object "$formal/manifest.json")[0]
$sourceMapBlob=@(& git hash-object "$formal/records/source-map.json")[0]
if($LASTEXITCODE -ne 0 -or $manifestBlob -notmatch '^[0-9a-f]{40}$' -or $sourceMapBlob -notmatch '^[0-9a-f]{40}$'){throw '索引对象计算失败'}
$record=[ordered]@{schema=1;member_count=67;source_sha256=$currentHash;manifest_blob=$manifestBlob;source_map_blob=$sourceMapBlob;historical_initial_exit=101;historical_allow_exit=0;structural_no_allow_exit=0;gpu_rerun=$false;product_test_rerun=$false;result='pass'}
[IO.File]::WriteAllText((Join-Path (Get-Location) $output),((($record|ConvertTo-Json -Depth 6).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
$record|ConvertTo-Json -Depth 6
