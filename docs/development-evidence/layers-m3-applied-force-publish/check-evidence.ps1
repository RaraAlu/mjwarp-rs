$ErrorActionPreference = 'Stop'
$task = 'docs/development-evidence/layers-m3-applied-force-publish'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-publish'
$sourcePath = 'src/physics/applied_force.rs'
$base = '6bcf8496477e154d5f904b4aa1884637ecbe5b9e'
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$required = @(
    'struct AppliedForcePublishSpec;',
    'struct AppliedForcePublishKernel {',
    'publish: AppliedForcePublishKernel',
    'self.publish.launch(',
    'unsafe { self.kernel.launch(metadata, current, delta, staged, dims) }',
    'struct AppliedForceValidateKernel {',
    'struct AppliedForceDeltaKernel {',
    'staged[4+i]=add?__fadd_rn(current[4+i],delta[4+i]):delta[4+i];'
)
foreach ($text in $required) {
    if (-not $source.Contains($text)) { throw "缺少源码证据：$text" }
}
if ($source.Contains('publish: SynchronousKernel')) { throw '发布入口仍是原始字段' }

$cuda = [regex]::Match(
    $source,
    'const CUDA: &str = r#"(.*?)"#;',
    [Text.RegularExpressions.RegexOptions]::Singleline
)
if (-not $cuda.Success) { throw '找不到CUDA源码' }
$sha = [Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($cuda.Groups[1].Value))
$cudaHash = [Convert]::ToHexString($sha).ToLowerInvariant()
if ($cudaHash -ne 'edb51d900a9f14c3779e6a411dc1c46998cdd91ca70d880ae4f862cf428ea953') {
    throw "CUDA源码漂移：$cudaHash"
}
$fixtureHash = (Get-FileHash -LiteralPath 'fixtures/applied-force/reference.json' -Algorithm SHA256).Hash.ToLowerInvariant()
if ($fixtureHash -ne '13391225d2761132a931b63de65e5bd965a2e7eb93a135f9c0db4684bb2c7cde') {
    throw "夹具漂移：$fixtureHash"
}
$diff = @(& git diff --unified=0 --no-ext-diff $base -- $sourcePath)
if ($LASTEXITCODE -ne 0) { throw "源码差异读取失败：$LASTEXITCODE" }
$toleranceDrift = @($diff | Where-Object {
    $_ -match '^[+-](?![+-])' -and $_ -match '(tolerance|epsilon|abs\(\)\s*<=|[0-9]e-[0-9])'
})
if ($toleranceDrift.Count -ne 0) { throw "容差源码漂移：$($toleranceDrift -join ' | ')" }

foreach ($path in @("$task/claims.json", "$task/execution-plan.json", "$task/unmigrated-launch-inventory.json")) {
    $null = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
}
$inventory = Get-Content -LiteralPath "$task/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
$expectedCounts = [ordered]@{
    actual_cuda_entry_kinds = 88
    typed_actual_entry_kinds = 22
    unmigrated_actual_entry_kinds = 66
    product_low_level_call_sites = 99
    typed_low_level_call_sites = 18
    unmigrated_low_level_call_sites = 81
    wrapper_orchestration_call_sites = 20
    kernel_abi_families = 18
}
foreach ($name in $expectedCounts.Keys) {
    if ($inventory.counts.$name -ne $expectedCounts[$name]) {
        throw "盘点数量错误：$name=$($inventory.counts.$name)"
    }
}
$publish = @($inventory.actual_cuda_entry_kinds | Where-Object entry -eq 'applied_force_publish')
if ($publish.Count -ne 1 -or $publish[0].migration_status -cne 'typed_safe_wrapper') {
    throw '发布入口没有完成重分类'
}

