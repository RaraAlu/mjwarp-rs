$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-applied-force-validate'
$evidence = "docs/development-evidence/layers-m3-applied-force-validate"
$source = 'src/physics/applied_force.rs'
$receiptPath = "docs/development-evidence/changes/$task.json"
& git add -- $source $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0) { throw "git diff退出码：$LASTEXITCODE" }
if ($staged.Count -eq 0) { throw '暂存区为空' }
foreach ($path in $staged) {
    if ($path -ne $source -and -not $path.StartsWith("$evidence/")) {
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
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::applied_force::tests::validate_wrapper_rejects_capacity_context_alias_metadata_and_work_faults -- --exact --test-threads=1';result='pass: 1 passed';evidence="$evidence/formal-run/host-retry.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::applied_force::tests::gpu_m3_validate_wrapper_preserves_bits_and_recovers_faults -- --ignored --exact --test-threads=1 --nocapture';result='pass on NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/run.json"},
        [ordered]@{command='cargo fmt --check';result='pass after recorded formatting correction';evidence="$evidence/formal-run/fmt-final.run.json"},
        [ordered]@{command='cargo clippy --locked --features cuda-probe --lib -- -D warnings';result='pass';evidence="$evidence/formal-run/clippy-final.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-applied-force-validate/check-evidence.ps1';result='pass';evidence="$evidence/formal-run/evidence-final.run.json"}
    )
    remaining = @(
        'applied_force_publish remains a raw launch',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'negative-parent and out-of-range-root GPU faults were not rerun in this slice',
        'U075 and NP01 through NP04 remain unaccepted',
        'parent M3 remains incomplete',
        'historical process gaps remain unchanged'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 8).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;result='staged'} | ConvertTo-Json
