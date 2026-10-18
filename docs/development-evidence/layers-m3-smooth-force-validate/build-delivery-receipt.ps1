$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-smooth-force-validate'
$evidence = "docs/development-evidence/layers-m3-smooth-force-validate"
$source = 'src/physics/smooth_force.rs'
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
$preserved = [Collections.Generic.List[object]]::new()
$preserved.Add([ordered]@{
    gate = 'host-wrapper'
    evidence = "$evidence/formal-run/host-first.run.json"
    exit_code = 101
    reason = '测试先错误期待容量错误；实际证明先拒绝元数据跨度不符'
})
$preserved.Add([ordered]@{
    gate = 'clippy'
    evidence = "$evidence/formal-run/clippy-first.run.json"
    exit_code = 101
    reason = '测试专用导入与方法未受cfg(test)约束'
})
$preserved.Add([ordered]@{
    gate = 'cached-diff'
    evidence = "$evidence/delivery-check/cached-diff-first.run.json"
    exit_code = 2
    reason = '五个证据脚本含新增尾空行'
})
$stagedFailure = "$evidence/delivery-check/staged-first.run.json"
if (Test-Path -LiteralPath $stagedFailure) {
    $preserved.Add([ordered]@{
        gate = 'Staged'
        evidence = $stagedFailure
        exit_code = 1
        reason = '回执引用的索引字节日志尚未进入索引'
    })
}
$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $task
    milestones = [ordered]@{
        implementation = 'complete'
        validation = 'passed'
        delivery_record = 'pending_commit'
    }
    process_compliance = [ordered]@{
        retry_pre_registration = 'missing'
        record = "$evidence/retry-registration-gap.json"
        scope_validation_passed = $true
        full_process_compliance = $false
        parent_m3_passed = $false
    }
    files = $files
    validation = @(
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::smooth_force::tests::validate_wrapper_rejects_capacity_context_alias_span_dimension_and_proof_faults -- --exact --test-threads=1';result='final pass after preserved assertion correction and evidence expansion: 1 passed';evidence="$evidence/formal-run/host-final.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::smooth_force::tests::gpu_m3_validate_wrapper_sleep_disabled_preserves_output_and_recovers -- --ignored --exact --test-threads=1 --nocapture';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/gpu-off-final.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::smooth_force::tests::gpu_m3_validate_wrapper_sleep_enabled_covers_sources_and_ntree_zero -- --ignored --exact --test-threads=1 --nocapture';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/gpu-on-final.run.json"},
        [ordered]@{command='cargo fmt --check';result='pass';evidence="$evidence/formal-run/fmt-final.run.json"},
        [ordered]@{command='cargo clippy --locked --features cuda-probe --lib -- -D warnings';result='pass after preserved cfg-only correction';evidence="$evidence/formal-run/clippy-final.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-smooth-force-validate/check-evidence.ps1';result='final registered pass after supervisor tool IDs were recorded';evidence="$evidence/post-freeze-check/result.json"},
        [ordered]@{command='git diff --cached --check';result='pass after preserved evidence-script trailing blank-line correction';evidence="$evidence/delivery-check/cached-diff-final.run.json"},
        [ordered]@{command='pwsh -NoProfile -File scripts/check-evidence-blobs.ps1 -Manifest docs/development-evidence/layers-m3-smooth-force-validate/formal-run/manifest.json -Map docs/development-evidence/layers-m3-smooth-force-validate/source-map.json -Revision INDEX';result='pass';evidence="$evidence/index-check/result.json"}
    )
    preserved_failures = @($preserved)
    remaining = @(
        'retry pre-registration is missing; retry-registration-gap.json is retrospective only and does not erase the historical process gap',
        '65 actual CUDA entry kinds and 80 product low-level call sites remain unmigrated',
        'smooth_force remains the next raw entry',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'U016 and NP01 through NP04 remain unaccepted',
        'parent M3 remains proposed and incomplete'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;result='staged'} | ConvertTo-Json
