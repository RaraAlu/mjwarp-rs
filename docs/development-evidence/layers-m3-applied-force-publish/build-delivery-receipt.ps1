$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-applied-force-publish'
$evidence = "docs/development-evidence/layers-m3-applied-force-publish"
$source = 'src/physics/applied_force.rs'
$checker = 'scripts/check-evidence-blobs.ps1'
$receiptPath = "docs/development-evidence/changes/$task.json"
& git add -- $source $checker $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0) { throw "git diff退出码：$LASTEXITCODE" }
if ($staged.Count -eq 0) { throw '暂存区为空' }
foreach ($path in $staged) {
    if ($path -ne $source -and $path -ne $checker -and -not $path.StartsWith("$evidence/")) {
        throw "暂存范围外文件：$path"
    }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})
$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $task
    milestones = [ordered]@{
        implementation = 'complete'
        validation = 'passed'
        delivery_record = 'pending_commit'
    }
    files = $files
    validation = @(
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::applied_force::tests::publish_wrapper_rejects_capacity_context_alias_and_dimension_faults -- --exact --test-threads=1';result='pass after two preserved assertion corrections: 1 passed';evidence="$evidence/formal-run/host-final.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::applied_force::tests::gpu_m3_publish_wrapper_preserves_bits_and_recovers_faults -- --ignored --exact --test-threads=1 --nocapture';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/gpu.run.json"},
        [ordered]@{command='cargo fmt --check';result='pass after preserved formatting failure';evidence="$evidence/formal-run/fmt-final.run.json"},
        [ordered]@{command='cargo clippy --locked --features cuda-probe --lib -- -D warnings';result='pass';evidence="$evidence/formal-run/clippy.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-applied-force-publish/check-evidence.ps1';result='pass';evidence="$evidence/formal-run/evidence.run.json"},
        [ordered]@{command='pwsh -NoProfile -File scripts/check-evidence-blobs.ps1 -Manifest docs/development-evidence/layers-m3-applied-force-publish/formal-run/manifest.json -Map docs/development-evidence/layers-m3-applied-force-publish/source-map.json -Revision INDEX';result='pass: 53 members match';evidence="$evidence/index-check/run.json"}
    )
    preserved_failures = @(
        [ordered]@{
            gate = 'Staged'
            source = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-publish/delivery-04-staged'
            evidence = "$evidence/delivery-check/staged-first.run.json"
            exit_code = 1
            reason = '回执引用的索引字节日志尚未进入索引'
        }
    )
    remaining = @(
        '66 actual CUDA entry kinds and 81 product low-level call sites remain unmigrated',
        'the next migration candidate remains unselected',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'U075 and NP01 through NP04 remain unaccepted',
        'parent M3 remains proposed and incomplete'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 8).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;result='staged'} | ConvertTo-Json
