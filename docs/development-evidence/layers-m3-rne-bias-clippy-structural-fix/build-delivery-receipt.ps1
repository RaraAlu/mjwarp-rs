$ErrorActionPreference='Stop'
$task='2026-10-19-layers-m3-rne-bias-clippy-structural-fix'
$source='src/physics/rne_bias.rs'
$evidence='docs/development-evidence/layers-m3-rne-bias-clippy-structural-fix'
$frozen="$evidence/formal-run/frozen/current-rne-bias.rs"
$receiptPath="docs/development-evidence/changes/$task.json"
if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash){throw '当前源码与成功检查归档不一致'}
$formal=Get-Content -LiteralPath "$evidence/formal-verification.json" -Raw|ConvertFrom-Json
if($formal.result -cne 'pass' -or $formal.structural_no_allow_exit -ne 0){throw '正式证据没有通过'}
& git add -- $source $evidence;if($LASTEXITCODE -ne 0){throw '任务文件暂存失败'}
$staged=@(& git diff --cached --name-only --diff-filter=ACDMRTUXB|Where-Object{$_ -ne $receiptPath})
if($LASTEXITCODE -ne 0 -or $staged.Count -eq 0){throw '暂存区为空'}
foreach($path in $staged){if($path -ne $source -and -not $path.StartsWith("$evidence/",[StringComparison]::Ordinal)){throw "暂存范围外文件：$path"}}
$files=@($staged|Sort-Object|ForEach-Object{$blob=@(& git rev-parse ":$_")[0];if($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$'){throw "对象解析失败：$_"};[ordered]@{path=$_;blob=$blob}})
$receipt=[ordered]@{
  schema=1;kind='delivery';task_id=$task;
  milestones=[ordered]@{implementation='complete';validation='passed';delivery_record='pending_commit'};
  files=$files;
  validation=@(
    [ordered]@{command='cargo test --locked --lib rne_bias_plan -- --test-threads=1';result='pass: 2 passed';evidence="$evidence/formal-run/runs/host/result.json"},
    [ordered]@{command='cargo fmt --check';result='pass after registered formatting repair and test-call sync';evidence="$evidence/formal-run/runs/format-final/result.json"},
    [ordered]@{command='cargo clippy --locked --all-targets --features cuda-probe -- -D warnings';result='pass without too_many_arguments allowance';evidence="$evidence/formal-run/runs/clippy/result.json"},
    [ordered]@{command='build-inventory.ps1';result='pass after anchor repair: 7 current rne_bias line anchors';evidence="$evidence/formal-run/runs/inventory/result.json"},
    [ordered]@{command='check-evidence.ps1';result='pass: ABI, semantics, CUDA and PTX unchanged';evidence="$evidence/formal-run/runs/evidence-check/result.json"},
    [ordered]@{command='check-formal-evidence.ps1';result='pass: 67 members and index objects';evidence="$evidence/formal-verification.json"}
  );
  preserved_failures=@(
    [ordered]@{gate='format';exit_code=1;reason='首次格式检查发现两处rustfmt差异';evidence="$evidence/formal-run/runs/format-first/result.json"},
    [ordered]@{gate='clippy';exit_code=101;reason='首次结构改动未同步cfg(cuda-probe)测试调用形状';evidence="$evidence/formal-run/runs/clippy-first/result.json"},
    [ordered]@{gate='inventory';exit_code=1;reason='首次编排行锚点仍匹配rustfmt前形状';evidence="$evidence/formal-run/runs/inventory-first/result.json"}
  );
  clippy_generations=@(
    [ordered]@{generation='initial';exit_code=101;meaning='too_many_arguments diagnostic active';evidence="$evidence/formal-run/historical/initial-exit-101/result.json"},
    [ordered]@{generation='allow';exit_code=0;meaning='passed with task-added allowance';evidence="$evidence/formal-run/historical/allow-exit-0/result.json"},
    [ordered]@{generation='structural';exit_code=0;meaning='passed after grouping inputs and removing allowance';evidence="$evidence/formal-run/runs/clippy/result.json"}
  );
  remaining=@('59 actual CUDA entry kinds remain unmigrated','74 product low-level call sites remain unmigrated','historical flg_acc=false still has 1042 failed blocks','NP01 through NP04 remain unaccepted','performance remains unaccepted','release and Linux were not run','GPU was not rerun','parent M3 remains incomplete')
}
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath),((($receipt|ConvertTo-Json -Depth 12).Replace("`r`n","`n"))+"`n"),[Text.UTF8Encoding]::new($false))
& git add -- $receiptPath;if($LASTEXITCODE -ne 0){throw '回执暂存失败'}
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;result='staged'}|ConvertTo-Json
