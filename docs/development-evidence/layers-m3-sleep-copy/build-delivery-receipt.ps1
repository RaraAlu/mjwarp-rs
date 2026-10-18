$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-sleep-copy'
$evidence = 'docs/development-evidence/layers-m3-sleep-copy'
$runtime = "target/normify-evidence/$task"
$source = 'src/physics/resident.rs'
$currentSource = "$evidence/current-resident.rs"
$receiptPath = "docs/development-evidence/changes/$task.json"

function Copy-New([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少交付原件：$Source" }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    if (Test-Path -LiteralPath $Destination) {
        if ((Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash -cne
            (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash) { throw "交付归档冲突：$Destination" }
        return
    }
    [IO.File]::WriteAllBytes($Destination, [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source)))
}

$postRoot = "$evidence/post-freeze-check"
foreach ($item in @(
    @{source="$runtime/check-11-evidence-final/result.json";destination="$postRoot/evidence-final.run.json"},
    @{source="$runtime/check-11-evidence-final/stdout.log";destination="$postRoot/evidence-final.stdout.log"},
    @{source="$runtime/check-11-evidence-final/stderr.log";destination="$postRoot/evidence-final.stderr.log"}
)) { Copy-New $item.source $item.destination }
$indexReady = Test-Path -LiteralPath "$runtime/check-12-index-bytes/result.json" -PathType Leaf
if ($indexReady) {
    foreach ($item in @(
        @{source="$runtime/registrations/index-bytes.json";destination="$evidence/index-check/registration.json"},
        @{source="$runtime/check-12-index-bytes/started.json";destination="$evidence/index-check/started.json"},
        @{source="$runtime/check-12-index-bytes/result.json";destination="$evidence/index-check/result.json"},
        @{source="$runtime/check-12-index-bytes/stdout.log";destination="$evidence/index-check/stdout.log"},
        @{source="$runtime/check-12-index-bytes/stderr.log";destination="$evidence/index-check/stderr.log"}
    )) { Copy-New $item.source $item.destination }
}
$postMembers = @(Get-ChildItem -LiteralPath $postRoot -Recurse -File |
    Where-Object Name -ne 'manifest.json' | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            path=$_.FullName.Substring((Resolve-Path -LiteralPath $postRoot).Path.Length + 1).Replace('\','/')
            bytes=$_.Length
            sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    })
$postManifest = [ordered]@{schema=1;root=$postRoot;self_excluded=$true;member_count=$postMembers.Count;members=$postMembers}
$postJson = ($postManifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'manifest.json'), $postJson, [Text.UTF8Encoding]::new($false))

if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -cne
    (Get-FileHash -LiteralPath $currentSource -Algorithm SHA256).Hash) {
    throw '当前源码与冻结原件不一致'
}
& git add -- $source $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$sourceBlob = @(& git rev-parse ":$source")[0].Trim()
$currentBlob = @(& git rev-parse ":$currentSource")[0].Trim()
if ($LASTEXITCODE -ne 0 -or $sourceBlob -cne $currentBlob) { throw '暂存源码与冻结原件不一致' }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0 -or $staged.Count -eq 0) { throw '暂存区读取失败或为空' }
foreach ($path in $staged) {
    if ($path -ne $source -and -not $path.StartsWith("$evidence/")) { throw "暂存范围外文件：$path" }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})
$inventory = Get-Content -LiteralPath "$evidence/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
$receipt = [ordered]@{
    schema=1
    kind='delivery'
    task_id=$task
    milestones=[ordered]@{implementation='complete';validation='passed';delivery_record='pending_commit'}
    files=$files
    validation=@(
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::resident::tests::sleep_copy_wrapper_checks_span_session_alias_grid_and_metadata_proof -- --exact --test-threads=1';result='pass: 1 passed';evidence="$evidence/formal-run/host.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::resident::tests::gpu_m3_sleep_copy_uses_real_resident_buffers_and_recovers -- --ignored --exact --test-threads=1';result='pass on NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/gpu-product.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::resident::tests::tendon_wake_readback_rejects_guards_with_static_flex_and_zero_trees -- --ignored --exact --test-threads=1';result='pass on NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/gpu-zero-tree.run.json"},
        [ordered]@{command='cargo fmt --check';result='first failed and preserved; final registered retry passed';evidence="$evidence/formal-run/fmt-final.run.json"},
        [ordered]@{command='cargo clippy --locked --features cuda-probe --lib -- -D warnings';result='pass';evidence="$evidence/formal-run/clippy.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-sleep-copy/build-precision-map-v2.ps1';result='pass: 18 groups, 5 outputs, 2 PTX entries';evidence="$evidence/formal-run/precision-map-v2.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-sleep-copy/check-evidence.ps1';result='pass after two meaningful checker fixes';evidence="$evidence/formal-run/evidence-final.run.json"}
    )
    preserved_failures=@(
        [ordered]@{gate='format';exit_code=1;evidence="$evidence/formal-run/fmt-first.run.json";reason='rustfmt差异'},
        [ordered]@{gate='old-t4-identity';exit_code=101;evidence="$evidence/formal-run/precision-old-gate.run.json";reason='resident.rs全文身份按设计漂移'},
        [ordered]@{gate='evidence-v1';exit_code=1;evidence="$evidence/formal-run/evidence-first.run.json";reason='缺少通用发射派生预期'},
        [ordered]@{gate='evidence-v1-retry';exit_code=1;evidence="$evidence/formal-run/evidence-retry-01.run.json";reason='NVRTC字段名不匹配'}
    )
    remaining=@(
        "$($inventory.counts.unmigrated_actual_entry_kinds) actual CUDA entry kinds remain unmigrated",
        "$($inventory.counts.unmigrated_low_level_call_sites) product low-level call sites remain unmigrated",
        'old whole-file T4 identity gate remains 1 passed and 3 failed by design',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'NP01 through NP04 remain unaccepted',
        'performance remains unaccepted',
        'parent M3 remains incomplete'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;index_evidence=$indexReady;result='staged'} | ConvertTo-Json
