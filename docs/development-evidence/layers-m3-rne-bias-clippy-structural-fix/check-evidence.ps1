$ErrorActionPreference='Stop'
$task='2026-10-19-layers-m3-rne-bias-clippy-structural-fix'
$root="target/normify-evidence/$task"
$evidence='docs/development-evidence/layers-m3-rne-bias-clippy-structural-fix'
$source='src/physics/rne_bias.rs'
$baseCommit='d029f4ed5fa0402d637a04cb1aa4d0c5d4159f7b'
$verification="$evidence/verification.json"
if(Test-Path -LiteralPath $verification){throw '核验结果已存在'}
foreach($id in @('host','format-final','clippy-retry','inventory-retry')){
  $result=Get-Content -LiteralPath "$root/$(switch($id){'host'{'check-01-host'};'format-final'{'check-02-format-final'};'clippy-retry'{'check-03-clippy-retry'};'inventory-retry'{'check-04-inventory-retry'}})/result.json" -Raw|ConvertFrom-Json
  if($result.exit_code -ne 0){throw "核心验证失败：$id"}
}
$formatFirst=Get-Content -LiteralPath "$root/check-02-format/result.json" -Raw|ConvertFrom-Json
$formatApply=Get-Content -LiteralPath "$root/check-02-format-apply/result.json" -Raw|ConvertFrom-Json
if($formatFirst.exit_code -ne 1 -or $formatApply.exit_code -ne 0){throw '格式修复证据不完整'}
$clippyFirst=Get-Content -LiteralPath "$root/check-03-clippy/result.json" -Raw|ConvertFrom-Json
if($clippyFirst.exit_code -ne 101){throw '首次本任务Clippy编译失败未保留'}
$inventoryFirst=Get-Content -LiteralPath "$root/check-04-inventory/result.json" -Raw|ConvertFrom-Json
if($inventoryFirst.exit_code -ne 1){throw '首次盘点锚点失败未保留'}
$initial=Get-Content -LiteralPath 'docs/development-evidence/layers-m3-rne-bias/formal-run/runs/clippy-first.run.json' -Raw|ConvertFrom-Json
$allowPass=Get-Content -LiteralPath 'docs/development-evidence/layers-m3-rne-bias/formal-run/runs/clippy-final.run.json' -Raw|ConvertFrom-Json
if($initial.exit_code -ne 101 -or $allowPass.exit_code -ne 0){throw '历史Clippy代际错误'}
$allowBlob=@(& git rev-parse 'd735b1bfcea2a60668421c197e96a56947c111ec:src/physics/rne_bias.rs')[0]
if($LASTEXITCODE -ne 0 -or $allowBlob -cne '8692a10a1874e266cb68d940fe4f6a86fabc58b7'){throw 'allow版本源码对象不符'}
$current=[IO.File]::ReadAllText((Resolve-Path -LiteralPath $source)).Replace("`r`n","`n")
$baseLines=@(& git show "${baseCommit}:$source");if($LASTEXITCODE -ne 0){throw '基线源码读取失败'}
$base=($baseLines -join "`n")+"`n"
$diff=@(& git diff --unified=0 $baseCommit -- $source);if($LASTEXITCODE -ne 0){throw '源码差异读取失败'}
if(@($diff|Where-Object{$_ -match '^\+\s*#\[allow\(clippy::'}).Count -ne 0){throw '本次新增了Clippy豁免'}
if($current -match '#\[allow\(clippy::too_many_arguments\)\]'){throw '参数数量豁免仍然存在'}
if($current -notmatch "struct RneBiasInputs<'a>"){throw '内部输入分组缺失'}
function Extract([string]$Text,[string]$Start,[string]$End){
  $a=$Text.IndexOf($Start,[StringComparison]::Ordinal);if($a -lt 0){throw "缺少起点：$Start"}
  $b=$Text.IndexOf($End,$a,[StringComparison]::Ordinal);if($b -lt 0){throw "缺少终点：$End"}
  $Text.Substring($a,$b-$a)
}
$baseAbi=Extract $base 'struct RneBiasSpec;' 'struct RneBiasMetadata;'
$currentAbi=Extract $current 'struct RneBiasSpec;' 'struct RneBiasMetadata;'
if($baseAbi -cne $currentAbi){throw '产品ABI声明发生变化'}
$baseCuda=Extract $base 'const RNE_BIAS_CUDA: &str = r#"' '"#;'
$currentCuda=Extract $current 'const RNE_BIAS_CUDA: &str = r#"' '"#;'
if($baseCuda -cne $currentCuda){throw 'CUDA字符串发生变化'}
$cudaBytes=[Text.UTF8Encoding]::new($false).GetBytes($currentCuda.Substring('const RNE_BIAS_CUDA: &str = r#"'.Length))
$cudaHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($cudaBytes)).ToLowerInvariant()
if($cudaHash -cne '78239ca47c569395cdfce2b74f749813b73bbb0c032c51dd5d4422e5a1066667'){throw 'CUDA摘要不符'}
$ptx='docs/development-evidence/layers-m3-rne-bias/frozen/primary/rne_bias.ptx'
$ptxHash=(Get-FileHash -LiteralPath $ptx -Algorithm SHA256).Hash.ToLowerInvariant()
if($ptxHash -cne '5e5433159d2749ba4d5c29bff60a239891c1cd75030a1e66ec84f1bacf32f934'){throw '冻结PTX摘要不符'}
foreach($anchor in @(
  '!self.session.same_session(session)',
  '!inputs.metadata.session.same_session(session)',
  'let proof = inputs.metadata.proof(worlds)?;',
  'let bindings = Self::bindings(inputs, output)?;',
  'rne_bias_plan(proof.dims(), &bindings, &[proof.binding(0)])'
)){if($current.IndexOf($anchor,[StringComparison]::Ordinal) -lt 0){throw "预检语义锚点缺失：$anchor"}}
$readyFalse=$current.IndexOf('data.ready = false;',[StringComparison]::Ordinal)
$preflight=$current.IndexOf('let launch = self.kernel.preflight(',[StringComparison]::Ordinal)
$launch=$current.IndexOf('.launch(&launch, &inputs, &mut data.output)?;',[StringComparison]::Ordinal)
$readyTrue=$current.IndexOf('data.ready = true;',[StringComparison]::Ordinal)
if(-not($readyFalse -lt $preflight -and $preflight -lt $launch -and $launch -lt $readyTrue)){throw 'ready或同步顺序发生变化'}
$lowLevel=@('inputs.metadata.as_buffer(),','inputs.qvel,','inputs.com,','inputs.velocity,','output,','dims.worlds,')
$cursor=$current.IndexOf('self.kernel.launch_flex_edges(',[StringComparison]::Ordinal)
foreach($anchor in $lowLevel){$next=$current.IndexOf($anchor,$cursor,[StringComparison]::Ordinal);if($next -lt $cursor){throw "低层ABI顺序错误：$anchor"};$cursor=$next+$anchor.Length}
$inventory=Get-Content -LiteralPath "$evidence/bias-inventory.json" -Raw|ConvertFrom-Json
if($inventory.source.sha256 -cne (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()){throw '盘点未绑定当前源码'}
$record=[ordered]@{schema=1;base_commit=$baseCommit;source_sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant();abi_unchanged=$true;cuda_sha256=$cudaHash;ptx_sha256=$ptxHash;allowance_present=$false;new_clippy_allowances=0;host_exit=0;format_first_exit=1;format_apply_exit=0;format_final_exit=0;clippy_compile_shape_exit=101;clippy_exit=0;inventory_first_exit=1;inventory_retry_exit=0;historical_initial_exit=101;historical_allow_pass_exit=0;gpu_rerun=$false;product_test_rerun=$false;result='pass'}
[IO.File]::WriteAllText((Join-Path (Get-Location) $verification),((($record|ConvertTo-Json -Depth 6).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
$record|ConvertTo-Json -Depth 6