$requiredRuns = [ordered]@{
    'check-01-host/run.json' = 101
    'check-01-host-retry/run.json' = 0
    'check-01-host-assertion-retry/run.json' = 0
    'check-02-device/run.json' = 0
    'run-01/run.json' = 0
    'run-02/run.json' = 0
    'check-03-format/run.json' = 1
    'check-03-format-retry/run.json' = 0
    'check-04-clippy/run.json' = 0
}
foreach ($relative in $requiredRuns.Keys) {
    $run = Get-Content -LiteralPath "$runtime/$relative" -Raw | ConvertFrom-Json
    if ($run.exit_code -ne $requiredRuns[$relative]) { throw "运行退出码错误：$relative" }
}
$device = Get-Content -LiteralPath "$runtime/check-02-device/stdout.txt" -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,') { throw '本机T4记录错误' }
if ($device.cuda_root -cne 'target/toolchains/cuda-12.8.1') { throw 'CUDA根目录错误' }
$report = Get-Content -LiteralPath "$runtime/run-02/publish-report.json" -Raw | ConvertFrom-Json
foreach ($flag in @('replace_ignores_old_nan', 'accumulate_bitwise', 'nonfinite_publish_rejected', 'current_unchanged_on_failure', 'same_plan_recovered')) {
    if ($report.$flag -ne $true) { throw "GPU报告缺少通过项：$flag" }
}
if ($report.guard_failures -ne 3 -or $report.task_gpu_memory -cne 'unknown') { throw 'GPU报告边界错误' }

$capture = "$runtime/run-02/gpu-capture"
$baseline = 'docs/development-evidence/layers-m3-applied-force-validate/formal-run/gpu-capture'
$records = @(Get-ChildItem -LiteralPath $capture -Filter '*.json' -File)
if ($records.Count -ne 3) { throw "捕获记录数错误：$($records.Count)" }
$entries = @($records | ForEach-Object { (Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json).entry })
$expectedEntries = @('applied_force_delta', 'applied_force_publish', 'applied_force_validate')
if (@($entries | Sort-Object -Unique).Count -ne 3) { throw '捕获入口存在重复' }
if ((Compare-Object ($entries | Sort-Object) $expectedEntries).Count -ne 0) { throw '捕获入口集合错误' }
$capturedFiles = @(Get-ChildItem -LiteralPath $capture -File | Sort-Object Name)
$baselineFiles = @(Get-ChildItem -LiteralPath $baseline -File | Sort-Object Name)
if ($capturedFiles.Count -ne 9 -or $baselineFiles.Count -ne 9) { throw '捕获文件数量错误' }
for ($index = 0; $index -lt $capturedFiles.Count; $index++) {
    if ($capturedFiles[$index].Name -cne $baselineFiles[$index].Name) { throw '捕获文件名漂移' }
    $currentHash = (Get-FileHash -LiteralPath $capturedFiles[$index].FullName -Algorithm SHA256).Hash
    $priorHash = (Get-FileHash -LiteralPath $baselineFiles[$index].FullName -Algorithm SHA256).Hash
    if ($currentHash -cne $priorHash) { throw "捕获字节漂移：$($capturedFiles[$index].Name)" }
}

$tracked = @(& git diff --name-only)
if ($LASTEXITCODE -ne 0) { throw "变更范围读取失败：$LASTEXITCODE" }
foreach ($path in $tracked) {
    if ($path -notin @('scripts/check-evidence-blobs.ps1', $sourcePath)) { throw "范围外跟踪文件：$path" }
}
& git diff --check
if ($LASTEXITCODE -ne 0) { throw "git diff --check退出码：$LASTEXITCODE" }
[ordered]@{
    schema = 1
    cuda_source_sha256 = $cudaHash
    fixture_sha256 = $fixtureHash
    capture_records = $records.Count
    capture_files = $capturedFiles.Count
    capture_entries = @($entries | Sort-Object)
    capture_matches_prior_bytes = $true
    typed_actual_entry_kinds = 22
    unmigrated_actual_entry_kinds = 66
    typed_low_level_call_sites = 18
    unmigrated_low_level_call_sites = 81
    wrapper_orchestration_call_sites = 20
    publish_typed = $true
    task_gpu = 'NVIDIA GeForce RTX 4070 Ti SUPER'
    task_memory = 'unknown'
} | ConvertTo-Json
