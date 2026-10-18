$ErrorActionPreference='Stop'
$task='2026-10-19-layers-m3-rne-bias-clippy-structural-fix'
$base='docs/development-evidence/layers-m3-rne-bias/unmigrated-launch-inventory.json'
$source='src/physics/rne_bias.rs'
$output='docs/development-evidence/layers-m3-rne-bias-clippy-structural-fix/bias-inventory.json'
if(Test-Path -LiteralPath $output){throw '盘点输出已存在'}
$baseline=Get-Content -LiteralPath $base -Raw|ConvertFrom-Json
$lines=[IO.File]::ReadAllLines((Resolve-Path -LiteralPath $source))
function Find-Line([string]$Name,[string]$Pattern){
  $matches=@(for($i=0;$i -lt $lines.Length;$i++){if($lines[$i] -match $Pattern){$i+1}})
  if($matches.Count -ne 1){throw "行号锚点不唯一：$Name"}
  $matches[0]
}
$baseBlob=@(& git rev-parse "HEAD:$base")[0]
if($LASTEXITCODE -ne 0 -or $baseBlob -notmatch '^[0-9a-f]{40}$'){throw '基线清单对象解析失败'}
$inventory=[ordered]@{
  schema=1;task_id=$task;
  baseline=[ordered]@{path=$base;sha256=(Get-FileHash -LiteralPath $base -Algorithm SHA256).Hash.ToLowerInvariant();blob=$baseBlob;modified=$false};
  source=[ordered]@{path=$source;sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()};
  inherited_counts=$baseline.counts;
  rne_bias=[ordered]@{
    migration_status='typed_safe_wrapper';
    spec_definition_line=Find-Line 'RneBiasSpec' '^struct RneBiasSpec;$';
    input_group_line=Find-Line 'RneBiasInputs' "^struct RneBiasInputs<'a> \{$";
    compile_line=Find-Line 'compile' 'SynchronousKernel::compile\(session, RNE_BIAS_CUDA, "rne_bias"\)\?';
    preflight_line=Find-Line 'preflight' '^    fn preflight\($';
    low_level_launch_line=Find-Line 'low-level launch' '^            self\.kernel\.launch_flex_edges\($';
    orchestration_line=Find-Line 'wrapper orchestration' '^            self\.kernel\.launch\(&launch, &inputs, &mut data\.output\)\?;$';
    cuda_source_line=Find-Line 'CUDA source' '^const RNE_BIAS_CUDA: &str = r#"$';
  };
  boundaries=[ordered]@{repository_counts_recomputed=$false;other_files_reclassified=$false;next_candidate_started=$false;gpu_rerun=$false}
}
[IO.File]::WriteAllText((Join-Path (Get-Location) $output),((($inventory|ConvertTo-Json -Depth 8).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
[ordered]@{schema=1;baseline_blob=$baseBlob;source_sha256=$inventory.source.sha256;updated_lines=7;result='pass'}|ConvertTo-Json
